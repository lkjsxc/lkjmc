# Web architecture

## Purpose

This area owns the private authenticated operator web surface.

## Table of contents

- [Routes](routes.md)
- [Security](security.md)
- [Audit](audit.md)

## Contract

The web listener is a presentation adapter over daemon commands. It does not
write product state directly, does not expose secrets, and stays private by
default.
