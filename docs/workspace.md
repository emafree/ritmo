# Ritmo Workspace — Quick Guide

This repository is organized as a Rust workspace. The purpose of this document is to explain how to work with the workspace.

## Workspace Members
The main crates are listed in the root Cargo.toml as `members`:
- ritmo_core: core logic and ebook management
- ritmo_cli: command-line interface
- ritmo_db / ritmo_db_core: database and metadata management
- ritmo_mapping: metadata mapping
- ritmo_errors: shared error types crate
Other crates (gui, search, ml, ebook_parser) may be present but commented out if not ready.

## Useful Commands
- Build the entire workspace:
  cargo build --workspace

- Run tests for the entire workspace:
  cargo test --workspace

- Format code:
  cargo fmt --all

- Run clippy (lint):
  cargo clippy --all -- -D warnings

- Add a new crate:
  1. Create the crate folder with `cargo new --lib crate_name` or `cargo new --bin crate_name`.
  2. Add the path to the `members` list in the root `Cargo.toml`.
  3. Document the crate in this file.

## Database
- For local development, SQLite is used. The example connection string is in `.env.example` (`DATABASE_URL`).
- Migrations (if used) must be versioned and documented in the `ritmo_db_core` crate.

## CI and Quality Gates
- The repo includes a CI workflow that runs: formatting (check), clippy, build, and tests for the entire workspace.
- Ensure CI jobs pass before opening a significant PR.

## Notes
- Do not commit `.env` files. Use `.env.example`.
- If you add native or binary dependencies (e.g., conversion engines), document how to install them in the `docs/build.md` section (not yet present).