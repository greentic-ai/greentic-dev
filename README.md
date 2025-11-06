# greentic-dev – schema-aware developer toolkit

`greentic-dev` is the command-line toolbox we use to design, validate, and iterate on Greentic components before they ever hit production. It bundles a schema-aware flow runner, mock services, a transcript viewer, and a component scaffolder into one workspace so that building new automation feels repeatable and safe.

If you want to:

* prove that a flow YAML matches the latest component schema,
* spin up a new component repo that already understands describe APIs, JSON Schema, and CI guardrails,
* emulate Greentic services locally without real credentials, and
* inspect transcripts that show **which values came from defaults vs overrides**,

…then this repository is where you start.

---

## What lives in this workspace?

| Crate / folder        | Purpose                                                                                  |
|-----------------------|------------------------------------------------------------------------------------------|
| `crates/dev-runner`   | Validates flows by compiling each node’s describe() schema and optional conformance kit. |
| `crates/dev-viewer`   | Renders transcripts and highlights defaults/overrides so you can reason about configs.    |
| `xtask`               | Provides `cargo xtask new-component <name>` scaffolding and helper scripts.              |
| `docs/`               | High-level guides (runner, mocks, viewer, scaffolder, developer guide).                  |
| `scripts/build_pages.py` | Builds the GitHub Pages site by combining Rustdoc output with the markdown guides.    |

You will also find mock-service helpers for HTTP, NATS, and vault-like secrets in `dev-runner`, ready to be wired into flows.

---

## Why schema awareness matters

Flows in Greentic are YAML documents describing a set of nodes. Historically it was easy to typo a field or forget a required input; you would only discover the mistake at runtime or during a conformance run. The runner in this repository flips that around:

1. Load your flow YAML.
2. For each node, call the component’s `describe()` (or use a registered schema stub).
3. Compile the JSON Schema (Draft 7) and validate the node configuration.
4. Merge defaults, capture resolved config, schema ID, and validation log in a transcript.

Because validation happens before execution, you can run it on every commit or as part of CI:

```bash
cargo run -p greentic-dev -- run -f examples/flows/min.yaml --validate-only
```

The `--validate-only` flag is deliberately fast—it skips tool execution but still produces transcripts at `.greentic/transcripts/`.

### Examining the transcript

Use the viewer to inspect the result:

```bash
cargo run -p dev-viewer -- --file .greentic/transcripts/min-<timestamp>.yaml
```

You will see output like:

```
inputs:
  client_id: abc (override)
  client_secret: null (default)
```

so you immediately know which fields rely on defaults versus user input.

---

## Cheatsheet: validate, view, iterate

| Action                          | Command                                                                 |
|---------------------------------|-------------------------------------------------------------------------|
| Validate a flow                 | `cargo run -p greentic-dev -- run -f <flow>.yaml --validate-only`       |
| View transcript                 | `cargo run -p dev-viewer -- --file .greentic/transcripts/<file>.yaml`   |
| Run full test suite             | `cargo test` \| `cargo test --features conformance`                     |
| Lint everything                 | `cargo clippy --all-targets --all-features -- -D warnings`              |
| Format                          | `cargo fmt`                                                             |
| Generate docs locally           | `cargo doc --workspace --no-deps`                                       |
| Build GitHub Pages bundle       | `python3 scripts/build_pages.py`                                        |

---

## Creating a component – the “why” and the “how”

Below is the full workflow we follow when creating a complex component that we can run with local mocks. Each step includes not only **how** to perform it, but **why** it matters in the Greentic ecosystem.

### 1. Scaffold with `cargo xtask`

```bash
cargo xtask new-component my-component
cd component-my-component
```

**Why**: Every component in Greentic must expose schema metadata, documentation, and CI guardrails. The scaffold bakes these in: JSON Schema, describe stub, docs, example flow, unit tests, and a GitHub Actions workflow. Starting from the scaffold keeps new components consistent and reduces the “blank page” problem.

The generated layout:

```
component-my-component/
├── Cargo.toml
├── schemas/v1/my-component.node.schema.json
├── src/describe.rs
├── src/lib.rs
├── tests/schema_validates_examples.rs
├── examples/flows/min.yaml
├── docs/
│   ├── getting-started.md
│   ├── config.md
│   ├── testing.md
│   ├── ci.md
│   └── security.md
└── .github/workflows/ci.yml
```

### 2. Design the schema and defaults

Open `schemas/v1/my-component.node.schema.json` and model the configuration surface. Provide defaults wherever possible so flows stay concise. The runner uses this schema for validation and merges defaults into transcripts, so keeping it authoritative is critical.

Run the schema test (why: ensures docs and examples match the schema):

```bash
cargo test
```

Update `docs/config.md` to explain each field, default values, and a sample YAML node. This becomes the contract you hand to flow authors.

### 3. Implement `describe()`

Inside `src/describe.rs`, wire up the `describe()` function so it returns a payload like:

```rust
json!({
    "component": "my-component",
    "version": 1,
    "schemas": {
        "node": node_schema_json
    }
})
```

**Why**: The runner prioritises the schema returned from `describe()` over registry stubs. Having the component own its schema guarantees validation is always using the latest version (including when packaged or deployed).

### 4. Build the business logic / pack

Depending on your delivery format (native binary, WASM pack), implement the runtime operations. We typically expose a CLI entry point:

```bash
cargo build
./target/debug/my-component describe            # returns schema
./target/debug/my-component run --config path   # executes using local config
```

If you are packaging as a Greentic pack, run `greentic-pack build` after compile-time checks pass.

### 5. Write unit and integration tests

Augment the generated `tests/schema_validates_examples.rs` with component-specific validation. Use golden files when comparing structured output. Regenerate goldens intentionally:

```bash
GOLDEN_ACCEPT=1 cargo test      # update goldens
cargo test                      # ensure clean run without regeneration
```

**Why**: Tests give confidence that the schema, docs, and behaviour never drift apart. Golden snapshots make it obvious when response shape changes, forcing a conscious decision to accept the new behaviour.

### 6. Document everything

The scaffolded docs are intentionally verbose. Update them as you build:

* `docs/getting-started.md` – how a teammate clones, builds, and tests the component.
* `docs/config.md` – every YAML field, defaults, and examples.
* `docs/testing.md` – how to run the harness, regenerate goldens, and debug failures.
* `docs/ci.md` – what to expect from PR/nightly pipelines (and secrets policy).
* `docs/security.md` – how tokens are handled (no raw secrets; only opaque handles).

These documents publish automatically via the main repo’s Pages workflow, so downstream teams can self-serve.

### 7. Validate flows locally

Back in the `greentic-dev` repo, add or update flow YAML files referencing your new component. Run validation:

```bash
cargo run -p greentic-dev -- run -f examples/flows/my-component.yaml --validate-only
```

Why: This ensures the flow is immediately compatible with the runner and surfaces schema errors before integrating with the larger conformance kit.

### 8. Inspect transcripts with the viewer

After validation, open the transcript produced in `.greentic/transcripts`. It shows:

* Which defaults were applied (useful for confirming environment-specific values).
* Which overrides came directly from the flow author.
* Schema IDs and any warnings.

```bash
cargo run -p dev-viewer -- --file .greentic/transcripts/my-component-<ts>.yaml
```

### 9. Run with mocks (optional execution)

The dev runner bundles loopback mocks for HTTP, NATS, and a vault-like secret store. Configure them via environment variables (`MOCK_HTTP_PORT`, `MOCK_NATS_PORT`, etc.) when developing features like retries or fault injection. See `docs/mocks.md` for the full matrix of knobs including failure injection such as `MOCK_HTTP_FAIL_PATTERN`.

While execution mode is evolving, you can already simulate flow behaviour by orchestrating components against these mocks or by using transcripts to drive manual checks.

### 10. Harden CI & publish docs

Before opening a PR:

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

The scaffolded CI workflow mirrors these commands. Once merged, GitHub Pages automatically rebuilds:

```
https://greentic-ai.github.io/greentic-dev/
```

The site now includes:

* Rust API docs (`cargo doc` output),
* Runner, mocks, viewer, scaffolder guides, and
* The developer guide (this document) so the process is documented once.

Finally, publish your component’s own schema (usually under `component-<name>/gh-pages`) so the runner can fetch it in describe() responses.

---

## Additional resources

* **Runner guide** – `docs/runner.md`
* **Mocks guide** – `docs/mocks.md`
* **Viewer guide** – `docs/viewer.md`
* **Scaffolder internals** – `docs/scaffolder.md`
* **Developer guide (HTML)** – `https://greentic-ai.github.io/greentic-dev/docs/developer-guide.html`
* **GitHub Pages index** – `https://greentic-ai.github.io/greentic-dev/`

If you need help wiring your component into the larger conformance suites, check the `greentic-conformance` crate (available on crates.io) and wire its flows into `dev-runner`’s validation APIs.

Happy building! This toolkit should make it painless to iterate on components with confidence before they enter the main platform.
