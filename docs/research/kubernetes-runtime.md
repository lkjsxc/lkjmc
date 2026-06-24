# Kubernetes runtime

## Purpose

This document defines the future Kubernetes runtime seam.

## Direction

The runtime adapter interface must allow a future implementation to manage
cluster objects from the same desired state stored in PostgreSQL. Open questions
include object ownership, networking, storage classes, log collection, and safe
player transfer during pod replacement.

## Current status

No Kubernetes adapter is implemented or registered.
