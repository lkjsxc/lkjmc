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

  private record Entry(Material icon, String title, String description, Runnable action) {}

  private record Input(long expires, Consumer<String> action) {}

  private final Map<UUID, Input> inputs = new ConcurrentHashMap<>();

  private static final class Menu implements InventoryHolder {
    Inventory inventory;
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
    Menu holder = new Menu();
    holder.inventory = Bukkit.createInventory(holder, 54, Component.text(title));
    int offset = page * 45;
    for (int i = 0; i < 45 && offset + i < entries.size(); i++)
      put(holder, i, entries.get(offset + i));
    if (page > 0)
      put(holder, 45, entry(Material.ARROW, "前のページ", "", () -> menu(p, title, entries, page - 1)));
    put(holder, 49, entry(Material.COMPASS, "メインメニュー", "", () -> root(p)));
    if (offset + 45 < entries.size())
      put(holder, 53, entry(Material.ARROW, "次のページ", "", () -> menu(p, title, entries, page + 1)));
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
              Arrays.stream(entry.description.split("\n"))
                  .filter(s -> !s.isBlank())
                  .map(
                      s ->
                          Component.text(s, NamedTextColor.GRAY)
                              .decoration(
                                  net.kyori.adventure.text.format.TextDecoration.ITALIC, false))
                  .toList());
        });
    holder.inventory.setItem(slot, stack);
    holder.actions.put(slot, entry.action);
  }

  private void inform(Player p, String text) {
    p.sendMessage(Component.text(text));
  }

  private void input(Player p, String question, Consumer<String> action) {
    p.closeInventory();
    inputs.put(p.getUniqueId(), new Input(System.currentTimeMillis() + 120000, action));
    inform(p, question + "\nチャットに入力してください（他の人には送信されません）。中止: cancel");
  }

  private void confirm(Player p, String title, String detail, Runnable action) {
    menu(
        p,
        title,
        List.of(
            entry(Material.LIME_CONCRETE, "実行する", detail, action),
            entry(Material.RED_CONCRETE, "取り消す", "", () -> root(p))),
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
    ctx.async(
        () -> {
          try {
            JsonObject data = view(p, name, new JsonObject());
            ctx.main(
                () -> {
                  if (p.isOnline()) render.accept(data);
                  return null;
                });
          } catch (Exception e) {
            error(p, e);
          }
        });
  }

  private void error(Player p, Exception e) {
    Bukkit.getScheduler().runTask(ctx.plugin(), () -> inform(p, "操作できません: " + e.getMessage()));
  }

  private void submit(Player p, JsonObject command) {
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
                  inform(p, result.has("job_id") ? "処理を受け付けました。完了後にお知らせします。" : "反映しました。");
                  if (result.has("code"))
                    inform(
                        p, "連携コード: " + result.get("code").getAsString() + "（10分以内・本人のアカウントだけに入力）");
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
                                ? "処理が完了しました。"
                                : "処理を完了できませんでした: "
                                    + CoreClient.string(status, "error", "詳細は通知をご確認ください。"));
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
        "相手の名前を入力してください",
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
    menu(
        p,
        "lkjmc",
        List.of(
            entry(Material.COMPASS, "遊びに行く", "サーバーを選んで参加", () -> servers(p)),
            entry(Material.OAK_DOOR, "暮らし", "土地・ホーム・実績・残高", () -> life(p)),
            entry(Material.EMERALD, "マーケット", "購入・出品・受け取り・素材買取", () -> market(p)),
            entry(Material.PLAYER_HEAD, "つながり", "フレンド・チャット・チーム・パーティー", () -> social(p)),
            entry(Material.ENDER_EYE, "冒険", "専用エンドを3時間開く", () -> adventures(p)),
            entry(Material.BELL, "招待・処理結果", "招待への返答と処理状況", () -> notifications(p)),
            entry(Material.NAME_TAG, "アカウント連携", "ゲームIDとWebアカウントを本人確認して連携", () -> link(p))),
        0);
  }

  private void link(Player p) {
    menu(
        p,
        "アカウント連携",
        List.of(
            entry(
                Material.PAPER,
                "コードを発行",
                "連携後に残すプレイデータを1つ選びます",
                () -> submit(p, command("link_begin"))),
            entry(
                Material.WRITABLE_BOOK,
                "コードを入力",
                "",
                () ->
                    input(
                        p,
                        "自分の別アカウントで発行したコード",
                        code -> submit(p, command("link_present", "code", code))))),
        0);
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
                    "状態: "
                        + s.get("observed").getAsString()
                        + "\n"
                        + s.get("players").getAsInt()
                        + "人 / "
                        + s.get("version").getAsString(),
                    () -> submit(p, command("server_join", "id", s.get("id").getAsString()))));
          }
          menu(p, "遊びに行く", list, 0);
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
                    "残高 "
                        + owner.getAsJsonObject("wallet").get("balance").getAsLong()
                        + " コイン\n土地 "
                        + owner.get("used_chunks")
                        + " / "
                        + owner.getAsJsonObject("land").get("chunks"),
                    () -> {}));
          }
          list.add(
              entry(
                  Material.RED_BED,
                  "今いる場所をホームに登録",
                  "最大3件",
                  () -> input(p, "ホームの名前", name -> submit(p, command("home_set", "name", name)))));
          for (JsonElement value : data.getAsJsonArray("homes")) {
            JsonObject h = value.getAsJsonObject();
            list.add(
                entry(
                    Material.OAK_DOOR,
                    h.get("name").getAsString(),
                    "移動または削除",
                    () ->
                        menu(
                            p,
                            h.get("name").getAsString(),
                            List.of(
                                entry(
                                    Material.ENDER_PEARL,
                                    "ここへ移動",
                                    "",
                                    () ->
                                        submit(
                                            p,
                                            command(
                                                "home_travel", "id", h.get("id").getAsString()))),
                                entry(
                                    Material.BARRIER,
                                    "ホーム登録を削除",
                                    "土地・建物はそのままです",
                                    () ->
                                        confirm(
                                            p,
                                            "ホーム削除",
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
                  "今いるチャンクを保護",
                  "16×16ブロック・生活ワールド限定",
                  () -> {
                    int x = p.getLocation().getBlockX() >> 4, z = p.getLocation().getBlockZ() >> 4;
                    input(
                        p,
                        "土地の名前",
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
                    c.get("chunks") + "チャンク / " + c.get("state").getAsString(),
                    () ->
                        confirm(
                            p,
                            "保護を解除する",
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
                    () -> {}));
          }
          menu(p, "暮らし", list, 0);
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
                  Material.CHEST,
                  "手持ちアイテムを預ける",
                  "持っている1スタックを保管し、出品できます",
                  () ->
                      input(
                          p,
                          "出品物の名前",
                          name ->
                              confirm(
                                  p,
                                  "アイテムを預ける",
                                  "手に持ったスタックが持ち物から移動します",
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
                  "素材をNPCへ売る",
                  "今日の残り " + data.get("npc_remaining") + " コイン",
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
                              "1個 " + price.get("price") + "コイン",
                              () ->
                                  input(
                                      p,
                                      "売る個数",
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
                    menu(p, "素材買取", prices, 0);
                  }));
          for (JsonElement value : data.getAsJsonArray("assets")) {
            JsonObject a = value.getAsJsonObject();
            if (!a.get("state").getAsString().equals("escrowed")) continue;
            list.add(
                entry(
                    Material.BARREL,
                    "預かり: " + a.get("title").getAsString(),
                    a.get("kind").getAsString(),
                    () -> {
                      List<Entry> actions = new ArrayList<>();
                      actions.add(
                          entry(
                              Material.EMERALD,
                              "出品する",
                              "成約時の手数料5%",
                              () ->
                                  input(
                                      p,
                                      "価格（コイン・整数）",
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
                                "持ち物へ受け取る",
                                "空きが足りない場合は預かりを継続",
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
                    l.get("price") + "コイン / " + l.get("seller_name").getAsString(),
                    () ->
                        confirm(
                            p,
                            "購入する",
                            l.get("title").getAsString() + " / " + l.get("price") + "コイン",
                            () ->
                                submit(
                                    p, command("listing_buy", "id", l.get("id").getAsString())))));
          }
          menu(p, "マーケット", list, 0);
        });
  }

  private void social(Player p) {
    fetch(
        p,
        "social",
        data -> {
          List<Entry> list = new ArrayList<>();
          list.add(
              entry(
                  Material.PLAYER_HEAD,
                  "フレンドを追加",
                  "相手の承認が必要です",
                  () ->
                      choosePlayer(
                          p,
                          "フレンド申請",
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
                  "会いに行く",
                  "相手に移動の承諾を求めます",
                  () ->
                      choosePlayer(
                          p,
                          "移動申請",
                          other ->
                              submit(
                                  p,
                                  command(
                                      "teleport_request",
                                      "target",
                                      other.get("id").getAsString())))));
          list.add(
              entry(
                  Material.WHITE_BANNER,
                  "チームを作る",
                  "1アカウント1チーム・共用の土地と残高",
                  () -> input(p, "チーム名", name -> submit(p, command("team_create", "name", name)))));
          list.add(
              entry(
                  Material.CAMPFIRE,
                  "パーティーを作る",
                  "一緒に遊ぶための一時グループ",
                  () ->
                      input(
                          p, "パーティー名", name -> submit(p, command("party_create", "name", name)))));
          list.add(
              entry(
                  Material.WRITABLE_BOOK,
                  "グループチャットを作る",
                  "",
                  () ->
                      input(p, "グループ名", name -> submit(p, command("room_create", "name", name)))));
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
                                    "フレンド申請を承認",
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
                                    "DMを開く",
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
                                    "ブロック",
                                    "",
                                    () ->
                                        confirm(
                                            p,
                                            "ブロックする",
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
          for (String key : List.of("team", "party"))
            if (!data.get(key).isJsonNull()) {
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
                                "メンバーを招待",
                                "",
                                () ->
                                    choosePlayer(
                                        p,
                                        "招待",
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
                                  "冒険の準備完了",
                                  "参加と費用に同意して準備完了にする",
                                  () -> submit(p, command("party_ready", "ready", true))));
                        actions.add(
                            entry(
                                Material.OAK_DOOR,
                                "脱退",
                                "リーダーは先に委譲が必要です",
                                () ->
                                    confirm(
                                        p,
                                        "脱退する",
                                        group.get("name").getAsString(),
                                        () -> submit(p, command(key + "_leave")))));
                        menu(p, group.get("name").getAsString(), actions, 0);
                      }));
            }
          for (JsonElement value : data.getAsJsonArray("rooms")) {
            JsonObject room = value.getAsJsonObject();
            list.add(
                entry(
                    Material.WRITABLE_BOOK,
                    room.get("name").getAsString(),
                    "未読 " + room.get("unread"),
                    () -> chat(p, room)));
          }
          menu(p, "つながり", list, 0);
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
                          "メッセージを送る",
                          "",
                          () ->
                              input(
                                  p,
                                  "メッセージ",
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
                          "メンバーを招待",
                          "",
                          () ->
                              choosePlayer(
                                  p,
                                  "招待",
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
                                : "削除されたメッセージ",
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
                  "専用エンドを開く",
                  "1,000コイン + エンダーアイ12個 / 3時間",
                  () ->
                      confirm(
                          p,
                          "専用エンドを開く",
                          "終了時に未回収のドロップは失われます",
                          () -> submit(p, command("adventure_create")))));
          for (JsonElement value : data.getAsJsonArray("adventures")) {
            JsonObject a = value.getAsJsonObject();
            String state = a.get("state").getAsString();
            if (state.equals("active"))
              list.add(
                  entry(
                      Material.END_STONE,
                      "エンドへ入る",
                      CoreClient.string(a, "expires_at", ""),
                      () -> submit(p, command("adventure_join", "id", a.get("id").getAsString()))));
            if (Set.of("preparing", "activating").contains(state))
              list.add(
                  entry(
                      Material.BARRIER,
                      "準備を取り消す",
                      "開始前なら費用が返却されます",
                      () ->
                          submit(p, command("adventure_cancel", "id", a.get("id").getAsString()))));
          }
          menu(p, "冒険", list, 0);
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
                    invite.get("sender_name").getAsString() + "からの招待",
                    invite.get("kind").getAsString(),
                    () ->
                        menu(
                            p,
                            "招待に返答",
                            List.of(
                                entry(
                                    Material.LIME_DYE,
                                    "承諾",
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
                                    "辞退",
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
                    () -> {}));
          }
          menu(p, "招待・処理結果", list, 0);
        });
  }

  @EventHandler(priority = EventPriority.HIGHEST)
  public void click(InventoryClickEvent e) {
    if (e.getView().getTopInventory().getHolder() instanceof Menu menu) {
      e.setCancelled(true);
      if (e.getRawSlot() >= 0 && e.getRawSlot() < 54 && menu.actions.containsKey(e.getRawSlot()))
        Bukkit.getScheduler().runTask(ctx.plugin(), menu.actions.get(e.getRawSlot()));
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
                inform(e.getPlayer(), "入力を取り消しました。");
                return;
              }
              try {
                input.action.accept(text);
              } catch (Exception error) {
                inform(e.getPlayer(), "入力を確認してください: " + error.getMessage());
              }
            });
  }

  @Override
  public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
    if (!(sender instanceof Player p)) {
      sender.sendMessage("ゲーム内から操作してください。");
      return true;
    }
    if (args.length > 0 && args[0].equalsIgnoreCase("cancel")) {
      inputs.remove(p.getUniqueId());
      inform(p, "入力を取り消しました。");
      return true;
    }
    switch (command.getName()) {
      case "home", "claim" -> life(p);
      case "tpa" ->
          choosePlayer(
              p,
              "移動申請",
              other ->
                  submit(p, command("teleport_request", "target", other.get("id").getAsString())));
      default -> root(p);
    }
    return true;
  }
}
