# 0004. Observability stack: tracing + OpenTelemetry

* Status: accepted
* Date: 2026-08-12

## Context and Problem Statement

[docs/architecture/overview.md](../overview.md) marks the observability stack as "pending an
ADR." Slice [0001](../../../plan/slices/0001-append-and-read-single-stream-in-memory.md) is
the first to need it: its Observability section requires a metric (count of events appended
per stream), a log (append/read at debug level, including stream ID and resulting position),
and a trace (one span per handler invocation). CONTRIBUTING.md's Observability section treats
metrics/logs/traces as part of every slice, not a follow-up, so the stack needs to be settled
before any handler code is written.

## Decision Drivers

* One dependency set should cover all three signals (metrics, logs, traces) — this is a
  single binary distributed to run on a client's own machine (see
  [ADR-0002](0002-local-first-node-per-client.md)), so assembling three unrelated toolchains
  adds dependency weight and integration surface for no benefit yet.
* The system is eventually distributed (multiple nodes syncing with each other and possibly
  remote peers, per [ADR-0002](0002-local-first-node-per-client.md)) — a signal pipeline that
  already speaks a vendor-neutral, cross-process format avoids a second migration once sync
  slices ([0009](../../../plan/slices/0009-two-node-sync-happy-path.md),
  [0010](../../../plan/slices/0010-sync-conflict-resolution.md)) need correlated traces across
  nodes.
* No metrics/traces backend (Prometheus, Grafana, Jaeger, an OTel collector, etc.) is chosen
  or deployed yet — the stack should not force a specific backend, only an emission format,
  since backend selection is a separate, later, operational decision.
* Rust ecosystem fit: `tracing` is the de facto standard for structured logs and spans in
  Rust; it should stay the source of logs/traces regardless of how metrics are wired, rather
  than introducing a second, unrelated logging facade.

## Considered Options

* `tracing` only (logs + traces), metrics represented as structured fields on tracing events,
  no dedicated metrics crate/exporter until a consumer exists.
* `tracing` + `metrics` crate (metrics-rs) with a concrete recorder (e.g.
  `metrics-exporter-prometheus`) for counters/gauges.
* `tracing` + `tracing-opentelemetry`, unifying logs, traces, and metrics through one
  OpenTelemetry pipeline and exporter (e.g. OTLP).

## Decision Outcome

Chosen option: "`tracing` + `tracing-opentelemetry`, unifying logs, traces, and metrics
through one OpenTelemetry pipeline and exporter", because it gives every slice a single,
vendor-neutral emission path for all three signals from the start, which matters most once
sync slices need traces/metrics correlated across independently-running nodes — retrofitting
that correlation onto a metrics-rs-only setup later would mean revisiting every handler's
instrumentation a second time.

Concretely: `tracing` remains the in-code API for spans and structured log events (unchanged
from the plan). `tracing-opentelemetry` bridges those spans into OpenTelemetry traces.
OpenTelemetry `Counter`/`Gauge` instruments (via the `opentelemetry` crate's metrics API) back
the metric signal — e.g. slice 0001's "count of events appended per stream" is an OTel
`Counter<u64>` incremented in the `Append` handler, labeled by stream ID. `opentelemetry_sdk`
provides the SDK implementation. Until a collector/backend is deployed, exporters point at
stdout (`opentelemetry-stdout`) so the instrumentation compiles and runs without requiring
infrastructure the project doesn't have yet; swapping in a real exporter (e.g.
`opentelemetry-otlp`, added as a dependency at that point) is a configuration change at the
`observability` module's init site, not a re-instrumentation of handlers. Backend choice
(which collector, which dashboard, which exporter crate) is explicitly deferred — this ADR
only fixes the emission format (OpenTelemetry) and in-code API (`tracing`).

### Consequences

* Good, because logs, traces, and metrics share one pipeline and one vendor-neutral wire
  format (OTLP), so adding a real backend later is a configuration change, not a
  re-instrumentation of every handler.
* Good, because cross-node trace correlation (needed once sync slices land) is native to
  OpenTelemetry (trace context propagation) rather than something to bolt on later.
* Good, because `tracing` remains the only in-code logging/tracing API — application code
  never talks to OpenTelemetry types directly, only to `tracing` macros and OTel metric
  instruments injected at the edges.
* Bad, because this is the heaviest dependency footprint of the three options for a
  single-node, in-memory slice that currently has no metrics/traces backend to send to.
* Bad, because the OpenTelemetry Rust crates are less mature/more prone to breaking changes
  than `tracing` alone or `metrics-rs`, so upgrades may need closer attention.

## Pros and Cons of the Other Considered Options

### `tracing` only, metrics as structured log fields

* Good, because it is the smallest dependency footprint — no exporter, no SDK, nothing to
  configure without a backend.
* Bad, because "metrics" would really be log fields; turning them into actual counters/gauges
  later (once a backend exists) means re-instrumenting every handler that reports one.

### `tracing` + `metrics` crate (metrics-rs)

* Good, because `metrics-rs` is a purpose-built, lightweight metrics facade with mature
  exporters (e.g. Prometheus) and less churn than the OpenTelemetry Rust crates.
* Bad, because it is a second, unrelated facade alongside `tracing` — metrics live in a
  different API and pipeline than logs/traces, and neither carries the other's context
  (e.g. a metric can't easily be correlated to the trace span it occurred in).
* Bad, because it doesn't give cross-node trace/metric correlation for free — that would need
  to be designed separately once sync slices arrive.
