import type { SystemMessage } from "./i18n";
export type ReadData = Record<string, any>;
export type ReadSnapshot = {
  result?: ReadData;
  error: Error | SystemMessage | string;
  busy: boolean;
  progress?: ReadData;
  updated?: number;
  revoked?: boolean;
};
export type ReadTransport = {
  submit: (
    type: string,
    values: ReadData,
    requestId: string,
  ) => Promise<ReadData>;
  job: (id: string) => Promise<ReadData>;
};
const isTerminal = (state: string) =>
  ["succeeded", "failed", "cancelled"].includes(state);
// One session per scoped read. A failed status read resumes the same durable job;
// an uncertain submission retries the same idempotency key. No timers live here.
export class ReadSession {
  private generation = 0;
  private revoked = false;
  private clock = Date.now;
  private jobId?: string;
  private requestId?: string;
  private inFlight?: Promise<ReadSnapshot>;
  private completed = 0;
  private needsRead = true;
  private started = 0;
  private nextAt = 0;
  private result?: ReadData;
  private error: Error | SystemMessage | string = "";
  private progress?: ReadData;
  readonly type: string;
  readonly values: ReadData;
  constructor(type: string, values: ReadData) {
    this.type = type;
    this.values = values;
  }
  snapshot(): ReadSnapshot {
    if (this.completed && this.clock() - this.completed >= 120000)
      this.result = undefined;
    return {
      revoked: this.revoked,
      result: this.result,
      error: this.error,
      busy: !this.error && (!!this.inFlight || !!this.jobId || this.needsRead),
      progress: this.progress,
      updated: this.completed,
    };
  }
  // Reopening requires a current authorization check before exposing any output.
  revalidate() {
    this.generation++;
    this.inFlight = undefined;
    this.result = undefined;
    this.progress = undefined;
    if (!this.error) {
      if (!this.jobId) this.needsRead = true;
      this.nextAt = 0;
    }
  }
  dispose() {
    this.generation++;
    this.result = undefined;
    this.progress = undefined;
    this.inFlight = undefined;
  }
  retry() {
    this.error = "";
    if (!this.jobId) this.needsRead = true;
    this.nextAt = 0;
  }
  delay(now: number, live: boolean): number | null {
    if (this.error) return null;
    if (!this.jobId && this.completed && !live && !this.needsRead) return null;
    return Math.max(0, this.nextAt - now);
  }
  advance(
    transport: ReadTransport,
    now: () => number = Date.now,
    live = false,
  ): Promise<ReadSnapshot> {
    this.clock = now;
    if (this.inFlight) return this.inFlight;
    const delay = this.delay(now(), live);
    if (delay === null || delay > 0) return Promise.resolve(this.snapshot());
    const generation = this.generation;
    this.inFlight = (async () => {
      try {
        if (this.jobId) {
          const job = await transport.job(this.jobId);
          if (generation !== this.generation) return this.snapshot();
          this.progress = job.progress;
          if (isTerminal(job.state)) {
            this.jobId = undefined;
            this.requestId = undefined;
            this.completed = now();
            // An authoritative failed READ cannot validate the previous output.
            // Only transport/status failures may keep last-known data visible.
            if (job.state !== "succeeded" || !job.result) {
              this.result = undefined;
              this.progress = undefined;
            }
            if (job.state !== "succeeded")
              throw job.error ?? { id: "text.the_read_failed_retry_when_the_server_is_available", params: {} };
            if (!job.result)
              throw { id: "text.the_server_returned_no_readable_result", params: {} };
            this.revoked = false;
            this.result = job.result;
            this.nextAt = now() + 15000;
          } else
            this.nextAt =
              now() +
              Math.min(
                10000,
                2000 + Math.floor((now() - this.started) / 10000) * 1000,
              );
        } else {
          this.requestId ??= crypto.randomUUID();
          const result = await transport.submit(
            this.type,
            this.values,
            this.requestId,
          );
          if (generation !== this.generation) return this.snapshot();
          this.started = now();
          this.needsRead = false;
          if (result.job_id) {
            this.jobId = result.job_id;
            this.nextAt = now() + 1500;
          } else {
            this.revoked = false;
            this.result = result;
            this.completed = now();
            this.requestId = undefined;
            this.nextAt = now() + 15000;
          }
        }
      } catch (e) {
        if (generation !== this.generation) return this.snapshot();
        const status = (e as { status?: number })?.status;
        if (status === 401 || status === 403 || status === 404) {
          this.result = undefined;
          this.progress = undefined;
          this.jobId = undefined;
          this.requestId = undefined;
          this.completed = 0;
          this.needsRead = true;
          this.revoked = true;
        }
        this.error = e as Error | SystemMessage | string;
      } finally {
        if (generation === this.generation) this.inFlight = undefined;
      }
      return this.snapshot();
    })();
    return this.inFlight;
  }
}
