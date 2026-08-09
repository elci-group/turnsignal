# Turnsignal

Turnsignal is a lightweight inter-agent routing and state-transition layer. It observes an
agent result or intended action, infers the most probable next step, and emits a small,
actionable signal. It does not solve the task; it answers **“given what just happened,
what should happen next?”**

The included engine is deterministic and local, so the control plane needs no model call.
The `Inferencer` and `Router` traits make it straightforward to replace the heuristics with
a cheap model or an application-specific policy.

## Try it

```sh
cargo run -- "Inspected the repository. Compliance tests are absent."
```

```text
→ Next: inspect test infrastructure
Reason: Tests or compliance checks appear to be absent
Confidence: 0.91
```

Pipe long agent output into Turnsignal and request the canonical schema:

```sh
agent-command | turnsignal --mode routing --state repository_inspected --json
```

Anticipate the consequences of an intended action:

```sh
turnsignal --projected --mode injective "Add Redis caching to the API"
```

```text
[TURNSIGNAL]
Next action: inspect existing abstractions and deployment.
Reason: The intended change crosses dependency, configuration, and operations boundaries.
Confidence: 0.90.
[/TURNSIGNAL]
```

## Operating modes

- **Advisory** prints a concise recommendation while leaving the decision to the caller.
- **Injective** emits a short-lived `[TURNSIGNAL]` context block (`ttl: 1`).
- **Routing** attaches a destination selected by the router. The built-in policy recognizes
  documentation → `goglz`, compliance → `isopod`, releases → `kaptaind`, scalability →
  `iris`, and implementation → `codex`.

Use `--json` with any mode to obtain the entire delivery envelope. `--compact` produces a
single-line representation suitable for pipes and agent context injection.

## Canonical signal

```json
{
  "mode": "routing",
  "signal": {
    "source": "agent",
    "transition": {
      "from": "repository_inspected",
      "to": "repository_analysis_complete"
    },
    "outcome": "test_gap_identified",
    "confidence": 0.91,
    "next": {
      "action": "inspect_test_infrastructure",
      "actor": "main_agent",
      "priority": "high",
      "reason": "Tests or compliance checks appear to be absent"
    },
    "evidence": {
      "items": ["Compliance tests are absent"]
    },
    "injection": {
      "mode": "context",
      "ttl": 1
    },
    "route": {
      "target": "isopod",
      "reason": "compliance work detected"
    }
  },
  "message": "..."
}
```

Signals are intentionally lossy. Evidence is capped at three short excerpts, confidence is
bounded to `[0, 1]`, and context injections live for one turn by default.

## Library API

```rust
use turnsignal::{Observation, SignalMode, Turnsignal};

let engine = Turnsignal::default();
let observation = Observation::projected("Add Redis caching to the API")
    .with_source("planner");
let delivery = engine.dispatch(
    &observation,
    Some("architecture_understood"),
    SignalMode::Routing,
);

assert_eq!(delivery.signal.transition.from.as_deref(), Some("architecture_understood"));
assert_eq!(delivery.signal.injection.ttl, 1);
```

Implement `Inferencer` to call a small language model, classifier, or learned transition
policy. Implement `Router` to map the resulting action onto your own agents and tools. The
engine keeps inference, transition representation, rendering, and routing separate.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
deliver --spec deliver.toml --strict
```

Turnsignal is complementary to long-term memory systems: strategic memory captures what
was learned over time; Turnsignal supplies the tactical transition needed now.
