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
  private final Set<UUID> working = ConcurrentHashMap.newKeySet();
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
                    proxy.getCommandManager().metaBuilder("worlds").plugin(this).build(),
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
                              if (invocation.arguments()[0].equalsIgnoreCase("cancel"))
                                cancelJoin(player);
                              else submitJoin(player, UUID.fromString(invocation.arguments()[0]));
                            } catch (IllegalArgumentException e) {
                              tell(player, "Use /worlds to choose a destination.");
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
                  com.lkjsxc.lkjmc.common.SystemMessage.of("text.the_connection_service_is_starting_wait_a_moment_and_reconnect").toString());
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
              throw new IllegalArgumentException(com.lkjsxc.lkjmc.common.SystemMessage.of("text.your_java_account_must_be_authenticated").toString());
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
              throw new IllegalArgumentException(com.lkjsxc.lkjmc.common.SystemMessage.of("text.this_account_is_already_connected").toString());
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
              throw new IllegalArgumentException(com.lkjsxc.lkjmc.common.SystemMessage.of("text.your_identity_has_not_been_verified_yet").toString());
            RegisteredServer target = event.getResult().getServer().orElseThrow();
            UUID id = id(target);
            if (session.serverId == null && !id.equals(lobby))
              throw new IllegalArgumentException(com.lkjsxc.lkjmc.common.SystemMessage.of("text.connect_to_the_lobby_first").toString());
            boolean recovery = id.equals(lobby) && session.recoveryLobby.getAndSet(false);
            JsonObject request = session.body(id);
            request.addProperty("recovery", recovery);
            JoinAttempt attempt = session.join;
            if (attempt != null && id.equals(attempt.target) && !recovery) {
              if (attempt.expired || session(event.getPlayer()) != session)
                throw new IllegalArgumentException(
                    com.lkjsxc.lkjmc.common.SystemMessage.of("text.this_travel_request_has_ended_choose_a_destination_again").toString());
              request = joinBody(attempt.job, "connect");
            }
            JsonObject route = core.post("/internal/v1/game/route", request);
            if (!route.get("ready").getAsBoolean())
              throw new IllegalArgumentException(
                  com.lkjsxc.lkjmc.common.SystemMessage.of("text.the_server_is_starting_please_wait_in_the_lobby").toString());
            JsonObject previous = session.serverId == null ? null : servers.get(session.serverId);
            if (!recovery
                && previous != null
                && previous.get("kind").getAsString().equals("official")
                && !id.equals(session.serverId)) gate(session, id);
            if (attempt != null && id.equals(attempt.target) && !recovery) {
              // The save handshake is asynchronous. Revalidate the same request after it.
              if (attempt.expired
                  || session(event.getPlayer()) != session
                  || !CoreClient.string(joinRoute(session, attempt.job, "connect"), "state", "")
                      .equals("leased"))
                throw new IllegalArgumentException(
                    com.lkjsxc.lkjmc.common.SystemMessage.of("text.this_travel_request_has_ended_choose_a_destination_again").toString());
            }
          } catch (Exception e) {
            release(session);
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
        throw new IllegalStateException(com.lkjsxc.lkjmc.common.SystemMessage.of("text.the_pre_transfer_save_check_could_not_be_sent").toString());
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
            if (session.player.getCurrentServer().isPresent()
                && !session.player.getCurrentServer().get().getServer().equals(event.getServer()))
              return;
            JoinAttempt attempt = session.join;
            UUID destination = id(event.getServer());
            synchronized (session.abandoned) {
              session.abandoned.values().removeIf(expiry -> expiry < System.nanoTime());
            }
            if (!destination.equals(lobby)
                && (session.abandoned.containsKey(destination)
                    || attempt != null && attempt.expired && destination.equals(attempt.target))) {
              synchronized (session) {
                session.serverId = destination;
                heartbeat(session); // Late arrival is still an observed physical effect.
              }
              if (attempt != null && destination.equals(attempt.target)) {
                try {
                  joinRoute(session, attempt.job, "complete");
                } catch (Exception ignored) {
                  /* Poll reconciles with its current lease. */
                }
              }
              release(session);
              recoverLobby(session);
              return;
            }
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
                confirmArrivals(session);
                JoinAttempt attempt = session.join;
                if (attempt != null
                    && attempt.expired
                    && attempt.connection != null
                    && attempt.connection.isDone()
                    && working.add(CoreClient.uuid(attempt.job, "id"))) {
                  try {
                    processJoin(attempt.job);
                  } catch (Exception ignored) {
                    /* A rotated lease is retried by poll(). */
                  } finally {
                    working.remove(CoreClient.uuid(attempt.job, "id"));
                  }
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
    JsonObject response = core.post("/internal/v1/game/heartbeat", session.body(session.serverId));
    if (response.has("join_results"))
      for (JsonElement value : response.getAsJsonArray("join_results")) {
        JsonObject job = value.getAsJsonObject();
        notice(
            session,
            job,
            "failed",
            "Travel to {0} failed: {1}",
            CoreClient.string(job.getAsJsonObject("payload"), "server_name", "server"),
            CoreClient.string(job, "error", "Choose the destination again."));
      }
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
            if (session(player) == s)
              tell(
                  player,
                  Messages.text(
                      languages.getOrDefault(player.getUniqueId(), "en"),
                      "text.travel_to_0_is_queued_stay_connected_progress_will_appe_d7984d2d4f",
                      CoreClient.string(
                          result.getAsJsonObject("result"), "server_name", target.toString())));
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
      if (session(player) == s) tell(player, "Choose a world. Sleeping worlds wake when you join.");
      for (JsonElement element : view.getAsJsonArray("servers")) {
        JsonObject server = element.getAsJsonObject();
        JsonObject capabilities = server.getAsJsonObject("capabilities");
        boolean compatible =
            capabilities != null
                && capabilities.has("proxy_join")
                && capabilities.get("proxy_join").getAsBoolean()
                && (!CoreClient.string(s.data, "client", "java").equals("bedrock")
                    || capabilities.has("bedrock") && capabilities.get("bedrock").getAsBoolean());
        Component label = Component.text(server.get("name").getAsString());
        boolean available =
            compatible
                && !CoreClient.string(server, "maintenance", "false").equals("true")
                && Set.of("running", "stopped")
                    .contains(CoreClient.string(server, "observed", "unknown"));
        if (available)
          label =
              label
                  .append(text(player, "  [Join]"))
                  .clickEvent(
                      net.kyori.adventure.text.event.ClickEvent.runCommand(
                          "/go " + server.get("id").getAsString()));
        else
          label =
              label.append(
                  text(
                      player,
                      compatible
                          ? "  [Preparing or unavailable]"
                          : "  [Lobby joining unavailable]"));
        if (session(player) == s) player.sendMessage(label);
      }
    } catch (Exception e) {
      tell(player, message(e));
    }
  }

  private void cancelJoin(Player player) {
    io.execute(
        () -> {
          Session s = session(player);
          if (s == null) return;
          try {
            JsonObject body = s.body(null);
            body.addProperty("join_phase", "cancel");
            JsonObject result = core.post("/internal/v1/game/route", body);
            if (session(player) == s)
              tell(
                  player,
                  result.get("cancelled").getAsInt() > 0
                      ? "Travel cancelled. Choose a server when you are ready."
                      : "There is no waiting travel request to cancel.");
          } catch (Exception e) {
            if (session(player) == s) tell(player, message(e));
          }
        });
  }

  private void poll() {
    if (!ready || working.size() >= 16) return;
    try {
      JsonElement value = core.post("/internal/v1/poll", new JsonObject()).get("job");
      if (value.isJsonNull()) return;
      JsonObject job = value.getAsJsonObject();
      UUID id = CoreClient.uuid(job, "id");
      if (!working.add(id)) return;
      io.execute(
          () -> {
            try {
              if (job.get("kind").getAsString().equals("player.join")) processJoin(job);
              else {
                Optional<JsonObject> receipt = receipts.read(id);
                JsonObject result;
                if (receipt.isPresent()) result = receipt.get().getAsJsonObject("result");
                else if (job.get("kind").getAsString().equals("player.kick")) {
                  JsonObject payload = job.getAsJsonObject("payload");
                  Session s = account(CoreClient.uuid(payload, "account_id"));
                  if (s != null)
                    s.player.disconnect(
                        text(
                            s.player,
                            CoreClient.string(
                                payload, "reason", "Disconnected by an administrator.")));
                  result = CoreClient.object("effect", "committed");
                } else
                  throw new IllegalArgumentException(com.lkjsxc.lkjmc.common.SystemMessage.of("text.this_connection_action_is_not_supported").toString());
                receipts.write(
                    id, CoreClient.object("id", id, "phase", "committed", "result", result));
                core.ack(job, "succeeded", result, null, null);
              }
            } catch (Exception e) {
              log.warn("Connection result awaits retry: {}", e.getMessage());
              if (job.get("kind").getAsString().equals("player.join")) {
                Session s = account(CoreClient.uuid(job, "actor"));
                JsonObject payload = job.getAsJsonObject("payload");
                if (s != null
                    && payload.has("session_id")
                    && CoreClient.uuid(s.data, "session_id")
                        .equals(CoreClient.uuid(payload, "session_id")))
                  notice(
                      s,
                      job,
                      "contact",
                      "Travel to {0} could not be confirmed by the connection service. Stay here;"
                          + " use /hub or try again when it recovers.",
                      CoreClient.string(
                          payload, "server_name", job.get("server_id").getAsString()));
              }
            } finally {
              working.remove(id);
            }
          });
    } catch (Exception e) {
      log.warn("Connection poll awaits retry: {}", e.getMessage());
    }
  }

  private JsonObject joinBody(JsonObject job, String phase) {
    JsonObject payload = job.getAsJsonObject("payload");
    JsonObject body =
        CoreClient.object(
            "account_id",
            job.get("actor"),
            "session_id",
            payload.get("session_id"),
            "server_id",
            job.get("server_id"),
            "join_job_id",
            job.get("id"),
            "lease_token",
            job.get("lease_token"),
            "join_phase",
            phase);
    return body;
  }

  private JsonObject joinRoute(Session s, JsonObject job, String phase) throws Exception {
    if (session(s.player) != s
        || !CoreClient.uuid(s.data, "session_id")
            .equals(CoreClient.uuid(job.getAsJsonObject("payload"), "session_id")))
      throw new IllegalArgumentException(
          com.lkjsxc.lkjmc.common.SystemMessage.of("text.your_game_session_has_changed_choose_the_destination_again").toString());
    return core.post("/internal/v1/game/route", joinBody(job, phase));
  }

  private void notice(Session s, JsonObject job, String phase, String key, Object... values) {
    if (session(s.player) != s || !s.player.isActive()) return;
    UUID id = CoreClient.uuid(job, "id");
    long now = System.nanoTime();
    synchronized (s.notices) {
      Notice old = s.notices.get(id);
      if (old != null
          && old.phase.equals(phase)
          && (Set.of("succeeded", "failed", "cancelled").contains(phase)
              || now - old.at < TimeUnit.SECONDS.toNanos(20))) return;
      if (s.notices.size() >= 64 && !s.notices.containsKey(id))
        s.notices.remove(s.notices.keySet().iterator().next());
      s.notices.put(id, new Notice(phase, now));
    }
    tell(
        s.player, Messages.text(languages.getOrDefault(s.player.getUniqueId(), "en"), key, values));
  }

  private void processJoin(JsonObject job) throws Exception {
    JsonObject payload = job.getAsJsonObject("payload");
    Session s = account(CoreClient.uuid(job, "actor"));
    // Old, unbound jobs are deliberately rejected. Never adopt a reconnected player.
    if (!payload.has("session_id")) {
      core.ack(
          job,
          "failed",
          CoreClient.object("effect", "none"),
          null,
          "This old travel request has no game session. Choose a destination again.");
      return;
    }
    if (s == null
        || !CoreClient.uuid(s.data, "session_id").equals(CoreClient.uuid(payload, "session_id"))) {
      // This proxy no longer owns the original Player connection; it cannot
      // perform that session's old connect, even if its Core lease remains.
      JsonObject body = joinBody(job, "fence");
      body.addProperty(
          "error",
          "The original game session ended. Choose the destination again after reconnecting.");
      core.post("/internal/v1/game/route", body);
      return;
    }
    UUID target = CoreClient.uuid(job, "server_id");
    String targetName = CoreClient.string(payload, "server_name", target.toString());
    JoinAttempt attempt = null;
    try {
      JoinAttempt previousAttempt = s.join;
      if (previousAttempt != null && previousAttempt.expired) {
        if (previousAttempt.connection != null && !previousAttempt.connection.isDone()) {
          // A timeout is not a fence. Keep the effect boundary until Velocity's
          // bounded connection/read timeout has actually ended the network attempt.
          core.ack(job, "leased", null, CoreClient.object("phase", "connecting"), null);
          return;
        }
        if (CoreClient.uuid(previousAttempt.job, "id").equals(CoreClient.uuid(job, "id"))) {
          synchronized (s) {
            s.serverId = s.player.getCurrentServer().map(c -> id(c.getServer())).orElse(null);
            heartbeat(s);
          }
          JsonObject fenced = joinRoute(s, job, "fence");
          s.join = null;
          release(s);
          notice(
              s,
              job,
              CoreClient.string(fenced, "state", "failed"),
              CoreClient.string(fenced, "state", "failed").equals("succeeded")
                  ? "Your arrival at {0} was confirmed."
                  : "Travel to {0} failed: {1}",
              targetName,
              "The destination did not confirm in time. Choose a server again, or use /hub.");
          return;
        }
      }
      synchronized (s.confirmations) {
        if (s.confirmations.size() >= 32
            && !s.confirmations.containsKey(CoreClient.uuid(job, "id")))
          throw new IllegalArgumentException(
              "Previous arrivals are still being confirmed. Try again when the connection service"
                  + " recovers.");
      }
      // Recover a real arrival before consulting startup freshness. A lost response
      // must never turn an already observed destination into a waiting startup job.
      if (s.player.getCurrentServer().isPresent()
          && id(s.player.getCurrentServer().get().getServer()).equals(target)
          && (s.join == null || !s.join.expired)) {
        attempt = new JoinAttempt(job, target);
        s.confirmations.put(CoreClient.uuid(job, "id"), job);
        synchronized (s) {
          s.serverId = target;
          heartbeat(s);
        }
        JsonObject completed = joinRoute(s, job, "complete");
        if (CoreClient.string(completed, "state", "").equals("succeeded")) {
          notice(s, job, "succeeded", "Arrived at {0}.", targetName);
          s.confirmations.remove(CoreClient.uuid(job, "id"));
          if (s.join != null
              && CoreClient.uuid(s.join.job, "id").equals(CoreClient.uuid(job, "id")))
            s.join = null;
        }
        s.confirmations.remove(CoreClient.uuid(job, "id"));
        return;
      }
      JsonObject route = joinRoute(s, job, "check");
      String state = CoreClient.string(route, "state", "");
      if (!state.equals("leased")) {
        if (s.join != null && CoreClient.uuid(s.join.job, "id").equals(CoreClient.uuid(job, "id")))
          s.join = null;
        if (state.equals("succeeded")
            && s.player.getCurrentServer().isPresent()
            && id(s.player.getCurrentServer().get().getServer()).equals(target))
          notice(s, job, state, "Arrived at {0}.", targetName);
        if (state.equals("failed"))
          notice(
              s,
              job,
              state,
              "Travel to {0} failed: {1}",
              targetName,
              CoreClient.string(route, "error", "Choose the destination again."));
        if (state.equals("cancelled"))
          notice(
              s,
              job,
              state,
              "Travel to {0} was cancelled. Choose a server when you are ready.",
              targetName);
        return;
      }
      if (payload.has("superseded") && s.replacements.put(CoreClient.uuid(job, "id"), true) == null)
        for (JsonElement old : payload.getAsJsonArray("superseded"))
          if (!old.isJsonNull())
            notice(
                s,
                job,
                "replaced",
                "Travel to {0} was cancelled because another destination was chosen.",
                old.getAsString());
      if (!route.get("ready").getAsBoolean()) {
        JsonObject startup =
            route.has("startup") && !route.get("startup").isJsonNull()
                ? route.getAsJsonObject("startup")
                : new JsonObject();
        String observed = CoreClient.string(route, "observed", "stopped");
        String phase =
            observed.equals("starting") || CoreClient.string(startup, "state", "").equals("leased")
                ? "preparing"
                : "waking";
        if (CoreClient.string(startup, "state", "").equals("failed"))
          throw new IllegalArgumentException(
              "Server startup failed. Check the server status or ask its administrator, then try"
                  + " again.");
        notice(
            s,
            job,
            phase,
            phase.equals("waking")
                ? "Waking {0}. Stay connected; cancel with /go cancel."
                : "Preparing {0}. Waiting for verified readiness; cancel with /go cancel.",
            targetName);
        core.ack(
            job,
            "waiting",
            null,
            CoreClient.object("phase", phase, "server_name", targetName),
            null);
        return;
      }
      if (session(s.player) != s)
        throw new IllegalArgumentException(
            com.lkjsxc.lkjmc.common.SystemMessage.of("text.your_game_session_has_changed_choose_the_destination_again").toString());
      RegisteredServer backend =
          proxy
              .getServer(name(target))
              .orElseThrow(() -> new Waiting("Registering the destination."));
      synchronized (s) {
        if (s.join != null
            && !s.join.expired
            && !CoreClient.uuid(s.join.job, "id").equals(CoreClient.uuid(job, "id")))
          throw new Waiting("Waiting for the previous connection to finish.");
        route = joinRoute(s, job, "connect");
        if (!CoreClient.string(route, "state", "").equals("leased")) return;
        attempt = new JoinAttempt(job, target);
        s.abandoned.remove(target); // A new explicit request can intentionally choose it again.
        s.join = attempt;
      }
      notice(s, job, "connecting", "{0} is ready. Saving and connecting now.", targetName);
      if (s.player.getCurrentServer().isEmpty()
          || !id(s.player.getCurrentServer().get().getServer()).equals(target)) {
        CompletableFuture<ConnectionRequestBuilder.Result> connection =
            s.player.createConnectionRequest(backend).connect();
        attempt.connection = connection;
        try {
          ConnectionRequestBuilder.Result result = connection.get(20, TimeUnit.SECONDS);
          if (!result.isSuccessful())
            throw new IllegalArgumentException(
                com.lkjsxc.lkjmc.common.SystemMessage.of("text.the_transfer_failed_check_access_pvp_cooldown_and_server_status").toString());
        } catch (TimeoutException e) {
          attempt.expired = true;
          s.abandoned.put(target, System.nanoTime() + TimeUnit.SECONDS.toNanos(60));
          // Do not cancel the future: cancellation only hides completion and
          // cannot prove the underlying backend connection has stopped.
          throw new IllegalArgumentException(
              "The destination did not confirm in time. You can stay here and choose a server"
                  + " again, or use /hub.");
        }
      }
      // The connection future and the event can complete in either order. Persist our
      // own observation before settling, instead of declaring success on acceptance.
      if (session(s.player) != s
          || s.player.getCurrentServer().isEmpty()
          || !id(s.player.getCurrentServer().get().getServer()).equals(target))
        throw new IllegalArgumentException(
            com.lkjsxc.lkjmc.common.SystemMessage.of("text.the_destination_was_not_observed_stay_here_and_choose_a_d55df68f19").toString());
      s.confirmations.put(CoreClient.uuid(job, "id"), job);
      synchronized (s) {
        s.serverId = target;
        heartbeat(s);
      }
      route = joinRoute(s, job, "complete");
      if (CoreClient.string(route, "state", "").equals("succeeded"))
        notice(s, job, "succeeded", "Arrived at {0}.", targetName);
      s.confirmations.remove(CoreClient.uuid(job, "id"));
      s.join = null;
    } catch (Waiting e) {
      notice(
          s,
          job,
          "registering",
          "{0} is preparing its connection. Stay connected; cancel with /go cancel.",
          targetName);
      core.ack(
          job,
          "waiting",
          null,
          CoreClient.object("phase", "registering", "server_name", targetName),
          null);
    } catch (Exception e) {
      // Arrival is an effect even if Core was temporarily unreachable. Keep this
      // job recoverable and retry its observation instead of claiming effect:none.
      if (session(s.player) == s
          && s.player.getCurrentServer().isPresent()
          && id(s.player.getCurrentServer().get().getServer()).equals(target)
          && !(e instanceof TimeoutException)
          && attempt != null
          && !attempt.expired) {
        s.join = null; // Physical connection finished; Core still fences an uncommitted job.
        notice(
            s,
            job,
            "confirming",
            "Connected to {0}. Confirming the arrival record; stay connected.",
            targetName);
        throw e;
      }
      s.confirmations.remove(CoreClient.uuid(job, "id"));
      if (attempt != null) attempt.expired = true;
      release(s);
      String error = message(e);
      // A lost API response may follow a real arrival. Recheck Core's terminal state;
      // never replace an observed success with a failure or replay a connection.
      if (attempt != null && attempt.connection != null && !attempt.connection.isDone()) {
        notice(s, job, "fencing", "Travel to {0} failed: {1}", targetName, error);
        // Keep the old attempt until its natural completion; timer retries with
        // the same lease and poll retries after lease rotation both observe it.
        return;
      }
      synchronized (s) {
        s.serverId = s.player.getCurrentServer().map(c -> id(c.getServer())).orElse(null);
        heartbeat(s);
      }
      JsonObject body = joinBody(job, attempt == null ? "fail" : "fence");
      body.addProperty("error", error);
      JsonObject result = core.post("/internal/v1/game/route", body);
      String state = CoreClient.string(result, "state", "failed");
      if (attempt != null) s.join = null;
      if (state.equals("succeeded")) notice(s, job, state, "Arrived at {0}.", targetName);
      else if (state.equals("cancelled"))
        notice(
            s,
            job,
            state,
            "Travel to {0} was cancelled. Choose a server when you are ready.",
            targetName);
      else
        notice(
            s,
            job,
            state,
            "Travel to {0} failed: {1}",
            targetName,
            Messages.error(languages.getOrDefault(s.player.getUniqueId(), "en"), error));
      if (attempt != null
          && attempt.expired
          && s.player.getCurrentServer().isPresent()
          && id(s.player.getCurrentServer().get().getServer()).equals(target)
          && !state.equals("succeeded")) recoverLobby(s);
    }
  }

  private void confirmArrivals(Session s) {
    if (session(s.player) != s) return;
    List<JsonObject> pending;
    synchronized (s.confirmations) {
      pending = List.copyOf(s.confirmations.values());
    }
    for (JsonObject job : pending) {
      try {
        String state = CoreClient.string(joinRoute(s, job, "complete"), "state", "");
        if (Set.of("succeeded", "failed", "cancelled").contains(state)) {
          s.confirmations.remove(CoreClient.uuid(job, "id"), job);
          String targetName =
              CoreClient.string(
                  job.getAsJsonObject("payload"),
                  "server_name",
                  job.get("server_id").getAsString());
          if (state.equals("succeeded")) {
            boolean stillHere =
                s.player.getCurrentServer().isPresent()
                    && id(s.player.getCurrentServer().get().getServer())
                        .equals(CoreClient.uuid(job, "server_id"));
            notice(
                s,
                job,
                state,
                stillHere ? "Arrived at {0}." : "Your arrival at {0} was confirmed.",
                targetName);
          }
        }
      } catch (Exception e) {
        log.warn("Arrival record awaits retry: {}", e.getMessage());
      }
    }
  }

  private void recoverLobby(Session s) {
    if (session(s.player) != s) return;
    proxy
        .getServer(name(lobby))
        .ifPresent(
            backend -> {
              s.recoveryLobby.set(true);
              s.player.createConnectionRequest(backend).connect();
            });
  }

  private void release(Session session) {
    if (session == null) return;
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
      throw new IllegalArgumentException(com.lkjsxc.lkjmc.common.SystemMessage.of("text.this_destination_is_not_registered_with_lkjmc").toString());
    UUID id = UUID.fromString(name.substring(6));
    if (!servers.containsKey(id))
      throw new IllegalArgumentException(com.lkjsxc.lkjmc.common.SystemMessage.of("text.the_destination_registration_was_not_found").toString());
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

  private static final class BoundedMap<K, V> extends LinkedHashMap<K, V> {
    private final int limit;

    BoundedMap(int limit) {
      this.limit = limit;
    }

    @Override
    protected boolean removeEldestEntry(Map.Entry<K, V> entry) {
      return size() > limit;
    }
  }

  private static final class Session {
    final Player player;
    final JsonObject data;
    final AtomicBoolean heartbeat = new AtomicBoolean();
    volatile UUID serverId;
    volatile long lastGood = System.nanoTime();
    volatile JsonObject departure;
    final AtomicBoolean recoveryLobby = new AtomicBoolean();
    volatile JoinAttempt join;
    final Map<UUID, Notice> notices = new LinkedHashMap<>();
    final Map<UUID, Boolean> replacements =
        java.util.Collections.synchronizedMap(new BoundedMap<>(32));
    final Map<UUID, Long> abandoned = java.util.Collections.synchronizedMap(new BoundedMap<>(32));
    final Map<UUID, JsonObject> confirmations =
        java.util.Collections.synchronizedMap(new LinkedHashMap<>());

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

  private static final class JoinAttempt {
    final JsonObject job;
    final UUID target;
    volatile boolean expired;
    volatile CompletableFuture<ConnectionRequestBuilder.Result> connection;

    JoinAttempt(JsonObject job, UUID target) {
      this.job = job;
      this.target = target;
    }
  }

  private record Notice(String phase, long at) {}

  private record Departure(Session session, UUID source, CompletableFuture<JsonObject> response) {}

  private static final class Waiting extends Exception {
    Waiting(String message) {
      super(message);
    }
  }
}
