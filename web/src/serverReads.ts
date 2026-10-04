import {
  identityEpoch,
  onIdentityReset,
  PrivateCache,
  onResourceReset,
} from "./identity";
import { useEffect, useState } from "react";
import { readJob, command, ApiError, type Data } from "./api";
import { terminal } from "./jobs";
import { message } from "./i18n";
import { ReadSession, type ReadSnapshot } from "./readSession";
import { useApp } from "./App";
const reads = new PrivateCache<ReadSession>(24);
const active = new Set<ReadSession>();
onIdentityReset(() => {
  for (const read of active) read.dispose();
});
export function clearServerReads(id: string) {
  for (const [key, read] of reads)
    if (
      read.values.id === id ||
      (id === read.values.id + "/files" && read.type !== "server_logs")
    ) {
      read.dispose();
      reads.delete(key);
    }
}
onResourceReset(clearServerReads);
const transport = {
  submit: command,
  job: (id: string) => readJob(id, new AbortController().signal),
};
export function useServerRead(
  type: string,
  values: Data,
  enabled: boolean,
  live = false,
) {
  const { panelsVisible, me } = useApp();
  const key = JSON.stringify([identityEpoch(), me.account.id, type, values]);
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
    // This effect is only a scope/open boundary; ordinary refreshes retain state.
    reads.get(key)?.revalidate();
  }, [key, enabled]);
  useEffect(() => {
    setStateKey(key);
    if (!enabled) {
      setState({ error: "", busy: false });
      return;
    }
    let alive = true;
    const session = reads.get(key) ?? new ReadSession(type, values);
    reads.set(key, session);
    active.add(session);
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
      setState(next);
      const delay = session.delay(Date.now(), live);
      if (delay !== null) timer = setTimeout(tick, Math.max(500, delay));
    }
    if (panelsVisible) void tick();
    return () => {
      alive = false;
      clearTimeout(timer);
      active.delete(session);
    };
  }, [key, enabled, live, revision, panelsVisible]);
  return {
    ...(enabled && stateKey === key ? state : { error: "", busy: enabled }),
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
        throw new ApiError(0, job.error ?? message("text.the_operation_did_not_complete_open_its_details_before_59bdcb5fd1"));
      return job.result ?? {};
    }
    await new Promise((resolve) => setTimeout(resolve, 2500));
  }
}
