package com.lkjmc.common.sync;

import java.net.URI;
import java.net.URLEncoder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.sql.Array;
import java.sql.Connection;
import java.sql.DriverManager;
import java.sql.PreparedStatement;
import java.sql.Statement;
import java.time.Duration;
import java.util.Comparator;
import java.util.HexFormat;
import java.util.List;
import java.util.UUID;

final class HarnessDatabase implements AutoCloseable {
    private final String baseUrl;
    private final String schema;
    private final String jdbcUrl;
    private final String daemonUrl;
    private final UUID player = UUID.randomUUID();
    private final String firstToken = "sync-harness-" + UUID.randomUUID();

    HarnessDatabase(Path root) throws Exception {
        baseUrl = requireDatabase();
        schema = "sync_harness_" + UUID.randomUUID().toString().replace("-", "");
        try (Connection control = DriverManager.getConnection(jdbc(baseUrl));
             Statement statement = control.createStatement()) {
            statement.execute("create schema " + schema);
        }
        jdbcUrl = parameter(jdbc(baseUrl), "currentSchema", schema);
        daemonUrl = parameter(baseUrl, "options", "-c search_path=" + schema + ",public");
        migrate(root.resolve("migrations"));
        createCredential(firstToken, "velocity");
        createPlayer(player);
        seedDomainRevisions();
    }

    String daemonUrl() { return daemonUrl; }
    String token() { return firstToken; }
    UUID player() { return player; }

    String createCredential(String token, String surface) throws Exception {
        String sql = "insert into daemon_tokens(credential_id,token_hash,surface,principal_kind,"
                + "principal_id,scopes,expires_at) values(?,?,?,?,?,?::text[],now()+interval '1 hour')";
        try (Connection connection = connect(); PreparedStatement statement = connection.prepareStatement(sql)) {
            statement.setObject(1, UUID.randomUUID());
            statement.setString(2, hash(token));
            statement.setString(3, surface);
            statement.setString(4, "service");
            statement.setString(5, "sync-harness");
            statement.setArray(6, connection.createArrayOf("text", new String[] {"lkjmc.sync.read"}));
            statement.executeUpdate();
        }
        return token;
    }

    void revoke(String token) throws Exception {
        try (Connection connection = connect(); PreparedStatement statement = connection.prepareStatement(
                "update daemon_tokens set revoked_at=now() where token_hash=?")) {
            statement.setString(1, hash(token));
            statement.executeUpdate();
        }
    }

    void createPlayer(UUID id) throws Exception {
        try (Connection connection = connect()) {
            try (PreparedStatement identity = connection.prepareStatement(
                    "insert into player_identities(player_uuid,current_name,metadata) values(?,'Sync','{}')")) {
                identity.setObject(1, id);
                identity.executeUpdate();
            }
            try (PreparedStatement settings = connection.prepareStatement(
                    "insert into player_settings(player_uuid,language) values(?,'en')")) {
                settings.setObject(1, id);
                settings.executeUpdate();
            }
        }
    }

    private void seedDomainRevisions() throws Exception {
        String sql = "insert into sync_domain_revisions(domain,key,revision) values(?,?,1) "
                + "on conflict(domain,key) do nothing";
        try (Connection connection = connect(); PreparedStatement statement = connection.prepareStatement(sql)) {
            String[][] keys = {{"permissions", "service:sync-harness"}, {"claims", "survival"},
                    {"profiles", player + ":profile"}, {"presence", "hub"},
                    {"routing", "network"}, {"settings", player.toString()}};
            for (String[] key : keys) {
                statement.setString(1, key[0]); statement.setString(2, key[1]); statement.addBatch();
            }
            statement.executeBatch();
        }
    }

    void language(UUID id, String language) throws Exception {
        try (Connection connection = connect(); PreparedStatement statement = connection.prepareStatement(
                "update player_settings set language=? where player_uuid=?")) {
            statement.setString(1, language);
            statement.setObject(2, id);
            statement.executeUpdate();
        }
    }

    Connection lockCredentialRevision() throws Exception {
        Connection connection = connect();
        connection.setAutoCommit(false);
        try (Statement statement = connection.createStatement()) {
            statement.executeUpdate("update daemon_token_revision set revision=revision where singleton=true");
        }
        return connection;
    }

    private Connection connect() throws Exception {
        return DriverManager.getConnection(jdbcUrl);
    }

    private void migrate(Path migrations) throws Exception {
        List<Path> files;
        try (var paths = Files.list(migrations)) {
            files = paths.filter(path -> path.getFileName().toString().matches("[0-9]{3}-.*\\.sql"))
                    .sorted(Comparator.comparing(path -> path.getFileName().toString())).toList();
        }
        try (Connection connection = connect(); Statement statement = connection.createStatement()) {
            statement.setQueryTimeout((int) Duration.ofSeconds(30).toSeconds());
            for (Path file : files) {
                statement.execute(Files.readString(file));
            }
        }
    }

    private static String requireDatabase() {
        String value = System.getenv("LKJMC_STORE_TEST_DATABASE_URL");
        if (value == null || !(value.startsWith("postgres://") || value.startsWith("postgresql://"))) {
            throw new IllegalStateException("valid LKJMC_STORE_TEST_DATABASE_URL is required");
        }
        return value;
    }

    private static String jdbc(String value) {
        URI uri = URI.create(value);
        StringBuilder result = new StringBuilder("jdbc:postgresql://")
                .append(uri.getHost()).append(':').append(uri.getPort() < 0 ? 5432 : uri.getPort())
                .append(uri.getPath());
        if (uri.getRawQuery() != null) {
            result.append('?').append(uri.getRawQuery());
        }
        if (uri.getUserInfo() != null) {
            String[] credentials = uri.getUserInfo().split(":", 2);
            result.append(result.indexOf("?") < 0 ? '?' : '&').append("user=")
                    .append(URLEncoder.encode(credentials[0], StandardCharsets.UTF_8));
            if (credentials.length == 2) {
                result.append("&password=").append(URLEncoder.encode(credentials[1], StandardCharsets.UTF_8));
            }
        }
        return result.toString();
    }

    private static String parameter(String url, String name, String value) {
        String separator = url.contains("?") ? "&" : "?";
        return url + separator + name + "="
                + URLEncoder.encode(value, StandardCharsets.UTF_8).replace("+", "%20");
    }

    private static String hash(String value) throws Exception {
        return "sha256:" + HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256")
                .digest(value.getBytes(StandardCharsets.UTF_8)));
    }

    @Override
    public void close() throws Exception {
        try (Connection control = DriverManager.getConnection(jdbc(baseUrl));
             Statement statement = control.createStatement()) {
            statement.execute("drop schema if exists " + schema + " cascade");
        }
    }
}
