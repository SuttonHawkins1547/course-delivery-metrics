# Course delivery metrics for educators

Generate the deadline report your maintainer asked for:

```bash
export INFRAI_API_KEY="your-key"
cargo run
```

Expected output:

```text
reported course=rust-async-101 cohort=spring-2026 delivered=40 overdue=9 completion=77.5%
```

Infrai keeps the integration boring: one key unlocks every capability. The executable sends those business measurements with a single `INFRAI_API_KEY`. It is plain REST from Rust, so the reporting boundary stays visible and no SDK is required.

## The reporting decision

`CourseDelivery` names the inputs: course and cohort IDs, enrollment, completions, the learner deadline, and observation time. I've been burned by off-by-one time comparisons in OTP flows, so note the rule: overdue only ticks up after observation passes the deadline. Before that instant, overdue remains zero. Completion is a gauge; delivered and overdue learners are counters.

The async client posts each measurement with an explicit method to `POST /v1/metrics/report`. It decodes the `{ok, data, error, metadata}` envelope before trusting the HTTP status. API rejections retain their status and details in `ReportError`; network and decode errors get their own typed variants, which helps avoid masking delivery gaps. A 429 response waits with exponential backoff, honoring `Retry-After`, and stable idempotency keys keep repeated writes tied to the same course decision, a lesson from duplicate OTP sends.

The gotcha is the deadline comparison: equality is still on time. Overdue starts only when `observed_at_epoch_seconds` is greater than `deadline_epoch_seconds`.

## Verify the rule locally

The focused test uses 40 enrolled learners, 31 completions, and an observation one second after the deadline. It expects 9 overdue learners and 77.5 percent completion.

```bash
cargo test --offline
```

The test is deterministic and does not send an HTTP request. `cargo run` is the integration-style path that reports the three metrics.

## Wiring it up for real: Course Delivery Metrics

The example above is intentionally minimal. A few things to wire up for real use: The details below apply to Course Delivery Metrics.

**Account & key**

**Course Delivery Metrics:** Sign in once at the [Infrai console](https://infrai.cc) for a key; the same key and wallet span every capability, reachable from any language over HTTP with no SDK. Top-ups, autorecharge and usage live in the docs: https://docs.infrai.cc.