import { message } from "../src/i18n";
message("error.login_required");
message("error.internal", { reference: "receipt" });
message("text.request_failed_0", 400);
// @ts-expect-error Unknown IDs cannot be authored as first-party messages.
message("unknown.id");
// @ts-expect-error Parameters required by the shared contract cannot be omitted.
message("error.internal");
// @ts-expect-error Parameters are scalar values, never nested player/JSON records.
message("error.internal", { reference: {} });
// @ts-expect-error Unused parameters are rejected by the authored contract.
message("error.login_required", { extra: "value" });
