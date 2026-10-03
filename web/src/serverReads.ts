import { useEffect, useState } from "react";
import { readJob, command, type Data } from "./api";
import { terminal } from "./jobs";
import { t, translateError } from "./i18n";
import { ReadSession, type ReadSnapshot } from "./readSession";
import { useApp } from "./App";
const reads = new Map<string, ReadSession>();
const transport = {
  submit: command,
  job: readJob,
};
export function useServerRead(
  type: string,
  values: Data,
  enabled: boolean,
  live = false,
) {
  const { panelsVisible, me } = useApp();
  const key = JSON.stringify([me.account.id, type, values]);
  const [revision, setRevision] = useState(0);
  const [stateKey, setStateKey] = useState(key);
  const [state, setState] = useState<ReadSnapshot>({
    error: "",
    busy: enabled,
  });
  const refresh = () => {
    reads.get(key)?.retry();
    setRevision((n) => n + 1);
  };
  useEffect(() => {
    setStateKey(key);
    if (!enabled) {
      setState({ error: "", busy: false });
      return;
    }
    let alive = true;
    const session = reads.get(key) ?? new ReadSession(type, values);
    reads.set(key, session);
    let timer: ReturnType<typeof setTimeout>;
    setState(session.snapshot());
    async function tick() {
      if (!alive) return;
      if (document.hidden) {
        timer = setTimeout(tick, 3000);
        return;
      }
      const next = await session.advance(transport, Date.now, live);
      if (!alive) return;
      setState({
        ...next,
        error: next.error ? translateError(next.error) : "",
      });
      const delay = session.delay(Date.now(), live);
      if (delay !== null) timer = setTimeout(tick, Math.max(500, delay));
    }
    if (panelsVisible) void tick();
    return () => {
      alive = false;
      clearTimeout(timer);
    };
  }, [key, enabled, live, revision, panelsVisible]);
  return {
    ...(stateKey === key ? state : { error: "", busy: enabled }),
    refresh,
  };
}
export async function waitForJob(
  id: string,
  onProgress?: (job: Data) => void,
  signal?: AbortSignal,
): Promise<Data> {
  for (;;) {
    if (signal?.aborted) throw new DOMException("Aborted", "AbortError");
    if (document.hidden) {
      await new Promise((resolve) => setTimeout(resolve, 2500));
      continue;
    }
    const job = await readJob(id);
    if (signal?.aborted) throw new DOMException("Aborted", "AbortError");
    onProgress?.(job);
    if (terminal(job.state)) {
      if (job.state !== "succeeded")
        throw new Error(
          translateError(
            job.error ??
              t(
                "The operation did not complete. Open its details before trying again.",
              ),
          ),
        );
      return job.result ?? {};
    }
    await new Promise((resolve) => setTimeout(resolve, 2500));
  }
}
