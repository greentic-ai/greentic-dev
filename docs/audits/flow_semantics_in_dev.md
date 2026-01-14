# Flow Semantics Implemented in greentic-dev

## Flow-Related Commands and Codepaths

| Command / Path | Upstream intent | Extra logic in greentic-dev | Classification | Location |
| --- | --- | --- | --- | --- |
| `flow ...` | `greentic-flow` CLI | No extra logic; arguments are passed through directly. | PASS-THROUGH | `src/main.rs` |

## Semantic Behaviors and Repro Snippets

- None: greentic-dev no longer adds flow-specific logic; behavior matches greentic-flow.
