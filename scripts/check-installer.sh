#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
cd "$root"
python3 -m unittest -v tests.test_deploy_release
printf '%s\n' 'ok immutable-update-check checkout-installer=withdrawn anchored-publication=pass migration-classification=pass'
