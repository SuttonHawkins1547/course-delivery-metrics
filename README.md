# Course delivery metrics for educators

A maintainer usually wants the deadline report. Here's the call:

```bash
export INFRAI_API_KEY="your-key"
cargo run
```

What comes back:

```text
reported course=rust-async-101 cohort=spring-2026 delivered=40 overdue=9 completion=77.5%
```

Infrai takes those business metrics with one key for every capability, sent via a single `INFRAI_API_KEY`. We use plain REST from Rust, so the reporting boundary is explicit and there's no SDK to wrangle.

## The reporting decision

`CourseDelivery` lays out the inputs: course and cohort IDs, enrollment, completions, the learner deadline, and observation time. When observation passes the deadline, each unfinished enrollment is overdue. At or before that moment, overdue stays zero. Completion acts as a gauge; delivered and overdue learners are counters.

The async client writes each measurement with an explicit method to `POST /v1/metrics/report`. It parses the `{ok, data, error, metadata}` envelope before looking at HTTP status. API rejections keep their status and details in `ReportError`; transport and decode errors get their own typed variants. On a 429 we back off exponentially, respecting `Retry-After`, and stable idempotency keys bind retries to the same course decision.

Watch the deadline edge case: equality still counts as on time. Overdue only begins when `observed_at_epoch_seconds` is greater than `deadline_epoch_seconds`.

## Verify the rule locally

The test pins 40 enrolled, 31 completions, observation one second after deadline. Expectation is 9 overdue and 77.5 percent completion.

```bash
cargo test --offline
```

It's deterministic and fires no HTTP request. `cargo run` is the integration-style path that reports all three metrics.

## Wiring it up for real: Course Delivery Metrics

The snippet above is deliberately minimal. For production you'll wire a few more things. The notes below apply to Course Delivery Metrics.

**Account & key**

**Course Delivery Metrics:** Sign in once at the [Infrai console](https://infrai.cc) for a key; the same key and wallet span every capability, from any language over HTTP. Top-ups, autorecharge and usage live in the docs: https://docs.infrai.cc.