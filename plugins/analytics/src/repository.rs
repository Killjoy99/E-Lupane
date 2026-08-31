use sqlx::PgPool;
use sqlx::QueryBuilder;
use uuid::Uuid;
use crate::models::*;

pub async fn insert_event(pool: &PgPool, event: &AnalyticsEvent) -> anyhow::Result<AnalyticsEvent> {
    let row = sqlx::query_as::<_, AnalyticsEvent>(
        r#"INSERT INTO analytics_events (id, event_type, user_id, course_id, resource_type, resource_id, metadata, occurred_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
           RETURNING id, event_type, user_id, course_id, resource_type, resource_id, metadata, occurred_at"#,
    )
    .bind(event.id)
    .bind(&event.event_type)
    .bind(event.user_id)
    .bind(event.course_id)
    .bind(&event.resource_type)
    .bind(event.resource_id)
    .bind(&event.metadata)
    .bind(event.occurred_at)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn list_events(pool: &PgPool, user_id: Option<Uuid>, course_id: Option<Uuid>, event_type: Option<&str>, start: Option<chrono::DateTime<chrono::Utc>>, end: Option<chrono::DateTime<chrono::Utc>>) -> anyhow::Result<Vec<AnalyticsEvent>> {
    let mut builder = QueryBuilder::new("SELECT * FROM analytics_events WHERE 1=1");
    if user_id.is_some() {
        builder.push(" AND user_id = ").push_bind(user_id);
    }
    if course_id.is_some() {
        builder.push(" AND course_id = ").push_bind(course_id);
    }
    if event_type.is_some() {
        builder.push(" AND event_type = ").push_bind(event_type);
    }
    if start.is_some() {
        builder.push(" AND occurred_at >= ").push_bind(start);
    }
    if end.is_some() {
        builder.push(" AND occurred_at <= ").push_bind(end);
    }
    builder.push(" ORDER BY occurred_at DESC");

    let rows = builder.build_query_as::<AnalyticsEvent>().fetch_all(pool).await?;
    Ok(rows)
}

pub async fn upsert_aggregate(pool: &PgPool, aggregate: &AnalyticsAggregate) -> anyhow::Result<AnalyticsAggregate> {
    let row = sqlx::query_as::<_, AnalyticsAggregate>(
        r#"INSERT INTO analytics_aggregates (id, bucket, bucket_start, bucket_end, user_id, course_id, metric, value, metadata, calculated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
           ON CONFLICT (bucket, bucket_start, bucket_end, user_id, course_id, metric) WHERE user_id IS NOT NULL AND course_id IS NOT NULL
           DO UPDATE SET value = EXCLUDED.value, metadata = EXCLUDED.metadata, calculated_at = EXCLUDED.calculated_at
           RETURNING id, bucket, bucket_start, bucket_end, user_id, course_id, metric, value, metadata, calculated_at"#,
    )
    .bind(aggregate.id)
    .bind(&aggregate.bucket)
    .bind(aggregate.bucket_start)
    .bind(aggregate.bucket_end)
    .bind(aggregate.user_id)
    .bind(aggregate.course_id)
    .bind(&aggregate.metric)
    .bind(aggregate.value)
    .bind(&aggregate.metadata)
    .bind(aggregate.calculated_at)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn get_aggregates(pool: &PgPool, bucket: &str, bucket_start: chrono::DateTime<chrono::Utc>, bucket_end: chrono::DateTime<chrono::Utc>, course_id: Option<Uuid>, user_id: Option<Uuid>, metric: Option<&str>) -> anyhow::Result<Vec<AnalyticsAggregate>> {
    let mut builder = QueryBuilder::new("SELECT * FROM analytics_aggregates WHERE bucket = ");
    builder.push_bind(bucket);
    builder.push(" AND bucket_start = ").push_bind(bucket_start);
    builder.push(" AND bucket_end = ").push_bind(bucket_end);
    if course_id.is_some() {
        builder.push(" AND course_id = ").push_bind(course_id);
    }
    if user_id.is_some() {
        builder.push(" AND user_id = ").push_bind(user_id);
    }
    if metric.is_some() {
        builder.push(" AND metric = ").push_bind(metric);
    }

    let rows = builder.build_query_as::<AnalyticsAggregate>().fetch_all(pool).await?;
    Ok(rows)
}

pub async fn create_report(pool: &PgPool, report: &AnalyticsReport) -> anyhow::Result<AnalyticsReport> {
    let row = sqlx::query_as::<_, AnalyticsReport>(
        r#"INSERT INTO analytics_reports (id, report_type, format, filters, data, file_path, generated_by, generated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
           RETURNING id, report_type, format, filters, data, file_path, generated_by, generated_at"#,
    )
    .bind(report.id)
    .bind(&report.report_type)
    .bind(&report.format)
    .bind(&report.filters)
    .bind(&report.data)
    .bind(&report.file_path)
    .bind(report.generated_by)
    .bind(report.generated_at)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn list_reports(pool: &PgPool, report_type: Option<&str>, generated_by: Option<Uuid>) -> anyhow::Result<Vec<AnalyticsReport>> {
    let mut builder = QueryBuilder::new("SELECT * FROM analytics_reports WHERE 1=1");
    if report_type.is_some() {
        builder.push(" AND report_type = ").push_bind(report_type);
    }
    if generated_by.is_some() {
        builder.push(" AND generated_by = ").push_bind(generated_by);
    }
    builder.push(" ORDER BY generated_at DESC");

    let rows = builder.build_query_as::<AnalyticsReport>().fetch_all(pool).await?;
    Ok(rows)
}
