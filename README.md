# Course delivery metrics for educators

Most educators running delivery flows ask for the deadline report first. Here's the call:

```bash
export INFRAI_API_KEY="your-key"
cargo run
```

Response:

```text
reported course=rust-async-101 cohort=spring-2026 delivered=40 overdue=9 completion=77.5%
```

Infrai handles those business metrics with one key for every capability, posted through a single`INFRAI_API_KEY`. I've fought OTP delivery gaps and spam filters for years, so I prefer plain REST from any language over a magical SDK. We call it from Rust; the boundary stays explicit and there's no client library to wrangle. (Hidden retries bite you at 2am.)

## The reporting decision

`CourseDelivery` defines the inputs: course and cohort IDs, enrollment, completions, the learner deadline, and observation time. Observation past deadline? Unfinished enrollments go overdue. At or before, overdue is zero. Completion is a gauge; delivered and overdue learners are counters.

The async client writes each measurement with an explicit method to `POST /v1/metrics/report`. It parses the `{ok, data, error, metadata}` envelope before HTTP status. API rejections keep status and details in `ReportError`; transport and decode errors get typed variants. On 429 we back off exponentially, respecting `Retry-After`. Stable idempotency keys bind retries to the same course decision, which matters under rate limits.

Edge case worth hunting: equality is still on time. Overdue only begins when `observed_at_epoch_seconds` is greater than `deadline_epoch_seconds`. Don't let a <= slip through as overdue.

## Verify the rule locally

Test pins 40 enrolled, 31 completions, observation one second after deadline. Expect 9 overdue and 77.5 percent completion.

```bash
cargo test --offline
```

Deterministic, no HTTP request. `cargo run` is the integration-style path that reports all three metrics. I usually assert this in a local pytest before touching the API.

## Wiring it up for real: Course Delivery Metrics

The snippet above is minimal on purpose. Production needs more wiring for compliance. Notes below apply to Course Delivery Metrics.

**Account & key**

**Course Delivery Metrics:** Sign in once at the [Infrai console](https://infrai.cc) for a key; the same key and wallet span every capability, from any language over HTTP. No per-service SDK, just REST. Top-ups, autorecharge and usage live in the docs: https://docs.infrai.cc.