# PADAGONIA Integration Roadmap

See `/home/sal/padagonia/docs/enterprise-integration-directives.md`.

## Modules

- `coordination_event_writer`: record agent, route, state, handoff, and policy
  events using transaction batches.
- `lease_correlator`: link live Turnsignal decisions to Locksmith lease IDs.
- `state_reader`: retrieve current and historical transitions with namespace
  and freshness bounds.
- `recovery_replayer`: reconstruct state after restart without duplicating
  handoffs or granting authority.
- `audit_export`: produce deterministic operator timelines.

## Acceptance gates

Padagonia is never treated as the live lock authority, handoffs are idempotent,
and recovery cannot create an unauthorized transition.
