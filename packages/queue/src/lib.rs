use parking_lot::Mutex;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::warn;

pub type DbPool = Arc<Mutex<Connection>>;

pub const QUEUE_SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS queue_tasks (
    id TEXT PRIMARY KEY,
    task_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending','running','completed','failed','cancelled')),
    payload TEXT NOT NULL DEFAULT '{}',
    result TEXT,
    error TEXT,
    progress TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    started_at TEXT,
    completed_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_queue_tasks_status ON queue_tasks(status);
CREATE INDEX IF NOT EXISTS idx_queue_tasks_type_status ON queue_tasks(task_type, status);
";

// --- Task Status ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QueueTaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl QueueTaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "running" => Some(Self::Running),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

// --- QueueTask ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueTask {
    pub id: String,
    pub task_type: String,
    pub status: QueueTaskStatus,
    pub payload: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub progress: Option<serde_json::Value>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

// --- QueueEvent ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum QueueEvent {
    TaskCreated { task: QueueTask },
    TaskStarted { task: QueueTask },
    TaskProgress { id: String, progress: serde_json::Value },
    TaskCompleted { task: QueueTask },
    TaskFailed { task: QueueTask },
    TaskCancelled { id: String },
    TaskRemoved { id: String },
}

// --- QueueManager ---

pub struct QueueManager {
    db: DbPool,
    tx: broadcast::Sender<QueueEvent>,
}

impl QueueManager {
    pub fn new(db: DbPool) -> Self {
        let (tx, _) = broadcast::channel(256);
        Self { db, tx }
    }

    /// Create the queue_tasks table if it doesn't exist.
    pub fn create_table(&self) {
        let conn = self.db.lock();
        conn.execute_batch(QUEUE_SCHEMA_SQL)
            .expect("Failed to create queue_tasks table");
    }

    /// Insert a new pending task and broadcast TaskCreated.
    pub fn enqueue(&self, task_type: &str, payload: serde_json::Value) -> QueueTask {
        let id = uuid::Uuid::new_v4().to_string();
        let payload_str = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());

        let conn = self.db.lock();
        conn.execute(
            "INSERT INTO queue_tasks (id, task_type, status, payload) VALUES (?1, ?2, 'pending', ?3)",
            rusqlite::params![id, task_type, payload_str],
        )
        .expect("Failed to insert queue task");

        let task = self.get_by_id_locked(&conn, &id).expect("Just inserted task not found");
        let _ = self.tx.send(QueueEvent::TaskCreated { task: task.clone() });
        task
    }

    /// Atomically claim the next pending task matching task_type prefix.
    /// Sets status to running and returns the task, or None if no pending tasks.
    pub fn claim_next(&self, task_type_prefix: &str) -> Option<QueueTask> {
        let conn = self.db.lock();
        let pattern = format!("{}%", task_type_prefix);
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // Atomic claim: UPDATE with subquery
        let rows = conn
            .execute(
                "UPDATE queue_tasks SET status = 'running', started_at = ?1
                 WHERE id = (
                     SELECT id FROM queue_tasks
                     WHERE status = 'pending' AND task_type LIKE ?2
                     ORDER BY created_at ASC
                     LIMIT 1
                 )",
                rusqlite::params![now, pattern],
            )
            .unwrap_or(0);

        if rows == 0 {
            return None;
        }

        // Find the task we just claimed (it's running with started_at = now)
        let task = conn
            .prepare(
                "SELECT id, task_type, status, payload, result, error, progress, created_at, started_at, completed_at
                 FROM queue_tasks WHERE status = 'running' AND started_at = ?1 AND task_type LIKE ?2
                 ORDER BY created_at ASC LIMIT 1",
            )
            .ok()
            .and_then(|mut stmt| {
                stmt.query_row(rusqlite::params![now, pattern], |row| row_to_task(row)).ok()
            });

        if let Some(ref task) = task {
            let _ = self.tx.send(QueueEvent::TaskStarted { task: task.clone() });
        }
        task
    }

    /// Mark a task as completed with a result.
    pub fn complete(&self, id: &str, result: serde_json::Value) {
        let result_str = serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string());
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let conn = self.db.lock();
        conn.execute(
            "UPDATE queue_tasks SET status = 'completed', result = ?1, completed_at = ?2 WHERE id = ?3",
            rusqlite::params![result_str, now, id],
        )
        .ok();

        if let Some(task) = self.get_by_id_locked(&conn, id) {
            let _ = self.tx.send(QueueEvent::TaskCompleted { task });
        }
    }

    /// Mark a task as failed with an error message.
    pub fn fail(&self, id: &str, error: &str) {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let conn = self.db.lock();
        conn.execute(
            "UPDATE queue_tasks SET status = 'failed', error = ?1, completed_at = ?2 WHERE id = ?3",
            rusqlite::params![error, now, id],
        )
        .ok();

        if let Some(task) = self.get_by_id_locked(&conn, id) {
            let _ = self.tx.send(QueueEvent::TaskFailed { task });
        }
    }

    /// Update the progress field of a running task.
    pub fn update_progress(&self, id: &str, progress: serde_json::Value) {
        let progress_str = serde_json::to_string(&progress).unwrap_or_else(|_| "{}".to_string());

        let conn = self.db.lock();
        conn.execute(
            "UPDATE queue_tasks SET progress = ?1 WHERE id = ?2",
            rusqlite::params![progress_str, id],
        )
        .ok();

        let _ = self.tx.send(QueueEvent::TaskProgress {
            id: id.to_string(),
            progress,
        });
    }

    /// Cancel a pending or running task.
    pub fn cancel(&self, id: &str) -> bool {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let conn = self.db.lock();
        let rows = conn
            .execute(
                "UPDATE queue_tasks SET status = 'cancelled', completed_at = ?1
                 WHERE id = ?2 AND status IN ('pending', 'running')",
                rusqlite::params![now, id],
            )
            .unwrap_or(0);

        if rows > 0 {
            let _ = self.tx.send(QueueEvent::TaskCancelled { id: id.to_string() });
            true
        } else {
            false
        }
    }

    /// Remove a task from the database.
    pub fn remove(&self, id: &str) -> bool {
        let conn = self.db.lock();
        let rows = conn
            .execute("DELETE FROM queue_tasks WHERE id = ?1", rusqlite::params![id])
            .unwrap_or(0);

        if rows > 0 {
            let _ = self.tx.send(QueueEvent::TaskRemoved { id: id.to_string() });
            true
        } else {
            false
        }
    }

    /// Get a task by ID.
    pub fn get(&self, id: &str) -> Option<QueueTask> {
        let conn = self.db.lock();
        self.get_by_id_locked(&conn, id)
    }

    /// List tasks with optional status and task_type filters.
    pub fn list(&self, status: Option<&str>, task_type: Option<&str>) -> Vec<QueueTask> {
        let conn = self.db.lock();
        let mut sql = String::from(
            "SELECT id, task_type, status, payload, result, error, progress, created_at, started_at, completed_at
             FROM queue_tasks WHERE 1=1",
        );
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(s) = status {
            sql.push_str(&format!(" AND status = ?{}", params.len() + 1));
            params.push(Box::new(s.to_string()));
        }
        if let Some(t) = task_type {
            sql.push_str(&format!(" AND task_type = ?{}", params.len() + 1));
            params.push(Box::new(t.to_string()));
        }

        sql.push_str(" ORDER BY created_at DESC");

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let mut stmt = match conn.prepare(&sql) {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to prepare list query: {}", e);
                return Vec::new();
            }
        };

        stmt.query_map(param_refs.as_slice(), |row| row_to_task(row))
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
    }

    /// Delete all completed, failed, and cancelled tasks.
    pub fn clear_completed(&self) -> usize {
        let conn = self.db.lock();
        conn.execute(
            "DELETE FROM queue_tasks WHERE status IN ('completed', 'failed', 'cancelled')",
            [],
        )
        .unwrap_or(0)
    }

    /// Subscribe to queue events.
    pub fn subscribe(&self) -> broadcast::Receiver<QueueEvent> {
        self.tx.subscribe()
    }

    fn get_by_id_locked(&self, conn: &Connection, id: &str) -> Option<QueueTask> {
        conn.prepare(
            "SELECT id, task_type, status, payload, result, error, progress, created_at, started_at, completed_at
             FROM queue_tasks WHERE id = ?1",
        )
        .ok()
        .and_then(|mut stmt| stmt.query_row(rusqlite::params![id], |row| row_to_task(row)).ok())
    }
}

fn row_to_task(row: &rusqlite::Row) -> rusqlite::Result<QueueTask> {
    let status_str: String = row.get(2)?;
    let payload_str: String = row.get(3)?;
    let result_str: Option<String> = row.get(4)?;
    let progress_str: Option<String> = row.get(6)?;

    Ok(QueueTask {
        id: row.get(0)?,
        task_type: row.get(1)?,
        status: QueueTaskStatus::from_str(&status_str).unwrap_or(QueueTaskStatus::Pending),
        payload: serde_json::from_str(&payload_str).unwrap_or(serde_json::Value::Object(Default::default())),
        result: result_str.and_then(|s| serde_json::from_str(&s).ok()),
        error: row.get(5)?,
        progress: progress_str.and_then(|s| serde_json::from_str(&s).ok()),
        created_at: row.get(7)?,
        started_at: row.get(8)?,
        completed_at: row.get(9)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> QueueManager {
        let conn = Connection::open_in_memory().unwrap();
        let db: DbPool = Arc::new(Mutex::new(conn));
        let manager = QueueManager::new(db);
        manager.create_table();
        manager
    }

    #[test]
    fn test_enqueue_and_get() {
        let mgr = setup();
        let task = mgr.enqueue("test:hello", serde_json::json!({ "foo": "bar" }));
        assert_eq!(task.task_type, "test:hello");
        assert_eq!(task.status, QueueTaskStatus::Pending);
        assert_eq!(task.payload["foo"], "bar");

        let fetched = mgr.get(&task.id).unwrap();
        assert_eq!(fetched.id, task.id);
    }

    #[test]
    fn test_claim_next() {
        let mgr = setup();
        mgr.enqueue("job:analyze", serde_json::json!({}));
        mgr.enqueue("other:task", serde_json::json!({}));

        let claimed = mgr.claim_next("job:").unwrap();
        assert_eq!(claimed.task_type, "job:analyze");
        assert_eq!(claimed.status, QueueTaskStatus::Running);
        assert!(claimed.started_at.is_some());

        // No more matching tasks to claim
        assert!(mgr.claim_next("job:").is_none());
    }

    #[test]
    fn test_complete() {
        let mgr = setup();
        let task = mgr.enqueue("job:analyze", serde_json::json!({}));
        mgr.claim_next("job:").unwrap();

        mgr.complete(&task.id, serde_json::json!({ "relevance": 85 }));

        let completed = mgr.get(&task.id).unwrap();
        assert_eq!(completed.status, QueueTaskStatus::Completed);
        assert_eq!(completed.result.unwrap()["relevance"], 85);
        assert!(completed.completed_at.is_some());
    }

    #[test]
    fn test_fail() {
        let mgr = setup();
        let task = mgr.enqueue("job:analyze", serde_json::json!({}));
        mgr.claim_next("job:").unwrap();

        mgr.fail(&task.id, "No model loaded");

        let failed = mgr.get(&task.id).unwrap();
        assert_eq!(failed.status, QueueTaskStatus::Failed);
        assert_eq!(failed.error.unwrap(), "No model loaded");
    }

    #[test]
    fn test_cancel() {
        let mgr = setup();
        let task = mgr.enqueue("job:analyze", serde_json::json!({}));

        assert!(mgr.cancel(&task.id));

        let cancelled = mgr.get(&task.id).unwrap();
        assert_eq!(cancelled.status, QueueTaskStatus::Cancelled);

        // Can't cancel again
        assert!(!mgr.cancel(&task.id));
    }

    #[test]
    fn test_remove() {
        let mgr = setup();
        let task = mgr.enqueue("test:remove", serde_json::json!({}));
        assert!(mgr.remove(&task.id));
        assert!(mgr.get(&task.id).is_none());
    }

    #[test]
    fn test_list_with_filters() {
        let mgr = setup();
        mgr.enqueue("job:a", serde_json::json!({}));
        mgr.enqueue("job:b", serde_json::json!({}));
        mgr.enqueue("other:c", serde_json::json!({}));

        let all = mgr.list(None, None);
        assert_eq!(all.len(), 3);

        let pending = mgr.list(Some("pending"), None);
        assert_eq!(pending.len(), 3);

        let job_a = mgr.list(None, Some("job:a"));
        assert_eq!(job_a.len(), 1);
    }

    #[test]
    fn test_clear_completed() {
        let mgr = setup();
        let t1 = mgr.enqueue("a", serde_json::json!({}));
        let t2 = mgr.enqueue("b", serde_json::json!({}));
        mgr.enqueue("c", serde_json::json!({}));

        mgr.claim_next("a").unwrap();
        mgr.complete(&t1.id, serde_json::json!({}));
        mgr.cancel(&t2.id);

        let cleared = mgr.clear_completed();
        assert_eq!(cleared, 2);

        let remaining = mgr.list(None, None);
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].status, QueueTaskStatus::Pending);
    }

    #[test]
    fn test_update_progress() {
        let mgr = setup();
        let task = mgr.enqueue("job:analyze", serde_json::json!({}));
        mgr.claim_next("job:").unwrap();

        mgr.update_progress(&task.id, serde_json::json!({ "tokens": 42 }));

        let updated = mgr.get(&task.id).unwrap();
        assert_eq!(updated.progress.unwrap()["tokens"], 42);
    }

    #[test]
    fn test_claim_respects_order() {
        let mgr = setup();
        let first = mgr.enqueue("job:a", serde_json::json!({ "order": 1 }));
        mgr.enqueue("job:b", serde_json::json!({ "order": 2 }));

        let claimed = mgr.claim_next("job:").unwrap();
        assert_eq!(claimed.id, first.id);
    }
}
