package com.lkjsxc.lkjmc.paper;

import com.google.gson.*;
import com.lkjsxc.lkjmc.common.*;
import io.papermc.paper.event.player.AsyncChatEvent;
import java.time.Instant;
import java.time.ZoneOffset;
import java.time.format.DateTimeFormatter;
import java.time.format.FormatStyle;
import java.util.*;
import java.util.concurrent.ConcurrentHashMap;
import java.util.function.Consumer;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.format.NamedTextColor;
import net.kyori.adventure.text.serializer.plain.PlainTextComponentSerializer;
import org.bukkit.*;
import org.bukkit.command.*;
import org.bukkit.entity.Player;
import org.bukkit.event.*;
import org.bukkit.event.inventory.*;
import org.bukkit.inventory.*;

/** Inventory menus and private chat input work in both Java and Geyser clients. */
public final class GameMenus implements Listener, CommandExecutor {
  private final PaperContext ctx;
  private final ClaimProtection claims;
  private final Map<UUID, Deque<Menu>> history = new HashMap<>();
  private final Map<UUID, Menu> current = new HashMap<>();
  private final Map<UUID, Long> launcherOpens = new HashMap<>();
  private final Map<UUID, UUID> launcherPending = new HashMap<>();
  private final Map<UUID, Long> requests = new ConcurrentHashMap<>();
  private final Map<UUID, String> selectedLanguages = new ConcurrentHashMap<>();

  private String language(Player p) {
    try {
      return CoreClient.string(ctx.session(p.getUniqueId()), "language", "en");
    } catch (Exception e) {
      return "en";
    }
  }

  private String effectiveLanguage(Player p) {
    return selectedLanguages.getOrDefault(p.getUniqueId(), language(p));
  }

  private String tr(Player p, String key, Object... values) {
    return Messages.text(effectiveLanguage(p), key, values);
  }

  private String systemText(Player p, JsonObject record, String field) {
    JsonElement message = record.get(field + "_message");
    return message != null && !message.isJsonNull()
        ? Messages.render(effectiveLanguage(p), message)
        : CoreClient.string(record, field, "");
  }

  private record Entry(Material icon, String title, String description, Runnable action) {}

  private record Input(long expires, UUID sessionId, long navigation, Consumer<String> action) {}

  private final Map<UUID, Input> inputs = new ConcurrentHashMap<>();

  private record PlacementPreview(JsonObject asset, JsonObject placement, JsonObject result) {}

  private final Map<UUID, PlacementPreview> previews = new ConcurrentHashMap<>();

  private static final class Menu implements InventoryHolder {
    Inventory inventory;
    Player player;
    String title;
    List<Entry> entries;
    int page;
    boolean consumed;
    boolean loading;
    UUID sessionId;
    final Map<Integer, Runnable> actions = new HashMap<>();

    @Override
    public Inventory getInventory() {
      return inventory;
    }
  }

  public GameMenus(PaperContext ctx, ClaimProtection claims) {
    this.ctx = ctx;
    this.claims = claims;
  }

  private Entry entry(Material icon, String title, String description, Runnable action) {
    return new Entry(icon, title, description, action);
  }

  private JsonObject command(String kind, Object... fields) {
    JsonObject c = CoreClient.object(fields);
    c.addProperty("type", kind);
    return c;
  }

  private void menu(Player p, String title, List<Entry> entries, int page) {
    render(p, title, entries, page, true);
  }

  private void render(Player p, String title, List<Entry> entries, int page, boolean remember) {
    render(p, title, entries, page, remember, false);
  }

  private void render(
      Player p, String title, List<Entry> entries, int page, boolean remember, boolean loading) {
    if (!currentPlayer(p) || sessionId(p) == null) return;
    Menu previous = current.get(p.getUniqueId());
    Deque<Menu> trail = history.computeIfAbsent(p.getUniqueId(), ignored -> new ArrayDeque<>());
    if (remember && previous != null && !previous.loading && previous.entries != entries) {
      if (trail.size() == 12) trail.removeFirst();
      trail.addLast(previous);
    }
    Menu holder = new Menu();
    holder.player = p;
    holder.sessionId = sessionId(p);
    holder.title = title;
    holder.entries = entries;
    holder.loading = loading;
    holder.page = Math.max(0, Math.min(page, Math.max(0, (entries.size() - 1) / 28)));
    holder.inventory = Bukkit.createInventory(holder, 54, Component.text(title));
    int offset = holder.page * 28;
    for (int i = 0; i < 28 && offset + i < entries.size(); i++)
      put(holder, 10 + (i / 7) * 9 + i % 7, entries.get(offset + i));
    if (entries.isEmpty())
      put(
          holder,
          22,
          entry(
              Material.GRAY_DYE,
              tr(p, "Nothing here yet"),
              tr(p, "Return to the previous menu to choose another action."),
              null));
    if (!trail.isEmpty())
      put(
          holder,
          45,
          entry(
              Material.ARROW,
              tr(p, "Back"),
              "",
              () -> {
                requests.merge(p.getUniqueId(), 1L, Long::sum);
                Menu back = trail.removeLast();
                render(p, back.title, back.entries, back.page, false);
              }));
    if (holder.page > 0)
      put(
          holder,
          48,
          entry(
              Material.ARROW,
              tr(p, "Previous page"),
              "",
              () -> render(p, title, entries, holder.page - 1, false)));
    put(holder, 49, entry(Material.COMPASS, tr(p, "Main menu"), "", () -> root(p)));
    if (offset + 28 < entries.size())
      put(
          holder,
          50,
          entry(
              Material.ARROW,
              tr(p, "Next page"),
              "",
              () -> render(p, title, entries, holder.page + 1, false)));
    put(
        holder,
        51,
        entry(
            Material.PAPER,
            tr(p, "Page {0} of {1}", holder.page + 1, Math.max(1, (entries.size() + 27) / 28)),
            "",
            null));
    put(holder, 53, entry(Material.BARRIER, tr(p, "Close"), "", p::closeInventory));
    current.put(p.getUniqueId(), holder);
    p.openInventory(holder.inventory);
  }

  private void put(Menu holder, int slot, Entry entry) {
    ItemStack stack = new ItemStack(entry.icon);
    stack.editMeta(
        meta -> {
          meta.displayName(
              Component.text(
                      entry.title,
                      entry.action == null ? NamedTextColor.WHITE : NamedTextColor.AQUA)
                  .decoration(net.kyori.adventure.text.format.TextDecoration.ITALIC, false));
          meta.lore(
              wrap(entry.description).stream()
                  .filter(s -> !s.isBlank())
                  .map(
                      s ->
                          Component.text(s, NamedTextColor.GRAY)
                              .decoration(
                                  net.kyori.adventure.text.format.TextDecoration.ITALIC, false))
                  .toList());
        });
    holder.inventory.setItem(slot, stack);
    if (entry.action != null) holder.actions.put(slot, entry.action);
  }

  private static List<String> wrap(String text) {
    List<String> result = new ArrayList<>();
    for (String line : text.split("\n")) {
      String remaining = line;
      while (remaining.codePointCount(0, remaining.length()) > 38) {
        int end = remaining.offsetByCodePoints(0, 38);
        int space = remaining.lastIndexOf(' ', end);
        if (space > end / 2) end = space;
        result.add(remaining.substring(0, end));
        remaining = remaining.substring(end).stripLeading();
      }
      if (!remaining.isBlank()) result.add(remaining);
    }
    return result;
  }

  private void inform(Player p, String text) {
    p.sendMessage(Component.text(text));
  }

  private void input(Player p, String question, Consumer<String> action) {
    requests.merge(p.getUniqueId(), 1L, Long::sum);
    p.closeInventory();
    long navigation = requests.merge(p.getUniqueId(), 1L, Long::sum);
    inputs.put(
        p.getUniqueId(),
        new Input(System.currentTimeMillis() + 120000, sessionId(p), navigation, action));
    inform(p, question + tr(p, "\nType in chat (only you can see it). Cancel: cancel"));
  }

  private void confirm(Player p, String title, String detail, Runnable action) {
    menu(
        p,
        title,
        List.of(
            entry(Material.LIME_CONCRETE, tr(p, "Confirm"), detail, action),
            entry(Material.RED_CONCRETE, tr(p, "Cancel"), "", () -> back(p))),
        0);
  }

  private JsonObject view(Player p, String name, JsonObject query) throws Exception {
    JsonObject session = ctx.session(p.getUniqueId());
    return ctx.core()
        .post(
            "/internal/v1/game/view",
            CoreClient.object(
                "account_id",
                session.get("account_id").getAsString(),
                "session_id",
                session.get("session_id").getAsString(),
                "view",
                name,
                "query",
                query));
  }

  private void fetch(Player p, String name, Consumer<JsonObject> render) {
    fetch(p, name, new JsonObject(), render);
  }

  private void fetch(Player p, String name, JsonObject query, Consumer<JsonObject> render) {
    UUID expectedSession = sessionId(p);
    if (!validPlayer(p, expectedSession)) return;
    Menu previous = current.get(p.getUniqueId());
    if (previous != null && !previous.loading) {
      Deque<Menu> trail = history.computeIfAbsent(p.getUniqueId(), ignored -> new ArrayDeque<>());
      if (trail.size() == 12) trail.removeFirst();
      trail.addLast(previous);
    }
    render(
        p,
        tr(p, "Loading"),
        List.of(
            entry(
                Material.CLOCK,
                tr(p, "Loading"),
                tr(p, "You can go back or close this menu."),
                null)),
        0,
        false,
        true);
    long request = requests.merge(p.getUniqueId(), 1L, Long::sum);
    Inventory expectedView = p.getOpenInventory().getTopInventory();
    ctx.async(
        () -> {
          try {
            JsonObject data = view(p, name, query);
            ctx.main(
                () -> {
                  if (validPlayer(p, expectedSession)
                      && p.getOpenInventory().getTopInventory() == expectedView
                      && Objects.equals(requests.get(p.getUniqueId()), request))
                    render.accept(data);
                  return null;
                });
          } catch (Exception e) {
            Bukkit.getScheduler()
                .runTask(
                    ctx.plugin(),
                    () -> {
                      if (validPlayer(p, expectedSession)
                          && p.getOpenInventory().getTopInventory() == expectedView
                          && Objects.equals(requests.get(p.getUniqueId()), request)) error(p, e);
                    });
          }
        });
  }

  private void back(Player p) {
    requests.merge(p.getUniqueId(), 1L, Long::sum);
    Deque<Menu> trail = history.get(p.getUniqueId());
    if (trail == null || trail.isEmpty()) root(p);
    else {
      Menu previous = trail.removeLast();
      render(p, previous.title, previous.entries, previous.page, false);
    }
  }

  private void error(Player p, Exception e) {
    selectedLanguages.remove(p.getUniqueId());
    Bukkit.getScheduler()
        .runTask(
            ctx.plugin(),
            () -> {
              if (currentPlayer(p))
                inform(
                    p,
                    tr(p, "Could not complete action: ") + Messages.error(effectiveLanguage(p), e));
            });
  }

  private void submit(Player p, JsonObject command) {
    submit(p, command, null);
  }

  private void submit(Player p, JsonObject command, Consumer<JsonObject> completed) {
    p.closeInventory();
    long navigation = requests.merge(p.getUniqueId(), 1L, Long::sum);
    Inventory expectedView = p.getOpenInventory().getTopInventory();
    UUID request = UUID.randomUUID();
    JsonObject submittedSession;
    try {
      submittedSession = ctx.session(p.getUniqueId());
    } catch (Exception e) {
      error(p, e);
      return;
    }
    boolean join = command.get("type").getAsString().equals("server_join");
    ctx.async(
        () -> {
          try {
            JsonObject result =
                ctx.core().command(submittedSession, command, request).getAsJsonObject("result");
            ctx.main(
                () -> {
                  if (!validPlayer(p, CoreClient.uuid(submittedSession, "session_id"))) return null;
                  if (join) {
                    inform(
                        p,
                        tr(
                            p,
                            "Travel to {0} is queued. Stay connected; progress will appear here."
                                + " Cancel: /go cancel",
                            CoreClient.string(
                                result, "server_name", command.get("id").getAsString())));
                    return null;
                  }
                  inform(
                      p,
                      result.has("job_id")
                          ? tr(p, "Request accepted. You will be notified when it finishes.")
                          : tr(p, "Saved."));
                  if (!result.has("job_id")
                      && completed != null
                      && Objects.equals(requests.get(p.getUniqueId()), navigation)
                      && p.getOpenInventory().getTopInventory() == expectedView)
                    completed.accept(result);
                  if (result.has("code"))
                    inform(
                        p,
                        tr(p, "Link code: ")
                            + result.get("code").getAsString()
                            + tr(p, " (expires in 10 minutes; use only on your own account)"));
                  return null;
                });
            if (join)
              return; // Proxy owns progress through actual arrival, including backend changes.
            if (result.has("job_id")) {
              for (int attempt = 0; attempt < 90 && p.isOnline(); attempt++) {
                Thread.sleep(2000);
                JsonObject status =
                    view(p, "job", CoreClient.object("id", result.get("job_id").getAsString()));
                String state = status.get("state").getAsString();
                if (Set.of("succeeded", "failed", "cancelled").contains(state)) {
                  ctx.main(
                      () -> {
                        if (!validPlayer(p, CoreClient.uuid(submittedSession, "session_id")))
                          return null;
                        inform(
                            p,
                            state.equals("succeeded")
                                ? tr(p, "Action completed.")
                                : tr(p, "Action failed: ")
                                    + (status.has("error") && !status.get("error").isJsonNull()
                                        ? Messages.render(effectiveLanguage(p), status.get("error"))
                                        : tr(p, "Check notifications for details.")));
                        if (state.equals("succeeded")
                            && completed != null
                            && Objects.equals(requests.get(p.getUniqueId()), navigation)
                            && p.getOpenInventory().getTopInventory() == expectedView)
                          completed.accept(status.getAsJsonObject("result"));
                        return null;
                      });
                  return;
                }
              }
            }
          } catch (Exception e) {
            error(p, e);
          }
        });
  }

  private void choosePlayer(Player p, String title, Consumer<JsonObject> selected) {
    input(
        p,
        tr(p, "Enter the other player’s name"),
        text ->
            fetch(
                p,
                "players",
                CoreClient.object("q", text),
                result -> {
                  List<Entry> list = new ArrayList<>();
                  for (JsonElement element : result.getAsJsonArray("players")) {
                    JsonObject item = element.getAsJsonObject();
                    list.add(
                        entry(
                            Material.PLAYER_HEAD,
                            item.get("name").getAsString(),
                            systemText(p, item, "rank"),
                            () -> selected.accept(item)));
                  }
                  menu(p, title, list, 0);
                }));
  }

  public void root(Player p) {
    requests.merge(p.getUniqueId(), 1L, Long::sum);
    inputs.remove(p.getUniqueId());
    history.remove(p.getUniqueId());
    current.remove(p.getUniqueId());
    selectedLanguages.remove(p.getUniqueId());
    List<Entry> entries = new ArrayList<>();
    if (ctx.official()) {
      entries.add(
          entry(
              Material.RED_BED,
              tr(p, "Homes"),
              tr(p, "Return home or save this place"),
              () -> life(p, "homes")));
      entries.add(
          entry(
              Material.OAK_FENCE,
              tr(p, "Land"),
              tr(p, "Protect your builds and manage shared land"),
              () -> life(p, "land")));
      entries.add(
          entry(
              Material.EMERALD,
              tr(p, "Market"),
              tr(p, "Buy, list, collect and sell materials"),
              () -> market(p)));
      entries.add(
          entry(
              Material.ENDER_EYE,
              tr(p, "Expeditions"),
              tr(p, "Explore a temporary world together"),
              () -> expeditions(p)));
      entries.add(
          entry(
              Material.PLAYER_HEAD,
              tr(p, "People"),
              tr(p, "Friends, teams and parties"),
              () -> people(p)));
      entries.add(
          entry(
              Material.OAK_DOOR,
              tr(p, "Return to SMP"),
              tr(p, "Leave an Expedition with the items you carry"),
              () -> returns(p)));
    } else {
      entries.add(
          entry(
              Material.COMPASS,
              tr(p, "Play"),
              tr(p, "Choose a world and start playing"),
              () -> servers(p)));
      entries.add(
          entry(
              Material.GRASS_BLOCK,
              tr(p, "Worlds"),
              tr(p, "Explore available worlds and their people"),
              () -> servers(p)));
      entries.add(
          entry(
              Material.PLAYER_HEAD,
              tr(p, "People"),
              tr(p, "Friends, teams and parties"),
              () -> people(p)));
    }
    entries.add(
        entry(
            Material.WRITABLE_BOOK,
            tr(p, "Timeline"),
            tr(p, "Conversations, invitations and activity"),
            () -> timeline(p)));
    if (ctx.official())
      entries.add(
          entry(
              Material.GRASS_BLOCK,
              tr(p, "Worlds"),
              tr(p, "Choose another world"),
              () -> servers(p)));
    entries.add(
        entry(
            Material.NAME_TAG,
            tr(p, "Account"),
            tr(p, "Language, account linking and achievements"),
            () -> account(p)));
    entries.add(
        entry(
            Material.BOOK,
            tr(p, "Help"),
            tr(p, "Getting started and useful commands"),
            () -> help(p)));
    menu(p, "lkjmc", entries, 0);
  }

  private void people(Player p) {
    menu(
        p,
        tr(p, "People"),
        List.of(
            entry(
                Material.PLAYER_HEAD,
                tr(p, "Friends"),
                tr(p, "Friends and requests"),
                () -> social(p, "friends")),
            entry(
                Material.WHITE_BANNER,
                tr(p, "Teams"),
                tr(p, "A lasting group with shared land and coins"),
                () -> social(p, "team")),
            entry(
                Material.CAMPFIRE,
                tr(p, "Parties"),
                tr(p, "A temporary group for playing together"),
                () -> social(p, "party"))),
        0);
  }

  private void timeline(Player p) {
    menu(
        p,
        tr(p, "Timeline"),
        List.of(
            entry(
                Material.WRITABLE_BOOK,
                tr(p, "Conversations"),
                tr(p, "Read and send messages in one conversation"),
                () -> social(p, "chat")),
            entry(
                Material.BELL,
                tr(p, "Invitations & activity"),
                tr(p, "Respond to invitations and check progress"),
                () -> notifications(p))),
        0);
  }

  private void account(Player p) {
    List<Entry> entries =
        new ArrayList<>(
            List.of(
                entry(
                    Material.WRITABLE_BOOK,
                    tr(p, "Language"),
                    tr(p, "Your language is shared with linked game accounts."),
                    () -> languages(p)),
                entry(
                    Material.NAME_TAG,
                    tr(p, "Account linking"),
                    tr(p, "Verify and link your game and Web identities"),
                    () -> link(p))));
    if (ctx.official())
      entries.add(
          entry(
              Material.EXPERIENCE_BOTTLE,
              tr(p, "Achievements & balance"),
              tr(p, "Your progress and shared coins"),
              () -> life(p, "progress")));
    menu(p, tr(p, "Account"), entries, 0);
  }

  private void languages(Player p) {
    List<Entry> entries = new ArrayList<>();
    String active = language(p);
    for (JsonElement value : Messages.languages()) {
      JsonObject locale = value.getAsJsonObject();
      String code = locale.get("code").getAsString();
      entries.add(
          entry(
              active.equals(code) ? Material.LIME_DYE : Material.GRAY_DYE,
              locale.get("name").getAsString(),
              active.equals(code) ? tr(p, "Selected") : tr(p, "Use this language"),
              () -> {
                selectedLanguages.put(p.getUniqueId(), code);
                submit(
                    p,
                    command("language", "language", code),
                    result -> {
                      ctx.async(
                          () -> {
                            try {
                              ctx.refreshProjection();
                              ctx.main(
                                  () -> {
                                    selectedLanguages.remove(p.getUniqueId());
                                    root(p);
                                    return null;
                                  });
                            } catch (Exception e) {
                              selectedLanguages.remove(p.getUniqueId());
                              error(p, e);
                            }
                          });
                    });
              }));
    }
    menu(p, tr(p, "Language"), entries, 0);
  }

  private void help(Player p) {
    menu(
        p,
        tr(p, "Help"),
        List.of(
            entry(
                Material.COMPASS,
                tr(p, "Play"),
                tr(
                    p,
                    "Choose a world. Stay connected while it wakes; progress and arrival appear in"
                        + " chat. Cancel: /go cancel"),
                () -> servers(p)),
            entry(
                Material.ENDER_EYE,
                tr(p, "Expeditions"),
                tr(
                    p,
                    "Temporary worlds close when their time ends. Keep what you carry; collect your"
                        + " items before leaving."),
                ctx.official() ? () -> expeditions(p) : () -> servers(p)),
            entry(
                Material.NAME_TAG,
                tr(p, "Account linking"),
                "https://lkjmc.lkjsxc.com",
                () -> link(p)),
            entry(
                Material.BOOK,
                tr(p, "Commands"),
                "/menu · /home · /claim · /tpa\n"
                    + "/expedition · /expedition return\n"
                    + "/worlds · /hub · /go cancel\n"
                    + "/menu cancel",
                null)),
        0);
  }

  private String serverState(Player p, JsonObject server) {
    if (CoreClient.string(server, "maintenance", "false").equals("true"))
      return tr(p, "Maintenance");
    return switch (CoreClient.string(server, "observed", "unknown")) {
      case "running" -> tr(p, "Ready to play");
      case "stopped" -> tr(p, "Sleeping — join to wake");
      case "starting" -> tr(p, "Preparing");
      case "stopping" -> tr(p, "Closing");
      case "error", "failed" -> tr(p, "Unavailable");
      default -> tr(p, "Checking availability");
    };
  }

  private void smp(Player p, JsonObject server) {
    String state = server.get("observed").getAsString();
    boolean available =
        proxyCompatible(p, server)
            && !CoreClient.string(server, "maintenance", "false").equals("true")
            && Set.of("running", "stopped").contains(state);
    List<Entry> entries = new ArrayList<>();
    entries.add(
        entry(
            available ? Material.GRASS_BLOCK : Material.GRAY_DYE,
            tr(p, "Join world"),
            serverState(p, server),
            available
                ? () -> submit(p, command("server_join", "id", server.get("id").getAsString()))
                : null));
    if (ctx.official()) {
      entries.add(
          entry(
              Material.RED_BED,
              tr(p, "Homes"),
              tr(p, "Return home or save this place"),
              () -> life(p, "homes")));
      entries.add(
          entry(
              Material.OAK_FENCE,
              tr(p, "Land"),
              tr(p, "Protect your builds and manage shared land"),
              () -> life(p, "land")));
      entries.add(
          entry(
              Material.EMERALD,
              tr(p, "Market"),
              tr(p, "Buy, list, collect and sell materials"),
              () -> market(p)));
      entries.add(
          entry(
              Material.ENDER_EYE,
              tr(p, "Expeditions"),
              tr(p, "Explore a temporary world together"),
              () -> expeditions(p)));
    } else {
      entries.add(
          entry(
              Material.BOOK,
              tr(p, "Life in SMP"),
              tr(p, "Join this world to use Homes, Land, Market and Expeditions."),
              null));
    }
    entries.add(
        entry(
            Material.PLAYER_HEAD,
            tr(p, "People"),
            tr(p, "Friends, teams and parties"),
            () -> people(p)));
    menu(p, server.get("name").getAsString(), entries, 0);
  }

  private void link(Player p) {
    menu(
        p,
        tr(p, "Account linking"),
        List.of(
            entry(
                Material.PAPER,
                tr(p, "Create code"),
                tr(p, "Choose one set of game data to keep using"),
                () -> submit(p, command("link_begin"))),
            entry(
                Material.WRITABLE_BOOK,
                tr(p, "Enter code"),
                "",
                () ->
                    input(
                        p,
                        tr(p, "Code from your other account"),
                        code -> submit(p, command("link_present", "code", code)))),
            entry(
                Material.CHEST,
                tr(p, "Choose game data"),
                tr(p, "Confirm on the account that created the code"),
                () -> linkChoices(p))),
        0);
  }

  private void linkChoices(Player p) {
    fetch(
        p,
        "settings",
        data -> {
          List<Entry> choices = new ArrayList<>();
          String account;
          try {
            account = ctx.session(p.getUniqueId()).get("account_id").getAsString();
          } catch (Exception e) {
            inform(p, Messages.error(effectiveLanguage(p), e));
            return;
          }
          for (JsonElement element : data.getAsJsonArray("links")) {
            JsonObject link = element.getAsJsonObject();
            if (!link.get("initiator").getAsString().equals(account)
                || !link.get("state").getAsString().equals("pending")
                || link.get("candidate").isJsonNull()) continue;
            for (JsonElement v : link.getAsJsonArray("profiles")) {
              JsonObject profile = v.getAsJsonObject();
              String name = profile.get("name").getAsString();
              String detail =
                  name
                      + " / "
                      + profile.getAsJsonObject("wallet").get("balance").getAsLong()
                      + tr(p, " coins\n")
                      + (profile.get("native_uuid").isJsonNull()
                          ? tr(p, "Start with a fresh inventory and achievements")
                          : tr(p, "Use this inventory and achievements"));
              choices.add(
                  entry(
                      Material.CHEST,
                      name,
                      detail,
                      () ->
                          confirm(
                              p,
                              tr(p, "Confirm game data"),
                              detail
                                  + tr(
                                      p,
                                      "\n"
                                          + "The other data is archived, not combined\n"
                                          + "Both game connections will be disconnected"),
                              () ->
                                  submit(
                                      p,
                                      command(
                                          "link_confirm",
                                          "id",
                                          link.get("id"),
                                          "selected_profile",
                                          profile.get("id"))))));
            }
          }
          if (choices.isEmpty())
            inform(
                p,
                tr(
                    p,
                    "No pending link. Enter the code on your other account, then reopen this"
                        + " menu."));
          else menu(p, tr(p, "Game data to keep"), choices, 0);
        });
  }

  private boolean proxyCompatible(Player p, JsonObject server) {
    JsonObject capabilities = server.getAsJsonObject("capabilities");
    if (capabilities == null
        || !capabilities.has("proxy_join")
        || !capabilities.get("proxy_join").getAsBoolean()) return false;
    try {
      return !CoreClient.string(ctx.session(p.getUniqueId()), "client", "java").equals("bedrock")
          || capabilities.has("bedrock") && capabilities.get("bedrock").getAsBoolean();
    } catch (Exception e) {
      return false;
    }
  }

  private void servers(Player p) {
    fetch(
        p,
        "play",
        data -> {
          List<Entry> list = new ArrayList<>();
          for (JsonElement value : data.getAsJsonArray("servers")) {
            JsonObject s = value.getAsJsonObject();
            list.add(
                entry(
                    Material.GRASS_BLOCK,
                    s.get("name").getAsString(),
                    serverState(p, s)
                        + "\n"
                        + s.get("players").getAsInt()
                        + tr(p, " players / ")
                        + s.get("version").getAsString()
                        + (proxyCompatible(p, s)
                            ? ""
                            : "\n"
                                + tr(
                                    p,
                                    "Joining this server through the lobby is not available for"
                                        + " your client.")),
                    s.get("kind").getAsString().equals("official")
                        ? () -> smp(p, s)
                        : proxyCompatible(p, s)
                                && !s.get("maintenance").getAsBoolean()
                                && Set.of("running", "stopped")
                                    .contains(s.get("observed").getAsString())
                            ? () ->
                                submit(p, command("server_join", "id", s.get("id").getAsString()))
                            : null));
          }
          menu(p, tr(p, "Worlds"), list, 0);
        });
  }

  private void life(Player p, String section) {
    fetch(
        p,
        "life",
        data -> {
          List<Entry> list = new ArrayList<>();
          if (!section.equals("homes"))
            for (JsonElement value : data.getAsJsonArray("owners")) {
              JsonObject owner = value.getAsJsonObject();
              list.add(
                  entry(
                      Material.GOLD_INGOT,
                      owner.get("name").getAsString(),
                      tr(p, "Balance: ")
                          + owner.getAsJsonObject("wallet").get("balance").getAsLong()
                          + tr(p, " coins\nLand: ")
                          + owner.get("used_chunks")
                          + " / "
                          + owner.getAsJsonObject("land").get("chunks"),
                      null));
            }
          if (section.equals("homes"))
            list.add(
                entry(
                    Material.RED_BED,
                    tr(p, "Set a home here"),
                    p.getWorld().getName().equals("living")
                        ? tr(p, "Up to three homes")
                        : tr(p, "Save homes in the survival world."),
                    p.getWorld().getName().equals("living")
                        ? () ->
                            input(
                                p,
                                tr(p, "Home name"),
                                name -> submit(p, command("home_set", "name", name)))
                        : null));
          if (section.equals("homes"))
            for (JsonElement value : data.getAsJsonArray("homes")) {
              JsonObject h = value.getAsJsonObject();
              list.add(
                  entry(
                      Material.OAK_DOOR,
                      h.get("name").getAsString(),
                      tr(p, "Travel or delete"),
                      () ->
                          menu(
                              p,
                              h.get("name").getAsString(),
                              List.of(
                                  entry(
                                      Material.ENDER_PEARL,
                                      tr(p, "Travel here"),
                                      "",
                                      () ->
                                          submit(
                                              p,
                                              command(
                                                  "home_travel", "id", h.get("id").getAsString()))),
                                  entry(
                                      Material.BARRIER,
                                      tr(p, "Delete home"),
                                      tr(p, "Land and buildings stay unchanged"),
                                      () ->
                                          confirm(
                                              p,
                                              tr(p, "Confirm home deletion"),
                                              h.get("name").getAsString(),
                                              () ->
                                                  submit(
                                                      p,
                                                      command(
                                                          "home_delete",
                                                          "id",
                                                          h.get("id").getAsString()))))),
                              0)));
            }
          if (section.equals("land"))
            list.add(
                entry(
                    Material.OAK_FENCE,
                    tr(p, "Protect this chunk"),
                    tr(p, "16 × 16 blocks, survival world only"),
                    p.getWorld().getName().equals("living")
                        ? () -> {
                          int x = p.getLocation().getBlockX() >> 4,
                              z = p.getLocation().getBlockZ() >> 4;
                          input(
                              p,
                              tr(p, "Claim name"),
                              name ->
                                  submit(
                                      p,
                                      command(
                                          "claim_create",
                                          "name",
                                          name,
                                          "min_x",
                                          x,
                                          "max_x",
                                          x,
                                          "min_z",
                                          z,
                                          "max_z",
                                          z)));
                        }
                        : null));
          if (section.equals("land"))
            for (JsonElement value : data.getAsJsonArray("claims")) {
              JsonObject c = value.getAsJsonObject();
              list.add(
                  entry(
                      Material.MAP,
                      c.get("name").getAsString(),
                      c.get("chunks") + tr(p, " chunks / ") + c.get("state").getAsString(),
                      () ->
                          confirm(
                              p,
                              tr(p, "Release protection"),
                              c.get("name").getAsString(),
                              () ->
                                  submit(
                                      p,
                                      command("claim_release", "id", c.get("id").getAsString())))));
            }
          if (section.equals("progress"))
            for (JsonElement value : data.getAsJsonArray("achievements")) {
              JsonObject a = value.getAsJsonObject();
              list.add(
                  entry(
                      Material.EXPERIENCE_BOTTLE,
                      systemText(p, a, "title"),
                      systemText(p, a, "description")
                          + "\n"
                          + a.get("progress")
                          + " / "
                          + a.get("target"),
                      null));
            }
          menu(
              p,
              switch (section) {
                case "homes" -> tr(p, "Homes");
                case "land" -> tr(p, "Land");
                default -> tr(p, "Achievements & balance");
              },
              list,
              0);
        });
  }

  private void market(Player p) {
    fetch(
        p,
        "market",
        data -> {
          List<Entry> list = new ArrayList<>();
          list.add(
              entry(
                  Material.BRICKS,
                  tr(p, "Deposit a building or land"),
                  tr(p, "Pack a building or sell it with its land"),
                  () -> captureBuilding(p)));
          if (previews.containsKey(p.getUniqueId()))
            list.add(
                entry(
                    Material.COMPASS,
                    tr(p, "Open last placement preview"),
                    tr(p, "Leave the area before confirming placement"),
                    () -> showPreview(p, previews.get(p.getUniqueId()))));
          list.add(
              entry(
                  Material.CHEST,
                  tr(p, "Deposit held item"),
                  tr(p, "Store and list the stack in your hand"),
                  () ->
                      input(
                          p,
                          tr(p, "Listing name"),
                          name ->
                              confirm(
                                  p,
                                  tr(p, "Deposit item"),
                                  tr(p, "The held stack moves out of your inventory"),
                                  () ->
                                      submit(
                                          p,
                                          command(
                                              "asset_capture",
                                              "kind",
                                              "items",
                                              "title",
                                              name,
                                              "selection",
                                              CoreClient.object(),
                                              "include_contents",
                                              true))))));
          list.add(
              entry(
                  Material.IRON_INGOT,
                  tr(p, "Sell materials"),
                  tr(p, "Remaining today: ") + data.get("npc_remaining") + tr(p, " coins"),
                  () -> {
                    List<Entry> prices = new ArrayList<>();
                    for (JsonElement value : data.getAsJsonArray("prices")) {
                      JsonObject price = value.getAsJsonObject();
                      Material material =
                          Material.matchMaterial(price.get("material").getAsString());
                      prices.add(
                          entry(
                              material == null ? Material.PAPER : material,
                              price.get("material").getAsString(),
                              tr(p, "Each: ") + price.get("price") + tr(p, " coins"),
                              () ->
                                  input(
                                      p,
                                      tr(p, "Quantity to sell"),
                                      text -> {
                                        int n = Integer.parseInt(text);
                                        submit(
                                            p,
                                            command(
                                                "npc_sell",
                                                "material",
                                                price.get("material").getAsString(),
                                                "amount",
                                                n));
                                      })));
                    }
                    menu(p, tr(p, "Material buyback"), prices, 0);
                  }));
          for (JsonElement value : data.getAsJsonArray("assets")) {
            JsonObject a = value.getAsJsonObject();
            if (a.get("state").getAsString().equals("capturing")
                && a.has("manifest_sha256")
                && !a.get("manifest_sha256").isJsonNull()) {
              list.add(
                  entry(
                      Material.WRITABLE_BOOK,
                      tr(p, "Awaiting consent: ") + systemText(p, a, "title"),
                      tr(p, "Review the building and agree or cancel"),
                      () -> {
                        List<Entry> actions = manifestEntries(p, a.getAsJsonObject("manifest"));
                        actions.add(
                            entry(
                                Material.LIME_DYE,
                                tr(p, "Agree to transfer pets"),
                                tr(p, "Pet ownership transfers with this building"),
                                () ->
                                    confirm(
                                        p,
                                        tr(p, "Agree to transfer"),
                                        tr(p, "These pets will be transferred to the buyer"),
                                        () ->
                                            submit(
                                                p,
                                                command(
                                                    "asset_consent",
                                                    "id",
                                                    a.get("id"),
                                                    "manifest_sha256",
                                                    a.get("manifest_sha256"))))));
                        actions.add(
                            entry(
                                Material.BARRIER,
                                tr(p, "Cancel packing"),
                                tr(p, "The owner can cancel a pending consent request"),
                                () -> submit(p, command("asset_withdraw", "id", a.get("id")))));
                        menu(p, tr(p, "Building contents"), actions, 0);
                      }));
            }
            if (!a.get("state").getAsString().equals("escrowed")) continue;
            list.add(
                entry(
                    Material.BARREL,
                    tr(p, "Stored: ") + systemText(p, a, "title"),
                    a.get("kind").getAsString(),
                    () -> {
                      List<Entry> actions = new ArrayList<>();
                      actions.addAll(manifestEntries(p, a.getAsJsonObject("manifest")));
                      if (a.get("kind").getAsString().equals("building"))
                        actions.add(
                            entry(
                                Material.BRICKS,
                                tr(p, "Place building"),
                                tr(p, "Check area and rotation from your position"),
                                () -> placeBuilding(p, a)));
                      if (a.get("kind").getAsString().equals("land"))
                        actions.add(
                            entry(
                                Material.BARRIER,
                                tr(p, "Release deposit"),
                                tr(p, "Allow editing of the claim again"),
                                () -> submit(p, command("asset_withdraw", "id", a.get("id")))));
                      actions.add(
                          entry(
                              Material.EMERALD,
                              tr(p, "Create listing"),
                              tr(p, "5% fee when sold"),
                              () ->
                                  input(
                                      p,
                                      tr(p, "Price (whole coins)"),
                                      price ->
                                          submit(
                                              p,
                                              command(
                                                  "listing_create",
                                                  "asset",
                                                  a.get("id").getAsString(),
                                                  "price",
                                                  Long.parseLong(price))))));
                      if (a.get("kind").getAsString().equals("items"))
                        actions.add(
                            entry(
                                Material.HOPPER,
                                tr(p, "Collect into inventory"),
                                tr(p, "Items stay stored if there is not enough space"),
                                () ->
                                    submit(
                                        p,
                                        command(
                                            "asset_receive", "id", a.get("id").getAsString()))));
                      menu(p, systemText(p, a, "title"), actions, 0);
                    }));
          }
          for (JsonElement value : data.getAsJsonArray("listings")) {
            JsonObject l = value.getAsJsonObject();
            list.add(
                entry(
                    Material.EMERALD,
                    systemText(p, l, "title"),
                    l.get("price") + tr(p, " coins / ") + l.get("seller_name").getAsString(),
                    () ->
                        confirm(
                            p,
                            tr(p, "Buy"),
                            systemText(p, l, "title") + " / " + l.get("price") + tr(p, " coins"),
                            () ->
                                submit(
                                    p, command("listing_buy", "id", l.get("id").getAsString())))));
          }
          menu(p, tr(p, "Market"), list, 0);
        });
  }

  private void selectPoint(Player p, boolean second) {
    if (!ctx.official())
      throw new IllegalArgumentException(tr(p, "Select the area in the official SMP."));
    org.bukkit.block.Block target = p.getTargetBlockExact(8);
    if (target == null)
      throw new IllegalArgumentException(tr(p, "Look at a building corner within eight blocks."));
    var actor = com.sk89q.worldedit.bukkit.BukkitAdapter.adapt(p);
    var session = com.sk89q.worldedit.WorldEdit.getInstance().getSessionManager().get(actor);
    var selector =
        session.getRegionSelector(com.sk89q.worldedit.bukkit.BukkitAdapter.adapt(p.getWorld()));
    var point =
        com.sk89q.worldedit.math.BlockVector3.at(target.getX(), target.getY(), target.getZ());
    var limit =
        com.sk89q.worldedit.extension.platform.permission.ActorSelectorLimits.forActor(actor);
    if (second) selector.selectSecondary(point, limit);
    else selector.selectPrimary(point, limit);
    inform(
        p,
        (second ? "2" : "1")
            + tr(p, " point: ")
            + target.getX()
            + ", "
            + target.getY()
            + ", "
            + target.getZ());
  }

  private List<Entry> manifestEntries(Player p, JsonObject manifest) {
    List<Entry> entries = new ArrayList<>();
    if (manifest == null) return entries;
    if (manifest.has("blocks"))
      entries.add(
          entry(
              Material.BRICKS,
              tr(p, "Building: ") + manifest.get("blocks") + tr(p, " blocks"),
              String.valueOf(manifest.get("dimensions")),
              null));
    if (manifest.has("materials"))
      for (var item : manifest.getAsJsonObject("materials").entrySet()) {
        Material material = Material.matchMaterial(item.getKey());
        entries.add(
            entry(
                material == null ? Material.PAPER : material,
                item.getKey(),
                item.getValue() + tr(p, " items"),
                null));
      }
    if (manifest.has("containers"))
      for (JsonElement item : manifest.getAsJsonArray("containers")) {
        JsonObject row = item.getAsJsonObject();
        Material material = Material.matchMaterial(row.get("material").getAsString());
        entries.add(
            entry(
                material == null ? Material.CHEST : material,
                tr(p, "Containers: ") + row.get("material").getAsString(),
                row.get("amount") + tr(p, " items / ") + CoreClient.string(row, "at", ""),
                null));
      }
    if (manifest.has("entities"))
      for (JsonElement item : manifest.getAsJsonArray("entities")) {
        JsonObject row = item.getAsJsonObject();
        entries.add(
            entry(
                Material.NAME_TAG,
                CoreClient.string(row, "name", row.get("type").getAsString()),
                row.get("type").getAsString()
                    + "\n"
                    + (row.has("owner") && !row.get("owner").isJsonNull()
                        ? tr(p, "Pet owner confirmation required")
                        : tr(p, "Moves with the building")),
                null));
      }
    return entries;
  }

  private void captureBuilding(Player p) {
    fetch(
        p,
        "life",
        data -> {
          List<Entry> options = new ArrayList<>();
          options.add(
              entry(
                  Material.COMPASS,
                  tr(p, "Select first corner"),
                  tr(p, "Look at a corner and click. Command: /lkjmc pos1"),
                  () -> {
                    p.closeInventory();
                    selectPoint(p, false);
                  }));
          options.add(
              entry(
                  Material.COMPASS,
                  tr(p, "Select second corner"),
                  tr(p, "Look at the opposite corner. Command: /lkjmc pos2"),
                  () -> {
                    p.closeInventory();
                    selectPoint(p, true);
                  }));
          for (JsonElement value : data.getAsJsonArray("claims")) {
            JsonObject claim = value.getAsJsonObject();
            if (!claim.get("state").getAsString().equals("active")) continue;
            options.add(
                entry(
                    Material.GRASS_BLOCK,
                    claim.get("name").getAsString(),
                    tr(p, "Pack or deposit with land in this claim"),
                    () -> {
                      List<Entry> kinds = new ArrayList<>();
                      for (String kind : List.of("building", "land"))
                        kinds.add(
                            entry(
                                Material.BRICKS,
                                kind.equals("building")
                                    ? tr(p, "Pack selected building")
                                    : tr(p, "Sell land with buildings"),
                                kind.equals("building")
                                    ? tr(p, "Remove the original and create a one-use asset")
                                    : tr(p, "Freeze editing until sale or cancellation"),
                                () ->
                                    input(
                                        p,
                                        tr(p, "Building or land name"),
                                        title -> {
                                          List<Entry> contents = new ArrayList<>();
                                          for (boolean include : List.of(false, true))
                                            contents.add(
                                                entry(
                                                    Material.CHEST,
                                                    include
                                                        ? tr(p, "Include container contents")
                                                        : tr(
                                                            p,
                                                            "Empty containers before depositing"),
                                                    tr(p, "Everyone must leave the selected area"),
                                                    () ->
                                                        confirm(
                                                            p,
                                                            tr(p, "Confirm deposit"),
                                                            tr(
                                                                p,
                                                                "Safely save the structure,"
                                                                    + " containers and entities"),
                                                            () ->
                                                                submit(
                                                                    p,
                                                                    command(
                                                                        "asset_capture",
                                                                        "kind",
                                                                        kind,
                                                                        "owner",
                                                                        claim.get("owner"),
                                                                        "title",
                                                                        title,
                                                                        "selection",
                                                                        CoreClient.object(
                                                                            "claim_id",
                                                                            claim.get("id")),
                                                                        "include_contents",
                                                                        include)))));
                                          menu(p, tr(p, "Container contents"), contents, 0);
                                        })));
                      menu(p, tr(p, "Deposit method"), kinds, 0);
                    }));
          }
          menu(p, tr(p, "Buildings & land"), options, 0);
        });
  }

  private void placeBuilding(Player p, JsonObject asset) {
    if (claims == null) {
      inform(p, tr(p, "Choose a placement location in the official SMP."));
      return;
    }
    Location point = p.getLocation();
    JsonObject claim = claims.claim(point.getBlock());
    if (claim == null) {
      inform(p, tr(p, "Stand inside the destination claim first."));
      return;
    }
    List<Entry> rotations = new ArrayList<>();
    for (int rotation : List.of(0, 90, 180, 270))
      rotations.add(
          entry(
              Material.COMPASS,
              rotation + tr(p, " degrees"),
              tr(p, "Origin: ")
                  + point.getBlockX()
                  + ", "
                  + point.getBlockY()
                  + ", "
                  + point.getBlockZ(),
              () -> {
                JsonObject placement =
                    CoreClient.object(
                        "claim_id",
                        claim.get("id"),
                        "x",
                        point.getBlockX(),
                        "y",
                        point.getBlockY(),
                        "z",
                        point.getBlockZ(),
                        "rotation",
                        rotation,
                        "preview",
                        true);
                inform(
                    p,
                    tr(
                        p,
                        "You cannot place while inside the area. Preview, move outside, then"
                            + " confirm."));
                submit(
                    p,
                    command("asset_place", "id", asset.get("id"), "placement", placement),
                    preview -> {
                      PlacementPreview saved =
                          new PlacementPreview(
                              asset.deepCopy(), placement.deepCopy(), preview.deepCopy());
                      previews.put(p.getUniqueId(), saved);
                      showPreview(p, saved);
                    });
              }));
    menu(p, tr(p, "Building rotation"), rotations, 0);
  }

  private void showPreview(Player p, PlacementPreview saved) {
    if (saved == null) return;
    JsonObject preview = saved.result(), placement = saved.placement().deepCopy();
    JsonObject box = preview.getAsJsonObject("footprint");
    String description =
        "X "
            + box.get("min_x")
            + "〜"
            + box.get("max_x")
            + " / Y "
            + box.get("min_y")
            + "〜"
            + box.get("max_y")
            + " / Z "
            + box.get("min_z")
            + "〜"
            + box.get("max_z")
            + tr(p, "\nRotation: ")
            + preview.get("rotation")
            + tr(p, " degrees\n")
            + preview.get("message").getAsString();
    List<Entry> actions = new ArrayList<>();
    actions.add(
        entry(Material.PAPER, tr(p, "Placement area"), description, () -> inform(p, description)));
    actions.add(
        entry(
            Material.LIME_CONCRETE,
            tr(p, "Place in this area"),
            tr(p, "Clear the area first. It is checked again before placement"),
            () -> {
              placement.addProperty("preview", false);
              placement.add("preview_hash", preview.get("preview_hash"));
              submit(
                  p,
                  command("asset_place", "id", saved.asset().get("id"), "placement", placement),
                  done -> previews.remove(p.getUniqueId()));
            }));
    actions.add(
        entry(
            Material.OAK_DOOR,
            tr(p, "Close and move outside the area"),
            tr(p, "Return to the preview from the Market menu"),
            p::closeInventory));
    menu(p, tr(p, "Placement preview"), actions, 0);
  }

  private void social(Player p, String section) {
    fetch(
        p,
        "social",
        data -> {
          List<Entry> list = new ArrayList<>();
          if (section.equals("friends")) {
            list.add(
                entry(
                    Material.PLAYER_HEAD,
                    tr(p, "Add friend"),
                    tr(p, "The other player must accept"),
                    () ->
                        choosePlayer(
                            p,
                            tr(p, "Friend request"),
                            other ->
                                submit(
                                    p,
                                    command(
                                        "friend_request",
                                        "target",
                                        other.get("id").getAsString())))));
            list.add(
                entry(
                    Material.ENDER_PEARL,
                    tr(p, "Meet up"),
                    tr(p, "Ask permission to teleport to the player"),
                    () ->
                        choosePlayer(
                            p,
                            tr(p, "Teleport request"),
                            other ->
                                submit(
                                    p,
                                    command(
                                        "teleport_request",
                                        "target",
                                        other.get("id").getAsString())))));
          }
          if (section.equals("team") && data.get("team").isJsonNull()) {
            list.add(
                entry(
                    Material.WHITE_BANNER,
                    tr(p, "Create team"),
                    tr(p, "One team per account, with shared land and coins"),
                    () ->
                        input(
                            p,
                            tr(p, "Team name"),
                            name -> submit(p, command("team_create", "name", name)))));
          }
          if (section.equals("party") && data.get("party").isJsonNull()) {
            list.add(
                entry(
                    Material.CAMPFIRE,
                    tr(p, "Create party"),
                    tr(p, "A temporary group for playing together"),
                    () ->
                        input(
                            p,
                            tr(p, "Party name"),
                            name -> submit(p, command("party_create", "name", name)))));
          }
          if (section.equals("chat")) {
            list.add(
                entry(
                    Material.WRITABLE_BOOK,
                    tr(p, "Create group chat"),
                    "",
                    () ->
                        input(
                            p,
                            tr(p, "Group name"),
                            name -> submit(p, command("room_create", "name", name)))));
          }
          if (section.equals("friends")) {
            for (JsonElement value : data.getAsJsonArray("friends")) {
              JsonObject friend = value.getAsJsonObject();
              list.add(
                  entry(
                      Material.PLAYER_HEAD,
                      friend.get("name").getAsString(),
                      friendState(p, friend),
                      () -> friend(p, friend)));
            }
          }
          for (String key : List.of("team", "party"))
            if (section.equals(key) && !data.get(key).isJsonNull()) {
              JsonObject group = data.getAsJsonObject(key);
              list.add(
                  entry(
                      Material.BELL,
                      group.get("name").getAsString(),
                      key.equals("party") ? tr(p, "Temporary party") : tr(p, "Lasting team"),
                      () -> {
                        List<Entry> actions = new ArrayList<>();
                        actions.add(
                            entry(
                                Material.PLAYER_HEAD,
                                tr(p, "Invite members"),
                                "",
                                () ->
                                    choosePlayer(
                                        p,
                                        tr(p, "Invite"),
                                        other ->
                                            submit(
                                                p,
                                                command(
                                                    "invite",
                                                    "kind",
                                                    key,
                                                    "resource",
                                                    group.get("id").getAsString(),
                                                    "target",
                                                    other.get("id").getAsString())))));
                        if (key.equals("party"))
                          actions.add(
                              entry(
                                  Material.LIME_DYE,
                                  tr(p, "Ready for Expedition"),
                                  tr(
                                      p,
                                      "Agree to join the next Expedition before preparation"
                                          + " commits"),
                                  () -> submit(p, command("party_ready", "ready", true))));
                        if (key.equals("party"))
                          actions.add(
                              entry(
                                  Material.GRAY_DYE,
                                  tr(p, "Withdraw next Expedition consent"),
                                  tr(p, "Committed Expeditions keep their participant roster."),
                                  () -> submit(p, command("party_ready", "ready", false))));
                        if (group.has("members"))
                          for (JsonElement value : group.getAsJsonArray("members")) {
                            JsonObject member = value.getAsJsonObject();
                            actions.add(
                                entry(
                                    Material.PLAYER_HEAD,
                                    member.get("name").getAsString(),
                                    key.equals("party")
                                        ? (flag(member, "ready")
                                            ? tr(p, "Ready")
                                            : tr(p, "Consent needed"))
                                        : tr(p, "Team member"),
                                    null));
                          }
                        actions.add(
                            entry(
                                Material.OAK_DOOR,
                                tr(p, "Leave group"),
                                tr(p, "Leaders must transfer leadership first"),
                                () ->
                                    confirm(
                                        p,
                                        tr(p, "Leave"),
                                        group.get("name").getAsString(),
                                        () -> submit(p, command(key + "_leave")))));
                        menu(p, group.get("name").getAsString(), actions, 0);
                      }));
            }
          if (section.equals("chat")) {
            for (JsonElement value : data.getAsJsonArray("rooms")) {
              JsonObject room = value.getAsJsonObject();
              list.add(
                  entry(
                      Material.WRITABLE_BOOK,
                      room.get("name").getAsString(),
                      tr(p, "Unread: ") + room.get("unread"),
                      () -> chat(p, room)));
            }
          }
          menu(
              p,
              switch (section) {
                case "team" -> tr(p, "Teams");
                case "party" -> tr(p, "Parties");
                case "chat" -> tr(p, "Conversations");
                default -> tr(p, "Friends");
              },
              list,
              0);
        });
  }

  private String friendState(Player p, JsonObject friend) {
    if (CoreClient.string(friend, "state", "pending").equals("accepted")) return tr(p, "Friends");
    try {
      return CoreClient.string(friend, "requester", "")
              .equals(ctx.session(p.getUniqueId()).get("account_id").getAsString())
          ? tr(p, "Request sent")
          : tr(p, "Friend request");
    } catch (Exception ignored) {
      return tr(p, "Friend request");
    }
  }

  private void friend(Player p, JsonObject friend) {
    String id = friend.get("id").getAsString();
    boolean accepted = CoreClient.string(friend, "state", "pending").equals("accepted");
    boolean outgoing;
    try {
      outgoing =
          CoreClient.string(friend, "requester", "")
              .equals(ctx.session(p.getUniqueId()).get("account_id").getAsString());
    } catch (Exception e) {
      error(p, e);
      return;
    }
    List<Entry> entries = new ArrayList<>();
    if (!accepted && !outgoing) {
      entries.add(
          entry(
              Material.LIME_DYE,
              tr(p, "Accept friend request"),
              "",
              () -> submit(p, command("friend_respond", "target", id, "accept", true))));
      entries.add(
          entry(
              Material.GRAY_DYE,
              tr(p, "Decline friend request"),
              "",
              () -> submit(p, command("friend_respond", "target", id, "accept", false))));
    }
    if (accepted)
      entries.add(
          entry(
              Material.WRITABLE_BOOK,
              tr(p, "Open DM"),
              "",
              () ->
                  submit(
                      p,
                      command("direct_room", "target", id),
                      result ->
                          chat(
                              p,
                              CoreClient.object(
                                  "id", result.get("room_id"), "name", friend.get("name"))))));
    entries.add(
        entry(
            Material.BARRIER,
            accepted ? tr(p, "Remove friend") : tr(p, "Cancel friend request"),
            "",
            () ->
                confirm(
                    p,
                    accepted ? tr(p, "Remove friend") : tr(p, "Cancel friend request"),
                    friend.get("name").getAsString(),
                    () -> submit(p, command("friend_remove", "target", id)))));
    entries.add(
        entry(
            Material.RED_DYE,
            tr(p, "Block player"),
            "",
            () ->
                confirm(
                    p,
                    tr(p, "Block player"),
                    friend.get("name").getAsString(),
                    () -> submit(p, command("block", "target", id, "blocked", true)))));
    menu(p, friend.get("name").getAsString(), entries, 0);
  }

  private void chat(Player p, JsonObject room) {
    fetch(
        p,
        "messages",
        CoreClient.object("room", room.get("id").getAsString()),
        history -> {
          List<Entry> entries = new ArrayList<>();
          entries.add(
              entry(
                  Material.WRITABLE_BOOK,
                  tr(p, "Send message"),
                  "",
                  () ->
                      input(
                          p,
                          tr(p, "Message"),
                          body ->
                              submit(
                                  p,
                                  command(
                                      "message_send",
                                      "room",
                                      room.get("id").getAsString(),
                                      "body",
                                      body)))));
          entries.add(
              entry(
                  Material.PLAYER_HEAD,
                  tr(p, "Invite members"),
                  "",
                  () ->
                      choosePlayer(
                          p,
                          tr(p, "Invite"),
                          other ->
                              submit(
                                  p,
                                  command(
                                      "invite",
                                      "kind",
                                      "room",
                                      "resource",
                                      room.get("id").getAsString(),
                                      "target",
                                      other.get("id").getAsString())))));
          for (JsonElement element : history.getAsJsonArray("messages")) {
            JsonObject message = element.getAsJsonObject();
            entries.add(
                entry(
                    Material.PAPER,
                    message.get("author_name").getAsString(),
                    message.get("deleted_at").isJsonNull()
                        ? message.get("body").getAsString()
                        : tr(p, "Deleted message"),
                    () -> inform(p, message.get("body").getAsString())));
          }
          menu(p, room.get("name").getAsString(), entries, 0);
        });
  }

  private boolean flag(JsonObject item, String key) {
    return item.has(key) && !item.get(key).isJsonNull() && item.get(key).getAsBoolean();
  }

  private String expeditionState(Player p, JsonObject expedition) {
    return switch (CoreClient.string(expedition, "state", "unknown")) {
      case "preparing", "reserved", "queued" -> tr(p, "Preparing");
      case "active" -> tr(p, "Open");
      case "closing" -> tr(p, "Closing");
      case "closed", "completed" -> tr(p, "Completed");
      case "cancelled" -> tr(p, "Cancelled");
      case "failed" -> tr(p, "Failed");
      default -> tr(p, "Checking availability");
    };
  }

  private String expeditionTitle(Player p, JsonObject expedition) {
    String id = expedition.get("id").getAsString();
    return tr(p, "End Expedition · {0}", id.substring(0, Math.min(8, id.length())));
  }

  private String expeditionSummary(Player p, JsonObject expedition) {
    String detail =
        tr(p, "The End · Temporary world · Participants only")
            + "\n"
            + expeditionState(p, expedition);
    if (CoreClient.string(expedition, "state", "").equals("active")
        && expedition.has("remaining_seconds")
        && !expedition.get("remaining_seconds").isJsonNull())
      detail +=
          "\n"
              + tr(
                  p,
                  "{0} minutes remaining",
                  Math.max(0, (expedition.get("remaining_seconds").getAsLong() + 59) / 60));
    else if (expedition.has("expires_at") && !expedition.get("expires_at").isJsonNull())
      detail += "\n" + tr(p, "Scheduled end: {0}", date(p, expedition.get("expires_at")));
    return detail;
  }

  private String date(Player p, JsonElement value) {
    try {
      return DateTimeFormatter.ofLocalizedDateTime(FormatStyle.MEDIUM)
              .withLocale(Locale.forLanguageTag(language(p)))
              .withZone(ZoneOffset.UTC)
              .format(Instant.parse(value.getAsString()))
          + " UTC";
    } catch (Exception ignored) {
      return tr(p, "Time unavailable");
    }
  }

  private String returnConsequences(Player p) {
    return tr(
        p,
        "Keep the items you carry. Placed blocks, containers and dropped items disappear when the"
            + " world closes.");
  }

  private void expeditions(Player p) {
    fetch(
        p,
        "expedition",
        data -> {
          List<Entry> entries = new ArrayList<>();
          entries.add(
              entry(
                  Material.ENDER_EYE,
                  tr(p, "Begin Expedition"),
                  preparationCost(p, data),
                  () -> preparation(p, data)));
          for (JsonElement value : data.getAsJsonArray("expeditions")) {
            JsonObject expedition = value.getAsJsonObject();
            entries.add(
                entry(
                    flag(expedition, "can_enter") ? Material.END_STONE : Material.BOOK,
                    expeditionTitle(p, expedition),
                    expeditionSummary(p, expedition),
                    () -> expedition(p, expedition)));
          }
          menu(p, tr(p, "Expeditions"), entries, 0);
        });
  }

  private String preparationCost(Player p, JsonObject data) {
    JsonObject cost = data.getAsJsonObject("cost");
    return tr(
        p,
        "{0} coins + {1} Eyes of Ender · {2} minutes from activation",
        cost.get("coins").getAsLong(),
        cost.get("ender_eyes").getAsInt(),
        data.get("duration_seconds").getAsLong() / 60);
  }

  private void preparation(Player p, JsonObject data) {
    JsonObject preparation = data.getAsJsonObject("preparation");
    boolean ready = preparation != null && flag(preparation, "can_prepare");
    List<Entry> entries = new ArrayList<>();
    entries.add(
        entry(
            Material.BOOK,
            tr(p, "Temporary world"),
            tr(p, "The End · Temporary world · Participants only")
                + "\n"
                + preparationCost(p, data)
                + "\n"
                + returnConsequences(p),
            null));
    entries.add(
        entry(
            ready ? Material.LIME_CONCRETE : Material.GRAY_CONCRETE,
            tr(p, "Prepare Expedition"),
            ready
                ? tr(
                    p,
                    "Participant consent is committed now. Party changes afterward do not remove"
                        + " participants.")
                : tr(p, "Resolve the requirements below, then refresh."),
            ready
                ? () ->
                    confirm(
                        p,
                        tr(p, "Prepare Expedition"),
                        preparationCost(p, data)
                            + "\n"
                            + tr(
                                p,
                                "Participant consent is committed now. Party changes afterward do"
                                    + " not remove participants."),
                        () -> submit(p, command("expedition_prepare"), ignored -> expeditions(p)))
                : null));
    if (preparation != null) {
      if (!flag(preparation, "is_leader"))
        entries.add(
            entry(
                Material.WHITE_BANNER,
                tr(p, "Party leader required"),
                tr(p, "Ask your party leader to prepare this Expedition."),
                null));
      if (preparation.has("available_coins")
          && preparation.get("available_coins").getAsLong()
              < data.getAsJsonObject("cost").get("coins").getAsLong())
        entries.add(
            entry(
                Material.GOLD_INGOT,
                tr(p, "More coins needed"),
                tr(p, "Available: {0} coins", preparation.get("available_coins").getAsLong()),
                null));
      if (preparation.has("participants"))
        for (JsonElement value : preparation.getAsJsonArray("participants")) {
          JsonObject participant = value.getAsJsonObject();
          List<String> requirements = new ArrayList<>();
          if (!flag(participant, "ready")) requirements.add(tr(p, "Consent needed"));
          if (!flag(participant, "online")) requirements.add(tr(p, "Must be online in SMP"));
          if (flag(participant, "in_combat")) requirements.add(tr(p, "Wait until combat ends"));
          if (flag(participant, "occupied"))
            requirements.add(tr(p, "Finish the current Expedition first"));
          entries.add(
              entry(
                  requirements.isEmpty() ? Material.LIME_DYE : Material.GRAY_DYE,
                  participant.get("name").getAsString(),
                  requirements.isEmpty() ? tr(p, "Ready") : String.join("\n", requirements),
                  null));
        }
    }
    entries.add(
        entry(
            Material.CAMPFIRE,
            tr(p, "Party readiness"),
            tr(p, "Review your party and give consent"),
            () -> social(p, "party")));
    entries.add(
        entry(
            Material.CLOCK,
            tr(p, "Refresh requirements"),
            "",
            () -> fetch(p, "expedition", refreshed -> preparation(p, refreshed))));
    menu(p, tr(p, "Prepare Expedition"), entries, 0);
  }

  private void expedition(Player p, JsonObject expedition) {
    String id = expedition.get("id").getAsString();
    List<Entry> entries = new ArrayList<>();
    entries.add(
        entry(
            Material.BOOK,
            tr(p, "Expedition journal"),
            expeditionSummary(p, expedition)
                + (expedition.has("opens_at") && !expedition.get("opens_at").isJsonNull()
                    ? "\n" + tr(p, "Opened: {0}", date(p, expedition.get("opens_at")))
                    : "")
                + "\n"
                + returnConsequences(p),
            null));
    if (flag(expedition, "can_enter"))
      entries.add(
          entry(
              Material.END_STONE,
              tr(p, "Enter Expedition"),
              tr(p, "Your return position is saved before you enter."),
              () -> submit(p, command("expedition_enter", "id", id))));
    if (canReturn(p, expedition))
      entries.add(
          entry(
              Material.OAK_DOOR,
              tr(p, "Return to SMP"),
              tr(p, "Return to your saved position when safe, otherwise a safe SMP spawn.")
                  + "\n"
                  + returnConsequences(p),
              () -> submit(p, command("expedition_return", "id", id))));
    if (flag(expedition, "can_cancel"))
      entries.add(
          entry(
              Material.BARRIER,
              tr(p, "Cancel preparation"),
              tr(p, "Release reserved coins and return reserved materials to storage"),
              () ->
                  confirm(
                      p,
                      tr(p, "Cancel preparation"),
                      tr(p, "Release reserved coins and return reserved materials to storage"),
                      () ->
                          submit(
                              p,
                              command("expedition_cancel", "id", id),
                              ignored -> expeditions(p)))));
    if (flag(expedition, "can_receive"))
      entries.add(
          entry(
              Material.ENDER_EYE,
              tr(p, "Collect refunded items"),
              tr(p, "Make room in your inventory first"),
              () ->
                  submit(
                      p,
                      command(
                          "asset_receive", "id", expedition.get("material_asset").getAsString()))));
    if (expedition.has("participants"))
      for (JsonElement value : expedition.getAsJsonArray("participants")) {
        JsonObject participant = value.getAsJsonObject();
        entries.add(
            entry(
                Material.PLAYER_HEAD,
                participant.get("name").getAsString(),
                tr(p, "Committed participant"),
                null));
      }
    entries.add(
        entry(
            Material.CLOCK,
            tr(p, "Refresh Expedition"),
            "",
            () ->
                fetch(
                    p,
                    "expedition",
                    data -> {
                      for (JsonElement value : data.getAsJsonArray("expeditions")) {
                        JsonObject refreshed = value.getAsJsonObject();
                        if (refreshed.get("id").getAsString().equals(id)) {
                          expedition(p, refreshed);
                          return;
                        }
                      }
                      expeditions(p);
                    })));
    menu(p, expeditionTitle(p, expedition), entries, 0);
  }

  private void returns(Player p) {
    fetch(
        p,
        "expedition",
        data -> {
          List<Entry> entries = new ArrayList<>();
          for (JsonElement value : data.getAsJsonArray("expeditions")) {
            JsonObject expedition = value.getAsJsonObject();
            if (!canReturn(p, expedition)) continue;
            entries.add(
                entry(
                    Material.OAK_DOOR,
                    expeditionTitle(p, expedition),
                    returnConsequences(p),
                    () ->
                        submit(
                            p,
                            command(
                                "expedition_return", "id", expedition.get("id").getAsString()))));
          }
          if (entries.isEmpty())
            entries.add(
                entry(
                    Material.GRASS_BLOCK,
                    tr(p, "No Expedition return needed"),
                    tr(p, "There is no Expedition to return from."),
                    null));
          menu(p, tr(p, "Return to SMP"), entries, 0);
        });
  }

  private boolean canReturn(Player p, JsonObject expedition) {
    return flag(expedition, "can_return")
        && p.getWorld().getName().equals("adventure_" + expedition.get("id").getAsString());
  }

  private void notifications(Player p) {
    fetch(
        p,
        "home",
        data -> {
          List<Entry> list = new ArrayList<>();
          for (JsonElement value : data.getAsJsonArray("invitations")) {
            JsonObject invite = value.getAsJsonObject();
            list.add(
                entry(
                    Material.PAPER,
                    invite.get("sender_name").getAsString() + tr(p, " invited you"),
                    invitationKind(p, CoreClient.string(invite, "kind", "")),
                    () ->
                        menu(
                            p,
                            tr(p, "Respond to invitation"),
                            List.of(
                                entry(
                                    Material.LIME_DYE,
                                    tr(p, "Accept"),
                                    "",
                                    () ->
                                        submit(
                                            p,
                                            command(
                                                "invite_respond",
                                                "id",
                                                invite.get("id").getAsString(),
                                                "accept",
                                                true))),
                                entry(
                                    Material.GRAY_DYE,
                                    tr(p, "Decline"),
                                    "",
                                    () ->
                                        submit(
                                            p,
                                            command(
                                                "invite_respond",
                                                "id",
                                                invite.get("id").getAsString(),
                                                "accept",
                                                false)))),
                            0)));
          }
          for (JsonElement value : data.getAsJsonArray("jobs")) {
            JsonObject job = value.getAsJsonObject();
            list.add(
                entry(
                    Material.CLOCK,
                    operationTitle(p, CoreClient.string(job, "kind", "")),
                    operationState(p, CoreClient.string(job, "state", ""))
                        + (job.has("error") && !job.get("error").isJsonNull()
                            ? "\n" + tr(p, "Check notifications for details.")
                            : ""),
                    null));
          }
          menu(p, tr(p, "Invitations & activity"), list, 0);
        });
  }

  private String invitationKind(Player p, String kind) {
    return switch (kind) {
      case "team" -> tr(p, "Team");
      case "party" -> tr(p, "Party");
      case "room" -> tr(p, "Group conversation");
      default -> tr(p, "Invitation");
    };
  }

  private String operationTitle(Player p, String kind) {
    if (kind.startsWith("adventure.")) return tr(p, "Expedition");
    if (kind.equals("server.join")) return tr(p, "World travel");
    if (kind.startsWith("asset.") || kind.startsWith("listing.") || kind.startsWith("material."))
      return tr(p, "Market");
    if (kind.startsWith("home.")) return tr(p, "Homes");
    if (kind.startsWith("claim.")) return tr(p, "Land");
    return tr(p, "Recent action");
  }

  private String operationState(Player p, String state) {
    return switch (state) {
      case "queued", "waiting" -> tr(p, "Waiting");
      case "leased" -> tr(p, "In progress");
      case "succeeded" -> tr(p, "Completed");
      case "failed" -> tr(p, "Failed");
      case "cancelled" -> tr(p, "Cancelled");
      case "delivery_unknown" -> tr(p, "Delivery could not be confirmed");
      default -> tr(p, "Checking progress");
    };
  }

  private boolean currentPlayer(Player player) {
    return player.isOnline() && Bukkit.getPlayer(player.getUniqueId()) == player;
  }

  private UUID sessionId(Player player) {
    try {
      return CoreClient.uuid(ctx.session(player.getUniqueId()), "session_id");
    } catch (Exception e) {
      return null;
    }
  }

  private boolean validPlayer(Player player, UUID session) {
    return session != null && currentPlayer(player) && session.equals(sessionId(player));
  }

  private boolean lobbyLauncher() {
    return ctx.plugin().getConfig().getString("role", "official").equals("lobby");
  }

  private NamespacedKey launcherKey() {
    return new NamespacedKey(ctx.plugin(), "menu_launcher");
  }

  private boolean launcher(ItemStack item) {
    if (item == null
        || !Set.of(Material.BOOK, Material.COMPASS).contains(item.getType())
        || !item.hasItemMeta()) return false;
    Byte token =
        item.getItemMeta()
            .getPersistentDataContainer()
            .get(launcherKey(), org.bukkit.persistence.PersistentDataType.BYTE);
    return token != null && token == 1;
  }

  private void installLauncher(Player player) {
    if (!lobbyLauncher() || !currentPlayer(player) || sessionId(player) == null) return;
    PlayerInventory inventory = player.getInventory();
    int owned = -1;
    for (int slot = 0; slot < inventory.getSize(); slot++) {
      if (!launcher(inventory.getItem(slot))) continue;
      if (owned < 0) owned = slot;
      else inventory.setItem(slot, null); // Only our PDC tokens are deduplicated.
    }
    if (launcher(player.getItemOnCursor())) {
      if (owned >= 0) player.setItemOnCursor(null);
      else return; // Do not mint another token while one is on the cursor.
    }
    if (owned < 0) {
      ItemStack existing = inventory.getItem(8);
      owned = existing == null || existing.getType().isAir() ? 8 : inventory.firstEmpty();
      if (owned < 0) return; // A full ordinary inventory is preserved; /menu still works.
    }
    ItemStack item = new ItemStack(Material.BOOK);
    item.editMeta(
        meta -> {
          meta.displayName(Component.text(tr(player, "Game menu"), NamedTextColor.AQUA));
          meta.lore(
              List.of(
                  Component.text(
                      tr(
                          player,
                          "Left/right-click or click this item in your inventory to open the menu"),
                      NamedTextColor.GRAY)));
          meta.getPersistentDataContainer()
              .set(launcherKey(), org.bukkit.persistence.PersistentDataType.BYTE, (byte) 1);
        });
    inventory.setItem(owned, item);
  }

  void installSoon(Player player) {
    UUID session = sessionId(player);
    Bukkit.getScheduler()
        .runTask(
            ctx.plugin(),
            () -> {
              // The same Player instance must still be online. If projection
              // populated its session after the join callback, validate it now.
              if (currentPlayer(player) && (session == null || validPlayer(player, session)))
                installLauncher(player);
            });
  }

  void localeChanged(Player player) {
    if (!currentPlayer(player)) return;
    boolean open = player.getOpenInventory().getTopInventory().getHolder() instanceof Menu;
    requests.merge(player.getUniqueId(), 1L, Long::sum);
    inputs.remove(player.getUniqueId());
    history.remove(player.getUniqueId());
    current.remove(player.getUniqueId());
    selectedLanguages.remove(player.getUniqueId());
    installLauncher(player);
    if (open) root(player);
  }

  private void openSoon(Player player, java.util.function.BooleanSupplier revalidate) {
    if (!lobbyLauncher()
        || ctx.departing(player.getUniqueId())
        || ctx.mustIsolate(player.getUniqueId())) return;
    UUID id = player.getUniqueId(), session = sessionId(player);
    long now = System.nanoTime();
    if (session == null
        || launcherPending.containsKey(id)
        || now - launcherOpens.getOrDefault(id, 0L) < 300_000_000L) return;
    launcherPending.put(id, session);
    launcherOpens.put(id, now);
    Inventory top = player.getOpenInventory().getTopInventory();
    Bukkit.getScheduler()
        .runTask(
            ctx.plugin(),
            () -> {
              launcherPending.remove(id, session);
              if (validPlayer(player, session)
                  && player.getOpenInventory().getTopInventory() == top
                  && !ctx.departing(id)
                  && !ctx.mustIsolate(id)
                  && revalidate.getAsBoolean()) root(player);
            });
  }

  @EventHandler
  public void joined(org.bukkit.event.player.PlayerJoinEvent event) {
    installSoon(event.getPlayer());
  }

  @EventHandler
  public void respawned(org.bukkit.event.player.PlayerRespawnEvent event) {
    installSoon(event.getPlayer());
  }

  @EventHandler(priority = EventPriority.HIGHEST)
  public void died(org.bukkit.event.entity.PlayerDeathEvent event) {
    if (lobbyLauncher()) event.getDrops().removeIf(this::launcher);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = false)
  public void openLauncher(org.bukkit.event.player.PlayerInteractEvent event) {
    if (!lobbyLauncher() || event.getHand() == null || !launcher(event.getItem())) return;
    if (!Set.of(
            org.bukkit.event.block.Action.LEFT_CLICK_AIR,
            org.bukkit.event.block.Action.LEFT_CLICK_BLOCK,
            org.bukkit.event.block.Action.RIGHT_CLICK_AIR,
            org.bukkit.event.block.Action.RIGHT_CLICK_BLOCK)
        .contains(event.getAction())) return;
    event.setCancelled(true);
    Player player = event.getPlayer();
    org.bukkit.inventory.EquipmentSlot hand = event.getHand();
    openSoon(player, () -> launcher(player.getInventory().getItem(hand)));
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = false)
  public void swingLauncher(org.bukkit.event.player.PlayerAnimationEvent event) {
    if (!lobbyLauncher()) return;
    org.bukkit.inventory.EquipmentSlot hand =
        event.getAnimationType() == org.bukkit.event.player.PlayerAnimationType.OFF_ARM_SWING
            ? org.bukkit.inventory.EquipmentSlot.OFF_HAND
            : org.bukkit.inventory.EquipmentSlot.HAND;
    Player player = event.getPlayer();
    if (!launcher(player.getInventory().getItem(hand))) return;
    event.setCancelled(true);
    openSoon(player, () -> launcher(player.getInventory().getItem(hand)));
  }

  private void entityLauncher(org.bukkit.event.player.PlayerInteractEntityEvent event) {
    if (!lobbyLauncher() || !launcher(event.getPlayer().getInventory().getItem(event.getHand())))
      return;
    event.setCancelled(true);
    openSoon(
        event.getPlayer(),
        () -> launcher(event.getPlayer().getInventory().getItem(event.getHand())));
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = false)
  public void interactEntity(org.bukkit.event.player.PlayerInteractEntityEvent event) {
    entityLauncher(event);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = false)
  public void interactAtEntity(org.bukkit.event.player.PlayerInteractAtEntityEvent event) {
    entityLauncher(event);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = false)
  public void attackEntity(org.bukkit.event.entity.EntityDamageByEntityEvent event) {
    if (lobbyLauncher()
        && event.getDamager() instanceof Player player
        && launcher(player.getInventory().getItemInMainHand())) {
      event.setCancelled(true);
      openSoon(player, () -> launcher(player.getInventory().getItemInMainHand()));
    }
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = false)
  public void click(InventoryClickEvent event) {
    if (event.getView().getTopInventory().getHolder() instanceof Menu menu) {
      event.setCancelled(true);
      if (!menu.consumed
          && event.getWhoClicked() == menu.player
          && validPlayer(menu.player, menu.sessionId)
          && event.getRawSlot() >= 54
          && launcher(event.getCurrentItem())
          && (event.getClick() == ClickType.LEFT || event.getClick() == ClickType.RIGHT)) {
        menu.consumed = true;
        Inventory inventory = event.getClickedInventory();
        int slot = event.getSlot();
        openSoon(menu.player, () -> inventory != null && launcher(inventory.getItem(slot)));
        return;
      }
      if (!menu.consumed
          && event.getWhoClicked() == menu.player
          && validPlayer(menu.player, menu.sessionId)
          && (event.getClick() == ClickType.LEFT || event.getClick() == ClickType.RIGHT)
          && menu.actions.containsKey(event.getRawSlot())) {
        menu.consumed = true;
        Runnable action = menu.actions.get(event.getRawSlot());
        Bukkit.getScheduler()
            .runTask(
                ctx.plugin(),
                () -> {
                  if (validPlayer(menu.player, menu.sessionId)
                      && menu.player.getOpenInventory().getTopInventory().getHolder() == menu
                      && !ctx.departing(menu.player.getUniqueId())
                      && !ctx.mustIsolate(menu.player.getUniqueId())) action.run();
                });
      }
      return;
    }
    if (!lobbyLauncher() || !(event.getWhoClicked() instanceof Player player)) return;
    boolean clicked = launcher(event.getCurrentItem());
    boolean hotbar =
        event.getClick() == ClickType.NUMBER_KEY
            && event.getHotbarButton() >= 0
            && launcher(player.getInventory().getItem(event.getHotbarButton()));
    boolean offhand =
        event.getClick() == ClickType.SWAP_OFFHAND
            && launcher(player.getInventory().getItemInOffHand());
    if (!clicked && !launcher(event.getCursor()) && !hotbar && !offhand) return;
    event.setCancelled(true);
    if (clicked && (event.getClick() == ClickType.LEFT || event.getClick() == ClickType.RIGHT)) {
      Inventory clickedInventory = event.getClickedInventory();
      int slot = event.getSlot();
      openSoon(player, () -> clickedInventory != null && launcher(clickedInventory.getItem(slot)));
    }
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = false)
  public void drag(InventoryDragEvent event) {
    if (event.getView().getTopInventory().getHolder() instanceof Menu) {
      event.setCancelled(true);
      return;
    }
    if (!lobbyLauncher()) return;
    if (launcher(event.getOldCursor())
        || event.getRawSlots().stream().anyMatch(slot -> launcher(event.getView().getItem(slot))))
      event.setCancelled(true);
  }

  @EventHandler
  public void closed(InventoryCloseEvent event) {
    if (event.getInventory().getHolder() instanceof Menu menu
        && current.get(event.getPlayer().getUniqueId()) == menu) {
      requests.merge(event.getPlayer().getUniqueId(), 1L, Long::sum);
    }
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = false)
  public void swap(org.bukkit.event.player.PlayerSwapHandItemsEvent event) {
    if (lobbyLauncher() && (launcher(event.getMainHandItem()) || launcher(event.getOffHandItem())))
      event.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = false)
  public void drop(org.bukkit.event.player.PlayerDropItemEvent event) {
    if (lobbyLauncher() && launcher(event.getItemDrop().getItemStack())) event.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = false)
  public void pickup(org.bukkit.event.entity.EntityPickupItemEvent event) {
    if (lobbyLauncher()
        && event.getEntity() instanceof Player
        && launcher(event.getItem().getItemStack())) {
      event.setCancelled(true);
      event.getItem().remove(); // A stale owned token is never an ordinary transferable item.
    }
  }

  @EventHandler(priority = EventPriority.LOWEST)
  public void inputChat(AsyncChatEvent e) {
    Input input = inputs.remove(e.getPlayer().getUniqueId());
    if (input == null) return;
    e.setCancelled(true);
    String text = PlainTextComponentSerializer.plainText().serialize(e.message());
    Bukkit.getScheduler()
        .runTask(
            ctx.plugin(),
            () -> {
              if (!validPlayer(e.getPlayer(), input.sessionId)
                  || !Objects.equals(requests.get(e.getPlayer().getUniqueId()), input.navigation))
                return;
              if (input.expires < System.currentTimeMillis() || text.equalsIgnoreCase("cancel")) {
                inform(e.getPlayer(), tr(e.getPlayer(), "Input cancelled."));
                return;
              }
              try {
                input.action.accept(text);
              } catch (Exception error) {
                inform(
                    e.getPlayer(),
                    tr(e.getPlayer(), "Check your input: ")
                        + Messages.error(effectiveLanguage(e.getPlayer()), error));
              }
            });
  }

  @EventHandler
  public void quit(org.bukkit.event.player.PlayerQuitEvent event) {
    UUID id = event.getPlayer().getUniqueId();
    inputs.remove(id);
    launcherOpens.remove(id);
    launcherPending.remove(id);
    previews.remove(id);
    history.remove(id);
    current.remove(id);
    requests.remove(id);
    selectedLanguages.remove(id);
  }

  @Override
  public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
    if (!(sender instanceof Player p)) {
      sender.sendMessage("Use this command in-game.");
      return true;
    }
    if (args.length > 0 && args[0].equalsIgnoreCase("cancel")) {
      inputs.remove(p.getUniqueId());
      requests.merge(p.getUniqueId(), 1L, Long::sum);
      inform(p, tr(p, "Input cancelled."));
      return true;
    }
    if (args.length > 0 && Set.of("pos1", "pos2").contains(args[0])) {
      try {
        selectPoint(p, args[0].equals("pos2"));
      } catch (Exception e) {
        inform(p, Messages.error(effectiveLanguage(p), e));
      }
      return true;
    }
    if (command.getName().equals("lkjmc") && args.length > 0) {
      switch (args[0].toLowerCase(Locale.ROOT)) {
        case "help" -> help(p);
        case "language" -> languages(p);
        case "people" -> people(p);
        case "worlds", "play" -> servers(p);
        case "timeline" -> timeline(p);
        default -> root(p);
      }
      return true;
    }
    switch (command.getName()) {
      case "home", "claim" -> {
        if (ctx.official()) life(p, command.getName().equals("home") ? "homes" : "land");
        else {
          inform(p, tr(p, "Join SMP to use Homes or Land."));
          servers(p);
        }
      }
      case "expedition" -> {
        if (!ctx.official()) {
          inform(p, tr(p, "Join SMP to begin or return from an Expedition."));
          servers(p);
        } else if (args.length > 0 && args[0].equalsIgnoreCase("return")) returns(p);
        else expeditions(p);
      }
      case "tpa" ->
          choosePlayer(
              p,
              tr(p, "Teleport request"),
              other ->
                  submit(p, command("teleport_request", "target", other.get("id").getAsString())));
      default -> root(p);
    }
    return true;
  }
}
