#!/bin/sh
set -eu
cat >&2 <<'EOF'
The checkout-based lkjmc installer has been withdrawn because it could build
ambient bytes, overwrite operator intent, and migrate without a complete
rollback boundary.

Build a clean immutable release with scripts/build-release.sh. Existing
supported system deployments are updated with the release-packaged
lkjmc-deploy-release command and an externally anchored manifest SHA-256.
Clean system installation remains intentionally unavailable until its full
PostgreSQL, credential, server-asset, EULA-record, and systemd drill passes.
EOF
exit 1
