#!/bin/sh
set -eu
exec ./scripts/check-truth-probes.py --expected-failures
