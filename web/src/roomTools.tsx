import { useEffect, useRef, useState } from "react";
import type { Room } from "livekit-client";
import { api, type Data } from "./api";
import { useApp } from "./App";
import { t, message, messageError } from "./i18n";
export function RoomTools({ room }: { room: Data }) {
  const { me, open } = useApp();
  const current = useRef<Room | null>(null);
  const alive = useRef(true);
  const controller = useRef(new AbortController());
  const audio = useRef<HTMLDivElement>(null);
  const [connected, setConnected] = useState(false);
  const [busy, setBusy] = useState(false);
  const [muted, setMuted] = useState(false);
  const [error, setError] = useState<unknown>(null);
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
      if (alive.current) setError(e);
    } finally {
      if (alive.current) setBusy(false);
    }
  }
  return (
    <div className="room-tools">
      <small>
        {room.kind === "dm"
          ? t("text.private_chat")
          : t("text.visible_to_group_members")}
      </small>
      <div className="actions">
        {me.voice_available ? (
          connected ? (
            <>
              <span>{t("text.in_voice_chat_not_recorded")}</span>
              <button
                onClick={async () => {
                  try {
                    await current.current?.localParticipant.setMicrophoneEnabled(
                      muted,
                    );
                    setMuted(!muted);
                  } catch (e) {
                    setError(e);
                  }
                }}
              >
                {muted
                  ? t("text.unmute_microphone")
                  : t("text.mute_microphone")}
              </button>
              <button onClick={() => void current.current?.disconnect()}>
                {t("text.leave_voice_chat")}
              </button>
            </>
          ) : (
            <button disabled={busy} onClick={() => void join()}>
              {busy ? t("text.connecting") : t("text.voice_chat")}
            </button>
          )
        ) : null}
        {room.kind === "group" && (
          <>
            <button
              onClick={() =>
                open({
                  title: message("text.send_an_invitation"),
                  type: "invite",
                  values: { kind: "room", resource: room.id },
                  fields: [
                    {
                      name: "target",
                      label: message("text.invite_a_player"),
                      type: "player",
                    },
                  ],
                  submit: message("text.invite"),
                })
              }
            >
              {t("text.invite_member")}
            </button>
            <button
              onClick={() =>
                open({
                  title: message("text.leave_group"),
                  type: "room_leave",
                  values: { room: room.id },
                  note: () => (
                    <p>
                      {t(
                        "text.you_will_lose_access_to_this_conversation_and_its_voice_room",
                      )}
                    </p>
                  ),
                  submit: message("text.leave_now"),
                })
              }
            >
              {t("text.leave_this_group")}
            </button>
          </>
        )}
      </div>
      {!!error && (
        <p role="alert" className="error">
          {messageError(error)}
        </p>
      )}
      <div className="remote-audio" ref={audio} />
    </div>
  );
}
