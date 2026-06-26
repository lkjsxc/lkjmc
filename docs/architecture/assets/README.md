# Asset architecture

## Purpose

This directory owns target contracts for server and plugin artifact storage.

## Table of contents

- [Download policy](download-policy.md)
- [Plugin jars](plugin-jars.md)
- [Server jars](server-jars.md)

## Contract

Assets are immutable, content-addressed files recorded in PostgreSQL. No command
may install or report an asset unless the stored file hash matches trusted
metadata.
