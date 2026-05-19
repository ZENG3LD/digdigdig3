use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::task::JoinHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JobId(pub u64);

static NEXT_JOB_ID: AtomicU64 = AtomicU64::new(1);

impl JobId {
    pub fn next() -> Self {
        JobId(NEXT_JOB_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct JobResult<T> {
    pub job_id: JobId,
    pub label: String,
    pub outcome: JobOutcome<T>,
}

pub enum JobOutcome<T> {
    Ok(T),
    TimedOut,
    Failed(String),
}

pub async fn run_jobs<F, Fut, T>(
    labels: Vec<String>,
    timeout: Duration,
    factory: F,
) -> Vec<JobResult<T>>
where
    F: Fn(JobId, String) -> Fut + Send + Sync + 'static + Clone,
    Fut: std::future::Future<Output = Result<T, String>> + Send + 'static,
    T: Send + 'static,
{
    let handles: Vec<JoinHandle<JobResult<T>>> = labels
        .into_iter()
        .map(|label| {
            let factory = factory.clone();
            tokio::spawn(async move {
                let job_id = JobId::next();
                let outcome =
                    match tokio::time::timeout(timeout, factory(job_id, label.clone())).await {
                        Ok(Ok(v)) => JobOutcome::Ok(v),
                        Ok(Err(e)) => JobOutcome::Failed(e),
                        Err(_) => JobOutcome::TimedOut,
                    };
                JobResult { job_id, label, outcome }
            })
        })
        .collect();

    futures_util::future::join_all(handles)
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .collect()
}
