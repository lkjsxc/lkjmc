# Action bar

## Purpose

This document owns action-bar and HUD behavior.

## Current status

The current HUD behavior is not the target contextual reducer. Treat prioritized
exchange, purchase, daily, temporary-instance, transfer, and admin diagnostics as
target behavior until implemented and verified.

## Sources

Action-bar state may use player settings, cached point balance, recent point
delta, daily reward availability, adventure or temporary instance countdown,
transfer status, exchange and purchase results, admin diagnostics, and claim
protection notices.

## Priority

1. Critical admin or daemon diagnostic.
2. Exchange or purchase result.
3. Adventure, temporary instance, or transfer countdown.
4. Claim protection denial or confirmation.
5. Daily reward availability.
6. Compact balance and HUD status.

## Rules

A pure reducer chooses the highest-priority unexpired frame. Passive state sends
only when changed or after a long refresh interval. Repeated identical failures
collapse behind a cooldown. Daemon failure disables daemon-backed passive data
only; local events may still render. Frames never contain secrets, raw JSON, or
stack traces.
