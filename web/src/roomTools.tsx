import { useEffect, useRef, useState } from "react";
import type { Room } from "livekit-client";
import { api, type Data } from "./api";
import { useApp } from "./App";
import { t } from "./i18n";
export function RoomTools({ room }: { room: Data }) {
  const { me, open } = useApp();
  const current = useRef<Room | null>(null);
  const alive = useRef(true);
  const controller = useRef(new AbortController());
  const audio = useRef<HTMLDivElement>(null);
  const [connected, setConnected] = useState(false);
  const [busy, setBusy] = useState(false);
  const [muted, setMuted] = useState(false);
  const [error, setError] = useState("");
  useEffect(
    () => () => {
      alive.current = false;
      controller.current.abort();
      void current.current?.disconnect();
    },
    [],
  );
  async function join() {
    setBusy(true);
    setError("");
    try {
      const { Room, RoomEvent, Track } = await import("livekit-client");
      if (!alive.current) return;
      const token = await api(`/api/v1/voice/${room.id}`, {
        method: "POST",
        body: "{}",
        signal: controller.current.signal,
      });
      if (!alive.current) return;
      const next = new Room({ adaptiveStream: true, dynacast: true });
      current.current = next;
      next.on(RoomEvent.TrackSubscribed, (track) => {
        if (track.kind === Track.Kind.Audio)
          audio.current?.appendChild(track.attach());
      });
      next.on(RoomEvent.TrackUnsubscribed, (track) =>
        track.detach().forEach((el) => el.remove()),
      );
      next.on(RoomEvent.Disconnected, () => {
        if (alive.current) setConnected(false);
      });
      await next.connect(token.url, token.token);
      if (!alive.current) {
        await next.disconnect();
        return;
      }
      await next.localParticipant.setMicrophoneEnabled(true);
      if (!alive.current) {
        await next.disconnect();
        return;
      }
      setConnected(true);
      setMuted(false);
    } catch (e) {
      await current.current?.disconnect();
      if (alive.current) setError((e as Error).message);
    } finally {
      if (alive.current) setBusy(false);
    }
  }
  return (
    <div className="room-tools">
      <small>
        {room.kind === "dm" ? t("Private chat") : t("Visible to group members")}
      </small>
      <div className="actions">
        {me.voice_available ? (
          connected ? (
            <>
              <span>{t("In voice chat · Not recorded")}</span>
              <button
                onClick={async () => {
                  try {
                    await current.current?.localParticipant.setMicrophoneEnabled(
                      muted,
                    );
                    setMuted(!muted);
                  } catch (e) {
                    setError((e as Error).message);
                  }
                }}
              >
                {muted ? t("Unmute microphone") : t("Mute microphone")}
              </button>
              <button onClick={() => void current.current?.disconnect()}>
                {t("Leave voice chat")}
              </button>
            </>
          ) : (
            <button disabled={busy} onClick={() => void join()}>
              {busy ? t("Connecting…") : t("Voice chat")}
            </button>
          )
        ) : (
          <span>{t("Voice service is being set up")}</span>
        )}
        {room.kind === "group" && (
          <>
            <button
              onClick={() =>
                open({
                  title: t("Send an invitation"),
                  type: "invite",
                  values: { kind: "room", resource: room.id },
                  fields: [
                    {
                      name: "target",
                      label: t("Invite a player"),
                      type: "player",
                    },
                  ],
                  submit: t("Invite"),
                })
              }
            >
              {t("Invite member")}
            </button>
            <button
              onClick={() =>
                open({
                  title: t("Leave group"),
                  type: "room_leave",
                  values: { room: room.id },
                  note: (
                    <p>
                      {t(
                        "You will lose access to this conversation and its voice room.",
                      )}
                    </p>
                  ),
                  submit: t("Leave now"),
                })
              }
            >
              {t("Leave this group")}
            </button>
          </>
        )}
      </div>
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
      <div className="remote-audio" ref={audio} />
    </div>
  );
}
