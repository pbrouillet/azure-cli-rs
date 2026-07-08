# Architecture

This document describes how `azrs` is put together. For auth specifics see
[AUTHENTICATION.md](AUTHENTICATION.md); for a feature-by-feature parity comparison with
Python `az` see [`../researches/feature-parity.md`](../researches/feature-parity.md).

All paths below are relative to the crate root, `az-cli-rs/`.

## Two-layer design

`azrs` separates *argument parsing* from *API execution*:

1. **CLI layer** — `src/cli.rs` + `src/main.rs`.
   `cli.rs` defines the full command tree with `clap` derive structs (a top-level
   `Commands` enum, per-service subcommand enums, and per-command argument structs plus a
   global-args struct). `main.rs` parses those args and dispatches each variant to an
   implementation, mostly through the `cmd_handlers` helper module.

2. **ARM command framework** — `src/commands/mod.rs`.
   The `ArmCommand` struct is the shared engine for every Azure Resource Manager (ARM)
   REST call. Service modules under `src/commands/` are thin wrappers that build a URL
   path and delegate to it.

Generated commands (see [Build-time code generation](#build-time-code-generation)) plug
into the same CLI tree via a flattened `Generated` variant bridged through
`src/generated.rs`.

## The `ArmCommand` framework

`ArmCommand` (in `src/commands/mod.rs`, with HTTP/pagination helpers in `src/arm.rs`)
centralizes everything a management-plane call needs:

- **Authentication** — acquires an access token for the current subscription's user and
  tenant using the cloud's default scope (`cloud.default_scope()`), via the token cache.
- **`{subscriptionId}` substitution** — service modules write paths with a literal
  `{subscriptionId}` placeholder; the framework substitutes the active subscription.
- **`api-version`** — supplied by each service module as a query parameter on the path.
- **Verbs** — `get()`, `put()`, `delete()`, `list()`, `exists()` (HEAD).
- **Pagination** — `list()` transparently follows `nextLink` across pages (`arm.rs`).
- **Error parsing** — ARM's `{ "error": { "code", "message" } }` envelope is parsed into a
  structured error by `parse_arm_error()`.

Data-plane services differ only in scope. For example `src/commands/keyvault.rs` targets
`https://<vault>.vault.azure.net/` with scope `https://vault.azure.net/.default` instead
of the management scope.

### Adding a service command (manual)

Follow `src/commands/group.rs` as the canonical example:

1. Create `src/commands/<service>.rs` with `async` functions that use `ArmCommand`.
2. Add `pub mod <service>;` to `src/commands/mod.rs`.
3. Add a CLI args struct + subcommand enum in `src/cli.rs`.
4. Wire the dispatch in `src/main.rs`.

Each function typically: builds the ARM path with a `{subscriptionId}` placeholder and an
`api-version`; calls `cmd.get()` / `put()` / `delete()` / `list()`; calls
`cmd.save_cache()` before returning; and returns `Result<serde_json::Value>` (or
`Vec<Value>` for lists, `()` for deletes).

## Build-time code generation

Most large services (`network`, `vm`, `cosmosdb`, `sql`, `monitor`, …) are **generated at
compile time** from Python AAZ command definitions rather than hand-written.

Pipeline:

```
gen_config.toml ──▶ build.rs ──▶ tools/aaz_gen (parser: Python AAZ ─▶ IR,
                                                emitter: IR ─▶ Rust)
                            ──▶ $OUT_DIR/generated/*.rs
                            ──▶ included via src/generated.rs
```

- **`gen_config.toml`** lists modules. Each `[[modules]]` entry names a `service`, a
  `cli_prefix` (must equal the `az` top-level group name, e.g. `network`), and an
  `aaz_subpath` into the Python source. `azure_cli_path` (default `../azure-cli`) is the
  root those subpaths resolve against.
- **`build.rs`** reads the config, runs the `aaz_gen` parser + emitter for each module
  whose AAZ path exists, and writes Rust into `$OUT_DIR/generated/`. It re-runs when
  `gen_config.toml` or the AAZ inputs change.
- **Stub fallback** — if `../azure-cli` (or a specific AAZ path) is absent, `build.rs`
  emits empty stubs so the crate still compiles; those commands are simply unavailable.
- **`tools/aaz_gen`** is both a library (used by `build.rs`) and a standalone CLI tool.

Conventions enforced by the emitter and config:

- **Nested subcommands** — generated commands mirror the `az` hierarchy as recursive
  `#[command(subcommand)]` enums (`azrs network nsg rule create`), never flattened
  (`azrs network-nsg-rule-create`).
- **One entry per top-level service** — a single `[[modules]]` entry points at the full
  AAZ directory for a service (all subgroups unified), not one entry per subgroup.
- **Keyword identifiers** — when a CLI arg name is a Rust keyword (e.g. `type`) it is
  sanitized to `type_` (not `r#type`) because `r#`-prefixed names are invalid inside
  `format!` strings. See `sanitize_ident()` in `build.rs`.

### Adding a generated service (config-only)

Add an entry to `gen_config.toml` and rebuild — no Rust changes needed:

```toml
[[modules]]
service = "my_service"
cli_prefix = "my-service"
aaz_subpath = "src/azure-cli/azure/cli/command_modules/xxx/aaz/latest/xxx"
```

Code generation runs **only** through `build.rs` at compile time. There is no separate
regeneration script — `cargo build` is the entire workflow.

## Module map

| Module | Purpose |
|--------|---------|
| `cli.rs` | `clap` command tree, args structs, global args, output-format enum |
| `main.rs` | Entry point; parses args and dispatches to handlers; login flow |
| `commands/mod.rs` | `ArmCommand` framework — shared auth + HTTP for service commands |
| `commands/group.rs` | Resource group CRUD — pattern to follow for new commands |
| `commands/keyvault.rs` | Key Vault secrets — data-plane pattern (different scope) |
| `arm.rs` | HTTP/pagination (`nextLink`), tenant/subscription discovery after login |
| `rest.rs` | Generic `azrs rest` — URL normalization, auto scope detection |
| `cloud.rs` | Cloud endpoints (Public/China/USGov), `resource_to_scope`, default scope |
| `config.rs` | `~/.azure/config` INI (default output/location/group) |
| `profile.rs` | Read/write `azureProfile.json`, subscription CRUD |
| `output.rs` | Output formatting (json/jsonc/table/tsv/yaml/yamlc/none) + `--query` |
| `selector.rs` | Interactive subscription picker (TTY, after login) |
| `http_client.rs` | `HttpClient` trait + `reqwest` impl; the record/playback seam for tests |
| `generated.rs` | `include!()` bridge for build-time generated commands |
| `error.rs` | `AzrsError` + `crate::error::Result<T>` |
| `auth/` | OAuth2 flows, token cache — see [AUTHENTICATION.md](AUTHENTICATION.md) |
| `testing/` | Cassette-based test framework (see below) |
| `build.rs` | Build-time AAZ codegen driver |

## Testing framework

Integration tests use record/playback "cassettes", inspired by the Python
`azure-cli-testsdk`. The framework lives in `src/testing/` (gated on `#[cfg(test)]`):

- **`http_client.rs`** defines the `HttpClient` trait. `testing/recording_client.rs`
  provides `RecordingHttpClient`, which implements that trait to either replay recorded
  interactions or record live ones — this trait is the seam that makes tests
  network-free.
- **Modes** — playback is the default (deterministic, no login). Setting
  `AZURE_TEST_RUN_LIVE=1` sends real HTTP and re-records the cassettes.
- **Cassettes** live under `tests/recordings/<service>/<test>.json`.
- **Support modules** — `processors.rs` (scrub secrets/volatile fields),
  `checkers.rs` (JMESPath assertions), `fixtures.rs`, `preparers.rs`, `scenario.rs`
  (`ScenarioTest` harness), `cassette.rs`.
- **Parity suites** — `tests/parity/*.toml` declare commands to run under `azrs` and
  compare against `az`, with `ignore_fields` for volatile values.

Run a single test with `cargo test <name>` (e.g. `cargo test test_group_crud`).

## Error handling

- Use `crate::error::Result<T>` throughout.
- Auth failures that should prompt a re-login use
  `AzrsError::AuthWithSuggestion { message, suggestion }`; `main.rs` prints the suggestion
  (e.g. a corrected `azrs login --tenant … --scope …`) on stderr.
- ARM error envelopes are parsed centrally in `commands/mod.rs::parse_arm_error()`.
