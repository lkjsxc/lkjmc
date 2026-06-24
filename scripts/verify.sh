#!/bin/sh
set -eu
./scripts/check-lines.py >/dev/null
./scripts/check-docs.py >/dev/null
printf '%s\n' 'ok verify'
