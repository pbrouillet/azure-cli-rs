# Feature Parity: Python `az` CLI vs Rust `azrs` CLI

> Auto-generated comparison based on codebase analysis (2026-04-10)

---

## Summary

| Metric | Python `az` | Rust `azrs` |
|--------|------------|-------------|
| Top-level command groups | **65** (35 AAZ + 30 manual) | **~61** (27 manual + 34 generated) |
| Auth flows | 6 (browser, device-code, SP secret, SP cert, managed identity, Cloud Shell) | 6 (browser PKCE, device-code, SP secret, SP cert, managed identity, Cloud Shell) |
| Output formats | 7 (json, jsonc, table, tsv, yaml, yamlc, none) | 7 (json, jsonc, table, tsv, yaml, yamlc, none) |
| Extension system | ✅ (wheel + dev extensions) | ❌ |
| JMESPath `--query` | ✅ | ✅ |
| Config system | ✅ (`~/.azure/config`) | ✅ (`~/.azure/config`) |
| Profile/subscriptions | ✅ (`azureProfile.json`) | ✅ (`azureProfile.json`, shared format) |

---

## 1. Authentication

| Feature | Python `az` | Rust `azrs` | Notes |
|---------|------------|-------------|-------|
| Interactive browser (auth-code + PKCE) | ✅ | ✅ | |
| Device code flow | ✅ | ✅ | |
| Service principal (client secret) | ✅ | ✅ | |
| Service principal (certificate) | ✅ | ✅ | PEM with PKCS#8 private key |
| Managed identity | ✅ | ✅ | IMDS + App Service endpoints |
| Cloud Shell identity | ✅ | ✅ | Auto-detected via MSI_ENDPOINT |
| Username/password (ROPC) | ✅ | ❌ | Deprecated in Python too |
| Token cache + refresh | ✅ (MSAL) | ✅ (custom, `~/.azure/azrs_token_cache.json`) | Different cache files |
| HTTP cache | ✅ | ❌ | |
| Multi-tenant discovery | ✅ | ✅ | |
| Interactive subscription selector | ✅ | ✅ |
| Identity Broker / WAM (silent) | ⚠️ (MSAL broker) | ✅ | `login --use-broker`; Linux D-Bus `com.microsoft.identity.broker1` + Windows WAM. Silent, credential-free on a joined machine, and used as an automatic fallback for any ARM call. | |

---

## 2. Core Features

| Feature | Python `az` | Rust `azrs` | Notes |
|---------|------------|-------------|-------|
| `login` / `logout` | ✅ | ✅ | |
| `account show/list/set` | ✅ | ✅ | |
| `account get-access-token` | ✅ | ✅ | |
| `account management-group` | ✅ | ✅ | |
| `account list-locations` | ✅ | ✅ | |
| `rest` (generic REST calls) | ✅ | ✅ | |
| `config set/get/unset` | ✅ | ✅ | |
| `configure` (interactive wizard) | ✅ | ✅ | |
| `completions` (shell completions) | ✅ | ✅ | |
| `find` (command search) | ✅ | ✅ | Aladdin API |
| `feedback` | ✅ | ❌ | |
| `extension` management | ✅ | ❌ | |
| `cloud` management | ✅ | ✅ | list/show/set |
| `--debug` flag | ✅ | ✅ | |
| `--subscription` flag | ✅ | ✅ | |
| `--output` / `-o` flag | ✅ | ✅ | |
| `--query` (JMESPath) | ✅ | ✅ | |
| `--verbose` flag | ✅ | ✅ | |
| `--only-show-errors` flag | ✅ | ✅ | |

---

## 3. Command Group Parity

### Legend
- ✅ = Implemented (manual or generated)
- 🔧 = Generated via AAZ (may have limited arg support)
- ❌ = Not implemented
- ⚠️ = Partially implemented

### Resource Management (ARM)

| Command Group | Python `az` | Rust `azrs` | Implementation |
|---------------|------------|-------------|----------------|
| `group` | ✅ | ✅ | Manual — full CRUD + wait + lock + export |
| `resource` | ✅ | ✅ | Manual — list/show/delete/create/update/tag/invoke-action/wait/lock/link |
| `provider` | ✅ | ✅ | Manual — list/show/register/unregister/operation/permission |
| `feature` | ✅ | ✅ | Manual — list/show/register/unregister + registration CRUD |
| `tag` | ✅ | ✅ | Manual — list/create/delete/update/add-value/remove-value |
| `lock` | ✅ | ✅ | Manual — create/delete/list/update |
| `deployment` | ✅ | ✅ | Manual — group/sub/mg/tenant scopes, what-if |
| `deployment-scripts` | ✅ | ✅ | Manual — list/show-log/delete |
| `ts` (template specs) | ✅ | ✅ | Manual — list/show/create/delete/export |
| `stack` | ✅ | ✅ | Manual — group/sub/mg scopes |
| `managedapp` | ✅ | ✅ | Manual — app + definition CRUD |
| `policy` | ✅ | 🔧 | Generated |
| `managedservices` | ✅ | 🔧 | Generated |

### Compute

| Command Group | Python `az` | Rust `azrs` | Implementation |
|---------------|------------|-------------|----------------|
| `vm` | ✅ | 🔧 | Generated (`vm_ext.rs` exists but unwired) |
| `vmss` | ✅ | 🔧 | Generated (`vmss_ext.rs` exists but unwired) |
| `disk` | ✅ | 🔧 | Generated |
| `disk-access` | ✅ | 🔧 | Generated |
| `disk-encryption-set` | ✅ | 🔧 | Generated |
| `image` | ✅ | 🔧 | Generated |
| `snapshot` | ✅ | 🔧 | Generated |
| `sig` (shared image gallery) | ✅ | 🔧 | Generated |
| `ppg` (proximity placement) | ✅ | 🔧 | Generated |
| `capacity` (reservations) | ✅ | 🔧 | Generated |
| `restore-point` | ✅ | 🔧 | Generated |
| `sshkey` | ✅ | 🔧 | Generated |
| `compute-fleet` | ✅ | 🔧 | Generated |
| `compute-recommender` | ✅ | ❌ | |

### Networking

| Command Group | Python `az` | Rust `azrs` | Implementation |
|---------------|------------|-------------|----------------|
| `network` | ✅ | 🔧 | Generated (`network.rs` manual module exists, merged with generated) |
| `afd` (Front Door) | ✅ | 🔧 | Generated |
| `cdn` | ✅ | 🔧 | Generated |
| `privatedns` | ✅ | ❌ | |

### Web / App Service

| Command Group | Python `az` | Rust `azrs` | Implementation |
|---------------|------------|-------------|----------------|
| `webapp` | ✅ | ✅ | Manual — extensive (config, deploy, identity, cors, vnet, logs, slots, webjobs, traffic) |
| `functionapp` | ✅ | ✅ | Manual — extensive (config, keys, functions, deploy, plan, slots, vnet, scale) |
| `appservice` | ✅ | ✅ | Manual — plan CRUD + identity, ASE, domain |
| `staticwebapp` | ✅ | ✅ | Manual — CRUD + appsettings, hostname, environment |
| `logicapp` | ✅ | ✅ | Manual — CRUD + config, deploy |
| `containerapp` | ✅ | ❌ | |

### Data / Storage

| Command Group | Python `az` | Rust `azrs` | Implementation |
|---------------|------------|-------------|----------------|
| `storage` | ✅ | ⚠️ | Manual account CRUD + generated blob/share-rm/sku |
| `cosmosdb` | ✅ | 🔧 | Generated |
| `sql` | ✅ | 🔧 | Generated |
| `mysql` | ✅ | ❌ | |
| `postgresql` | ✅ | ❌ | |
| `redis` | ✅ | ❌ | |
| `dls` (Data Lake Store) | ✅ | ❌ | |
| `netappfiles` | ✅ | 🔧 | Generated |

### Security / Identity

| Command Group | Python `az` | Rust `azrs` | Implementation |
|---------------|------------|-------------|----------------|
| `keyvault` | ✅ | ✅ | Manual — secret set/show/list/delete (data-plane) |
| `identity` | ✅ | 🔧 | Generated |
| `security` | ✅ | 🔧 | Generated |
| `role` | ✅ | ❌ | |

### Monitoring / Analytics

| Command Group | Python `az` | Rust `azrs` | Implementation |
|---------------|------------|-------------|----------------|
| `monitor` | ✅ | 🔧 | Generated |
| `policyinsights` | ✅ | ❌ | |
| `synapse` | ✅ | ❌ | |

### Messaging / Events

| Command Group | Python `az` | Rust `azrs` | Implementation |
|---------------|------------|-------------|----------------|
| `servicebus` | ✅ | 🔧 | Generated |
| `eventhubs` | ✅ | 🔧 | Generated |
| `relay` | ✅ | 🔧 | Generated |
| `eventgrid` | ✅ | ❌ | |
| `signalr` | ✅ | ❌ | |

### Containers / Kubernetes

| Command Group | Python `az` | Rust `azrs` | Implementation |
|---------------|------------|-------------|----------------|
| `aks` | ✅ | 🔧 | Generated |
| `acr` | ✅ | ❌ | |
| `acs` | ✅ | ❌ | Deprecated in Python |
| `container` | ✅ | ❌ | |

### Other Services

| Command Group | Python `az` | Rust `azrs` | Implementation |
|---------------|------------|-------------|----------------|
| `billing` | ✅ | 🔧 | Generated |
| `consumption` | ✅ | 🔧 | Generated |
| `search` | ✅ | 🔧 | Generated |
| `lab` | ✅ | 🔧 | Generated |
| `databoxedge` | ✅ | 🔧 | Generated |
| `servicefabric` | ✅ | ❌ | |
| `hdinsight` | ✅ | ❌ | |
| `aro` | ✅ | ❌ | |
| `advisor` | ✅ | ❌ | |
| `ams` (Media Services) | ✅ | ❌ | |
| `apim` (API Management) | ✅ | ❌ | |
| `appconfig` | ✅ | ❌ | |
| `backup` | ✅ | ❌ | |
| `batch` | ✅ | ❌ | |
| `batchai` | ✅ | ❌ | |
| `botservice` | ✅ | ❌ | |
| `cognitiveservices` | ✅ | ❌ | |
| `dms` (Database Migration) | ✅ | ❌ | |
| `iot` | ✅ | ❌ | |
| `maps` | ✅ | ❌ | |
| `marketplaceordering` | ✅ | ❌ | |
| `rdbms` | ✅ | ❌ | |
| `serviceconnector` | ✅ | ❌ | |
| `sqlvm` | ✅ | ❌ | |

---

## 4. Generated Command Limitations

The AAZ code generator (`tools/aaz_gen/`) has known limitations:

- **Parser** only recognizes a subset of AAZ patterns (string args, resource-group, location, dict args, URL params, limited `set_prop` body construction)
- **Emitter** has TODOs for body construction and unsupported HTTP methods
- Generated commands may lack full argument coverage compared to Python equivalents
- Build falls back to stubs if `azure-cli/` source isn't present
- Commands marked 🔧 above should be validated against their Python equivalents for arg completeness

---

## 5. Missing Core Features (not command-specific)

| Feature | Status | Priority | Notes |
|---------|--------|----------|-------|
| SP certificate auth | ✅ | — | Implemented |
| Managed identity auth | ✅ | — | Implemented |
| Cloud Shell auth | ✅ | — | Implemented |
| `yamlc` output format | ✅ | — | Implemented |
| `--verbose` flag | ✅ | — | Implemented |
| `--only-show-errors` flag | ✅ | — | Implemented |
| Extension system | ❌ | Low | Large architectural effort |
| `configure` interactive wizard | ✅ | — | Implemented |
| `find` command search | ✅ | — | Implemented |
| `cloud` CLI commands | ✅ | — | Implemented |
| HTTP response caching | ❌ | Low | Performance optimization |
| `role` RBAC commands | ❌ | High | Used in nearly every deployment |
| `acr` container registry | ❌ | High | Critical for container workflows |

---

## 6. Parity Score

**By command group coverage:**
- Groups with any Rust implementation (manual or generated): **~40/65** ≈ **62%**
- Groups with full manual implementation: **~24/65** ≈ **37%**
- Groups with generated (potentially incomplete) coverage: **~34** (some overlap with manual)
- Groups completely missing from Rust: **~25/65** ≈ **38%**

**By auth feature coverage:** **6/6** = **100%** (excluding deprecated ROPC)

**By core feature coverage:** **~16/18** ≈ **89%** (missing: extension system, feedback)

---

## 7. Highest-Impact Gaps

These are the most commonly used `az` features missing from `azrs`:

1. **`role` (RBAC)** — assignment create/delete/list, definition CRUD — used in nearly every deployment
2. **`acr` (Container Registry)** — login, build, push, import — critical for container workflows
3. **Managed identity auth** — required for CI/CD and automation scenarios
4. **SP certificate auth** — enterprise standard for service principals
5. **`container` / `containerapp`** — growing usage for serverless containers
6. **`iot`** — large ecosystem
7. **`appconfig`** — frequently used with app service
8. **`eventgrid`** — common event-driven architecture component
9. **`vm_ext` / `vmss_ext` wiring** — manual extensions exist but aren't dispatched
10. **Generated command arg completeness** — 🔧 commands need validation against Python equivalents
