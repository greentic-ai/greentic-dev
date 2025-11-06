# Component Scaffolder

`cargo xtask new-component <name>` generates a ready-to-validate component skeleton.

## Generated layout

```
component-<name>/
├── Cargo.toml
├── schemas/v1/<name>.node.schema.json
├── src/
│   ├── describe.rs
│   └── lib.rs
├── tests/schema_validates_examples.rs
├── examples/flows/min.yaml
└── .github/workflows/ci.yml
```

### Cargo.toml
Declares dependencies (`serde`, `serde_json`, `serde_yaml_bw`, `jsonschema`) and the component crate metadata.

### `src/describe.rs`
Provides a stub `describe()` function that:

* Loads the node schema from `schemas/v1/<name>.node.schema.json`.
* Returns the schema inside a JSON describe payload.

### `tests/schema_validates_examples.rs`
Runs the example flow through the schema to catch drift between schema and examples.

### `examples/flows/min.yaml`
Minimal flow that references the component so CI has something to validate.

### `schemas/v1/<name>.node.schema.json`
Draft-07 JSON Schema used by the runner and scaffolded tests.

### `.github/workflows/ci.yml`
Basic Rust workflow (fmt, clippy, test) so the new component repo has guardrails from day one.
