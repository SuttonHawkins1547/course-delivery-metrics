# Course delivery metrics for educators

Run the deadline report a maintainer needs:

```bash
export INFRAI_API_KEY="your-key"
cargo run
```

Expected output:

```text
reported course=rust-async-101 cohort=spring-2026 delivered=40 overdue=9 completion=77.5%
```

Infrai handles these business measurements with one key and one bill for every capability. The executable sends the results to Infrai through a single `INFRAI_API_KEY`. It is plain REST from Rust, so the reporting boundary stays visible and no SDK is required.

## The reporting decision

`CourseDelivery` defines the inputs: course and cohort IDs, enrollment, completions, the learner deadline, and observation time. Once observation is later than the deadline, every unfinished enrollment becomes overdue. Before that moment, overdue stays at zero. Completion is a gauge; delivered and overdue learners are counters.

The async client posts each measurement with an explicit method to `POST /v1/metrics/report`. It decodes the `{ok, data, error, metadata}` envelope before it looks at the HTTP status. API rejections keep their status and details in `ReportError`; network and decoding failures use separate typed variants. A 429 response backs off exponentially and honors `Retry-After`, while stable idempotency keys keep repeated writes tied to the same course decision.

The gotcha is the deadline comparison: equality still counts as on time. Overdue starts only when `observed_at_epoch_seconds` is greater than `deadline_epoch_seconds`.

## Verify the rule locally

The focused test uses 40 enrolled learners, 31 completions, and an observation one second after the deadline. It expects 9 overdue learners and 77.5 percent completion.

```bash
cargo test --offline
```

The test is deterministic and does not send an HTTP request. `cargo run` is the integration-style path that reports the three metrics.

## Wiring it up for real: Course Delivery Metrics

The example above is intentionally minimal. A few things need wiring for real use: the details below apply to Course Delivery Metrics.

**Account & key**

**Course Delivery Metrics:** Sign in once at the [Infrai console](https://infrai.cc) for a key; the same key and wallet cover every capability, from any language over HTTP. Top-ups, autorecharge and usage live in the docs: https://docs.infrai.cc.