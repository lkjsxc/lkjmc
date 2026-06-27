package com.lkjmc.smoke;

import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.ArrayList;
import java.util.UUID;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import net.kyori.adventure.text.serializer.plain.PlainTextComponentSerializer;
import org.cloudburstmc.math.vector.Vector3d;
import org.cloudburstmc.math.vector.Vector3i;
import org.geysermc.mcprotocollib.auth.GameProfile;
import org.geysermc.mcprotocollib.network.ClientSession;
import org.geysermc.mcprotocollib.network.Session;
import org.geysermc.mcprotocollib.network.event.session.DisconnectedEvent;
import org.geysermc.mcprotocollib.network.event.session.SessionAdapter;
import org.geysermc.mcprotocollib.network.factory.ClientNetworkSessionFactory;
import org.geysermc.mcprotocollib.network.packet.Packet;
import org.geysermc.mcprotocollib.protocol.MinecraftProtocol;
import org.geysermc.mcprotocollib.protocol.data.game.entity.object.Direction;
import org.geysermc.mcprotocollib.protocol.data.game.entity.player.Hand;
import org.geysermc.mcprotocollib.protocol.data.game.entity.player.PlayerAction;
import org.geysermc.mcprotocollib.protocol.packet.ingame.clientbound.ClientboundSystemChatPacket;
import org.geysermc.mcprotocollib.protocol.packet.ingame.clientbound.entity.player.ClientboundPlayerPositionPacket;
import org.geysermc.mcprotocollib.protocol.packet.ingame.serverbound.ServerboundChatCommandPacket;
import org.geysermc.mcprotocollib.protocol.packet.ingame.serverbound.level.ServerboundAcceptTeleportationPacket;
import org.geysermc.mcprotocollib.protocol.packet.ingame.serverbound.player.*;

public final class MinecraftClaimProtocolSmoke {
    private static final String OWNER = "ClaimOwner";
    private static final String STRANGER = "ClaimStranger";

    private MinecraftClaimProtocolSmoke() {}

    public static void main(String[] args) throws Exception {
        if (args.length != 2) {
            throw new IllegalArgumentException("usage: <host> <port>");
        }
        var host = args[0];
        var port = Integer.parseInt(args[1]);
        try (var owner = new Client(host, port, OWNER); var stranger = new Client(host, port, STRANGER)) {
            owner.connect();
            stranger.connect();
            owner.command("claim create ProtocolBase");
            owner.awaitMessages("Claim created.", 1, Duration.ofSeconds(20));
            Thread.sleep(3000L);
            owner.command("tp " + STRANGER + " " + OWNER);
            stranger.awaitSameChunk(owner, Duration.ofSeconds(10));
            owner.command("setblock " + stranger.blockTarget() + " minecraft:stone replace");
            owner.command("item replace entity " + STRANGER + " hotbar.0 with minecraft:dirt 1");
            Thread.sleep(1500L);
            stranger.breakBlock();
            stranger.awaitMessages("This chunk is claimed by " + OWNER + ".", 1, Duration.ofSeconds(20));
            Thread.sleep(2500L);
            stranger.placeBlock();
            stranger.awaitMessages("This chunk is claimed by " + OWNER + ".", 2, Duration.ofSeconds(20));
        }
        System.out.println("ok minecraft claim protocol smoke");
    }

    private static final class Client implements AutoCloseable {
        private final String host;
        private final int port;
        private final String name;
        private final CountDownLatch joined = new CountDownLatch(1);
        private final ArrayList<String> messages = new ArrayList<>();
        private ClientSession session;
        private volatile Vector3d position = Vector3d.from(0.5D, 64.0D, 0.5D);
        private int sequence = 1;

        Client(String host, int port, String name) {
            this.host = host;
            this.port = port;
            this.name = name;
        }

        void connect() throws InterruptedException {
            var profile = new GameProfile(offlineUuid(name), name);
            var protocol = new MinecraftProtocol(profile, "");
            session = ClientNetworkSessionFactory.factory()
                .setAddress(host, port)
                .setProtocol(protocol)
                .create();
            session.addListener(new Listener());
            session.connect(false);
            if (!joined.await(45, TimeUnit.SECONDS)) {
                throw new IllegalStateException(name + " did not join play state");
            }
        }

        void command(String command) {
            session.send(new ServerboundChatCommandPacket(command));
        }

        void breakBlock() {
            var block = floorBlock();
            session.send(new ServerboundSwingPacket(Hand.MAIN_HAND));
            session.send(new ServerboundPlayerActionPacket(
                PlayerAction.START_DIGGING, block, Direction.UP, sequence++
            ));
            session.send(new ServerboundPlayerActionPacket(
                PlayerAction.FINISH_DIGGING, block, Direction.UP, sequence++
            ));
        }

        void placeBlock() {
            session.send(new ServerboundSetCarriedItemPacket(0));
            session.send(new ServerboundSwingPacket(Hand.MAIN_HAND));
            session.send(new ServerboundUseItemOnPacket(
                floorBlock(), Direction.EAST, Hand.MAIN_HAND, 1.0F, 0.5F, 0.5F, false, false, sequence++
            ));
        }

        void awaitMessages(String needle, int count, Duration timeout) throws InterruptedException {
            var deadline = System.nanoTime() + timeout.toNanos();
            while (System.nanoTime() < deadline) {
                synchronized (messages) {
                    var matches = messages.stream().filter(message -> message.contains(needle)).count();
                    if (matches >= count) {
                        return;
                    }
                }
                Thread.sleep(100L);
            }
            throw new IllegalStateException(name + " did not receive " + needle + " x" + count);
        }

        void awaitSameChunk(Client other, Duration timeout) throws InterruptedException {
            var deadline = System.nanoTime() + timeout.toNanos();
            while (System.nanoTime() < deadline) {
                if (chunkX() == other.chunkX() && chunkZ() == other.chunkZ()) {
                    return;
                }
                Thread.sleep(100L);
            }
            throw new IllegalStateException(name + " was not teleported to " + other.name);
        }

        @Override
        public void close() {
            if (session != null && session.isConnected()) {
                session.disconnect("smoke complete");
            }
        }

        private Vector3i floorBlock() {
            return Vector3i.from(
                (int) Math.floor(position.getX()),
                (int) Math.floor(position.getY() - 1.0D),
                (int) Math.floor(position.getZ())
            );
        }

        private int chunkX() {
            return ((int) Math.floor(position.getX())) >> 4;
        }

        private int chunkZ() {
            return ((int) Math.floor(position.getZ())) >> 4;
        }

        private String blockTarget() {
            var block = floorBlock();
            return block.getX() + " " + block.getY() + " " + block.getZ();
        }

        private final class Listener extends SessionAdapter {
            @Override
            public void packetReceived(Session ignored, Packet packet) {
                if (packet instanceof ClientboundPlayerPositionPacket positionPacket) {
                    position = positionPacket.getPosition();
                    session.send(new ServerboundAcceptTeleportationPacket(positionPacket.getId()));
                    session.send(new ServerboundMovePlayerPosRotPacket(true, false,
                        position.getX(), position.getY(), position.getZ(),
                        positionPacket.getYRot(), positionPacket.getXRot()));
                    joined.countDown();
                }
                if (packet instanceof ClientboundSystemChatPacket chat) {
                    var text = PlainTextComponentSerializer.plainText().serialize(chat.getContent());
                    synchronized (messages) {
                        messages.add(text);
                    }
                }
            }

            @Override
            public void disconnected(DisconnectedEvent event) {
                synchronized (messages) {
                    messages.add("disconnect: " + event.getReason());
                }
            }
        }
    }

    private static UUID offlineUuid(String name) {
        return UUID.nameUUIDFromBytes(("OfflinePlayer:" + name).getBytes(StandardCharsets.UTF_8));
    }
}
