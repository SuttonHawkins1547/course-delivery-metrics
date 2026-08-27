use course_delivery_metrics::course_reporter::{CourseDelivery, CourseMetricsReporter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let delivery = CourseDelivery {
        course_id: "rust-async-101".into(),
        cohort_id: "spring-2026".into(),
        enrolled: 40,
        completed: 31,
        deadline_epoch_seconds: 1_787_679_600,
        observed_at_epoch_seconds: 1_787_683_200,
    };

    let report = CourseMetricsReporter::from_env()?.report_delivery(&delivery).await?;
    println!(
        "reported course={} cohort={} delivered={} overdue={} completion={:.1}%",
        delivery.course_id, delivery.cohort_id, report.delivered, report.overdue, report.completion_percent
    );
    Ok(())
}

