# azrs — Azure CLI, reimplemented in Rust

`azrs` is a drop-in reimplementation of the Azure CLI (`az`) written in Rust. It targets
**behavioral parity** with the Python `az`: the same command hierarchy, argument names,
output formats, config files, and profile/token storage — so it can share `~/.azure/`
state with the official CLI.

> Status: work in progress. See [`researches/feature-parity.md`](researches/feature-parity.md)
> for a detailed, up-to-date comparison against Python `az`.

## Parity snapshot

| Area | Python `az` | Rust `azrs` |
|------|-------------|-------------|
| Top-level command groups | ~65 | ~61 (manual + build-time generated) |
| Auth flows | 6 | 6 (browser PKCE, device code, SP secret, SP cert, managed identity, Cloud Shell) |
| Output formats | 7 | 7 (`json`, `jsonc`, `table`, `tsv`, `yaml`, `yamlc`, `none`) |
| JMESPath `--query` | ✅ | ✅ |
| Config (`~/.azure/config`) | ✅ | ✅ |
| Profile (`azureProfile.json`) | ✅ | ✅ (shared format) |
| Extensions | ✅ | ❌ |

## Repository layout

```
az-cli-rs/            The Rust crate (binary name: azrs)
  src/                CLI, auth, ARM framework, service commands
  tools/aaz_gen/      Python AAZ -> IR -> Rust code generator (used by build.rs)
  tests/              Cassette-based integration + parity tests
  gen_config.toml     Which AAZ services to generate, and from where
  build.rs            Runs the generator at compile time
researches/           Analysis notes (feature-parity comparison)
../azure-cli          OPTIONAL sibling checkout of Azure/azure-cli (Python).
                      Source for build-time code generation and the behavioral
                      reference. When absent, generated commands are stubbed out.
docs/                 ARCHITECTURE.md, AUTHENTICATION.md
```

## Build & run

Requires a recent stable Rust toolchain (edition 2021).

```sh
cd az-cli-rs
cargo build                  # build
cargo run -- --help          # list all commands
cargo run -- group list      # run a command
```

To include the auto-generated service commands (e.g. `network`, `vm`, `cosmosdb`), clone
[`Azure/azure-cli`](https://github.com/Azure/azure-cli) as a sibling directory
(`../azure-cli`, matching `azure_cli_path` in `gen_config.toml`) before building. Without
it, the crate still builds — those commands are simply omitted.

## Quickstart

```sh
azrs login                                   # interactive browser login (default)
azrs login --use-device-code                 # device code flow
azrs login --service-principal -u <appId> -p <secret> --tenant <tenant>
azrs login --identity                        # managed identity

azrs account list                            # subscriptions
azrs account set --subscription <name-or-id> # set active subscription

azrs group list
azrs group create -n myrg -l eastus

azrs rest --method get --url \
  "https://management.azure.com/subscriptions?api-version=2022-12-01"
```

## Global options

Available on every command:

- `-o, --output <json|jsonc|table|tsv|yaml|yamlc|none>` — output format (default `json`).
- `--query <JMESPath>` — filter/reshape output (see <http://jmespath.org/>).
- `--subscription <id-or-name>` — override the active subscription.
- `--debug` / `--verbose` / `--only-show-errors` — logging verbosity.

## State files (`~/.azure/`)

`azrs` reads and writes the same directory as Python `az`:

- `azureProfile.json` — subscriptions and the active subscription (shared format).
- `config` — INI config (default output, location, resource group).
- `azrs_token_cache.json` — access/refresh token cache (`azrs`-specific).
- `azrs_sp_entries.json` — service principal entries (`azrs`-specific).

## Testing

Integration tests use a cassette record/playback framework (no network by default):

```sh
cd az-cli-rs
cargo test                     # playback recorded HTTP interactions (deterministic)
cargo test test_group_crud     # run a single test by name
AZURE_TEST_RUN_LIVE=1 cargo test   # re-record against live Azure (requires login)
```

Command-level parity suites live in `tests/parity/*.toml`.

## Documentation

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — internal design: the two-layer command
  model, the ARM command framework, build-time AAZ code generation, and the testing
  framework.
- [docs/AUTHENTICATION.md](docs/AUTHENTICATION.md) — the six auth flows, token/profile
  storage, and OAuth2 scope logic.
- [.github/copilot-instructions.md](.github/copilot-instructions.md) — conventions and
  critical rules for contributors (and AI agents).
