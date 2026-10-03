// Private client state belongs to one authenticated session, not just an account.
let epoch = 0;
let identity = "";
let controller = new AbortController();
const resets = new Set<() => void>();
const listeners = new Set<() => void>();
export const identityEpoch = () => epoch;
export const identitySignal = () => controller.signal;
export const onIdentityReset = (reset: () => void) => {
  resets.add(reset);
  return () => {
    resets.delete(reset);
  };
};
export const subscribeIdentity = (listener: () => void) => {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
};
export function resetIdentity(next = "") {
  identity = next;
  epoch++;
  controller.abort();
  controller = new AbortController();
  for (const reset of resets) reset();
  for (const listener of listeners) listener();
}
export function acceptIdentity(account: string, csrf: string) {
  const next = JSON.stringify([account, csrf]);
  if (identity !== next) resetIdentity(next);
}
export function assertIdentity(expected: number) {
  if (expected !== epoch)
    throw new DOMException("Session changed", "AbortError");
}
export const unreadable = (error: unknown) =>
  [401, 403, 404].includes((error as { status?: number })?.status ?? 0);

// Small insertion-ordered caches; never retained across login boundaries.
export class PrivateCache<V> extends Map<string, V> {
  private limit: number;
  constructor(limit = 32) {
    super();
    this.limit = limit;
    onIdentityReset(() => this.clear());
  }
  override set(key: string, value: V): this {
    this.delete(key);
    super.set(key, value);
    while (this.size > this.limit) this.delete(this.keys().next().value!);
    return this;
  }
}
const resourceResets = new Set<(id: string) => void>();
export function onResourceReset(reset: (id: string) => void) {
  resourceResets.add(reset);
}
export function resetResource(id: string) {
  for (const reset of resourceResets) reset(id);
}
