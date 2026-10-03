package com.lkjsxc.lkjmc.proxy;

import com.google.gson.*;
import com.google.inject.Inject;
import com.lkjsxc.lkjmc.common.*;
import com.velocitypowered.api.command.SimpleCommand;
import com.velocitypowered.api.event.*;
import com.velocitypowered.api.event.connection.*;
import com.velocitypowered.api.event.player.*;
import com.velocitypowered.api.event.proxy.*;
import com.velocitypowered.api.plugin.*;
import com.velocitypowered.api.plugin.annotation.DataDirectory;
import com.velocitypowered.api.proxy.*;
import com.velocitypowered.api.proxy.messages.*;
import com.velocitypowered.api.proxy.server.*;
import java.net.*;
import java.nio.file.*;
import java.time.*;
import java.util.*;
import java.util.concurrent.*;
import java.util.concurrent.atomic.AtomicBoolean;
import net.kyori.adventure.text.Component;
import org.geysermc.floodgate.api.FloodgateApi;
import org.slf4j.Logger;

@Plugin(
    id = "lkjmc",
    name = "lkjmc",
    version = "0.1.0",
    dependencies = {@Dependency(id = "floodgate", optional = true)})
public final class LkjmcProxy {
  private final ProxyServer proxy;
  private final Logger log;
  private final Path directory;
  private final ExecutorService io = Executors.newVirtualThreadPerTaskExecutor();
  private final ScheduledExecutorService timers = Executors.newScheduledThreadPool(3);
  private final ConcurrentMap<UUID, Session> sessions = new ConcurrentHashMap<>();
  private final ConcurrentMap<UUID, String> languages = new ConcurrentHashMap<>();
  private final ConcurrentMap<UUID, JsonObject> servers = new ConcurrentHashMap<>();
  private final ConcurrentMap<UUID, Departure> departures = new ConcurrentHashMap<>();
  private final MinecraftChannelIdentifier channel =
      MinecraftChannelIdentifier.from(SignedBridge.CHANNEL);
  private CoreClient core;
  private SignedBridge bridge;
  private Journal receipts;
  private UUID lobby;
  private boolean offlineFixture;
  private volatile long projectionContact;
  private volatile boolean ready;

  @Inject
  public LkjmcProxy(ProxyServer proxy, Logger log, @DataDirectory Path directory) {
    this.proxy = proxy;
    this.log = log;
    this.directory = directory;
  }

  @Subscribe
  public EventTask initialize(ProxyInitializeEvent event) {
    return EventTask.async(
        () -> {
          try {
            JsonObject config =
                JsonParser.parseString(Files.readString(directory.resolve("config.json")))
                    .getAsJsonObject();
            core =
                new CoreClient(
                    URI.create(config.get("core_url").getAsString()),
                    Path.of(config.get("credential_file").getAsString()));
            bridge = new SignedBridge(Path.of(config.get("departure_key_file").getAsString()));
            receipts = new Journal(directory.resolve("job-receipts"));
            lobby = CoreClient.uuid(config, "lobby_id");
            refresh();
            if (!proxy.getConfiguration().isOnlineMode() && !offlineFixture)
              throw new IllegalStateException(
                  "Public proxies must authenticate Java accounts in online mode");
            if (proxy.getPluginManager().isLoaded("floodgate")) {
              var links = FloodgateApi.getInstance().getPlayerLink();
              if (!links.isEnabled() || !links.getName().equals("lkjmc") || links.isAllowLinking())
                throw new IllegalStateException(
                    "Floodgate must use the read-only lkjmc local linking extension");
            }
            proxy.getChannelRegistrar().register(channel);
            proxy
                .getCommandManager()
                .register(
                    proxy
                        .getCommandManager()
                        .metaBuilder("hub")
                        .aliases("lobby")
                        .plugin(this)
                        .build(),
                    (SimpleCommand)
                        invocation -> {
                          if (invocation.source() instanceof Player player)
                            submitJoin(player, lobby);
                        });
            proxy
                .getCommandManager()
                .register(
                    proxy.getCommandManager().metaBuilder("servers").plugin(this).build(),
                    (SimpleCommand)
                        invocation -> {
                          if (invocation.source() instanceof Player player)
                            io.execute(() -> listServers(player));
                        });
            proxy
                .getCommandManager()
                .register(
                    proxy.getCommandManager().metaBuilder("go").plugin(this).build(),
                    (SimpleCommand)
                        invocation -> {
                          if (invocation.source() instanceof Player player) {
                            try {
                              if (invocation.arguments().length != 1)
                                throw new IllegalArgumentException();
                              submitJoin(player, UUID.fromString(invocation.arguments()[0]));
                            } catch (IllegalArgumentException e) {
                              tell(player, "Use /servers to choose a destination.");
                            }
                          }
                        });
            ready = true;
            timers.scheduleWithFixedDelay(
                () -> {
                  try {
                    refresh();
                  } catch (Exception e) {
                    log.warn("Registry awaits retry: {}", e.getMessage());
                  }
                },
                2,
                5,
                TimeUnit.SECONDS);
            timers.scheduleWithFixedDelay(this::heartbeatAll, 1, 10, TimeUnit.SECONDS);
            timers.scheduleWithFixedDelay(this::poll, 1, 1, TimeUnit.SECONDS);
            log.info("lkjmc proxy ready; authenticated accounts start in lobby {}", lobby);
          } catch (Exception e) {
            log.error("Cannot initialize lkjmc proxy", e);
            proxy.shutdown(Component.text("The connection service could not be prepared."));
          }
        });
  }

  private void refresh() throws Exception {
    JsonObject projection = core.get("/internal/v1/projection");
    if (!projection.get("role").getAsString().equals("proxy"))
      throw new IllegalStateException("Wrong service role");
    offlineFixture =
        Boolean.getBoolean("lkjmc.testOffline") && projection.get("development").getAsBoolean();
    Set<UUID> active = new HashSet<>();
    for (JsonElement item : projection.getAsJsonArray("servers")) {
      JsonObject data = item.getAsJsonObject();
      UUID id = CoreClient.uuid(data, "id");
      active.add(id);
      servers.put(id, data);
      String address = CoreClient.string(data, "address", "");
      if (address.isEmpty()) continue;
      URI uri = URI.create("tcp://" + address);
      if (uri.getHost() == null
          || uri.getPort() < 1
          || uri.getPort() > 65535
          || uri.getUserInfo() != null
          || !uri.getPath().isEmpty())
        throw new IllegalArgumentException("Invalid backend address");
      ServerInfo info =
          new ServerInfo(name(id), new InetSocketAddress(uri.getHost(), uri.getPort()));
      Optional<RegisteredServer> old = proxy.getServer(name(id));
      if (old.isPresent() && !old.get().getServerInfo().getAddress().equals(info.getAddress()))
        proxy.unregisterServer(old.get().getServerInfo());
      if (proxy.getServer(name(id)).isEmpty()) proxy.registerServer(info);
    }
    for (UUID id : List.copyOf(servers.keySet()))
      if (!active.contains(id)) {
        servers.remove(id);
        proxy.getServer(name(id)).ifPresent(s -> proxy.unregisterServer(s.getServerInfo()));
      }
    Set<UUID> connected = new HashSet<>();
    for (JsonElement value : projection.getAsJsonArray("sessions")) {
      JsonObject account = value.getAsJsonObject();
      UUID nativeId = CoreClient.uuid(account, "native_uuid");
      connected.add(nativeId);
      languages.put(nativeId, CoreClient.string(account, "language", "en"));
    }
    languages.keySet().retainAll(connected);
    projectionContact = System.nanoTime();
  }

  private boolean privateFixture(Player player) {
    return offlineFixture
        && proxy.getBoundAddress() != null
        && proxy.getBoundAddress().getAddress().isLoopbackAddress()
        && player.getRemoteAddress().getAddress().isLoopbackAddress();
  }

  @Subscribe(priority = -32000)
  public EventTask login(LoginEvent event) {
    return EventTask.async(
        () -> {
          if (!event.getResult().isAllowed()) return;
          Player player = event.getPlayer();
          try {
            if (!ready || System.nanoTime() - projectionContact > TimeUnit.SECONDS.toNanos(30))
              throw new IllegalArgumentException(
                  "The connection service is starting. Wait a moment and reconnect.");
            String issuer = "java",
                subject = player.getUniqueId().toString(),
                display = player.getUsername();
            var floodgate =
                proxy.getPluginManager().isLoaded("floodgate")
                    ? FloodgateApi.getInstance().getPlayer(player.getUniqueId())
                    : null;
            if (floodgate != null) {
              issuer = "bedrock";
              subject = floodgate.getXuid();
              display = floodgate.getUsername();
            } else if (!player.isOnlineMode() && !privateFixture(player))
              throw new IllegalArgumentException("Your Java account must be authenticated.");
            JsonObject result =
                core.post(
                    "/internal/v1/game/connect",
                    CoreClient.object(
                        "issuer",
                        issuer,
                        "subject",
                        subject,
                        "display_name",
                        display,
                        "native_uuid",
                        player.getUniqueId(),
                        "session_id",
                        UUID.randomUUID()));
            languages.put(player.getUniqueId(), CoreClient.string(result, "language", "en"));
            Session session = new Session(player, result);
            if (sessions.putIfAbsent(player.getUniqueId(), session) != null) {
              core.post("/internal/v1/game/disconnect", session.body(null));
              throw new IllegalArgumentException("This account is already connected.");
            }
          } catch (Exception e) {
            event.setResult(ResultedEvent.ComponentResult.denied(Component.text(message(e))));
          }
        });
  }

  @Subscribe(priority = -32000)
  public void initial(PlayerChooseInitialServerEvent event) {
    RegisteredServer target = proxy.getServer(name(lobby)).orElse(null);
    if (!ready || target == null || session(event.getPlayer()) == null) {
      event
          .getPlayer()
          .disconnect(text(event.getPlayer(), "The lobby is unavailable. Please reconnect."));
      return;
    }
    event.setInitialServer(target);
  }

  @Subscribe(priority = -32000)
  public EventTask connecting(ServerPreConnectEvent event) {
    return EventTask.async(
        () -> {
          if (!event.getResult().isAllowed()) return;
          Session session = session(event.getPlayer());
          try {
            if (session == null)
              throw new IllegalArgumentException("Your identity has not been verified yet.");
            RegisteredServer target = event.getResult().getServer().orElseThrow();
            UUID id = id(target);
            if (session.serverId == null && !id.equals(lobby))
              throw new IllegalArgumentException("Connect to the lobby first.");
            boolean recovery = id.equals(lobby) && session.recoveryLobby.getAndSet(false);
            JsonObject request = session.body(id);
            request.addProperty("recovery", recovery);
            JsonObject route = core.post("/internal/v1/game/route", request);
            if (!route.get("ready").getAsBoolean())
              throw new IllegalArgumentException(
                  "The server is starting. Please wait in the lobby.");
            JsonObject previous = session.serverId == null ? null : servers.get(session.serverId);
            if (!recovery
                && previous != null
                && previous.get("kind").getAsString().equals("official")
                && !id.equals(session.serverId)) gate(session, id);
          } catch (Exception e) {
            event.setResult(ServerPreConnectEvent.ServerResult.denied());
            tell(event.getPlayer(), message(e));
          }
        });
  }

  private void gate(Session session, UUID destination) throws Exception {
    ServerConnection current = session.player.getCurrentServer().orElseThrow();
    UUID nonce = UUID.randomUUID();
    JsonObject body =
        CoreClient.object(
            "op",
            "prepare",
            "nonce",
            nonce,
            "native_uuid",
            session.player.getUniqueId(),
            "session_id",
            session.data.get("session_id").getAsString(),
            "destination",
            destination,
            "expires_at",
            System.currentTimeMillis() + 15_000);
    Departure waiting = new Departure(session, session.serverId, new CompletableFuture<>());
    departures.put(nonce, waiting);
    session.departure = body;
    try {
      if (!current.sendPluginMessage(channel, bridge.encode(body)))
        throw new IllegalStateException("The pre-transfer save check could not be sent.");
      JsonObject response = waiting.response.get(5, TimeUnit.SECONDS);
      if (!response.get("allowed").getAsBoolean())
        throw new IllegalArgumentException(
            CoreClient.string(response, "reason", "You cannot transfer right now."));
    } catch (Exception e) {
      release(session);
      throw e;
    } finally {
      departures.remove(nonce);
    }
  }

  @Subscribe
  public void bridge(PluginMessageEvent event) {
    if (!event.getIdentifier().equals(channel)) return;
    event.setResult(PluginMessageEvent.ForwardResult.handled());
    if (!(event.getSource() instanceof ServerConnection backend)
        || !(event.getTarget() instanceof Player player)) return;
    try {
      JsonObject value = bridge.decode(event.getData());
      Departure waiting = departures.get(CoreClient.uuid(value, "nonce"));
      if (waiting != null
          && waiting.session.player == player
          && id(backend.getServer()).equals(waiting.source)
          && CoreClient.uuid(value, "native_uuid").equals(player.getUniqueId())
          && value.get("op").getAsString().equals("result")) waiting.response.complete(value);
    } catch (Exception e) {
      log.warn("Rejected departure response: {}", e.getMessage());
    }
  }

  @Subscribe
  public void backendFailure(KickedFromServerEvent event) {
    Session session = session(event.getPlayer());
    if (session == null) return;
    try {
      if (id(event.getServer()).equals(lobby)) return;
      RegisteredServer target = proxy.getServer(name(lobby)).orElse(null);
      if (target == null) return;
      session.recoveryLobby.set(true);
      event.setResult(
          KickedFromServerEvent.RedirectPlayer.create(
              target,
              text(event.getPlayer(), "Your server connection ended. Returning to the lobby.")));
    } catch (Exception e) {
      log.warn("Cannot recover backend connection: {}", e.getMessage());
    }
  }

  @Subscribe
  public EventTask connected(ServerConnectedEvent event) {
    return EventTask.async(
        () -> {
          Session session = session(event.getPlayer());
          if (session == null) {
            event
                .getPlayer()
                .disconnect(text(event.getPlayer(), "Connection verification was lost."));
            return;
          }
          try {
            synchronized (session) {
              session.serverId = id(event.getServer());
              heartbeat(session);
              session.departure = null;
            }
          } catch (Exception e) {
            event.getPlayer().disconnect(text(event.getPlayer(), message(e)));
          }
        });
  }

  @Subscribe
  public EventTask disconnected(DisconnectEvent event) {
    Session session = session(event.getPlayer());
    if (session == null) return null;
    sessions.remove(event.getPlayer().getUniqueId(), session);
    return EventTask.async(
        () -> {
          try {
            core.post("/internal/v1/game/disconnect", session.body(null));
          } catch (Exception e) {
            log.warn("Disconnect will expire by session lease: {}", e.getMessage());
          }
        });
  }

  private void heartbeatAll() {
    for (Session session : sessions.values())
      if (session.heartbeat.compareAndSet(false, true))
        io.execute(
            () -> {
              try {
                synchronized (session) {
                  heartbeat(session);
                }
              } catch (Exception e) {
                if (e instanceof CoreClient.CoreFailure f && (f.status == 403 || f.status == 409)
                    || System.nanoTime() - session.lastGood > TimeUnit.SECONDS.toNanos(30))
                  session.player.disconnect(
                      text(
                          session.player,
                          "Your identity could not be refreshed. Reconnect after saving."));
              } finally {
                session.heartbeat.set(false);
              }
            });
  }

  private void heartbeat(Session session) throws Exception {
    core.post("/internal/v1/game/heartbeat", session.body(session.serverId));
    session.lastGood = System.nanoTime();
  }

  private Session session(Player player) {
    Session s = sessions.get(player.getUniqueId());
    return s != null && s.player == player ? s : null;
  }

  private Session account(UUID account) {
    return sessions.values().stream()
        .filter(s -> CoreClient.uuid(s.data, "account_id").equals(account))
        .findFirst()
        .orElse(null);
  }

  private void submitJoin(Player player, UUID target) {
    io.execute(
        () -> {
          try {
            Session s = Objects.requireNonNull(session(player));
            JsonObject result =
                core.command(
                    s.data,
                    CoreClient.object("type", "server_join", "id", target),
                    UUID.randomUUID());
            tell(player, "Transfer requested. Checking server startup and saves.");
          } catch (Exception e) {
            tell(player, message(e));
          }
        });
  }

  private void listServers(Player player) {
    try {
      Session s = Objects.requireNonNull(session(player));
      JsonObject request = s.body(null);
      request.remove("server_id");
      request.addProperty("view", "play");
      request.add("query", new JsonObject());
      JsonObject view = core.post("/internal/v1/game/view", request);
      for (JsonElement element : view.getAsJsonArray("servers")) {
        JsonObject server = element.getAsJsonObject();
        player.sendMessage(
            Component.text(server.get("name").getAsString())
                .append(text(player, "  [Join]"))
                .clickEvent(
                    net.kyori.adventure.text.event.ClickEvent.runCommand(
                        "/go " + server.get("id").getAsString())));
      }
    } catch (Exception e) {
      tell(player, message(e));
    }
  }

  private void poll() {
    if (!ready) return;
    JsonObject job = null;
    try {
      JsonElement value = core.post("/internal/v1/poll", new JsonObject()).get("job");
      if (value.isJsonNull()) return;
      job = value.getAsJsonObject();
      UUID jobId = CoreClient.uuid(job, "id");
      Optional<JsonObject> receipt = receipts.read(jobId);
      JsonObject result;
      if (receipt.isPresent()) result = receipt.get().getAsJsonObject("result");
      else {
        JsonObject payload = job.getAsJsonObject("payload");
        switch (job.get("kind").getAsString()) {
          case "player.kick" -> {
            Session s = account(CoreClient.uuid(payload, "account_id"));
            if (s != null)
              s.player.disconnect(
                  text(
                      s.player,
                      CoreClient.string(payload, "reason", "Disconnected by an administrator.")));
            result = CoreClient.object("effect", "committed");
          }
          case "player.join" -> {
            Session s = account(CoreClient.uuid(job, "actor"));
            if (s == null)
              throw new IllegalArgumentException(
                  "The transfer ended because you disconnected. Reconnect and choose your"
                      + " destination again.");
            UUID target = CoreClient.uuid(job, "server_id");
            if (!target.equals(s.serverId)) {
              JsonObject route = core.post("/internal/v1/game/route", s.body(target));
              if (!route.get("ready").getAsBoolean()) throw new Waiting("Starting the server.");
              RegisteredServer backend =
                  proxy
                      .getServer(name(target))
                      .orElseThrow(() -> new Waiting("Registering the destination."));
              ConnectionRequestBuilder.Result connection;
              try {
                connection =
                    s.player.createConnectionRequest(backend).connect().get(10, TimeUnit.SECONDS);
              } catch (TimeoutException e) {
                release(s);
                s.player.disconnect(
                    text(
                        s.player,
                        "The destination did not confirm in time. Reconnect through the lobby."));
                throw new IllegalArgumentException("Transfer confirmation timed out.");
              }
              if (!connection.isSuccessful()) {
                release(s);
                throw new IllegalArgumentException(
                    "The transfer failed. Check access, PvP cooldown, and server status.");
              }
              // Observe the actual backend, not merely the request being accepted.
              if (s.player.getCurrentServer().isEmpty()
                  || !id(s.player.getCurrentServer().get().getServer()).equals(target))
                throw new Waiting("Waiting for the destination to confirm.");
            }
            result = CoreClient.object("effect", "committed", "server_id", target);
          }
          default -> throw new IllegalArgumentException("This connection action is not supported.");
        }
      }
      receipts.write(jobId, CoreClient.object("id", jobId, "phase", "committed", "result", result));
      core.ack(job, "succeeded", result, null, null);
    } catch (Waiting e) {
      if (job != null)
        try {
          if (Instant.parse(job.get("created_at").getAsString())
              .isBefore(Instant.now().minusSeconds(900)))
            core.ack(
                job,
                "failed",
                CoreClient.object("effect", "none"),
                null,
                "Startup confirmation is taking longer than expected. Check the server’s"
                    + " progress.");
          else core.ack(job, "waiting", null, CoreClient.object("message", e.getMessage()), null);
        } catch (Exception failure) {
          log.warn("Connection wait: {}", failure.getMessage());
        }
    } catch (IllegalArgumentException | CoreClient.CoreFailure e) {
      if (job != null)
        try {
          if (e instanceof CoreClient.CoreFailure f && f.status >= 500) throw e;
          core.ack(job, "failed", CoreClient.object("effect", "none"), null, message(e));
        } catch (Exception failure) {
          log.warn("Connection result awaits retry: {}", failure.getMessage());
        }
    } catch (Exception e) {
      log.warn("Connection job awaits retry: {}", e.getMessage());
    }
  }

  private void release(Session session) {
    JsonObject body = session.departure;
    session.departure = null;
    if (body == null) return;
    try {
      body = body.deepCopy();
      body.addProperty("op", "release");
      if (session.player.getCurrentServer().isPresent())
        session.player.getCurrentServer().get().sendPluginMessage(channel, bridge.encode(body));
    } catch (Exception e) {
      log.warn("Departure will expire: {}", e.getMessage());
    }
  }

  private static String name(UUID id) {
    return "lkjmc-" + id;
  }

  private UUID id(RegisteredServer server) {
    String name = server.getServerInfo().getName();
    if (!name.startsWith("lkjmc-"))
      throw new IllegalArgumentException("This destination is not registered with lkjmc.");
    UUID id = UUID.fromString(name.substring(6));
    if (!servers.containsKey(id))
      throw new IllegalArgumentException("The destination registration was not found.");
    return id;
  }

  private Component text(Player player, String message) {
    return Component.text(
        Messages.error(languages.getOrDefault(player.getUniqueId(), "en"), message));
  }

  private void tell(Player player, String message) {
    player.sendMessage(Component.text("[lkjmc] ").append(text(player, message)));
  }

  private static String message(Exception e) {
    return e instanceof CoreClient.CoreFailure || e instanceof IllegalArgumentException
        ? Objects.toString(e.getMessage(), "The action could not be verified.")
        : "The connection service did not confirm. Wait a moment and try again.";
  }

  @Subscribe
  public void shutdown(ProxyShutdownEvent event) {
    ready = false;
    timers.shutdownNow();
    io.shutdown();
    if (core != null) core.close();
  }

  private static final class Session {
    final Player player;
    final JsonObject data;
    final AtomicBoolean heartbeat = new AtomicBoolean();
    volatile UUID serverId;
    volatile long lastGood = System.nanoTime();
    volatile JsonObject departure;
    final AtomicBoolean recoveryLobby = new AtomicBoolean();

    Session(Player player, JsonObject data) {
      this.player = player;
      this.data = data;
    }

    JsonObject body(UUID target) {
      return CoreClient.object(
          "account_id",
          data.get("account_id").getAsString(),
          "session_id",
          data.get("session_id").getAsString(),
          "server_id",
          target);
    }
  }

  private record Departure(Session session, UUID source, CompletableFuture<JsonObject> response) {}

  private static final class Waiting extends Exception {
    Waiting(String message) {
      super(message);
    }
  }
}
