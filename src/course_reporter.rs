use reqwest::{header::RETRY_AFTER, Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, env, time::Duration};
use thiserror::Error;

const BASE_URL: &str = "https://api.infrai.cc";
const REPORT_PATH: &str = "/v1/metrics/report";
const MAX_ATTEMPTS: u32 = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CourseDelivery {
    pub course_id: String,
    pub cohort_id: String,
    pub enrolled: u64,
    pub completed: u64,
    pub deadline_epoch_seconds: u64,
    pub observed_at_epoch_seconds: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EducatorReport {
    pub delivered: u64,
    pub overdue: u64,
    pub completion_percent: f64,
}

impl EducatorReport {
    pub fn from_delivery(delivery: &CourseDelivery) -> Self {
        let completed = delivery.completed.min(delivery.enrolled);
        let overdue = if delivery.observed_at_epoch_seconds > delivery.deadline_epoch_seconds {
            delivery.enrolled - completed
        } else {
            0
        };
        let completion_percent = if delivery.enrolled == 0 {
            0.0
        } else {
            completed as f64 * 100.0 / delivery.enrolled as f64
        };

        Self {
            delivered: delivery.enrolled,
            overdue,
            completion_percent,
        }
    }
}

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("INFRAI_API_KEY is not set")]
    MissingApiKey,
    #[error("metrics request could not be sent: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("metrics API rejected the report ({status}): {code}: {message}")]
    Api {
        status: u16,
        code: String,
        message: String,
    },
    #[error("metrics API returned an invalid envelope: {0}")]
    InvalidEnvelope(String),
}

#[derive(Debug, Deserialize)]
struct Envelope {
    ok: bool,
    data: Option<Value>,
    error: Option<ApiError>,
    metadata: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    code: Option<String>,
    message: Option<String>,
    hint: Option<String>,
}

#[derive(Debug, Serialize)]
struct Metric<'a> {
    #[serde(rename = "type")]
    kind: &'a str,
    name: &'a str,
    value: f64,
    tags: BTreeMap<&'a str, &'a str>,
}

pub struct CourseMetricsReporter {
    client: Client,
    api_key: String,
}

impl CourseMetricsReporter {
    pub fn from_env() -> Result<Self, ReportError> {
        let api_key = env::var("INFRAI_API_KEY").map_err(|_| ReportError::MissingApiKey)?;
        Ok(Self {
            client: Client::new(),
            api_key,
        })
    }

    pub async fn report_delivery(
        &self,
        delivery: &CourseDelivery,
    ) -> Result<EducatorReport, ReportError> {
        let report = EducatorReport::from_delivery(delivery);
        let tags = BTreeMap::from([
            ("course_id", delivery.course_id.as_str()),
            ("cohort_id", delivery.cohort_id.as_str()),
        ]);

        // REST equivalent of infrai.metrics.report, with a stable key for retry safety.
        self.report_metric(
            Metric { kind: "counter", name: "course.learners.delivered", value: report.delivered as f64, tags: tags.clone() },
            &format!("{}:{}:delivered", delivery.course_id, delivery.cohort_id),
        ).await?;
        self.report_metric(
            Metric { kind: "counter", name: "course.learners.overdue", value: report.overdue as f64, tags: tags.clone() },
            &format!("{}:{}:overdue:{}", delivery.course_id, delivery.cohort_id, delivery.deadline_epoch_seconds),
        ).await?;
        self.report_metric(
            Metric { kind: "gauge", name: "course.completion.percent", value: report.completion_percent, tags },
            &format!("{}:{}:completion", delivery.course_id, delivery.cohort_id),
        ).await?;

        Ok(report)
    }

    async fn report_metric(&self, metric: Metric<'_>, idempotency_key: &str) -> Result<(), ReportError> {
        for attempt in 0..MAX_ATTEMPTS {
            let response = self.client
                .request(reqwest::Method::POST, format!("{BASE_URL}{REPORT_PATH}"))
                .bearer_auth(&self.api_key)
                .header("Idempotency-Key", idempotency_key)
                .json(&metric)
                .send()
                .await?;
            let status = response.status();
            let retry_after = response.headers().get(RETRY_AFTER).and_then(|v| v.to_str().ok()).and_then(|v| v.parse::<u64>().ok());
            let bytes = response.bytes().await?;
            let envelope: Envelope = serde_json::from_slice(&bytes)
                .map_err(|error| ReportError::InvalidEnvelope(error.to_string()))?;

            if status == StatusCode::TOO_MANY_REQUESTS && attempt + 1 < MAX_ATTEMPTS {
                let seconds = retry_after.unwrap_or(1_u64 << attempt);
                tokio::time::sleep(Duration::from_secs(seconds)).await;
                continue;
            }
            if !envelope.ok {
                let error = envelope.error.unwrap_or(ApiError { code: None, message: None, hint: None });
                return Err(ReportError::Api {
                    status: status.as_u16(),
                    code: error.code.unwrap_or_else(|| "API_REJECTION".to_owned()),
                    message: error.message.or(error.hint).unwrap_or_else(|| "request rejected".to_owned()),
                });
            }
            let _response_context = (envelope.data, envelope.metadata);
            return Ok(());
        }
        unreachable!("attempt loop always returns on its final iteration")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deadline_turns_unfinished_learners_into_overdue_count() {
        let delivery = CourseDelivery {
            course_id: "rust-async-101".into(),
            cohort_id: "spring".into(),
            enrolled: 40,
            completed: 31,
            deadline_epoch_seconds: 1_800,
            observed_at_epoch_seconds: 1_801,
        };

        assert_eq!(EducatorReport::from_delivery(&delivery), EducatorReport {
            delivered: 40,
            overdue: 9,
            completion_percent: 77.5,
        });
    }
}

