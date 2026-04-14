use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::thread;

use entity::id::{SessionId, TimeslotId};
use rusqlite::types::{Value, ValueRef};
use tauri::{AppHandle, Manager, Runtime, ipc::Channel};
use tauri_plugin_dialog::DialogExt;

/// AI-generated (GPT-5.3-codex).
#[derive(Debug, thiserror::Error, serde::Serialize, specta::Type)]
pub enum SqlError {
    /// AI-generated (GPT-5.3-codex).
    #[error("SQL proxy error: {0}")]
    SqlProxy(String),
    /// AI-generated (GPT-5.3-codex).
    #[error("Lock poison error: {0}")]
    LockPoison(String),
}

/// AI-generated (GPT-5.3-codex).
#[derive(Debug, thiserror::Error, serde::Serialize, specta::Type)]
pub enum InitSolverError {
    /// AI-generated (GPT-5.3-codex).
    #[error("Solver initialization lock error: {0}")]
    LockPoison(String),
    /// AI-generated (GPT-5.3-codex).
    #[error("Solver initialization error: {0}")]
    Solver(String),
    /// AI-generated (GPT-5.3-codex).
    #[error("Solver initialization receive error: {0}")]
    Recv(String),
}

/// AI-generated (GPT-5.3-codex).
#[derive(Debug, thiserror::Error, serde::Serialize, specta::Type)]
pub enum SolveSessionControlError {
    /// AI-generated (GPT-5.3-codex).
    #[error("Lock poison error: {0}")]
    LockPoison(String),
    /// AI-generated (GPT-5.3-codex).
    #[error("Control channel send error: {0}")]
    Send(String),
    /// AI-generated (GPT-5.3-codex).
    #[error("Invalid session id: {0}")]
    InvalidSessionId(usize),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum SqlParam {
    Null,
    Bool(bool),
    Int(i32),
    Float(f64),
    Text(String),
}

impl From<SqlParam> for Value {
    fn from(param: SqlParam) -> Self {
        match param {
            SqlParam::Null => rusqlite::types::Value::Null,
            SqlParam::Bool(boolean) => rusqlite::types::Value::Integer(i64::from(boolean)),
            SqlParam::Int(integer) => rusqlite::types::Value::Integer(i64::from(integer)),
            SqlParam::Float(float) => rusqlite::types::Value::Real(float),
            SqlParam::Text(text) => rusqlite::types::Value::Text(text),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum SqlValue {
    Null,
    Bool(bool),
    Int(i32),
    Float(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl From<ValueRef<'_>> for SqlValue {
    fn from(value: ValueRef) -> Self {
        match value {
            ValueRef::Null => SqlValue::Null,
            ValueRef::Integer(i) => SqlValue::Int(i as i32),
            ValueRef::Real(f) => SqlValue::Float(f),
            ValueRef::Text(t) => SqlValue::Text(String::from_utf8_lossy(t).to_string()),
            ValueRef::Blob(b) => SqlValue::Blob(b.to_vec()),
        }
    }
}

#[derive(Debug, serde::Serialize, specta::Type)]
pub struct SqlQueryResult {
    pub rows: Vec<Vec<SqlValue>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct SqlBatchQuery {
    pub sql: String,
    pub params: Vec<SqlParam>,
    pub method: String,
}

/// AI-generated (GPT-5.3-codex).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct SolveBatch {
    pub solutions: Vec<HashMap<i32, TimeslotId>>,
    pub done: bool,
}

/// AI-generated (GPT-5.3-codex).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct SolveSessionStart {
    pub session_id: usize,
}

pub enum SolveSessionCommand {
    Pause,
    Stop,
}

/// AI-generated (GPT-5.3-codex).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct LockedTimetableSlot {
    pub session_id: SessionId,
    pub timeslot_id: TimeslotId,
}

/// AI-generated (GPT-5.2-codex).
fn query_database(
    connection: &rusqlite::Connection,
    sql: &str,
    params: Vec<SqlParam>,
    method: &str,
) -> Result<SqlQueryResult, SqlError> {
    let sql_params = params.into_iter().map(Value::from);

    let method = method.trim().to_ascii_lowercase();

    if method == "all" || method == "values" || method == "get" {
        let mut statement = connection
            .prepare_cached(sql)
            .map_err(|err| SqlError::SqlProxy(err.to_string()))?;

        let mut rows = statement
            .query(rusqlite::params_from_iter(sql_params))
            .map_err(|err| SqlError::SqlProxy(err.to_string()))?;

        let mut all_rows = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|err| SqlError::SqlProxy(err.to_string()))?
        {
            let mut values = Vec::new();
            for column_index in 0..row.as_ref().column_count() {
                let value = row
                    .get_ref(column_index)
                    .map_err(|err| SqlError::SqlProxy(err.to_string()))?;
                values.push(value.into());
            }
            all_rows.push(values);
        }

        if method == "get" {
            return Ok(SqlQueryResult {
                rows: all_rows.into_iter().take(1).collect(),
            });
        }

        return Ok(SqlQueryResult { rows: all_rows });
    }

    connection
        .execute(sql, rusqlite::params_from_iter(sql_params))
        .map_err(|err| SqlError::SqlProxy(err.to_string()))?;

    Ok(SqlQueryResult { rows: Vec::new() })
}

/// AI-generated (GPT-5.2-codex).
fn query_database_batch(
    connection: &mut rusqlite::Connection,
    queries: Vec<SqlBatchQuery>,
) -> Result<Vec<SqlQueryResult>, SqlError> {
    let tx = connection
        .transaction()
        .map_err(|err| SqlError::SqlProxy(err.to_string()))?;
    let mut results = Vec::with_capacity(queries.len());
    for query in queries {
        results.push(query_database(
            &tx,
            &query.sql,
            query.params,
            &query.method,
        )?);
    }
    tx.commit()
        .map_err(|err| SqlError::SqlProxy(err.to_string()))?;
    Ok(results)
}

#[taurpc::procedures(export_to = "../src/lib/backend.ts")]
pub trait Api {
    async fn sql<R: Runtime>(
        app_handle: AppHandle<R>,
        sql: String,
        params: Vec<SqlParam>,
        method: String,
    ) -> Result<SqlQueryResult, SqlError>;

    async fn sql_batch<R: Runtime>(
        app_handle: AppHandle<R>,
        queries: Vec<SqlBatchQuery>,
    ) -> Result<Vec<SqlQueryResult>, SqlError>;

    async fn start_solve_session<R: Runtime>(
        app_handle: AppHandle<R>,
        on_new_timetable: Channel<NewTimetableUpdate>,
        locked_slots: Vec<LockedTimetableSlot>,
    ) -> Result<SolveSessionStart, InitSolverError>;

    async fn solve_single<R: Runtime>(
        app_handle: AppHandle<R>,
        locked_slots: Vec<LockedTimetableSlot>,
    ) -> Result<HashMap<i32, TimeslotId>, InitSolverError>;

    async fn pause_solve_session<R: Runtime>(
        app_handle: AppHandle<R>,
        session_id: usize,
    ) -> Result<(), SolveSessionControlError>;

    async fn stop_solve_session<R: Runtime>(
        app_handle: AppHandle<R>,
        session_id: usize,
    ) -> Result<(), SolveSessionControlError>;

    async fn generate_timetable_pdf<R: Runtime>(
        app_handle: AppHandle<R>,
        data: crate::renderer::TimetableData,
    ) -> Result<(), String>;
}

#[derive(Clone)]
pub struct ApiImpl;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum NewTimetableUpdate {
    /// Key: session ID
    /// Value: timeslot ID
    ///
    /// (Specta doesn't like non-number/string keys)
    Timetable(HashMap<i32, TimeslotId>),
    Done,
}

#[taurpc::resolvers]
impl Api for ApiImpl {
    async fn sql<R: Runtime>(
        self,
        app: AppHandle<R>,
        sql: String,
        params: Vec<SqlParam>,
        method: String,
    ) -> Result<SqlQueryResult, SqlError> {
        let state = app.state::<crate::AppState>();
        let sql_proxy_conn = state
            .sql_proxy_conn
            .lock()
            .map_err(|err| SqlError::LockPoison(err.to_string()))?;

        query_database(&sql_proxy_conn, &sql, params, &method)
    }

    async fn sql_batch<R: Runtime>(
        self,
        app: AppHandle<R>,
        queries: Vec<SqlBatchQuery>,
    ) -> Result<Vec<SqlQueryResult>, SqlError> {
        let state = app.state::<crate::AppState>();
        let mut sql_proxy_conn = state
            .sql_proxy_conn
            .lock()
            .map_err(|err| SqlError::LockPoison(err.to_string()))?;

        query_database_batch(&mut sql_proxy_conn, queries)
    }

    async fn start_solve_session<R: Runtime>(
        self,
        app: AppHandle<R>,
        on_new_timetable: Channel<NewTimetableUpdate>,
        locked_slots: Vec<LockedTimetableSlot>,
    ) -> Result<SolveSessionStart, InitSolverError> {
        let (command_tx, command_rx) = mpsc::channel::<SolveSessionCommand>();
        let (init_tx, init_rx) = mpsc::channel::<Result<(), InitSolverError>>();

        let app_for_worker = app.clone();
        thread::spawn(move || {
            let mut paused = false;
            let state = app_for_worker.state::<crate::AppState>();
            let mut conn = match state.db_conn.lock() {
                Ok(conn) => conn,
                Err(err) => {
                    let _ = init_tx.send(Err(InitSolverError::LockPoison(err.to_string())));
                    return;
                }
            };

            let fixed_slots = locked_slots
                .iter()
                .map(|slot| (slot.session_id, slot.timeslot_id))
                .collect::<Vec<_>>();

            // Use the streaming solve (multiple solutions iterator)
            let mut solutions =
                match backend::solve_with_locked_assignments(&mut conn, &fixed_slots) {
                    Ok(solutions) => solutions.map(|solution| {
                        solution
                            .into_iter()
                            .map(|(session_id, timeslot_id)| (session_id.0, timeslot_id))
                            .collect::<HashMap<i32, TimeslotId>>()
                    }),
                    Err(err) => {
                        init_tx
                            .send(Err(InitSolverError::Solver(err.to_string())))
                            .expect("Failed to send solver error to main thread");
                        return;
                    }
                };

            drop(conn);

            init_tx
                .send(Ok(()))
                .expect("Failed to send initialization result to main thread");

            loop {
                if paused {
                    match command_rx.recv() {
                        Ok(SolveSessionCommand::Pause) => paused = false,
                        Ok(SolveSessionCommand::Stop) | Err(_) => break,
                    }
                    continue;
                }

                match command_rx.try_recv() {
                    Ok(SolveSessionCommand::Pause) => {
                        paused = true;
                        continue;
                    }
                    Ok(SolveSessionCommand::Stop) => break,
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => break,
                }

                if let Some(solution) = solutions.next() {
                    if on_new_timetable
                        .send(NewTimetableUpdate::Timetable(solution))
                        .is_err()
                    {
                        break;
                    }
                } else {
                    break;
                }
            }

            let _ = on_new_timetable.send(NewTimetableUpdate::Done);
        });

        match init_rx.recv() {
            Ok(Ok(())) => {
                let state = app.state::<crate::AppState>();

                let session_id = state
                    .solve_sessions
                    .lock()
                    .expect("Failed to lock solve sessions")
                    .insert(command_tx);

                Ok(SolveSessionStart { session_id })
            }
            Ok(Err(err)) => Err(err),
            Err(err) => Err(InitSolverError::Recv(err.to_string())),
        }
    }

    async fn solve_single<R: Runtime>(
        self,
        app: AppHandle<R>,
        locked_slots: Vec<LockedTimetableSlot>,
    ) -> Result<HashMap<i32, TimeslotId>, InitSolverError> {
        let state = app.state::<crate::AppState>();
        let mut conn = state
            .db_conn
            .lock()
            .map_err(|err| InitSolverError::LockPoison(err.to_string()))?;

        let fixed_slots = locked_slots
            .iter()
            .map(|slot| (slot.session_id, slot.timeslot_id))
            .collect::<Vec<_>>();

        let solution = backend::solve_one(&mut conn, &fixed_slots)
            .map_err(|err| InitSolverError::Solver(err.to_string()))?;

        Ok(solution
            .into_iter()
            .map(|(session_id, timeslot_id)| (session_id.0, timeslot_id))
            .collect())
    }

    async fn pause_solve_session<R: Runtime>(
        self,
        app: AppHandle<R>,
        session_id: usize,
    ) -> Result<(), SolveSessionControlError> {
        let state = app.state::<crate::AppState>();

        let command_tx = {
            let sessions = state
                .solve_sessions
                .lock()
                .map_err(|err| SolveSessionControlError::LockPoison(err.to_string()))?;

            sessions.get(session_id).cloned()
        };

        if let Some(sender) = command_tx {
            sender
                .send(SolveSessionCommand::Pause)
                .map_err(|err| SolveSessionControlError::Send(err.to_string()))?;
        } else {
            return Err(SolveSessionControlError::InvalidSessionId(session_id));
        }

        Ok(())
    }

    async fn stop_solve_session<R: Runtime>(
        self,
        app: AppHandle<R>,
        session_id: usize,
    ) -> Result<(), SolveSessionControlError> {
        let app_state = app.state::<crate::AppState>();

        let sender = {
            let mut sessions = app_state
                .solve_sessions
                .lock()
                .map_err(|err| SolveSessionControlError::LockPoison(err.to_string()))?;

            if sessions.contains(session_id) {
                Some(sessions.remove(session_id))
            } else {
                None
            }
        };

        if let Some(sender) = sender {
            sender
                .send(SolveSessionCommand::Stop)
                .map_err(|err| SolveSessionControlError::Send(err.to_string()))?;
        } else {
            return Err(SolveSessionControlError::InvalidSessionId(session_id));
        }

        Ok(())
    }

    /// AI-generated (Gemini).
    async fn generate_timetable_pdf<R: Runtime>(
        self,
        app: AppHandle<R>,
        data: crate::renderer::TimetableData,
    ) -> Result<(), String> {
        let pdf_bytes = crate::renderer::render_pdf(&data)?;

        let save_path = app
            .dialog()
            .file()
            .add_filter("PDF document", &["pdf"])
            .set_file_name("timetable.pdf")
            .blocking_save_file();

        if let Some(path) = save_path {
            let path_buf = match path {
                tauri_plugin_dialog::FilePath::Path(p) => p,
                tauri_plugin_dialog::FilePath::Url(u) => u
                    .to_file_path()
                    .map_err(|_| "Failed to convert URL to file path".to_string())?,
            };
            std::fs::write(path_buf, pdf_bytes).map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}
