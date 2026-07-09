# Copilot Instructions for azclirs

## Repository Structure

This is a Rust rewrite of the Azure CLI (`az`). The Rust app lives in `az-cli-rs/`. The reference Python CLI is expected as a sibling checkout at `../azure-cli` (path set by `azure_cli_path` in `gen_config.toml`); it is read-only and used both for behavioral comparison and as the source for build-time code generation. When it is absent, `build.rs` falls back to empty stubs so the crate still builds — generated service commands just won't be present.

## Build & Run

```sh
cd az-cli-rs
cargo build              # build
cargo run -- <command>   # run (e.g. cargo run -- group list)
cargo run -- --help      # see all commands
```

## Test

```sh
cd az-cli-rs
cargo test                      # run all tests (cassette playback, no network)
cargo test test_group_crud      # run a single test by name
```

Integration tests live in `src/testing/` and use a cassette record/playback framework inspired by `azure-cli-testsdk`:
- **Playback (default)** — HTTP interactions are replayed from JSON cassettes under `tests/recordings/`. No network or login required, so `cargo test` is deterministic and safe in CI.
- **Recording** — set `AZURE_TEST_RUN_LIVE=1` to send real HTTP requests (requires `azrs login`) and re-record the cassettes.

`tests/parity/*.toml` declare command-level parity suites (compare `azrs <cmd>` output against `az <cmd>`, with `ignore_fields` for volatile values). Beyond automated tests, still verify parity-sensitive changes manually against live Azure.

## Architecture

### Two-layer design

1. **CLI layer** (`cli.rs` + `main.rs`) — clap derive structs define args, `cmd_handlers` module in `main.rs` dispatches to implementations.

2. **ARM command framework** (`commands/mod.rs`) — `ArmCommand` struct provides authenticated `get()`, `put()`, `delete()`, `list()`, `exists()` against ARM REST APIs. Handles token injection, `{subscriptionId}` replacement, pagination via `nextLink`, and ARM error envelope parsing. Service commands in `commands/` are thin wrappers.

### Auth flow

Own OAuth2 implementation (not `azure_identity`) using Azure CLI's public client ID (`04b07795-8ddb-461a-bbee-02f9e1bf7b46`). Three flows:
- `auth/interactive.rs` — Browser auth-code + PKCE with a local `tiny_http` redirect server
- `auth/device_code.rs` — Device code polling
- `auth/service_principal.rs` — Client credentials grant (client_id + client_secret)

Plus a native **Identity Broker / WAM** path (`auth/broker.rs`): silent, credential-free token acquisition via the Linux session-D-Bus `com.microsoft.identity.broker1` (Intune broker / himmelblau) and the Windows Web Account Manager (WAM) WinRT API. Exposed as `azrs login --use-broker`, and used automatically inside `TokenCache::get_access_token` as a last-resort silent re-acquisition before erroring. Broker tokens carry no refresh token — re-auth is just another silent broker call. macOS reports the broker unavailable and falls back to the flows above. Deps are target-gated (`zbus` on Linux, `windows` on Windows).

Tokens cached in `~/.azure/azrs_token_cache.json`. Profile (subscriptions) in `~/.azure/azureProfile.json` (same format as Python `az`). SP entries in `~/.azure/azrs_sp_entries.json`.

### Build-time code generation

`build.rs` reads `gen_config.toml` and generates Rust command modules from Python AAZ files at build time.
Generated code lives in `$OUT_DIR/generated/` and is included via `src/generated.rs`.
To add a new generated service, add an entry to `gen_config.toml` — no code changes needed.

The `tools/aaz_gen/` crate provides the parser (Python AAZ → IR) and emitter (IR → Rust) as a library used by `build.rs` and as a standalone CLI tool.

### Key modules

| Module | Purpose |
|--------|---------|
| `cloud.rs` | Cloud endpoint definitions (Public/China/USGov), default scope derivation |
| `config.rs` | CLI configuration (`~/.azure/config` INI file), defaults for group/location/output |
| `profile.rs` | Read/write `azureProfile.json`, subscription CRUD |
| `auth/token_cache.rs` | Token persistence, refresh token exchange, cache key: `username\|tenant` |
| `auth/service_principal.rs` | SP auth flow + SP entry store |
| `arm.rs` | Tenant/subscription discovery after login (ARM `/tenants` + `/subscriptions`) |
| `rest.rs` | Generic `azrs rest` command — URL normalization, auto-scope detection from URL |
| `http_client.rs` | Shared HTTP client used by auth + commands (also the recording/playback seam for tests) |
| `testing/` | Cassette-based test framework (record/playback, processors, checkers, fixtures, preparers, scenario) |
| `commands/mod.rs` | `ArmCommand` framework — shared auth+HTTP for all service commands |
| `commands/group.rs` | Resource group CRUD — pattern to follow for new service commands |
| `commands/keyvault.rs` | Key Vault secrets — data-plane pattern (different auth scope) |
| `selector.rs` | Interactive subscription picker (shown after login in TTY) |
| `output.rs` | Output formatting (json/jsonc/table/tsv/yaml/none) + JMESPath --query |
| `generated.rs` | include!() bridge for build-time generated commands |
| `build.rs` | Build-time AAZ codegen — reads gen_config.toml, runs parser+emitter |

## Conventions

### Adding a new service command

Follow the pattern in `commands/group.rs`:

1. Create `commands/<service>.rs` with async functions using `ArmCommand`
2. Add `pub mod <service>;` to `commands/mod.rs`
3. Add CLI args struct + subcommand enum to `cli.rs`
4. Wire dispatch in `main.rs`

Each command function should:
- Create `ArmCommand::new()?`
- Build the ARM path with `{subscriptionId}` placeholder and `api-version` query param
- Call `cmd.get()`, `cmd.put()`, `cmd.delete()`, or `cmd.list()`
- Call `cmd.save_cache()?` before returning
- Return `Result<serde_json::Value>` (or `Vec<Value>` for list, `()` for delete)

### Adding a generated service (no code changes)

Add to `gen_config.toml`:
```toml
[[modules]]
service = "my_service"
cli_prefix = "my-service"
aaz_subpath = "src/azure-cli/azure/cli/command_modules/xxx/aaz/latest/xxx"
```
Then `cargo build` — the commands appear automatically.

### Error handling

- Use `crate::error::Result<T>` everywhere
- For auth errors that should suggest re-login, use `AzrsError::AuthWithSuggestion { message, suggestion }` — the suggestion is printed on stderr by `main.rs`
- ARM errors are auto-parsed from the `{ "error": { "code", "message" } }` envelope in `commands/mod.rs::parse_arm_error()`

### Default scope logic

Matches Python `az` exactly: `{cloud.active_directory_resource_id}/.default` → `https://management.core.windows.net//.default` for public cloud. The double slash is intentional.

### Reference codebase

The Python `az` CLI in `azure-cli/` is the behavioral reference. Key files:
- `src/azure-cli-core/azure/cli/core/aaz/` — AAZ auto-generated command framework
- `src/azure-cli-core/azure/cli/core/_profile.py` — profile/subscription management
- `src/azure-cli-core/azure/cli/core/auth/` — MSAL-based auth
- `src/azure-cli/azure/cli/command_modules/` — 40+ service modules

When implementing a new command, check the Python equivalent for exact arg names, default values, and API versions.

## Critical Rules

### `az` parity is non-negotiable
This project is a drop-in replacement for `az`. Do not defer features that affect CLI parity (command hierarchy, argument names, output format, error messages). When implementing or modifying any command, run both `az <command>` and `azrs <command>` to compare output. Pay special attention to:
- Error messages (should suggest re-login with correct tenant/scope)
- Login flow (interactive subscription selector, multi-tenant discovery)
- Output format and field names
- Command hierarchy must match `az` exactly

### Generated commands must be properly nested
Generated commands MUST use nested clap subcommands matching the `az` hierarchy: `azrs network nsg rule create`, NOT `azrs network-nsg-rule-create`. The `build.rs` emitter produces recursive `#[command(subcommand)]` enums for each group level.

### One gen_config entry per top-level service
In `gen_config.toml`, use ONE entry per top-level service pointing to the full AAZ directory. For example, use `network/aaz/latest/network` (all network subgroups unified), NOT separate entries for each subgroup. The `cli_prefix` must match the `az` top-level group name (e.g. `network`, `vm`, `cosmosdb`).

### Code generation runs via build.rs only
Code generation runs at compile time via `build.rs`, not via external scripts. Generated code goes to `$OUT_DIR/generated/` and is included via `src/generated.rs`. Never propose a separate regeneration script — `cargo build` is the only command needed.

### Keyword identifiers in format strings
When a CLI arg name is a Rust keyword (e.g. `type`), sanitize it to `type_` (not `r#type`) because `format!("{r#type}")` is invalid. See `sanitize_ident()` in `build.rs`.
