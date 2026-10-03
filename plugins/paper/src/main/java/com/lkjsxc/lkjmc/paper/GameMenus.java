package com.lkjsxc.lkjmc.paper;

import com.google.gson.*;
import com.lkjsxc.lkjmc.common.*;
import io.papermc.paper.event.player.AsyncChatEvent;
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
  private final Map<UUID, Long> requests = new ConcurrentHashMap<>();
  private final Map<UUID, String> selectedLanguages = new ConcurrentHashMap<>();

  private String language(Player p) {
    try {
      return CoreClient.string(ctx.session(p.getUniqueId()), "language", "en");
    } catch (Exception e) {
      return "en";
    }
  }

  private String tr(Player p, String key, Object... values) {
    return Messages.text(selectedLanguages.getOrDefault(p.getUniqueId(), language(p)), key, values);
  }

  private record Entry(Material icon, String title, String description, Runnable action) {}

  private record Input(long expires, Consumer<String> action) {}

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
    if (!p.isOnline()) return;
    Menu previous = current.get(p.getUniqueId());
    Deque<Menu> trail = history.computeIfAbsent(p.getUniqueId(), ignored -> new ArrayDeque<>());
    if (remember && previous != null && previous.entries != entries) {
      if (trail.size() == 12) trail.removeFirst();
      trail.addLast(previous);
    }
    Menu holder = new Menu();
    holder.player = p;
    holder.title = title;
    holder.entries = entries;
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
              Component.text(entry.title, NamedTextColor.AQUA)
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
    p.closeInventory();
    inputs.put(p.getUniqueId(), new Input(System.currentTimeMillis() + 120000, action));
    inform(p, question + tr(p, "\nType in chat (only you can see it). Cancel: cancel"));
  }

  private void confirm(Player p, String title, String detail, Runnable action) {
    menu(
        p,
        title,
        List.of(
            entry(Material.LIME_CONCRETE, tr(p, "Confirm"), detail, action),
            entry(Material.RED_CONCRETE, tr(p, "Cancel"), "", () -> root(p))),
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
    p.closeInventory();
    long request = requests.merge(p.getUniqueId(), 1L, Long::sum);
    ctx.async(
        () -> {
          try {
            JsonObject data = view(p, name, new JsonObject());
            ctx.main(
                () -> {
                  if (p.isOnline() && Objects.equals(requests.get(p.getUniqueId()), request))
                    render.accept(data);
                  return null;
                });
          } catch (Exception e) {
            error(p, e);
          }
        });
  }

  private void error(Player p, Exception e) {
    selectedLanguages.remove(p.getUniqueId());
    Bukkit.getScheduler()
        .runTask(
            ctx.plugin(),
            () ->
                inform(
                    p,
                    tr(p, "Could not complete action: ")
                        + Messages.error(language(p), e.getMessage())));
  }

  private void submit(Player p, JsonObject command) {
    submit(p, command, null);
  }

  private void submit(Player p, JsonObject command, Consumer<JsonObject> completed) {
    p.closeInventory();
    UUID request = UUID.randomUUID();
    ctx.async(
        () -> {
          try {
            JsonObject result =
                ctx.core()
                    .command(ctx.session(p.getUniqueId()), command, request)
                    .getAsJsonObject("result");
            ctx.main(
                () -> {
                  inform(
                      p,
                      result.has("job_id")
                          ? tr(p, "Request accepted. You will be notified when it finishes.")
                          : tr(p, "Saved."));
                  if (!result.has("job_id") && completed != null) completed.accept(result);
                  if (result.has("code"))
                    inform(
                        p,
                        tr(p, "Link code: ")
                            + result.get("code").getAsString()
                            + tr(p, " (expires in 10 minutes; use only on your own account)"));
                  return null;
                });
            if (result.has("job_id")) {
              for (int attempt = 0; attempt < 90 && p.isOnline(); attempt++) {
                Thread.sleep(2000);
                JsonObject status =
                    view(p, "job", CoreClient.object("id", result.get("job_id").getAsString()));
                String state = status.get("state").getAsString();
                if (Set.of("succeeded", "failed", "cancelled").contains(state)) {
                  ctx.main(
                      () -> {
                        inform(
                            p,
                            state.equals("succeeded")
                                ? tr(p, "Action completed.")
                                : tr(p, "Action failed: ")
                                    + CoreClient.string(
                                        status,
                                        "error",
                                        tr(p, "Check notifications for details.")));
                        if (state.equals("succeeded") && completed != null)
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
            ctx.async(
                () -> {
                  try {
                    JsonObject result = view(p, "players", CoreClient.object("q", text));
                    ctx.main(
                        () -> {
                          List<Entry> list = new ArrayList<>();
                          for (JsonElement element : result.getAsJsonArray("players")) {
                            JsonObject item = element.getAsJsonObject();
                            list.add(
                                entry(
                                    Material.PLAYER_HEAD,
                                    item.get("name").getAsString(),
                                    CoreClient.string(item, "rank", ""),
                                    () -> selected.accept(item)));
                          }
                          menu(p, title, list, 0);
                          return null;
                        });
                  } catch (Exception e) {
                    error(p, e);
                  }
                }));
  }

  public void root(Player p) {
    requests.merge(p.getUniqueId(), 1L, Long::sum);
    history.remove(p.getUniqueId());
    current.remove(p.getUniqueId());
    selectedLanguages.remove(p.getUniqueId());
    menu(
        p,
        "lkjmc",
        List.of(
            entry(
                Material.COMPASS,
                tr(p, "Servers"),
                tr(p, "Join a server. SMP tools live in its details."),
                () -> servers(p)),
            entry(
                Material.PLAYER_HEAD,
                tr(p, "Friends"),
                tr(p, "Friends and requests"),
                () -> social(p, "friends")),
            entry(
                Material.WRITABLE_BOOK,
                tr(p, "Chat"),
                tr(p, "Private and group conversations"),
                () -> social(p, "chat")),
            entry(
                Material.WHITE_BANNER,
                tr(p, "Teams"),
                tr(p, "Shared land, coins and permissions"),
                () -> social(p, "team")),
            entry(
                Material.CAMPFIRE,
                tr(p, "Parties"),
                tr(p, "Invitations and adventure readiness"),
                () -> social(p, "party")),
            entry(
                Material.BELL,
                tr(p, "Invitations & activity"),
                tr(p, "Respond to invitations and check progress"),
                () -> notifications(p)),
            entry(
                Material.NAME_TAG,
                tr(p, "Account linking"),
                tr(p, "Verify and link your game and Web identities"),
                () -> link(p)),
            entry(
                Material.WRITABLE_BOOK,
                tr(p, "Language"),
                tr(p, "Your language is shared with linked game accounts."),
                () -> languages(p)),
            entry(
                Material.BOOK,
                tr(p, "Help"),
                tr(p, "Getting started and useful commands"),
                () -> help(p))),
        0);
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
                tr(p, "Join a server"),
                tr(p, "Open Servers, choose a server, then Join."),
                () -> servers(p)),
            entry(
                Material.GRASS_BLOCK,
                tr(p, "SMP tools"),
                tr(p, "Open the SMP server details for land, market and adventures."),
                () -> servers(p)),
            entry(
                Material.NAME_TAG,
                tr(p, "Account linking"),
                "https://lkjmc.lkjsxc.com",
                () -> link(p)),
            entry(
                Material.BOOK,
                tr(p, "Commands"),
                "/lkjmc · /home · /claim · /tpa\n/lkjmc cancel",
                null)),
        0);
  }

  private void smp(Player p, JsonObject server) {
    String state = server.get("observed").getAsString();
    boolean available =
        !CoreClient.string(server, "maintenance", "false").equals("true")
            && Set.of("running", "stopped").contains(state);
    menu(
        p,
        server.get("name").getAsString(),
        List.of(
            entry(
                available ? Material.GRASS_BLOCK : Material.GRAY_DYE,
                tr(p, "Join"),
                tr(p, "Status: ") + state,
                available
                    ? () -> submit(p, command("server_join", "id", server.get("id").getAsString()))
                    : null),
            entry(
                Material.OAK_DOOR,
                tr(p, "Land & assets"),
                tr(p, "Claims, homes, achievements and balance"),
                () -> life(p)),
            entry(
                Material.EMERALD,
                tr(p, "Market"),
                tr(p, "Buy, list, collect and sell materials"),
                () -> market(p)),
            entry(
                Material.ENDER_EYE,
                tr(p, "Private End"),
                tr(p, "Open a private End for three hours"),
                () -> adventures(p))),
        0);
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
            inform(p, e.getMessage());
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
                    tr(p, "Status: ")
                        + s.get("observed").getAsString()
                        + "\n"
                        + s.get("players").getAsInt()
                        + tr(p, " players / ")
                        + s.get("version").getAsString(),
                    s.get("kind").getAsString().equals("official")
                        ? () -> smp(p, s)
                        : !s.get("maintenance").getAsBoolean()
                                && Set.of("running", "stopped")
                                    .contains(s.get("observed").getAsString())
                            ? () ->
                                submit(p, command("server_join", "id", s.get("id").getAsString()))
                            : null));
          }
          menu(p, tr(p, "Servers"), list, 0);
        });
  }

  private void life(Player p) {
    fetch(
        p,
        "life",
        data -> {
          List<Entry> list = new ArrayList<>();
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
          list.add(
              entry(
                  Material.RED_BED,
                  tr(p, "Set a home here"),
                  tr(p, "Up to three homes"),
                  () ->
                      input(
                          p,
                          tr(p, "Home name"),
                          name -> submit(p, command("home_set", "name", name)))));
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
          list.add(
              entry(
                  Material.OAK_FENCE,
                  tr(p, "Protect this chunk"),
                  tr(p, "16 × 16 blocks, survival world only"),
                  () -> {
                    int x = p.getLocation().getBlockX() >> 4, z = p.getLocation().getBlockZ() >> 4;
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
                  }));
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
          for (JsonElement value : data.getAsJsonArray("achievements")) {
            JsonObject a = value.getAsJsonObject();
            list.add(
                entry(
                    Material.EXPERIENCE_BOTTLE,
                    a.get("title").getAsString(),
                    a.get("description").getAsString()
                        + "\n"
                        + a.get("progress")
                        + " / "
                        + a.get("target"),
                    null));
          }
          menu(p, tr(p, "Land & assets"), list, 0);
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
                      tr(p, "Awaiting consent: ") + a.get("title").getAsString(),
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
                    tr(p, "Stored: ") + a.get("title").getAsString(),
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
                      menu(p, a.get("title").getAsString(), actions, 0);
                    }));
          }
          for (JsonElement value : data.getAsJsonArray("listings")) {
            JsonObject l = value.getAsJsonObject();
            list.add(
                entry(
                    Material.EMERALD,
                    l.get("title").getAsString(),
                    l.get("price") + tr(p, " coins / ") + l.get("seller_name").getAsString(),
                    () ->
                        confirm(
                            p,
                            tr(p, "Buy"),
                            l.get("title").getAsString() + " / " + l.get("price") + tr(p, " coins"),
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
              JsonObject f = value.getAsJsonObject();
              list.add(
                  entry(
                      Material.PLAYER_HEAD,
                      f.get("name").getAsString(),
                      f.get("state").getAsString(),
                      () ->
                          menu(
                              p,
                              f.get("name").getAsString(),
                              List.of(
                                  entry(
                                      Material.LIME_DYE,
                                      tr(p, "Accept friend request"),
                                      "",
                                      () ->
                                          submit(
                                              p,
                                              command(
                                                  "friend_respond",
                                                  "target",
                                                  f.get("id").getAsString(),
                                                  "accept",
                                                  true))),
                                  entry(
                                      Material.PAPER,
                                      tr(p, "Open DM"),
                                      "",
                                      () ->
                                          submit(
                                              p,
                                              command(
                                                  "direct_room",
                                                  "target",
                                                  f.get("id").getAsString()))),
                                  entry(
                                      Material.BARRIER,
                                      tr(p, " blocks"),
                                      "",
                                      () ->
                                          confirm(
                                              p,
                                              tr(p, "Block player"),
                                              f.get("name").getAsString(),
                                              () ->
                                                  submit(
                                                      p,
                                                      command(
                                                          "block",
                                                          "target",
                                                          f.get("id").getAsString(),
                                                          "blocked",
                                                          true))))),
                              0)));
            }
          }
          for (String key : List.of("team", "party"))
            if (section.equals(key) && !data.get(key).isJsonNull()) {
              JsonObject group = data.getAsJsonObject(key);
              list.add(
                  entry(
                      Material.BELL,
                      group.get("name").getAsString(),
                      key,
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
                                  tr(p, "Ready for adventure"),
                                  tr(p, "Agree to join and pay your share"),
                                  () -> submit(p, command("party_ready", "ready", true))));
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
                case "chat" -> tr(p, "Chat");
                default -> tr(p, "Friends");
              },
              list,
              0);
        });
  }

  private void chat(Player p, JsonObject room) {
    ctx.async(
        () -> {
          try {
            JsonObject history =
                view(p, "messages", CoreClient.object("room", room.get("id").getAsString()));
            ctx.main(
                () -> {
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
                  return null;
                });
          } catch (Exception e) {
            error(p, e);
          }
        });
  }

  private void adventures(Player p) {
    fetch(
        p,
        "adventure",
        data -> {
          List<Entry> list = new ArrayList<>();
          list.add(
              entry(
                  Material.ENDER_EYE,
                  tr(p, "Open private End"),
                  tr(p, "1,000 coins + 12 Eyes of Ender / 3 hours"),
                  () ->
                      confirm(
                          p,
                          tr(p, "Open private End"),
                          tr(p, "Uncollected drops are lost when the world closes"),
                          () -> submit(p, command("adventure_create")))));
          for (JsonElement value : data.getAsJsonArray("adventures")) {
            JsonObject a = value.getAsJsonObject();
            String state = a.get("state").getAsString();
            if (state.equals("active"))
              list.add(
                  entry(
                      Material.END_STONE,
                      tr(p, "Enter End"),
                      CoreClient.string(a, "expires_at", ""),
                      () -> submit(p, command("adventure_join", "id", a.get("id").getAsString()))));
            if (a.get("can_cancel").getAsBoolean())
              list.add(
                  entry(
                      Material.BARRIER,
                      tr(p, "Cancel preparation"),
                      tr(p, "Release reserved coins and return reserved materials to storage"),
                      () ->
                          submit(p, command("adventure_cancel", "id", a.get("id").getAsString()))));
            if (a.get("can_receive").getAsBoolean())
              list.add(
                  entry(
                      Material.ENDER_EYE,
                      tr(p, "Collect refunded items"),
                      tr(p, "Make room in your inventory first"),
                      () ->
                          submit(
                              p,
                              command(
                                  "asset_receive", "id", a.get("material_asset").getAsString()))));
          }
          menu(p, tr(p, "Adventures"), list, 0);
        });
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
                    invite.get("kind").getAsString(),
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
                    job.get("kind").getAsString(),
                    job.get("state").getAsString() + "\n" + CoreClient.string(job, "error", ""),
                    null));
          }
          menu(p, tr(p, "Invitations & activity"), list, 0);
        });
  }

  private NamespacedKey launcherKey() {
    return new NamespacedKey(ctx.plugin(), "menu_launcher");
  }

  private boolean launcher(ItemStack item) {
    return item != null
        && item.hasItemMeta()
        && item.getItemMeta()
            .getPersistentDataContainer()
            .has(launcherKey(), org.bukkit.persistence.PersistentDataType.BYTE);
  }

  @EventHandler
  public void joined(org.bukkit.event.player.PlayerJoinEvent event) {
    // Only the explicitly configured lobby role owns this optional hotbar item.
    if (ctx.official()) return;
    Player player = event.getPlayer();
    ItemStack existing = player.getInventory().getItem(8);
    if (existing != null && !existing.getType().isAir() && !launcher(existing)) return;
    if (!launcher(existing))
      for (ItemStack item : player.getInventory().getContents()) if (launcher(item)) return;
    ItemStack item = new ItemStack(Material.COMPASS);
    item.editMeta(
        meta -> {
          meta.displayName(Component.text("lkjmc", NamedTextColor.AQUA));
          meta.lore(
              List.of(
                  Component.text(tr(player, "Right-click to open the menu"), NamedTextColor.GRAY)));
          meta.getPersistentDataContainer()
              .set(launcherKey(), org.bukkit.persistence.PersistentDataType.BYTE, (byte) 1);
        });
    player.getInventory().setItem(8, item);
  }

  @EventHandler(priority = EventPriority.HIGHEST)
  public void openLauncher(org.bukkit.event.player.PlayerInteractEvent event) {
    if (ctx.official()
        || event.getHand() != org.bukkit.inventory.EquipmentSlot.HAND
        || !launcher(event.getItem())) return;
    if (event.getAction() == org.bukkit.event.block.Action.RIGHT_CLICK_AIR
        || event.getAction() == org.bukkit.event.block.Action.RIGHT_CLICK_BLOCK) {
      event.setCancelled(true);
      root(event.getPlayer());
    }
  }

  @EventHandler(priority = EventPriority.HIGHEST)
  public void click(InventoryClickEvent e) {
    if (e.getView().getTopInventory().getHolder() instanceof Menu menu) {
      e.setCancelled(true);
      if (!menu.consumed
          && (e.getClick() == ClickType.LEFT || e.getClick() == ClickType.RIGHT)
          && e.getRawSlot() >= 0
          && e.getRawSlot() < 54
          && menu.actions.containsKey(e.getRawSlot())) {
        menu.consumed = true;
        Bukkit.getScheduler()
            .runTask(
                ctx.plugin(),
                () -> {
                  if (menu.player.isOnline()
                      && menu.player.getOpenInventory().getTopInventory().getHolder() == menu)
                    menu.actions.get(e.getRawSlot()).run();
                });
      }
    }
  }

  @EventHandler(priority = EventPriority.HIGHEST)
  public void drag(InventoryDragEvent e) {
    if (e.getView().getTopInventory().getHolder() instanceof Menu) e.setCancelled(true);
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
              if (input.expires < System.currentTimeMillis() || text.equalsIgnoreCase("cancel")) {
                inform(e.getPlayer(), tr(e.getPlayer(), "Input cancelled."));
                return;
              }
              try {
                input.action.accept(text);
              } catch (Exception error) {
                inform(e.getPlayer(), tr(e.getPlayer(), "Check your input: ") + error.getMessage());
              }
            });
  }

  @EventHandler
  public void quit(org.bukkit.event.player.PlayerQuitEvent event) {
    UUID id = event.getPlayer().getUniqueId();
    inputs.remove(id);
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
      inform(p, tr(p, "Input cancelled."));
      return true;
    }
    if (args.length > 0 && Set.of("pos1", "pos2").contains(args[0])) {
      try {
        selectPoint(p, args[0].equals("pos2"));
      } catch (Exception e) {
        inform(p, e.getMessage());
      }
      return true;
    }
    switch (command.getName()) {
      case "home", "claim" -> life(p);
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
