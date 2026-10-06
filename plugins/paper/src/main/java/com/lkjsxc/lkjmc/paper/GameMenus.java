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
              tr(p, "text.nothing_here_yet"),
              tr(p, "text.return_to_the_previous_menu_to_choose_another_action"),
              null));
    if (!trail.isEmpty())
      put(
          holder,
          45,
          entry(
              Material.ARROW,
              tr(p, "text.back"),
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
              tr(p, "text.previous_page"),
              "",
              () -> render(p, title, entries, holder.page - 1, false)));
    put(holder, 49, entry(Material.COMPASS, tr(p, "text.main_menu"), "", () -> root(p)));
    if (offset + 28 < entries.size())
      put(
          holder,
          50,
          entry(
              Material.ARROW,
              tr(p, "text.next_page"),
              "",
              () -> render(p, title, entries, holder.page + 1, false)));
    put(
        holder,
        51,
        entry(
            Material.PAPER,
            tr(p, "text.page_0_of_1", holder.page + 1, Math.max(1, (entries.size() + 27) / 28)),
            "",
            null));
    put(holder, 53, entry(Material.BARRIER, tr(p, "text.close"), "", p::closeInventory));
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
    inform(p, question + tr(p, "text.type_in_chat_only_you_can_see_it_cancel_cancel"));
  }

  private void confirm(Player p, String title, String detail, Runnable action) {
    menu(
        p,
        title,
        List.of(
            entry(Material.LIME_CONCRETE, tr(p, "text.confirm"), detail, action),
            entry(Material.RED_CONCRETE, tr(p, "text.cancel"), "", () -> back(p))),
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
        tr(p, "text.loading_dc380888"),
        List.of(
            entry(
                Material.CLOCK,
                tr(p, "text.loading_dc380888"),
                tr(p, "text.you_can_go_back_or_close_this_menu"),
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
                    tr(p, "text.could_not_complete_action")
                        + Messages.error(effectiveLanguage(p), e));
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
                            "text.travel_to_0_is_queued_stay_connected_progress_will_appe_d7984d2d4f",
                            CoreClient.string(
                                result, "server_name", command.get("id").getAsString())));
                    return null;
                  }
                  inform(
                      p,
                      result.has("job_id")
                          ? tr(p, "text.request_accepted_you_will_be_notified_when_it_finishes")
                          : tr(p, "text.saved"));
                  if (!result.has("job_id")
                      && completed != null
                      && Objects.equals(requests.get(p.getUniqueId()), navigation)
                      && p.getOpenInventory().getTopInventory() == expectedView)
                    completed.accept(result);
                  if (result.has("code"))
                    inform(
                        p,
                        tr(p, "text.link_code")
                            + result.get("code").getAsString()
                            + tr(p, "text.expires_in_10_minutes_use_only_on_your_own_account"));
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
                                ? tr(p, "text.action_completed_d466be6f")
                                : tr(p, "text.action_failed")
                                    + (status.has("error") && !status.get("error").isJsonNull()
                                        ? Messages.render(effectiveLanguage(p), status.get("error"))
                                        : tr(p, "text.check_notifications_for_details")));
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
        tr(p, "text.enter_the_other_player_s_name"),
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
              tr(p, "text.homes"),
              tr(p, "text.return_home_or_save_this_place"),
              () -> life(p, "homes")));
      entries.add(
          entry(
              Material.OAK_FENCE,
              tr(p, "text.land_b6baff93"),
              tr(p, "text.protect_your_builds_and_manage_shared_land"),
              () -> life(p, "land")));
      entries.add(
          entry(
              Material.EMERALD,
              tr(p, "text.market"),
              tr(p, "text.buy_list_collect_and_sell_materials"),
              () -> market(p)));
      entries.add(
          entry(
              Material.ENDER_EYE,
              tr(p, "text.expeditions"),
              tr(p, "text.explore_a_temporary_world_together"),
              () -> expeditions(p)));
      entries.add(
          entry(
              Material.PLAYER_HEAD,
              tr(p, "text.people_7db20897"),
              tr(p, "text.friends_teams_and_parties"),
              () -> people(p)));
      entries.add(
          entry(
              Material.OAK_DOOR,
              tr(p, "text.return_to_smp"),
              tr(p, "text.leave_an_expedition_with_the_items_you_carry"),
              () -> returns(p)));
    } else {
      entries.add(
          entry(
              Material.COMPASS,
              tr(p, "text.play"),
              tr(p, "text.choose_a_world_and_start_playing"),
              () -> servers(p)));
      entries.add(
          entry(
              Material.GRASS_BLOCK,
              tr(p, "text.worlds"),
              tr(p, "text.explore_available_worlds_and_their_people"),
              () -> servers(p)));
      entries.add(
          entry(
              Material.PLAYER_HEAD,
              tr(p, "text.people_7db20897"),
              tr(p, "text.friends_teams_and_parties"),
              () -> people(p)));
    }
    entries.add(
        entry(
            Material.WRITABLE_BOOK,
            tr(p, "text.timeline"),
            tr(p, "text.conversations_invitations_and_activity"),
            () -> timeline(p)));
    if (ctx.official())
      entries.add(
          entry(
              Material.GRASS_BLOCK,
              tr(p, "text.worlds"),
              tr(p, "text.choose_another_world"),
              () -> servers(p)));
    entries.add(
        entry(
            Material.NAME_TAG,
            tr(p, "text.account"),
            tr(p, "text.language_account_linking_and_achievements"),
            () -> account(p)));
    entries.add(
        entry(
            Material.BOOK,
            tr(p, "text.help"),
            tr(p, "text.getting_started_and_useful_commands"),
            () -> help(p)));
    menu(p, "lkjmc", entries, 0);
  }

  private void people(Player p) {
    menu(
        p,
        tr(p, "text.people_7db20897"),
        List.of(
            entry(
                Material.PLAYER_HEAD,
                tr(p, "text.friends"),
                tr(p, "text.friends_and_requests"),
                () -> social(p, "friends")),
            entry(
                Material.WHITE_BANNER,
                tr(p, "text.teams"),
                tr(p, "text.a_lasting_group_with_shared_land_and_coins"),
                () -> teams(p)),
            entry(
                Material.CAMPFIRE,
                tr(p, "text.parties"),
                tr(p, "text.a_temporary_group_for_playing_together"),
                () -> social(p, "party"))),
        0);
  }

  private void timeline(Player p) {
    menu(
        p,
        tr(p, "text.timeline"),
        List.of(
            entry(
                Material.WRITABLE_BOOK,
                tr(p, "text.conversations"),
                tr(p, "text.read_and_send_messages_in_one_conversation"),
                () -> social(p, "chat")),
            entry(
                Material.BELL,
                tr(p, "text.invitations_activity"),
                tr(p, "text.respond_to_invitations_and_check_progress"),
                () -> notifications(p))),
        0);
  }

  private void account(Player p) {
    List<Entry> entries =
        new ArrayList<>(
            List.of(
                entry(
                    Material.WRITABLE_BOOK,
                    tr(p, "text.language"),
                    tr(p, "text.your_language_is_shared_with_linked_game_accounts"),
                    () -> languages(p)),
                entry(
                    Material.NAME_TAG,
                    tr(p, "text.account_linking"),
                    tr(p, "text.verify_and_link_your_game_and_web_identities"),
                    () -> link(p))));
    if (ctx.official())
      entries.add(
          entry(
              Material.EXPERIENCE_BOTTLE,
              tr(p, "text.achievements_balance"),
              tr(p, "text.your_progress_and_shared_coins"),
              () -> life(p, "progress")));
    menu(p, tr(p, "text.account"), entries, 0);
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
              active.equals(code)
                  ? tr(p, "text.selected_57fd7a0c")
                  : tr(p, "text.use_this_language"),
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
    menu(p, tr(p, "text.language"), entries, 0);
  }

  private void help(Player p) {
    menu(
        p,
        tr(p, "text.help"),
        List.of(
            entry(
                Material.COMPASS,
                tr(p, "text.play"),
                tr(p, "text.choose_a_world_stay_connected_while_it_wakes_progress_a_6ddb929112"),
                () -> servers(p)),
            entry(
                Material.ENDER_EYE,
                tr(p, "text.expeditions"),
                tr(p, "text.temporary_worlds_close_when_their_time_ends_keep_what_y_a0926b867d"),
                ctx.official() ? () -> expeditions(p) : () -> servers(p)),
            entry(
                Material.NAME_TAG,
                tr(p, "text.account_linking"),
                "https://lkjmc.lkjsxc.com",
                () -> link(p)),
            entry(
                Material.BOOK,
                tr(p, "text.commands"),
                "/menu · /home · /claim · /tpa\n"
                    + "/expedition · /expedition return\n"
                    + "/worlds · /hub · /go cancel\n"
                    + "/menu cancel",
                null)),
        0);
  }

  private String serverState(Player p, JsonObject server) {
    if (CoreClient.string(server, "maintenance", "false").equals("true"))
      return tr(p, "text.maintenance");
    JsonObject status = server.getAsJsonObject("status");
    return switch (status == null
        ? "unknown"
        : CoreClient.string(status, "game_state", "unknown")) {
      case "running" -> tr(p, "text.ready_to_play");
      case "stopped" -> tr(p, "text.sleeping_join_to_wake");
      case "starting" -> tr(p, "text.preparing");
      case "stopping" -> tr(p, "text.closing");
      case "error", "failed" -> tr(p, "text.unavailable");
      default -> tr(p, "text.checking_availability");
    };
  }

  private boolean joinAvailable(Player p, JsonObject server) {
    JsonObject status = server.getAsJsonObject("status");
    JsonObject actions = status == null ? null : status.getAsJsonObject("actions");
    JsonObject join = actions == null ? null : actions.getAsJsonObject("join");
    return proxyCompatible(p, server) && join != null && flag(join, "allowed");
  }

  private void smp(Player p, JsonObject server) {
    boolean available = joinAvailable(p, server);
    List<Entry> entries = new ArrayList<>();
    entries.add(
        entry(
            available ? Material.GRASS_BLOCK : Material.GRAY_DYE,
            tr(p, "text.join_world"),
            serverState(p, server),
            available
                ? () -> submit(p, command("server_join", "id", server.get("id").getAsString()))
                : null));
    if (ctx.official()) {
      entries.add(
          entry(
              Material.RED_BED,
              tr(p, "text.homes"),
              tr(p, "text.return_home_or_save_this_place"),
              () -> life(p, "homes")));
      entries.add(
          entry(
              Material.OAK_FENCE,
              tr(p, "text.land_b6baff93"),
              tr(p, "text.protect_your_builds_and_manage_shared_land"),
              () -> life(p, "land")));
      entries.add(
          entry(
              Material.EMERALD,
              tr(p, "text.market"),
              tr(p, "text.buy_list_collect_and_sell_materials"),
              () -> market(p)));
      entries.add(
          entry(
              Material.ENDER_EYE,
              tr(p, "text.expeditions"),
              tr(p, "text.explore_a_temporary_world_together"),
              () -> expeditions(p)));
    } else {
      entries.add(
          entry(
              Material.BOOK,
              tr(p, "text.life_in_smp"),
              tr(p, "text.join_this_world_to_use_homes_land_market_and_expeditions"),
              null));
    }
    entries.add(
        entry(
            Material.PLAYER_HEAD,
            tr(p, "text.people_7db20897"),
            tr(p, "text.friends_teams_and_parties"),
            () -> people(p)));
    menu(p, server.get("name").getAsString(), entries, 0);
  }

  private void link(Player p) {
    menu(
        p,
        tr(p, "text.account_linking"),
        List.of(
            entry(
                Material.PAPER,
                tr(p, "text.create_code"),
                tr(p, "text.choose_one_set_of_game_data_to_keep_using"),
                () -> submit(p, command("link_begin"))),
            entry(
                Material.WRITABLE_BOOK,
                tr(p, "text.enter_code"),
                "",
                () ->
                    input(
                        p,
                        tr(p, "text.code_from_your_other_account"),
                        code -> submit(p, command("link_present", "code", code)))),
            entry(
                Material.CHEST,
                tr(p, "text.choose_game_data"),
                tr(p, "text.confirm_on_the_account_that_created_the_code"),
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
                      + tr(p, "text.coins_e4f74364")
                      + (profile.get("native_uuid").isJsonNull()
                          ? tr(p, "text.start_with_a_fresh_inventory_and_achievements")
                          : tr(p, "text.use_this_inventory_and_achievements"));
              choices.add(
                  entry(
                      Material.CHEST,
                      name,
                      detail,
                      () ->
                          confirm(
                              p,
                              tr(p, "text.confirm_game_data"),
                              detail
                                  + tr(
                                      p,
                                      "text.the_other_data_is_archived_not_combined_both_game_conne_2816df7862"),
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
                tr(p, "text.no_pending_link_enter_the_code_on_your_other_account_th_eeab762a83"));
          else menu(p, tr(p, "text.game_data_to_keep"), choices, 0);
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
                        + tr(p, "text.players")
                        + s.get("version").getAsString()
                        + (proxyCompatible(p, s)
                            ? ""
                            : "\n"
                                + tr(
                                    p,
                                    "text.joining_this_server_through_the_lobby_is_not_available_c8c4ad4528")),
                    s.get("kind").getAsString().equals("official")
                        ? () -> smp(p, s)
                        : joinAvailable(p, s)
                            ? () ->
                                submit(p, command("server_join", "id", s.get("id").getAsString()))
                            : null));
          }
          menu(p, tr(p, "text.worlds"), list, 0);
        });
  }

  private void life(Player p, String section) {
    fetch(
        p,
        "life",
        data -> {
          if (section.equals("progress")) {
            progress(p, data);
            return;
          }
          List<Entry> list = new ArrayList<>();
          if (!section.equals("homes"))
            for (JsonElement value : data.getAsJsonArray("owners")) {
              JsonObject owner = value.getAsJsonObject();
              list.add(
                  entry(
                      Material.GOLD_INGOT,
                      owner.get("name").getAsString(),
                      tr(p, "text.balance")
                          + owner.getAsJsonObject("wallet").get("balance").getAsLong()
                          + tr(p, "text.coins_land")
                          + owner.get("used_chunks")
                          + " / "
                          + owner.getAsJsonObject("land").get("chunks"),
                      null));
            }
          if (section.equals("homes"))
            list.add(
                entry(
                    Material.RED_BED,
                    tr(p, "text.set_a_home_here"),
                    p.getWorld().getName().equals("living")
                        ? tr(p, "text.up_to_three_homes")
                        : tr(p, "text.save_homes_in_the_survival_world"),
                    p.getWorld().getName().equals("living")
                        ? () ->
                            input(
                                p,
                                tr(p, "text.home_name"),
                                name -> submit(p, command("home_set", "name", name)))
                        : null));
          if (section.equals("homes"))
            for (JsonElement value : data.getAsJsonArray("homes")) {
              JsonObject h = value.getAsJsonObject();
              list.add(
                  entry(
                      Material.OAK_DOOR,
                      h.get("name").getAsString(),
                      tr(p, "text.travel_or_delete"),
                      () ->
                          menu(
                              p,
                              h.get("name").getAsString(),
                              List.of(
                                  entry(
                                      Material.ENDER_PEARL,
                                      tr(p, "text.travel_here"),
                                      "",
                                      () ->
                                          submit(
                                              p,
                                              command(
                                                  "home_travel", "id", h.get("id").getAsString()))),
                                  entry(
                                      Material.BARRIER,
                                      tr(p, "text.delete_home"),
                                      tr(p, "text.land_and_buildings_stay_unchanged"),
                                      () ->
                                          confirm(
                                              p,
                                              tr(p, "text.confirm_home_deletion"),
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
                    tr(p, "text.protect_this_chunk"),
                    tr(p, "text.16_16_blocks_survival_world_only"),
                    p.getWorld().getName().equals("living")
                        ? () -> {
                          int x = p.getLocation().getBlockX() >> 4,
                              z = p.getLocation().getBlockZ() >> 4;
                          claimOwner(p, x, z);
                        }
                        : null));
          if (section.equals("land"))
            for (JsonElement value : data.getAsJsonArray("claims")) {
              JsonObject c = value.getAsJsonObject();
              JsonObject owner = owner(data, c.get("owner").getAsString());
              list.add(
                  entry(
                      Material.MAP,
                      c.get("name").getAsString(),
                      tr(
                              p,
                              "game.land.owner_named",
                              owner == null ? "" : owner.get("name").getAsString())
                          + "\n"
                          + c.get("chunks")
                          + tr(p, "text.chunks_580b8611")
                          + claimState(p, c),
                      TeamMenuPolicy.allowed(owner, "can_sell")
                          ? () ->
                              confirm(
                                  p,
                                  tr(p, "text.release_protection"),
                                  c.get("name").getAsString(),
                                  () ->
                                      submit(
                                          p,
                                          command(
                                              "claim_release", "id", c.get("id").getAsString())))
                          : null));
            }
          menu(
              p,
              switch (section) {
                case "homes" -> tr(p, "text.homes");
                case "land" -> tr(p, "text.land_b6baff93");
                default -> tr(p, "text.achievements_balance");
              },
              list,
              0);
        });
  }

  private JsonObject owner(JsonObject data, String id) {
    for (JsonElement value : data.getAsJsonArray("owners"))
      if (value.getAsJsonObject().get("id").getAsString().equals(id))
        return value.getAsJsonObject();
    return null;
  }

  private String ownerSummary(Player p, JsonObject owner) {
    return tr(p, "text.balance")
        + owner.getAsJsonObject("wallet").get("balance").getAsLong()
        + tr(p, "text.coins_land")
        + owner.get("used_chunks")
        + " / "
        + owner.getAsJsonObject("land").get("chunks");
  }

  private String claimState(Player p, JsonObject claim) {
    return switch (CoreClient.string(claim, "state", "pending")) {
      case "active" -> tr(p, "text.active");
      case "released" -> tr(p, "text.released");
      case "transferring" -> tr(p, "text.transferring");
      case "releasing" -> tr(p, "text.in_progress_c1f88e9d");
      default -> tr(p, "text.pending");
    };
  }

  private void claimOwner(Player p, int x, int z) {
    fetch(
        p,
        "life",
        data -> {
          List<Entry> owners = new ArrayList<>();
          for (JsonObject owner :
              TeamMenuPolicy.eligibleOwners(data.getAsJsonArray("owners"), "can_build")) {
            String id = owner.get("id").getAsString();
            boolean capacity = TeamMenuPolicy.canClaim(owner);
            owners.add(
                entry(
                    CoreClient.string(owner, "kind", "account").equals("team")
                        ? Material.WHITE_BANNER
                        : Material.PLAYER_HEAD,
                    owner.get("name").getAsString(),
                    ownerSummary(p, owner) + "\n" + tr(p, "game.land.owner_hint"),
                    capacity
                        ? () ->
                            input(
                                p,
                                tr(p, "text.claim_name"),
                                name ->
                                    confirm(
                                        p,
                                        tr(p, "text.protect_this_chunk"),
                                        tr(
                                                p,
                                                "game.land.owner_named",
                                                owner.get("name").getAsString())
                                            + "\n"
                                            + name
                                            + "\n"
                                            + x
                                            + ", "
                                            + z,
                                        () ->
                                            submit(
                                                p,
                                                command(
                                                    "claim_create",
                                                    "owner",
                                                    id,
                                                    "name",
                                                    name,
                                                    "min_x",
                                                    x,
                                                    "max_x",
                                                    x,
                                                    "min_z",
                                                    z,
                                                    "max_z",
                                                    z))))
                        : null));
          }
          menu(p, tr(p, "game.land.choose_owner"), owners, 0);
        });
  }

  private void progress(Player p, JsonObject data) {
    List<Entry> groups = new ArrayList<>();
    for (JsonElement value : data.getAsJsonArray("achievements")) {
      JsonObject group = value.getAsJsonObject();
      JsonObject principal = group.getAsJsonObject("owner");
      JsonObject balance = owner(data, principal.get("id").getAsString());
      String title =
          CoreClient.string(principal, "kind", "account").equals("team")
              ? tr(p, "game.progress.team", principal.get("name").getAsString())
              : tr(p, "game.progress.personal");
      groups.add(
          entry(
              Material.EXPERIENCE_BOTTLE,
              title,
              balance == null ? "" : ownerSummary(p, balance),
              () -> {
                List<Entry> achievements = new ArrayList<>();
                if (balance != null)
                  achievements.add(
                      entry(
                          Material.GOLD_INGOT,
                          principal.get("name").getAsString(),
                          ownerSummary(p, balance),
                          null));
                for (JsonElement item : group.getAsJsonArray("achievements")) {
                  JsonObject achievement = item.getAsJsonObject();
                  achievements.add(
                      entry(
                          Material.EXPERIENCE_BOTTLE,
                          systemText(p, achievement, "title"),
                          systemText(p, achievement, "description")
                              + "\n"
                              + achievement.get("progress")
                              + " / "
                              + achievement.get("target")
                              + (achievement.has("earned_at")
                                      && !achievement.get("earned_at").isJsonNull()
                                  ? "\n" + tr(p, "text.earned")
                                  : ""),
                          null));
                }
                menu(p, title, achievements, 0);
              }));
    }
    menu(p, tr(p, "game.progress.choose_owner"), groups, 0);
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
                  tr(p, "text.deposit_a_building_or_land"),
                  tr(p, "text.pack_a_building_or_sell_it_with_its_land"),
                  () -> captureBuilding(p)));
          if (previews.containsKey(p.getUniqueId())
              && TeamMenuPolicy.allowed(
                  owner(data, previews.get(p.getUniqueId()).asset().get("owner").getAsString()),
                  "can_build"))
            list.add(
                entry(
                    Material.COMPASS,
                    tr(p, "text.open_last_placement_preview"),
                    tr(p, "text.leave_the_area_before_confirming_placement"),
                    () -> showPreview(p, previews.get(p.getUniqueId()))));
          list.add(
              entry(
                  Material.CHEST,
                  tr(p, "text.deposit_held_item"),
                  tr(p, "text.store_and_list_the_stack_in_your_hand"),
                  () -> captureItems(p)));
          list.add(
              entry(
                  Material.IRON_INGOT,
                  tr(p, "text.sell_materials"),
                  tr(p, "text.remaining_today_3f1b5c93")
                      + data.get("npc_remaining")
                      + tr(p, "text.coins"),
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
                              tr(p, "text.each") + price.get("price") + tr(p, "text.coins"),
                              () ->
                                  input(
                                      p,
                                      tr(p, "text.quantity_to_sell"),
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
                    menu(p, tr(p, "text.material_buyback"), prices, 0);
                  }));
          for (JsonElement value : data.getAsJsonArray("assets")) {
            JsonObject a = value.getAsJsonObject();
            JsonObject assetOwner = owner(data, a.get("owner").getAsString());
            if (a.get("state").getAsString().equals("capturing")
                && a.has("manifest_sha256")
                && !a.get("manifest_sha256").isJsonNull()) {
              list.add(
                  entry(
                      Material.WRITABLE_BOOK,
                      tr(p, "text.awaiting_consent") + systemText(p, a, "title"),
                      tr(p, "text.review_the_building_and_agree_or_cancel"),
                      () -> {
                        List<Entry> actions = manifestEntries(p, a.getAsJsonObject("manifest"));
                        if (TeamMenuPolicy.canConsent(a, accountId(p)))
                          actions.add(
                              entry(
                                  Material.LIME_DYE,
                                  tr(p, "text.agree_to_transfer_pets"),
                                  tr(p, "text.pet_ownership_transfers_with_this_building"),
                                  () ->
                                      confirm(
                                          p,
                                          tr(p, "text.agree_to_transfer"),
                                          tr(p, "text.these_pets_will_be_transferred_to_the_buyer"),
                                          () ->
                                              submit(
                                                  p,
                                                  command(
                                                      "asset_consent",
                                                      "id",
                                                      a.get("id"),
                                                      "manifest_sha256",
                                                      a.get("manifest_sha256"))))));
                        if (TeamMenuPolicy.allowed(assetOwner, "can_sell"))
                          actions.add(
                              entry(
                                  Material.BARRIER,
                                  tr(p, "text.cancel_packing"),
                                  tr(p, "text.the_owner_can_cancel_a_pending_consent_request"),
                                  () -> submit(p, command("asset_withdraw", "id", a.get("id")))));
                        menu(p, tr(p, "text.building_contents"), actions, 0);
                      }));
            }
            if (!a.get("state").getAsString().equals("escrowed")) continue;
            list.add(
                entry(
                    Material.BARREL,
                    tr(p, "text.stored_581b2378") + systemText(p, a, "title"),
                    tr(
                        p,
                        "game.land.owner_named",
                        assetOwner == null ? "" : assetOwner.get("name").getAsString()),
                    () -> {
                      List<Entry> actions = new ArrayList<>();
                      if (assetOwner != null)
                        actions.add(
                            entry(
                                Material.WHITE_BANNER,
                                assetOwner.get("name").getAsString(),
                                ownerSummary(p, assetOwner),
                                null));
                      actions.addAll(manifestEntries(p, a.getAsJsonObject("manifest")));
                      if (a.get("kind").getAsString().equals("building")
                          && TeamMenuPolicy.allowed(assetOwner, "can_build"))
                        actions.add(
                            entry(
                                Material.BRICKS,
                                tr(p, "text.place_building"),
                                tr(p, "text.check_area_and_rotation_from_your_position"),
                                () -> placeBuilding(p, a)));
                      if (a.get("kind").getAsString().equals("land")
                          && TeamMenuPolicy.allowed(assetOwner, "can_sell"))
                        actions.add(
                            entry(
                                Material.BARRIER,
                                tr(p, "text.release_deposit"),
                                tr(p, "text.allow_editing_of_the_claim_again"),
                                () -> submit(p, command("asset_withdraw", "id", a.get("id")))));
                      if (TeamMenuPolicy.allowed(assetOwner, "can_sell"))
                        actions.add(
                            entry(
                                Material.EMERALD,
                                tr(p, "text.create_listing"),
                                tr(p, "text.5_fee_when_sold"),
                                () ->
                                    input(
                                        p,
                                        tr(p, "text.price_whole_coins"),
                                        price ->
                                            submit(
                                                p,
                                                command(
                                                    "listing_create",
                                                    "asset",
                                                    a.get("id").getAsString(),
                                                    "price",
                                                    Long.parseLong(price))))));
                      if (a.get("kind").getAsString().equals("items")
                          && TeamMenuPolicy.allowed(assetOwner, "can_spend"))
                        actions.add(
                            entry(
                                Material.HOPPER,
                                tr(p, "text.collect_into_inventory"),
                                tr(p, "text.items_stay_stored_if_there_is_not_enough_space"),
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
                    l.get("price")
                        + tr(p, "text.coins_cbe3e329")
                        + l.get("seller_name").getAsString(),
                    () -> listing(p, l, data)));
          }
          menu(p, tr(p, "text.market"), list, 0);
        });
  }

  private void chooseMarketOwner(
      Player p,
      String title,
      String hint,
      String capability,
      JsonObject listing,
      Consumer<JsonObject> chosen) {
    fetch(
        p,
        "market",
        data -> {
          List<Entry> choices = new ArrayList<>();
          for (JsonObject owner :
              TeamMenuPolicy.eligibleOwners(data.getAsJsonArray("owners"), capability)) {
            if (listing != null && !TeamMenuPolicy.canBuy(owner, listing)) continue;
            choices.add(
                entry(
                    CoreClient.string(owner, "kind", "account").equals("team")
                        ? Material.WHITE_BANNER
                        : Material.PLAYER_HEAD,
                    owner.get("name").getAsString(),
                    ownerSummary(p, owner) + "\n" + tr(p, hint),
                    () -> chosen.accept(owner)));
          }
          menu(p, tr(p, title), choices, 0);
        });
  }

  private void captureItems(Player p) {
    chooseMarketOwner(
        p,
        "game.market.choose_deposit_owner",
        "game.market.deposit_owner_hint",
        "can_sell",
        null,
        owner ->
            input(
                p,
                tr(p, "text.listing_name"),
                name ->
                    confirm(
                        p,
                        tr(p, "text.deposit_item"),
                        tr(p, "game.land.owner_named", owner.get("name").getAsString())
                            + "\n"
                            + tr(p, "text.the_held_stack_moves_out_of_your_inventory"),
                        () ->
                            submit(
                                p,
                                command(
                                    "asset_capture",
                                    "owner",
                                    owner.get("id"),
                                    "kind",
                                    "items",
                                    "title",
                                    name,
                                    "selection",
                                    CoreClient.object(),
                                    "include_contents",
                                    true),
                                done -> market(p)))));
  }

  private void listing(Player p, JsonObject listing, JsonObject market) {
    List<Entry> actions = new ArrayList<>();
    actions.add(
        entry(
            Material.EMERALD,
            tr(p, "text.buy"),
            listing.get("price") + tr(p, "text.coins"),
            () ->
                chooseMarketOwner(
                    p,
                    "game.market.choose_buyer",
                    "game.market.buyer_hint",
                    "can_spend",
                    listing,
                    owner ->
                        confirm(
                            p,
                            tr(p, "text.buy"),
                            tr(p, "game.land.owner_named", owner.get("name").getAsString())
                                + "\n"
                                + systemText(p, listing, "title")
                                + " / "
                                + listing.get("price")
                                + tr(p, "text.coins"),
                            () ->
                                submit(
                                    p,
                                    command(
                                        "listing_buy",
                                        "id",
                                        listing.get("id"),
                                        "owner",
                                        owner.get("id")),
                                    done -> market(p))))));
    JsonObject seller = owner(market, listing.get("seller").getAsString());
    if (TeamMenuPolicy.allowed(seller, "can_sell"))
      actions.add(
          entry(
              Material.BARRIER,
              tr(p, "text.withdraw_listing"),
              tr(p, "game.land.owner_named", seller.get("name").getAsString()),
              () ->
                  confirm(
                      p,
                      tr(p, "text.withdraw_listing"),
                      systemText(p, listing, "title") + "\n" + seller.get("name").getAsString(),
                      () ->
                          submit(
                              p,
                              command("listing_cancel", "id", listing.get("id")),
                              done -> market(p)))));
    menu(p, systemText(p, listing, "title"), actions, 0);
  }

  private void selectPoint(Player p, boolean second) {
    if (!ctx.official())
      throw new IllegalArgumentException(tr(p, "text.select_the_area_in_the_official_smp"));
    org.bukkit.block.Block target = p.getTargetBlockExact(8);
    if (target == null)
      throw new IllegalArgumentException(
          tr(p, "text.look_at_a_building_corner_within_eight_blocks"));
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
            + tr(p, "text.point")
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
              tr(p, "text.building") + manifest.get("blocks") + tr(p, "text.blocks"),
              String.valueOf(manifest.get("dimensions")),
              null));
    if (manifest.has("materials"))
      for (var item : manifest.getAsJsonObject("materials").entrySet()) {
        Material material = Material.matchMaterial(item.getKey());
        entries.add(
            entry(
                material == null ? Material.PAPER : material,
                item.getKey(),
                item.getValue() + tr(p, "text.items_0f12f4b3"),
                null));
      }
    if (manifest.has("containers"))
      for (JsonElement item : manifest.getAsJsonArray("containers")) {
        JsonObject row = item.getAsJsonObject();
        Material material = Material.matchMaterial(row.get("material").getAsString());
        entries.add(
            entry(
                material == null ? Material.CHEST : material,
                tr(p, "text.containers") + row.get("material").getAsString(),
                row.get("amount") + tr(p, "text.items_11fafae7") + CoreClient.string(row, "at", ""),
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
                        ? tr(p, "text.pet_owner_confirmation_required")
                        : tr(p, "text.moves_with_the_building")),
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
                  tr(p, "text.select_first_corner"),
                  tr(p, "text.look_at_a_corner_and_click_command_lkjmc_pos1"),
                  () -> {
                    p.closeInventory();
                    selectPoint(p, false);
                  }));
          options.add(
              entry(
                  Material.COMPASS,
                  tr(p, "text.select_second_corner"),
                  tr(p, "text.look_at_the_opposite_corner_command_lkjmc_pos2"),
                  () -> {
                    p.closeInventory();
                    selectPoint(p, true);
                  }));
          for (JsonElement value : data.getAsJsonArray("claims")) {
            JsonObject claim = value.getAsJsonObject();
            if (!claim.get("state").getAsString().equals("active")) continue;
            JsonObject claimOwner = owner(data, claim.get("owner").getAsString());
            if (!TeamMenuPolicy.allowed(claimOwner, "can_sell")) continue;
            String context =
                tr(p, "game.land.owner_named", claimOwner.get("name").getAsString())
                    + "\n"
                    + tr(p, "text.claim_name")
                    + ": "
                    + claim.get("name").getAsString()
                    + "\n"
                    + claim.get("min_x")
                    + ", "
                    + claim.get("min_z")
                    + " – "
                    + claim.get("max_x")
                    + ", "
                    + claim.get("max_z");
            options.add(
                entry(
                    Material.GRASS_BLOCK,
                    claim.get("name").getAsString(),
                    context + "\n" + tr(p, "text.pack_or_deposit_with_land_in_this_claim"),
                    () -> {
                      List<Entry> kinds = new ArrayList<>();
                      for (String kind : List.of("building", "land"))
                        kinds.add(
                            entry(
                                Material.BRICKS,
                                kind.equals("building")
                                    ? tr(p, "text.pack_selected_building")
                                    : tr(p, "text.sell_land_with_buildings"),
                                kind.equals("building")
                                    ? tr(p, "text.remove_the_original_and_create_a_one_use_asset")
                                    : tr(p, "text.freeze_editing_until_sale_or_cancellation"),
                                () ->
                                    input(
                                        p,
                                        tr(p, "text.building_or_land_name"),
                                        title -> {
                                          List<Entry> contents = new ArrayList<>();
                                          for (boolean include : List.of(false, true))
                                            contents.add(
                                                entry(
                                                    Material.CHEST,
                                                    include
                                                        ? tr(p, "text.include_container_contents")
                                                        : tr(
                                                            p,
                                                            "text.empty_containers_before_depositing"),
                                                    tr(
                                                        p,
                                                        "text.everyone_must_leave_the_selected_area"),
                                                    () ->
                                                        confirm(
                                                            p,
                                                            tr(p, "text.confirm_deposit"),
                                                            context
                                                                + "\n"
                                                                + title
                                                                + "\n"
                                                                + tr(
                                                                    p,
                                                                    "text.safely_save_the_structure_containers_and_entities"),
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
                                          menu(
                                              p,
                                              tr(p, "text.container_contents_0d10bbfc"),
                                              contents,
                                              0);
                                        })));
                      menu(p, tr(p, "text.deposit_method"), kinds, 0);
                    }));
          }
          menu(p, tr(p, "text.buildings_land"), options, 0);
        });
  }

  private void placeBuilding(Player p, JsonObject asset) {
    if (claims == null) {
      inform(p, tr(p, "text.choose_a_placement_location_in_the_official_smp"));
      return;
    }
    Location point = p.getLocation();
    JsonObject claim = claims.claim(point.getBlock());
    if (claim == null) {
      inform(p, tr(p, "text.stand_inside_the_destination_claim_first"));
      return;
    }
    List<Entry> rotations = new ArrayList<>();
    for (int rotation : List.of(0, 90, 180, 270))
      rotations.add(
          entry(
              Material.COMPASS,
              rotation + tr(p, "text.degrees"),
              tr(p, "text.origin")
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
                        "text.you_cannot_place_while_inside_the_area_preview_move_out_11b5e547bf"));
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
    menu(p, tr(p, "text.building_rotation"), rotations, 0);
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
            + tr(p, "text.rotation_c86233fe")
            + preview.get("rotation")
            + tr(p, "text.degrees_1903c2a4")
            + Messages.render(effectiveLanguage(p), preview.get("message"));
    List<Entry> actions = new ArrayList<>();
    actions.add(
        entry(
            Material.PAPER,
            tr(p, "text.placement_area"),
            description,
            () -> inform(p, description)));
    actions.add(
        entry(
            Material.LIME_CONCRETE,
            tr(p, "text.place_in_this_area"),
            tr(p, "text.clear_the_area_first_it_is_checked_again_before_placement"),
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
            tr(p, "text.close_and_move_outside_the_area"),
            tr(p, "text.return_to_the_preview_from_the_market_menu"),
            p::closeInventory));
    menu(p, tr(p, "text.placement_preview"), actions, 0);
  }

  private String accountId(Player p) {
    try {
      return ctx.session(p.getUniqueId()).get("account_id").getAsString();
    } catch (Exception ignored) {
      return "";
    }
  }

  private String permissionSummary(Player p, JsonObject flags) {
    List<String> lines = new ArrayList<>();
    for (TeamMenuPolicy.Permission permission : TeamMenuPolicy.PERMISSIONS)
      lines.add(
          tr(p, permission.titleId())
              + ": "
              + tr(
                  p,
                  flag(flags, permission.field())
                      ? "game.teams.permission_allowed"
                      : "game.teams.permission_denied"));
    return String.join("\n", lines);
  }

  private void teams(Player p) {
    fetch(
        p,
        "social",
        CoreClient.object("section", "teams"),
        data -> {
          List<Entry> entries = new ArrayList<>();
          String selected = tr(p, "game.teams.no_contribution");
          for (JsonElement value : data.getAsJsonArray("teams")) {
            JsonObject team = value.getAsJsonObject();
            if (flag(team, "is_contribution_team")) selected = team.get("name").getAsString();
          }
          entries.add(
              entry(
                  Material.EXPERIENCE_BOTTLE,
                  tr(p, "game.teams.contribution_title"),
                  tr(p, "game.teams.contribution_current", selected)
                      + "\n"
                      + tr(p, "game.teams.contribution_hint"),
                  () -> contributionTeams(p)));
          for (JsonElement value : data.getAsJsonArray("teams")) {
            JsonObject team = value.getAsJsonObject();
            entries.add(
                entry(
                    Material.WHITE_BANNER,
                    team.get("name").getAsString(),
                    tr(p, "game.teams.member_count", team.get("member_count").getAsInt())
                        + (flag(team, "is_contribution_team")
                            ? "\n" + tr(p, "game.teams.contribution_title")
                            : "")
                        + "\n"
                        + tr(p, "text.share_land_coins_and_buildings_with_your_team"),
                    () -> team(p, team.get("id").getAsString(), null, false)));
          }
          entries.add(
              entry(
                  Material.WRITABLE_BOOK,
                  tr(p, "text.create_team"),
                  tr(p, "game.teams.create_hint"),
                  () ->
                      input(
                          p,
                          tr(p, "text.team_name"),
                          name ->
                              submit(p, command("team_create", "name", name), done -> teams(p)))));
          menu(p, tr(p, "text.teams"), entries, 0);
        });
  }

  private void contributionTeams(Player p) {
    fetch(
        p,
        "social",
        CoreClient.object("section", "teams"),
        data -> {
          List<Entry> choices = new ArrayList<>();
          boolean none = data.get("contribution_team_id").isJsonNull();
          choices.add(
              entry(
                  none ? Material.LIME_DYE : Material.GRAY_DYE,
                  tr(p, "game.teams.no_contribution"),
                  tr(p, "game.teams.no_contribution_hint"),
                  none
                      ? null
                      : () -> setContribution(p, null, tr(p, "game.teams.no_contribution"))));
          for (JsonElement value : data.getAsJsonArray("teams")) {
            JsonObject team = value.getAsJsonObject();
            boolean selected = flag(team, "is_contribution_team");
            choices.add(
                entry(
                    selected ? Material.LIME_DYE : Material.WHITE_BANNER,
                    team.get("name").getAsString(),
                    tr(
                        p,
                        selected
                            ? "game.teams.contribution_selected"
                            : "game.teams.contribution_select"),
                    selected
                        ? null
                        : () ->
                            setContribution(
                                p, team.get("id").getAsString(), team.get("name").getAsString())));
          }
          menu(p, tr(p, "game.teams.contribution_title"), choices, 0);
        });
  }

  private void setContribution(Player p, String teamId, String name) {
    confirm(
        p,
        tr(p, "game.teams.contribution_confirm"),
        name + "\n" + tr(p, "game.teams.contribution_hint"),
        () -> submit(p, command("team_contribution_set", "team", teamId), done -> teams(p)));
  }

  private void team(Player p, String id, String after, boolean members) {
    JsonObject query = CoreClient.object("id", id);
    if (after != null) query.addProperty("after", after);
    fetch(
        p,
        "team",
        query,
        data -> {
          JsonObject team = data.getAsJsonObject("team");
          if (members) teamMembers(p, team);
          else teamDetails(p, team);
        });
  }

  private void teamDetails(Player p, JsonObject team) {
    String id = team.get("id").getAsString(), name = team.get("name").getAsString();
    boolean leader = TeamMenuPolicy.leader(team, accountId(p));
    List<Entry> actions = new ArrayList<>();
    actions.add(
        entry(
            Material.WHITE_BANNER,
            name,
            tr(p, "game.teams.member_count", team.get("member_count").getAsInt()),
            null));
    actions.add(
        entry(
            Material.EXPERIENCE_BOTTLE,
            tr(p, "game.teams.contribution_title"),
            tr(
                p,
                flag(team, "is_contribution_team")
                    ? "game.teams.contribution_selected"
                    : "game.teams.contribution_select"),
            flag(team, "is_contribution_team") ? null : () -> setContribution(p, id, name)));
    actions.add(
        entry(
            Material.PAPER,
            tr(p, "game.teams.your_permissions"),
            tr(p, "game.teams.permissions_scope")
                + "\n"
                + permissionSummary(p, team.getAsJsonObject("permissions")),
            null));
    actions.add(
        entry(
            Material.PLAYER_HEAD,
            tr(p, "text.members_and_permissions"),
            tr(p, "game.teams.member_count", team.get("member_count").getAsInt()),
            () -> teamMembers(p, team)));
    if (TeamMenuPolicy.allowed(team, "can_manage_members"))
      actions.add(
          entry(
              Material.PLAYER_HEAD,
              tr(p, "text.invite_member"),
              name,
              () ->
                  choosePlayer(
                      p,
                      tr(p, "text.invite"),
                      other ->
                          submit(
                              p,
                              command(
                                  "invite",
                                  "kind",
                                  "team",
                                  "resource",
                                  id,
                                  "target",
                                  other.get("id")),
                              done -> team(p, id, null, false)))));
    if (leader) {
      actions.add(
          entry(
              Material.GOLDEN_HELMET,
              tr(p, "text.transfer_leadership"),
              name,
              () -> transferTeamLeader(p, team)));
      actions.add(
          entry(
              Material.BARRIER,
              tr(p, "text.disband_team"),
              tr(p, "text.disband_a_team_after_disposing_of_its_land_stored_asset_3fd66c1a99"),
              () ->
                  confirm(
                      p,
                      tr(p, "text.confirm_disbanding"),
                      name,
                      () -> submit(p, command("team_disband", "team", id), done -> teams(p)))));
    } else {
      actions.add(
          entry(
              Material.OAK_DOOR,
              tr(p, "text.leave_team"),
              tr(p, "game.teams.leave_detail", name),
              () ->
                  confirm(
                      p,
                      tr(p, "text.leave_team"),
                      tr(p, "game.teams.leave_detail", name),
                      () -> submit(p, command("team_leave", "team", id), done -> teams(p)))));
    }
    menu(p, name, actions, 0);
  }

  private void teamMembers(Player p, JsonObject team) {
    List<Entry> members = new ArrayList<>();
    for (JsonElement value : team.getAsJsonArray("members")) {
      JsonObject member = value.getAsJsonObject();
      members.add(
          entry(
              Material.PLAYER_HEAD,
              member.get("name").getAsString(),
              (member.get("account_id").equals(team.get("leader"))
                      ? tr(p, "text.leader") + "\n"
                      : "")
                  + permissionSummary(p, TeamMenuPolicy.effectiveMemberPermissions(team, member)),
              TeamMenuPolicy.canEditMember(team, member, accountId(p))
                  ? () -> teamPermissions(p, team, member, member.deepCopy(), true)
                  : null));
    }
    JsonElement next = team.get("members_next_after");
    if (next != null && !next.isJsonNull())
      members.add(
          entry(
              Material.ARROW,
              tr(p, "game.teams.members_next"),
              team.get("name").getAsString(),
              () -> team(p, team.get("id").getAsString(), next.getAsString(), true)));
    menu(
        p,
        tr(p, "text.members_and_permissions") + " · " + team.get("name").getAsString(),
        members,
        0);
  }

  private void teamPermissions(
      Player p, JsonObject team, JsonObject member, JsonObject draft, boolean remember) {
    if (!TeamMenuPolicy.canEditMember(team, member, accountId(p))) return;
    List<Entry> permissions = new ArrayList<>();
    permissions.add(
        entry(
            Material.WHITE_BANNER,
            team.get("name").getAsString(),
            tr(p, "text.permissions_for_0", member.get("name").getAsString())
                + "\n"
                + tr(p, "game.teams.permissions_scope")
                + "\n"
                + tr(p, "game.teams.permission_roles_hint"),
            null));
    for (TeamMenuPolicy.Permission permission : TeamMenuPolicy.PERMISSIONS) {
      boolean allowed = flag(draft, permission.field());
      boolean editable =
          !permission.field().equals("can_administer") || TeamMenuPolicy.leader(team, accountId(p));
      permissions.add(
          entry(
              allowed ? Material.LIME_DYE : Material.GRAY_DYE,
              tr(p, permission.titleId()),
              tr(p, allowed ? "game.teams.permission_allowed" : "game.teams.permission_denied")
                  + "\n"
                  + tr(p, "game.teams.permissions_scope"),
              editable
                  ? () -> {
                    draft.addProperty(permission.field(), !allowed);
                    teamPermissions(p, team, member, draft, false);
                  }
                  : null));
    }
    permissions.add(
        entry(
            Material.LIME_CONCRETE,
            tr(p, "text.save"),
            tr(p, "game.teams.member_permissions_hint"),
            () ->
                confirm(
                    p,
                    tr(p, "text.save"),
                    team.get("name").getAsString()
                        + "\n"
                        + member.get("name").getAsString()
                        + "\n"
                        + permissionSummary(p, draft),
                    () ->
                        submit(
                            p,
                            TeamMenuPolicy.permissionsCommand(team, member, draft),
                            done -> team(p, team.get("id").getAsString(), null, false)))));
    render(
        p,
        tr(p, "text.permissions_for_0", member.get("name").getAsString()),
        permissions,
        0,
        remember);
  }

  private void transferTeamLeader(Player p, JsonObject team) {
    if (!TeamMenuPolicy.leader(team, accountId(p))) return;
    List<Entry> members = new ArrayList<>();
    for (JsonElement value : team.getAsJsonArray("members")) {
      JsonObject member = value.getAsJsonObject();
      if (member.get("account_id").getAsString().equals(accountId(p))) continue;
      members.add(
          entry(
              Material.PLAYER_HEAD,
              member.get("name").getAsString(),
              tr(p, "text.transfer_leadership"),
              () ->
                  confirm(
                      p,
                      tr(p, "text.transfer_leadership_now"),
                      team.get("name").getAsString() + "\n" + member.get("name").getAsString(),
                      () ->
                          submit(
                              p,
                              command(
                                  "team_transfer",
                                  "team",
                                  team.get("id"),
                                  "target",
                                  member.get("account_id")),
                              done -> team(p, team.get("id").getAsString(), null, false)))));
    }
    JsonElement next = team.get("members_next_after");
    if (next != null && !next.isJsonNull())
      members.add(
          entry(
              Material.ARROW,
              tr(p, "game.teams.members_next"),
              team.get("name").getAsString(),
              () ->
                  fetch(
                      p,
                      "team",
                      CoreClient.object("id", team.get("id"), "after", next),
                      data -> transferTeamLeader(p, data.getAsJsonObject("team")))));
    menu(p, tr(p, "text.transfer_leadership") + " · " + team.get("name").getAsString(), members, 0);
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
                    tr(p, "text.add_friend"),
                    tr(p, "text.the_other_player_must_accept"),
                    () ->
                        choosePlayer(
                            p,
                            tr(p, "text.friend_request"),
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
                    tr(p, "text.meet_up"),
                    tr(p, "text.ask_permission_to_teleport_to_the_player"),
                    () ->
                        choosePlayer(
                            p,
                            tr(p, "text.teleport_request"),
                            other ->
                                submit(
                                    p,
                                    command(
                                        "teleport_request",
                                        "target",
                                        other.get("id").getAsString())))));
          }
          if (section.equals("party") && data.get("party").isJsonNull()) {
            list.add(
                entry(
                    Material.CAMPFIRE,
                    tr(p, "text.create_party"),
                    tr(p, "text.a_temporary_group_for_playing_together"),
                    () ->
                        input(
                            p,
                            tr(p, "text.party_name"),
                            name -> submit(p, command("party_create", "name", name)))));
          }
          if (section.equals("chat")) {
            list.add(
                entry(
                    Material.WRITABLE_BOOK,
                    tr(p, "text.create_group_chat"),
                    "",
                    () ->
                        input(
                            p,
                            tr(p, "text.group_name"),
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
          if (section.equals("party") && !data.get("party").isJsonNull()) {
            JsonObject group = data.getAsJsonObject("party");
            list.add(
                entry(
                    Material.BELL,
                    group.get("name").getAsString(),
                    tr(p, "text.temporary_party"),
                    () -> {
                      List<Entry> actions = new ArrayList<>();
                      actions.add(
                          entry(
                              Material.PLAYER_HEAD,
                              tr(p, "text.invite_members"),
                              "",
                              () ->
                                  choosePlayer(
                                      p,
                                      tr(p, "text.invite"),
                                      other ->
                                          submit(
                                              p,
                                              command(
                                                  "invite",
                                                  "kind",
                                                  "party",
                                                  "resource",
                                                  group.get("id").getAsString(),
                                                  "target",
                                                  other.get("id").getAsString())))));
                      actions.add(
                          entry(
                              Material.LIME_DYE,
                              tr(p, "text.ready_for_expedition"),
                              tr(
                                  p,
                                  "text.agree_to_join_the_next_expedition_before_preparation_commits"),
                              () -> submit(p, command("party_ready", "ready", true))));
                      actions.add(
                          entry(
                              Material.GRAY_DYE,
                              tr(p, "text.withdraw_next_expedition_consent"),
                              tr(p, "text.committed_expeditions_keep_their_participant_roster"),
                              () -> submit(p, command("party_ready", "ready", false))));
                      if (group.has("members"))
                        for (JsonElement value : group.getAsJsonArray("members")) {
                          JsonObject member = value.getAsJsonObject();
                          actions.add(
                              entry(
                                  Material.PLAYER_HEAD,
                                  member.get("name").getAsString(),
                                  (flag(member, "ready")
                                      ? tr(p, "text.ready")
                                      : tr(p, "text.consent_needed")),
                                  null));
                        }
                      actions.add(
                          entry(
                              Material.OAK_DOOR,
                              tr(p, "text.leave_group"),
                              tr(p, "text.leaders_must_transfer_leadership_first"),
                              () ->
                                  confirm(
                                      p,
                                      tr(p, "text.leave"),
                                      group.get("name").getAsString(),
                                      () -> submit(p, command("party_leave")))));
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
                      tr(p, "text.unread_f551d88e") + room.get("unread"),
                      () -> chat(p, room)));
            }
          }
          menu(
              p,
              switch (section) {
                case "party" -> tr(p, "text.parties");
                case "chat" -> tr(p, "text.conversations");
                default -> tr(p, "text.friends");
              },
              list,
              0);
        });
  }

  private String friendState(Player p, JsonObject friend) {
    if (CoreClient.string(friend, "state", "pending").equals("accepted"))
      return tr(p, "text.friends");
    try {
      return CoreClient.string(friend, "requester", "")
              .equals(ctx.session(p.getUniqueId()).get("account_id").getAsString())
          ? tr(p, "text.request_sent")
          : tr(p, "text.friend_request");
    } catch (Exception ignored) {
      return tr(p, "text.friend_request");
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
              tr(p, "text.accept_friend_request"),
              "",
              () -> submit(p, command("friend_respond", "target", id, "accept", true))));
      entries.add(
          entry(
              Material.GRAY_DYE,
              tr(p, "text.decline_friend_request"),
              "",
              () -> submit(p, command("friend_respond", "target", id, "accept", false))));
    }
    if (accepted)
      entries.add(
          entry(
              Material.WRITABLE_BOOK,
              tr(p, "text.open_dm"),
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
            accepted ? tr(p, "text.remove_friend") : tr(p, "text.cancel_friend_request"),
            "",
            () ->
                confirm(
                    p,
                    accepted ? tr(p, "text.remove_friend") : tr(p, "text.cancel_friend_request"),
                    friend.get("name").getAsString(),
                    () -> submit(p, command("friend_remove", "target", id)))));
    entries.add(
        entry(
            Material.RED_DYE,
            tr(p, "text.block_player"),
            "",
            () ->
                confirm(
                    p,
                    tr(p, "text.block_player"),
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
                  tr(p, "text.send_message"),
                  "",
                  () ->
                      input(
                          p,
                          tr(p, "text.message"),
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
                  tr(p, "text.invite_members"),
                  "",
                  () ->
                      choosePlayer(
                          p,
                          tr(p, "text.invite"),
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
                        : tr(p, "text.deleted_message"),
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
      case "preparing", "reserved", "queued" -> tr(p, "text.preparing");
      case "active" -> tr(p, "text.open");
      case "closing" -> tr(p, "text.closing");
      case "closed", "completed" -> tr(p, "text.completed");
      case "cancelled" -> tr(p, "text.cancelled");
      case "failed" -> tr(p, "text.failed");
      default -> tr(p, "text.checking_availability");
    };
  }

  private String expeditionTitle(Player p, JsonObject expedition) {
    String id = expedition.get("id").getAsString();
    return tr(p, "text.end_expedition_0", id.substring(0, Math.min(8, id.length())));
  }

  private String expeditionSummary(Player p, JsonObject expedition) {
    String detail =
        tr(p, "text.the_end_temporary_world_participants_only")
            + "\n"
            + expeditionState(p, expedition);
    if (CoreClient.string(expedition, "state", "").equals("active")
        && expedition.has("remaining_seconds")
        && !expedition.get("remaining_seconds").isJsonNull())
      detail +=
          "\n"
              + tr(
                  p,
                  "text.0_minutes_remaining",
                  Math.max(0, (expedition.get("remaining_seconds").getAsLong() + 59) / 60));
    else if (expedition.has("expires_at") && !expedition.get("expires_at").isJsonNull())
      detail += "\n" + tr(p, "text.scheduled_end_0", date(p, expedition.get("expires_at")));
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
      return tr(p, "text.time_unavailable");
    }
  }

  private String returnConsequences(Player p) {
    return tr(p, "text.keep_the_items_you_carry_placed_blocks_containers_and_d_76fc5d269b");
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
                  tr(p, "text.begin_expedition"),
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
          menu(p, tr(p, "text.expeditions"), entries, 0);
        });
  }

  private String preparationCost(Player p, JsonObject data) {
    JsonObject cost = data.getAsJsonObject("cost");
    return tr(
        p,
        "text.0_coins_1_eyes_of_ender_2_minutes_from_activation",
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
            tr(p, "text.temporary_world"),
            tr(p, "text.the_end_temporary_world_participants_only")
                + "\n"
                + preparationCost(p, data)
                + "\n"
                + returnConsequences(p),
            null));
    entries.add(
        entry(
            ready ? Material.LIME_CONCRETE : Material.GRAY_CONCRETE,
            tr(p, "text.prepare_expedition_62b3cfde"),
            ready
                ? tr(p, "text.participant_consent_is_committed_now_party_changes_afte_323bb8646b")
                : tr(p, "text.resolve_the_requirements_below_then_refresh"),
            ready
                ? () ->
                    confirm(
                        p,
                        tr(p, "text.prepare_expedition_62b3cfde"),
                        preparationCost(p, data)
                            + "\n"
                            + tr(
                                p,
                                "text.participant_consent_is_committed_now_party_changes_afte_323bb8646b"),
                        () -> submit(p, command("expedition_prepare"), ignored -> expeditions(p)))
                : null));
    if (preparation != null) {
      if (!flag(preparation, "is_leader"))
        entries.add(
            entry(
                Material.WHITE_BANNER,
                tr(p, "text.party_leader_required"),
                tr(p, "text.ask_your_party_leader_to_prepare_this_expedition"),
                null));
      if (preparation.has("available_coins")
          && preparation.get("available_coins").getAsLong()
              < data.getAsJsonObject("cost").get("coins").getAsLong())
        entries.add(
            entry(
                Material.GOLD_INGOT,
                tr(p, "text.more_coins_needed"),
                tr(p, "text.available_0_coins", preparation.get("available_coins").getAsLong()),
                null));
      if (preparation.has("participants"))
        for (JsonElement value : preparation.getAsJsonArray("participants")) {
          JsonObject participant = value.getAsJsonObject();
          List<String> requirements = new ArrayList<>();
          if (!flag(participant, "ready")) requirements.add(tr(p, "text.consent_needed"));
          if (!flag(participant, "online")) requirements.add(tr(p, "text.must_be_online_in_smp"));
          if (flag(participant, "in_combat"))
            requirements.add(tr(p, "text.wait_until_combat_ends"));
          if (flag(participant, "occupied"))
            requirements.add(tr(p, "text.finish_the_current_expedition_first"));
          entries.add(
              entry(
                  requirements.isEmpty() ? Material.LIME_DYE : Material.GRAY_DYE,
                  participant.get("name").getAsString(),
                  requirements.isEmpty() ? tr(p, "text.ready") : String.join("\n", requirements),
                  null));
        }
    }
    entries.add(
        entry(
            Material.CAMPFIRE,
            tr(p, "text.party_readiness"),
            tr(p, "text.review_your_party_and_give_consent"),
            () -> social(p, "party")));
    entries.add(
        entry(
            Material.CLOCK,
            tr(p, "text.refresh_requirements"),
            "",
            () -> fetch(p, "expedition", refreshed -> preparation(p, refreshed))));
    menu(p, tr(p, "text.prepare_expedition_62b3cfde"), entries, 0);
  }

  private void expedition(Player p, JsonObject expedition) {
    String id = expedition.get("id").getAsString();
    List<Entry> entries = new ArrayList<>();
    entries.add(
        entry(
            Material.BOOK,
            tr(p, "text.expedition_journal"),
            expeditionSummary(p, expedition)
                + (expedition.has("opens_at") && !expedition.get("opens_at").isJsonNull()
                    ? "\n" + tr(p, "text.opened_0", date(p, expedition.get("opens_at")))
                    : "")
                + "\n"
                + returnConsequences(p),
            null));
    if (flag(expedition, "can_enter"))
      entries.add(
          entry(
              Material.END_STONE,
              tr(p, "text.enter_expedition_137f6536"),
              tr(p, "text.your_return_position_is_saved_before_you_enter"),
              () -> submit(p, command("expedition_enter", "id", id))));
    if (canReturn(p, expedition))
      entries.add(
          entry(
              Material.OAK_DOOR,
              tr(p, "text.return_to_smp"),
              tr(p, "text.return_to_your_saved_position_when_safe_otherwise_a_saf_dba355b350")
                  + "\n"
                  + returnConsequences(p),
              () -> submit(p, command("expedition_return", "id", id))));
    if (flag(expedition, "can_cancel"))
      entries.add(
          entry(
              Material.BARRIER,
              tr(p, "text.cancel_preparation"),
              tr(p, "text.release_reserved_coins_and_return_reserved_materials_to_storage"),
              () ->
                  confirm(
                      p,
                      tr(p, "text.cancel_preparation"),
                      tr(p, "text.release_reserved_coins_and_return_reserved_materials_to_storage"),
                      () ->
                          submit(
                              p,
                              command("expedition_cancel", "id", id),
                              ignored -> expeditions(p)))));
    if (flag(expedition, "can_receive"))
      entries.add(
          entry(
              Material.ENDER_EYE,
              tr(p, "text.collect_refunded_items"),
              tr(p, "text.make_room_in_your_inventory_first"),
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
                tr(p, "text.committed_participant"),
                null));
      }
    entries.add(
        entry(
            Material.CLOCK,
            tr(p, "text.refresh_expedition"),
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
                    tr(p, "text.no_expedition_return_needed"),
                    tr(p, "text.there_is_no_expedition_to_return_from"),
                    null));
          menu(p, tr(p, "text.return_to_smp"), entries, 0);
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
                    invite.get("sender_name").getAsString() + tr(p, "text.invited_you"),
                    invitationKind(p, CoreClient.string(invite, "kind", "")),
                    () ->
                        menu(
                            p,
                            tr(p, "text.respond_to_invitation"),
                            List.of(
                                entry(
                                    Material.LIME_DYE,
                                    tr(p, "text.accept"),
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
                                    tr(p, "text.decline"),
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
                            ? "\n" + tr(p, "text.check_notifications_for_details")
                            : ""),
                    null));
          }
          menu(p, tr(p, "text.invitations_activity"), list, 0);
        });
  }

  private String invitationKind(Player p, String kind) {
    return switch (kind) {
      case "team" -> tr(p, "text.team");
      case "party" -> tr(p, "text.party");
      case "room" -> tr(p, "text.group_conversation");
      default -> tr(p, "text.invitation_af4ea028");
    };
  }

  private String operationTitle(Player p, String kind) {
    if (kind.startsWith("adventure.")) return tr(p, "text.expedition");
    if (kind.equals("server.join")) return tr(p, "text.world_travel");
    if (kind.startsWith("asset.") || kind.startsWith("listing.") || kind.startsWith("material."))
      return tr(p, "text.market");
    if (kind.startsWith("home.")) return tr(p, "text.homes");
    if (kind.startsWith("claim.")) return tr(p, "text.land_b6baff93");
    return tr(p, "text.recent_action");
  }

  private String operationState(Player p, String state) {
    return switch (state) {
      case "queued", "waiting" -> tr(p, "text.waiting");
      case "leased" -> tr(p, "text.in_progress_c1f88e9d");
      case "succeeded" -> tr(p, "text.completed");
      case "failed" -> tr(p, "text.failed");
      case "cancelled" -> tr(p, "text.cancelled");
      case "delivery_unknown" -> tr(p, "text.delivery_could_not_be_confirmed");
      default -> tr(p, "text.checking_progress");
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
          meta.displayName(Component.text(tr(player, "text.game_menu"), NamedTextColor.AQUA));
          meta.lore(
              List.of(
                  Component.text(
                      tr(
                          player,
                          "text.left_right_click_or_click_this_item_in_your_inventory_t_d462da7e22"),
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
                inform(e.getPlayer(), tr(e.getPlayer(), "text.input_cancelled"));
                return;
              }
              try {
                input.action.accept(text);
              } catch (Exception error) {
                inform(
                    e.getPlayer(),
                    tr(e.getPlayer(), "text.check_your_input")
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
      inform(p, tr(p, "text.input_cancelled"));
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
          inform(p, tr(p, "text.join_smp_to_use_homes_or_land"));
          servers(p);
        }
      }
      case "expedition" -> {
        if (!ctx.official()) {
          inform(p, tr(p, "text.join_smp_to_begin_or_return_from_an_expedition"));
          servers(p);
        } else if (args.length > 0 && args[0].equalsIgnoreCase("return")) returns(p);
        else expeditions(p);
      }
      case "tpa" ->
          choosePlayer(
              p,
              tr(p, "text.teleport_request"),
              other ->
                  submit(p, command("teleport_request", "target", other.get("id").getAsString())));
      default -> root(p);
    }
    return true;
  }
}
