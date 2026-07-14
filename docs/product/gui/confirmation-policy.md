# Confirmation policy

## Purpose

This document defines confirmation-route and mutation admission rules.

## Status

implemented

## Routes

A confirmation route names its reason and uses the stable confirmation pair.
Cancel is typed Back and does not close. Confirm is a closed mutation operation;
it contains no daemon command or generic request body.

## Admission

Rendering a confirmation is not authorization. Confirm requires current typed
dependencies, a current exact capability, trusted attestation for the session
request, and an implemented typed mutation port. Any missing condition produces
a localized denial and no effect. This task intentionally supplies no daemon
mutation port.

EULA or other informed consent requires an owner-defined typed operation and
cannot be inferred from opening or clicking an unrelated route.
