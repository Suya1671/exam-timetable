use rusqlite::types::ValueRef;
use tauri::{AppHandle, Manager, Runtime};

#[derive(Debug, thiserror::Error, serde::Serialize, specta::Type)]
pub enum ApiError {
    /// AI-generated (GPT-5.2-codex).
    #[error("SQL proxy error: {0}")]
    SqlProxyError(String),
}

/// AI-generated (GPT-5.2-codex).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum SqlParam {
    /// AI-generated (GPT-5.2-codex).
    Null,
    /// AI-generated (GPT-5.2-codex).
    Bool(bool),
    /// AI-generated (GPT-5.2-codex).
    Int(i32),
    /// AI-generated (GPT-5.2-codex).
    Float(f64),
    /// AI-generated (GPT-5.2-codex).
    Text(String),
}

/// AI-generated (GPT-5.2-codex).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub enum SqlValue {
    /// AI-generated (GPT-5.2-codex).
    Null,
    /// AI-generated (GPT-5.2-codex).
    Bool(bool),
    /// AI-generated (GPT-5.2-codex).
    Int(i32),
    /// AI-generated (GPT-5.2-codex).
    Float(f64),
    /// AI-generated (GPT-5.2-codex).
    Text(String),
    /// AI-generated (GPT-5.2-codex).
    Blob(Vec<u8>),
}

/// AI-generated (GPT-5.2-codex).
#[derive(Debug, serde::Serialize, specta::Type)]
pub struct SqlQueryResult {
    pub rows: Vec<Vec<SqlValue>>,
}

/// AI-generated (GPT-5.2-codex).
fn query_database(
    connection: &mut rusqlite::Connection,
    sql: &str,
    params: Vec<SqlParam>,
    method: &str,
) -> Result<SqlQueryResult, ApiError> {
    let sql_params = params
        .into_iter()
        .map(sql_param_to_rusqlite)
        .collect::<Vec<_>>();

    let method = method.trim().to_ascii_lowercase();

    if method == "all" || method == "values" || method == "get" {
        let mut statement = connection
            .prepare(sql)
            .map_err(|err| ApiError::SqlProxyError(err.to_string()))?;

        let mut rows = statement
            .query(rusqlite::params_from_iter(sql_params.iter()))
            .map_err(|err| ApiError::SqlProxyError(err.to_string()))?;

        let mut all_rows = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|err| ApiError::SqlProxyError(err.to_string()))?
        {
            let mut values = Vec::new();
            for column_index in 0..row.as_ref().column_count() {
                let value = row
                    .get_ref(column_index)
                    .map_err(|err| ApiError::SqlProxyError(err.to_string()))?;
                values.push(rusqlite_value_to_sql_value(value));
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
        .execute(sql, rusqlite::params_from_iter(sql_params.iter()))
        .map_err(|err| ApiError::SqlProxyError(err.to_string()))?;

    Ok(SqlQueryResult { rows: Vec::new() })
}

/// AI-generated (GPT-5.2-codex).
fn sql_param_to_rusqlite(param: SqlParam) -> rusqlite::types::Value {
    match param {
        SqlParam::Null => rusqlite::types::Value::Null,
        SqlParam::Bool(boolean) => rusqlite::types::Value::Integer(i64::from(boolean)),
        SqlParam::Int(integer) => rusqlite::types::Value::Integer(i64::from(integer)),
        SqlParam::Float(float) => rusqlite::types::Value::Real(float),
        SqlParam::Text(text) => rusqlite::types::Value::Text(text),
    }
}

/// AI-generated (GPT-5.2-codex).
fn rusqlite_value_to_sql_value(value: ValueRef<'_>) -> SqlValue {
    match value {
        ValueRef::Null => SqlValue::Null,
        ValueRef::Integer(integer) => {
            if let Ok(value) = i32::try_from(integer) {
                SqlValue::Int(value)
            } else {
                SqlValue::Float(integer as f64)
            }
        }
        ValueRef::Real(float) => SqlValue::Float(float),
        ValueRef::Text(text) => SqlValue::Text(String::from_utf8_lossy(text).to_string()),
        ValueRef::Blob(blob) => SqlValue::Blob(blob.to_vec()),
    }
}

#[taurpc::procedures(export_to = "../src/lib/backend.ts")]
pub trait Api {
    async fn sql<R: Runtime>(
        app_handle: AppHandle<R>,
        sql: String,
        params: Vec<SqlParam>,
        method: String,
    ) -> Result<SqlQueryResult, ApiError>;
}

#[derive(Clone)]
pub struct ApiImpl;

#[taurpc::resolvers]
impl Api for ApiImpl {
    async fn sql<R: Runtime>(
        self,
        app: AppHandle<R>,
        sql: String,
        params: Vec<SqlParam>,
        method: String,
    ) -> Result<SqlQueryResult, ApiError> {
        let sql_proxy_db = app.state::<crate::SqlProxyConn>();
        let mut sql_proxy_conn = sql_proxy_db
            .lock()
            .map_err(|err| ApiError::SqlProxyError(err.to_string()))?;

        query_database(&mut sql_proxy_conn, &sql, params, &method)
    }
}
