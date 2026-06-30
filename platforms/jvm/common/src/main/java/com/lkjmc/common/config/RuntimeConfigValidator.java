package com.lkjmc.common.config;

import com.lkjmc.common.daemon.DaemonHttpConfigStatus;
import java.net.URI;
import java.net.URISyntaxException;
import java.util.Map;
import java.util.Optional;
import java.util.function.Function;
import java.util.regex.Pattern;

public final class RuntimeConfigValidator {
    private static final Pattern INSTANCE = Pattern.compile("[a-z0-9][a-z0-9-]{0,62}");

    private RuntimeConfigValidator() {}

    public static RuntimeConfigValidation fromEnv() {
        return validate(System.getenv(), DaemonHttpConfigStatus::readTokenFileSafe);
    }

    public static RuntimeConfigValidation validate(
        Map<String, String> env,
        Function<String, Optional<String>> reader
    ) {
        var url = value(env, "LKJMC_DAEMON_HTTP_URL");
        if (url.isPresent() && !validUrl(url.get())) {
            return RuntimeConfigValidation.invalid("schema.invalid_url");
        }
        var daemon = DaemonHttpConfigStatus.from(env, reader);
        if (url.isPresent() && !daemon.configured()) {
            return RuntimeConfigValidation.invalid(daemon.code());
        }
        var instance = value(env, "LKJMC_INSTANCE_ID");
        if (instance.isPresent() && !INSTANCE.matcher(instance.get()).matches()) {
            return RuntimeConfigValidation.invalid("schema.invalid_instance_id");
        }
        var locale = value(env, "LKJMC_DEFAULT_LOCALE");
        if (locale.isPresent() && !locale.get().matches("[a-z]{2}(-[A-Z]{2})?")) {
            return RuntimeConfigValidation.invalid("schema.invalid_locale");
        }
        return RuntimeConfigValidation.ok();
    }

    private static boolean validUrl(String value) {
        try {
            var uri = new URI(value);
            return ("http".equals(uri.getScheme()) || "https".equals(uri.getScheme())) && uri.getHost() != null;
        } catch (URISyntaxException error) {
            return false;
        }
    }

    private static Optional<String> value(Map<String, String> env, String key) {
        return Optional.ofNullable(env.get(key)).map(String::trim).filter(value -> !value.isBlank());
    }
}
