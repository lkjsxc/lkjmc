package com.lkjmc.paper;

import com.lkjmc.common.claim.ClaimChunk;
import com.lkjmc.common.claim.ClaimDecision;
import com.lkjmc.common.claim.ClaimEventKind;
import com.lkjmc.common.claim.ClaimProtectionPolicy;
import com.lkjmc.common.i18n.MessageRenderer;
import com.lkjmc.common.permission.PermissionNodes;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import org.bukkit.block.Block;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.block.Action;
import org.bukkit.event.block.BlockBreakEvent;
import org.bukkit.event.block.BlockPlaceEvent;
import org.bukkit.event.player.PlayerInteractEvent;

public final class ClaimProtectionListener implements Listener {
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;
    private final boolean protocolSmoke;
    private final ConcurrentHashMap<UUID, Long> lastMessage = new ConcurrentHashMap<>();

    public ClaimProtectionListener(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
        this.protocolSmoke = "1".equals(System.getenv("LKJMC_CLAIM_PROTOCOL_SMOKE"));
    }

    @EventHandler(ignoreCancelled = true)
    public void onBreak(BlockBreakEvent event) {
        if (deny(event.getPlayer(), event.getBlock(), ClaimEventKind.BREAK)) {
            event.setCancelled(true);
        }
    }

    @EventHandler(ignoreCancelled = true)
    public void onPlace(BlockPlaceEvent event) {
        if (deny(event.getPlayer(), event.getBlock(), ClaimEventKind.PLACE)) {
            event.setCancelled(true);
        }
    }

    @EventHandler(ignoreCancelled = true)
    public void onInteract(PlayerInteractEvent event) {
        var block = event.getClickedBlock();
        if (block != null && !handledByMutationEvent(event)
            && deny(event.getPlayer(), block, ClaimEventKind.INTERACT)) {
            event.setCancelled(true);
        }
    }

    private static boolean handledByMutationEvent(PlayerInteractEvent event) {
        if (event.getAction() == Action.LEFT_CLICK_BLOCK) {
            return true;
        }
        var item = event.getItem();
        return event.getAction() == Action.RIGHT_CLICK_BLOCK && item != null && item.getType().isBlock();
    }

    private boolean deny(Player player, Block block, ClaimEventKind event) {
        var decision = ClaimProtectionPolicy.decide(
            plugin.claims().snapshot(),
            player.getUniqueId().toString(),
            player.hasPermission(PermissionNodes.ADMIN_CLAIM),
            chunk(block),
            event
        );
        if (decision.allowed()) {
            return false;
        }
        logSmoke(event);
        notify(player, decision);
        return true;
    }

    private void logSmoke(ClaimEventKind event) {
        if (protocolSmoke) {
            plugin.getLogger().info("lkjmc claim protocol denied " + event.name().toLowerCase());
        }
    }

    private void notify(Player player, ClaimDecision decision) {
        var now = System.currentTimeMillis();
        var previous = lastMessage.getOrDefault(player.getUniqueId(), 0L);
        if (now - previous < 2000L) {
            return;
        }
        lastMessage.put(player.getUniqueId(), now);
        var owner = decision.claim().map(claim -> claim.ownerName()).orElse("");
        player.sendMessage(renderer.render(plugin.localeService().locale(player), "claim.denied", Map.of("owner", owner)));
    }

    private static ClaimChunk chunk(Block block) {
        var chunk = block.getChunk();
        return new ClaimChunk(instanceId(), block.getWorld().getName(), chunk.getX(), chunk.getZ());
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
