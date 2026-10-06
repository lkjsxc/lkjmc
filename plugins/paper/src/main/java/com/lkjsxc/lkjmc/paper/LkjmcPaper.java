package com.lkjsxc.lkjmc.paper;

import com.google.gson.*;
import com.lkjsxc.lkjmc.common.*;
import io.papermc.paper.event.player.AsyncPlayerSpawnLocationEvent;
import java.net.URI;
import java.nio.file.Path;
import java.util.*;
import java.util.concurrent.*;
import java.util.logging.Level;
import net.kyori.adventure.text.Component;
import org.bukkit.*;
import org.bukkit.entity.Player;
import org.bukkit.event.*;
import org.bukkit.event.player.*;
import org.bukkit.generator.ChunkGenerator;
import org.bukkit.plugin.java.JavaPlugin;

public final class LkjmcPaper extends JavaPlugin implements PaperContext, Listener {
  private CoreClient core;
  private UUID serverId;
  private volatile JsonObject projection =
      CoreClient.object(
          "worlds", List.of(), "sessions", List.of(), "claims", List.of(), "assets", List.of());
  private final Map<UUID, JsonObject> sessions = new ConcurrentHashMap<>();
  private final ExecutorService executor = Executors.newVirtualThreadPerTaskExecutor();
  private final ScheduledExecutorService scheduler = Executors.newScheduledThreadPool(2);
  private volatile boolean ready = false;
  private volatile long lastCoreContact;
  private SpawnPolicy spawns;
  private ClaimProtection claims;
  private PaperJobs jobs;
  private GameEvents events;
  private IdentityOwnership ownership;
  private GameMenus menus;
  private WorldLocks worldLocks;
  private DepartureGate departures;
  private boolean proxyJoin;

  @Override
  public void onLoad() {
    saveDefaultConfig();
  }

  @Override
  public ChunkGenerator getDefaultWorldGenerator(String name, String id) {
    return new SpawnPolicy.VoidGenerator();
  }

  @Override
  public void onEnable() {
    Bukkit.getPluginManager().registerEvents(this, this);
    try {
      serverId = UUID.fromString(getConfig().getString("server-id"));
      core =
          new CoreClient(
              URI.create(getConfig().getString("core-url")),
              Path.of(getConfig().getString("credential-file")));
      // STARTUP plugins run before default worlds have loaded.
      Bukkit.getScheduler()
          .runTask(
              this,
              () ->
                  async(
                      () -> {
                        try {
                          initialize();
                        } catch (Exception e) {
                          fatal("World initialization failed", e);
                        }
                      }));
    } catch (Exception e) {
      fatal("Invalid adapter configuration", e);
    }
  }

  private void initialize() throws Exception {
    refresh();
    JsonArray identities =
        main(
            () -> {
              String primary = official() ? "holding" : "lobby";
              if (Bukkit.getWorlds().isEmpty()
                  || !Bukkit.getWorlds().getFirst().getName().equals(primary))
                throw new IllegalStateException(
                    "level-name must be " + primary + "; refusing a shared SMP default spawn");
              proxyJoin =
                  org.bukkit.configuration.file.YamlConfiguration.loadConfiguration(
                          new java.io.File("config/paper-global.yml"))
                      .getBoolean("proxies.velocity.enabled");
              if (proxyJoin) {
                departures =
                    new DepartureGate(this, Path.of(getConfig().getString("departure-key-file")));
                Bukkit.getPluginManager().registerEvents(departures, this);
              }
              JsonArray registered = new JsonArray();
              List<World> registeredWorlds = new ArrayList<>();
              for (JsonElement element : projection.getAsJsonArray("worlds")) {
                JsonObject entry = element.getAsJsonObject();
                if (!entry.get("enabled").getAsBoolean()) continue;
                String name = entry.get("name").getAsString(),
                    kind = entry.get("kind").getAsString();
                if (!name.matches("[a-z0-9_-]{1,80}"))
                  throw new IllegalStateException("Invalid world name");
                World.Environment environment =
                    switch (kind) {
                      case "nether" -> World.Environment.NETHER;
                      case "end", "private_end" -> World.Environment.THE_END;
                      default -> World.Environment.NORMAL;
                    };
                WorldCreator creator = new WorldCreator(name).environment(environment);
                if (Set.of("holding", "lobby").contains(kind))
                  creator.generator(new SpawnPolicy.VoidGenerator());
                World world = creator.createWorld();
                if (world == null) throw new IllegalStateException("World creation failed");
                if (world.getEnvironment() != environment)
                  throw new IllegalStateException("World environment mismatch");
                registeredWorlds.add(world);
                registered.add(
                    CoreClient.object(
                        "id",
                        entry.get("id").getAsString(),
                        "name",
                        name,
                        "native_uuid",
                        world.getUID()));
              }
              WorldDurability.flushWorldIdentities(registeredWorlds);
              if (official()) {
                spawns =
                    new SpawnPolicy(
                        this,
                        Objects.requireNonNull(Bukkit.getWorld("living")),
                        Objects.requireNonNull(Bukkit.getWorld("holding")));
                claims = new ClaimProtection(this);
                Provenance provenance = new Provenance(this);
                worldLocks = new WorldLocks(this, claims);
                Bukkit.getPluginManager().registerEvents(spawns, this);
                Bukkit.getPluginManager().registerEvents(claims, this);
                Bukkit.getPluginManager().registerEvents(provenance, this);
                Bukkit.getPluginManager().registerEvents(worldLocks, this);
                claims.apply();
                ownership = new IdentityOwnership(this);
                Bukkit.getPluginManager().registerEvents(ownership, this);
                ownership.reconcileLoaded();
                events = new GameEvents(this);
                Bukkit.getPluginManager().registerEvents(events, this);
                jobs =
                    new PaperJobs(this, spawns, claims, provenance, worldLocks, ownership, events);
                jobs.recover();
              } else {
                buildLobby();
              }
              menus = new GameMenus(this, claims);
              Bukkit.getPluginManager().registerEvents(menus, this);
              for (String command :
                  List.of(
                      "lkjmc",
                      "home",
                      "claim",
                      "tpa",
                      "tpahere",
                      "tpaccept",
                      "tpdeny",
                      "expedition")) Objects.requireNonNull(getCommand(command)).setExecutor(menus);
              return registered;
            });
    core.post("/internal/v1/worlds/ready", identities);
    if (jobs != null) jobs.recoverIdentityReceipts();
    ready = true;
    lastCoreContact = System.nanoTime();
    observe();
    scheduler.scheduleWithFixedDelay(
        () -> {
          try {
            refresh();
            if (claims != null)
              main(
                  () -> {
                    claims.apply();
                    return null;
                  });
            observe();
          } catch (Exception e) {
            getLogger().warning("Core synchronization: " + e.getMessage());
          }
        },
        2,
        5,
        TimeUnit.SECONDS);
    if (jobs != null) scheduler.scheduleWithFixedDelay(jobs::poll, 1, 1, TimeUnit.SECONDS);
    Bukkit.getScheduler()
        .runTaskTimer(
            this,
            () -> {
              if (System.nanoTime() - lastCoreContact > TimeUnit.SECONDS.toNanos(40))
                for (Player p : Bukkit.getOnlinePlayers())
                  p.kick(
                      text(
                          p.getUniqueId(),
                          "text.the_shared_service_is_unavailable_reconnect_through_the_d9ff5afde8"));
              if (spawns != null)
                for (Player p : Bukkit.getOnlinePlayers())
                  if (!p.isDead())
                    try {
                      spawns.remember(p.getUniqueId(), p.getLocation());
                    } catch (Exception e) {
                      fatal("Player position journal failed", e);
                    }
            },
            200,
            200);
    getLogger().info("lkjmc adapter ready: " + serverId);
  }

  private synchronized void refresh() throws Exception {
    JsonObject next = core.get("/internal/v1/projection");
    if (!next.get("role").getAsString().equals(official() ? "official" : "lobby"))
      throw new IllegalStateException("Credential role mismatch");
    projection = next;
    lastCoreContact = System.nanoTime();
    for (JsonElement element : next.getAsJsonArray("sessions")) {
      JsonObject session = element.getAsJsonObject();
      UUID nativeId = CoreClient.uuid(session, "native_uuid");
      JsonObject previous = sessions.put(nativeId, session);
      boolean languageChanged =
          previous != null
              && !CoreClient.string(previous, "language", "en")
                  .equals(CoreClient.string(session, "language", "en"));
      if (menus != null
          && (previous == null
              || !previous.get("session_id").equals(session.get("session_id"))
              || languageChanged))
        Bukkit.getScheduler()
            .runTask(
                this,
                () -> {
                  Player player = Bukkit.getPlayer(nativeId);
                  if (player != null
                      && session
                          .get("session_id")
                          .equals(
                              sessions.getOrDefault(nativeId, new JsonObject()).get("session_id")))
                    if (languageChanged) menus.localeChanged(player);
                    else menus.installSoon(player);
                });
    }
  }

  private void observe() throws Exception {
    JsonObject body =
        main(
            () ->
                CoreClient.object(
                    "server_id",
                    serverId,
                    "observed",
                    "running",
                    "players",
                    Bukkit.getOnlinePlayers().size(),
                    "metrics",
                    CoreClient.object(
                        "tps",
                        Bukkit.getTPS(),
                        "mspt",
                        Bukkit.getAverageTickTime(),
                        "minecraft_version",
                        Bukkit.getMinecraftVersion(),
                        "adapter_ready",
                        ready),
                    "capabilities",
                    CoreClient.object(
                        "proxy_join",
                        proxyJoin,
                        "bedrock",
                        getConfig().getBoolean("bedrock-compatible", false),
                        "vanilla_client",
                        true,
                        "official_progression",
                        official(),
                        "adapter_ready",
                        ready)));
    core.post("/internal/v1/observations", body);
  }

  private void buildLobby() {
    World world = Objects.requireNonNull(Bukkit.getWorld("lobby"));
    world.setSpawnLocation(0, 66, 0);
    world.setDifficulty(Difficulty.PEACEFUL);
    for (int x = -20; x <= 20; x++)
      for (int z = -20; z <= 20; z++) {
        double radius = Math.hypot(x, z);
        if (radius > 20) continue;
        world
            .getBlockAt(x, 64, z)
            .setType(radius > 18 ? Material.SMOOTH_QUARTZ : Material.DEEPSLATE_TILES, false);
        if (radius > 19) world.getBlockAt(x, 65, z).setType(Material.GLASS, false);
      }
    for (int x : new int[] {-12, 12})
      for (int z : new int[] {-12, 12})
        for (int y = 65; y < 70; y++)
          world
              .getBlockAt(x, y, z)
              .setType(y == 69 ? Material.SEA_LANTERN : Material.QUARTZ_PILLAR, false);
  }

  private Component loginMessage(UUID nativeId, String id) {
    JsonObject cached = sessions.get(nativeId);
    String language = cached == null ? "en" : CoreClient.string(cached, "language", "en");
    return Component.text(com.lkjsxc.lkjmc.common.Messages.text(language, id));
  }

  @EventHandler(priority = EventPriority.LOWEST)
  public void prelogin(AsyncPlayerPreLoginEvent e) {
    if (jobs != null && jobs.identityBlocked(e.getUniqueId())) {
      e.disallow(
          AsyncPlayerPreLoginEvent.Result.KICK_OTHER,
          loginMessage(
              e.getUniqueId(),
              "text.finishing_account_links_and_land_protection_wait_a_mome_19a06be16a"));
      return;
    }
    if (!ready || System.nanoTime() - lastCoreContact > TimeUnit.SECONDS.toNanos(30)) {
      e.disallow(
          AsyncPlayerPreLoginEvent.Result.KICK_OTHER,
          loginMessage(
              e.getUniqueId(), "text.the_server_is_being_prepared_please_wait_in_the_lobby"));
      return;
    }
    try {
      sessions.put(e.getUniqueId(), core.get("/internal/v1/game/profile/" + e.getUniqueId()));
    } catch (Exception error) {
      e.disallow(
          AsyncPlayerPreLoginEvent.Result.KICK_OTHER,
          loginMessage(
              e.getUniqueId(),
              "text.your_game_identity_could_not_be_verified_reconnect_thro_7ee258323c"));
    }
  }

  @EventHandler(priority = EventPriority.LOWEST)
  public void spawnGate(AsyncPlayerSpawnLocationEvent e) {
    if (!ready
        || !sessions.containsKey(e.getConnection().getProfile().getId())
        || jobs != null && jobs.identityBlocked(e.getConnection().getProfile().getId()))
      e.getConnection()
          .disconnect(
              text(
                  e.getConnection().getProfile().getId(),
                  "text.connect_through_the_lobby_after_your_session_is_verified"));
    else if (!official())
      try {
        e.setSpawnLocation(main(() -> Bukkit.getWorld("lobby").getSpawnLocation()));
      } catch (Exception error) {
        e.getConnection()
            .disconnect(
                text(
                    e.getConnection().getProfile().getId(),
                    "text.the_lobby_could_not_be_prepared"));
      }
  }

  @EventHandler(priority = EventPriority.LOWEST)
  public void join(PlayerJoinEvent e) {
    if (jobs != null)
      try {
        jobs.recoverPlayer(e.getPlayer());
      } catch (Exception error) {
        e.getPlayer().kick(text(e.getPlayer().getUniqueId(), "text.recovering_inventory_saves"));
        fatal("Inventory reconciliation failed", error);
        return;
      }
    e.getPlayer()
        .sendMessage(
            text(e.getPlayer().getUniqueId(), "text.welcome_to_lkjmc_use_menu_to_open_the_menu"));
    if (!official()) {
      e.getPlayer().setGameMode(GameMode.ADVENTURE);
      e.getPlayer().setInvulnerable(true);
    }
  }

  @EventHandler(priority = EventPriority.MONITOR)
  public void quit(PlayerQuitEvent e) {
    sessions.remove(e.getPlayer().getUniqueId());
  }

  private void fatal(String message, Throwable error) {
    getLogger().log(Level.SEVERE, message, error);
    ready = false;
    Bukkit.getScheduler().runTask(this, Bukkit::shutdown);
  }

  @Override
  public void onDisable() {
    ready = false;
    if (events != null) events.close();
    if (jobs != null) jobs.close();
    scheduler.shutdownNow();
    executor.shutdownNow();
    if (core != null)
      try {
        core.close();
      } catch (Exception ignored) {
      }
  }

  @Override
  public JavaPlugin plugin() {
    return this;
  }

  @Override
  public CoreClient core() {
    return core;
  }

  @Override
  public UUID serverId() {
    return serverId;
  }

  @Override
  public JsonObject projection() {
    return projection;
  }

  @Override
  public void refreshProjection() throws Exception {
    refresh();
  }

  @Override
  public boolean mustIsolate(UUID id) {
    return jobs != null && jobs.needsRecovery(id);
  }

  @Override
  public boolean quarantined(Location location) {
    return worldLocks != null && worldLocks.locked(location);
  }

  @Override
  public JsonObject spawnClaim(Location location) {
    return claims == null || location == null ? null : claims.claim(location.getBlock());
  }

  @Override
  public boolean mayRespawn(UUID nativeId, Location location) {
    if (location == null || quarantined(location)) return false;
    JsonObject claim = spawnClaim(location);
    if (claim == null) return true;
    try {
      return claim.get("state").getAsString().equals("active")
          && claims.canBuild(CoreClient.uuid(session(nativeId), "account_id"), location.getBlock());
    } catch (Exception e) {
      return false;
    }
  }

  @Override
  public boolean inCombat(UUID id) {
    return events != null && events.inCombat(id);
  }

  @Override
  public boolean departing(UUID id) {
    return departures != null && departures.leaving(id);
  }

  @Override
  public void assignPetOwner(org.bukkit.entity.Tameable pet, UUID owner) {
    if (ownership == null) throw new IllegalStateException("Official pet ownership is not ready");
    ownership.assign(pet, owner);
  }

  @Override
  public JsonObject session(UUID id) throws Exception {
    JsonObject existing = sessions.get(id);
    if (existing != null) return existing.deepCopy();
    if (Bukkit.isPrimaryThread()) throw new IllegalStateException("No authenticated game session");
    JsonObject value = core.get("/internal/v1/game/profile/" + id);
    sessions.put(id, value);
    return value.deepCopy();
  }

  @Override
  public void async(Runnable work) {
    executor.execute(
        () -> {
          try {
            work.run();
          } catch (Throwable error) {
            fatal("Unexpected adapter failure", error);
          }
        });
  }
}
