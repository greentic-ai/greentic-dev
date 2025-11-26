# Component Scaffolder (`greentic-dev component`)

The scaffolder now emits a lean component workspace that builds with `cargo component`, consumes `greentic-interfaces-guest`, and exposes provider metadata out of the box. This guide walks through what the scaffold contains and how to iterate on it.

---

## Generated layout

```
component-<name>/
├── Cargo.toml
├── provider.toml
├── README.md
├── schemas/
│   └── v1/config.schema.json
└── src/
    └── lib.rs
```

### Key files

- **`Cargo.toml`** – Minimal manifest that depends on `greentic-interfaces-guest`, `serde`, and `serde_json`, and declares the component metadata used by tooling.
- **`provider.toml`** – Canonical metadata (name, version, ABI pins, capabilities, artifact location). `greentic-dev component validate` and `greentic-dev component pack` both consume this file.
- **`README.md`** – Quickstart for the component author (build, validate, pack).
- **`schemas/v1/config.schema.json`** – Draft 7 JSON Schema for the node configuration used by the runner and transcripts.
- **`src/lib.rs`** – Hello-world implementation using the guest bindings. It exports the `greentic:component/node` world and touches secrets/state/HTTP/telemetry to illustrate imports.

Older assets (`src/describe.rs`, `tests/schema_validates_examples.rs`, `examples/flows/min.ygtc`, `.github/workflows/ci.yml`) are intentionally no longer generated; they live in the main repository instead.

---

## Typical workflow inside the scaffold

1. **Build:** `cargo component build --release --target wasm32-wasip2`  
   Compiles to `target/wasm32-wasip2/release/<name>.wasm`. The scaffolder sets `CARGO_COMPONENT_CACHE_DIR` to a local folder so the command works offline once the cargo cache is warmed.

2. **Validate:** `greentic-dev component validate --path .`  
   Checks that the artifact exists, decodes embedded WIT metadata, compares it against `provider.toml`, and (if WASI host shims are present) inspects the manifest via the current host/runtime hooks. Missing WASI support produces a warning but doesn’t fail validation.

3. **Pack (optional):** `greentic-dev component pack --path .`  
   Copies the `.wasm`, writes `meta.json` (provider metadata + sha + timestamp), and generates `SHA256SUMS` under `packs/<name>/<version>/`.

4. **Wire into flows:** Back in the main workspace, point a flow node at the component (`using: <name>`) and run `greentic-dev flow validate -f <flow>.ygtc --json` (or `cargo run -p greentic-dev -- flow …` during local development). When you are ready to exercise the pack end-to-end, follow up with `greentic-dev pack build …` and `greentic-dev pack run …`.

---

## Customising the scaffold

- **Schema & defaults:** Edit `schemas/v1/config.schema.json` and keep it in sync with the behaviour inside `src/lib.rs`. Greentic transcripts record defaults vs overrides directly from this schema.
- **Provider metadata:** Update `provider.toml` as your component evolves (capabilities, WIT package requirements, artifact path).
- **Guest imports:** Pull in additional guest modules from `greentic-interfaces-guest` (e.g., OAuth broker, lifecycle) as needed; no local WIT vending is required.
- **Documentation:** Extend the scaffolded `README.md` or add a `docs/` directory in the component repo to mirror the patterns we use across Greentic components.

---

## FAQ

**Why does `greentic-dev component validate` sometimes skip manifest exports?**  
Until the runtime bundles WASI Preview 2 shims, components that import `wasi:*` interfaces cannot be instantiated locally. The validator spots this case, prints a warning, and finishes without the runtime checks.

**How do I stay current with Greentic interface upgrades?**  
Update the Greentic workspace to the new crate versions, bump the `greentic-interfaces-guest` version in the scaffolder, and regenerate components as needed so the bindings and metadata stay aligned.

**Where did the old `describe.rs` go?**  
The WASM component now exposes `describe` capabilities directly through the generated guest bindings. The CLI runner prefers the schema that comes from the component artifact, so no extra stub is needed in the scaffold.

---

Use this document any time you scaffold a new component to ensure you understand the generated pieces and how they factor into validation.
