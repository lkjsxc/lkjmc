export type ReadData = Record<string, any>;
export type ReadSnapshot = {
  result?: ReadData;
  error: string;
  busy: boolean;
  progress?: ReadData;
  updated?: number;
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
  private jobId?: string;
  private requestId?: string;
  private inFlight?: Promise<ReadSnapshot>;
  private completed = 0;
  private needsRead = true;
  private started = 0;
  private nextAt = 0;
  private result?: ReadData;
  private error = "";
  private progress?: ReadData;
  readonly type: string;
  readonly values: ReadData;
  constructor(type: string, values: ReadData) {
    this.type = type;
    this.values = values;
  }
  snapshot(): ReadSnapshot {
    return {
      result: this.result,
      error: this.error,
      busy: !this.error && (!!this.inFlight || !!this.jobId || this.needsRead),
      progress: this.progress,
      updated: this.completed,
    };
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
    if (this.inFlight) return this.inFlight;
    const delay = this.delay(now(), live);
    if (delay === null || delay > 0) return Promise.resolve(this.snapshot());
    this.inFlight = (async () => {
      try {
        if (this.jobId) {
          const job = await transport.job(this.jobId);
          this.progress = job.progress;
          if (isTerminal(job.state)) {
            this.jobId = undefined;
            this.requestId = undefined;
            this.completed = now();
            if (job.state !== "succeeded")
              throw new Error(
                typeof job.error === "string"
                  ? job.error
                  : "The read failed. Retry when the server is available.",
              );
            if (!job.result)
              throw new Error("The server returned no readable result.");
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
          this.started = now();
          this.needsRead = false;
          if (result.job_id) {
            this.jobId = result.job_id;
            this.nextAt = now() + 1500;
          } else {
            this.result = result;
            this.completed = now();
            this.requestId = undefined;
            this.nextAt = now() + 15000;
          }
        }
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
      } finally {
        this.inFlight = undefined;
      }
      return this.snapshot();
    })();
    return this.inFlight;
  }
}
