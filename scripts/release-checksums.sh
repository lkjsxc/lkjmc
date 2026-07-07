#!/bin/sh
set -eu
if [ "$#" -eq 0 ]; then
  echo "usage: scripts/release-checksums.sh FILE..." >&2
  exit 1
fi
sha256sum "$@"
