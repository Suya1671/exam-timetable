use diesel::{Connection, SqliteConnection, connection::SimpleConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use rusqlite::Connection as RusqliteConnection;
use slab::Slab;
use specta_typescript::BigIntExportBehavior;
use std::sync::Mutex;
use std::sync::mpsc;
use tauri::{App, Manager};
use tracing::info;

mod api;
mod renderer;
use crate::api::Api;
use crate::api::ApiImpl;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../../migrations");
const SQLITE_PRAGMAS: &str = r#"
    PRAGMA busy_timeout = 2000;
    PRAGMA journal_mode = WAL;
    PRAGMA synchronous = NORMAL;
    PRAGMA wal_autocheckpoint = 1000;
    PRAGMA wal_checkpoint(TRUNCATE);
"#;

type DbConn = Mutex<SqliteConnection>;
type SqlProxyConn = Mutex<RusqliteConnection>;
type SolveSessions = Mutex<Slab<mpsc::Sender<api::SolveSessionCommand>>>;

pub struct AppState {
    pub db_conn: DbConn,
    pub sql_proxy_conn: SqlProxyConn,
    pub solve_sessions: SolveSessions,
}

pub async fn run() {
    let router = taurpc::Router::new()
        .export_config(taurpc::Typescript::default().bigint(BigIntExportBehavior::Number))
        .merge(ApiImpl.into_handler());

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(router.into_handler())
        .setup(|app| {
            let db_conn = establish_connection(app);
            let sql_proxy_conn = establish_sql_proxy_connection(app);

            app.manage(AppState {
                db_conn: Mutex::new(db_conn),
                sql_proxy_conn: Mutex::new(sql_proxy_conn),
                solve_sessions: Mutex::new(Slab::new()),
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn establish_connection(app: &App) -> SqliteConnection {
    info!("start establish_connection()",);

    let database_path = ensure_database_path(app);

    let database_url = format!("sqlite://{}", database_path.to_string_lossy());

    let mut conn = SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to the database: {}", database_url));

    conn.batch_execute(SQLITE_PRAGMAS)
        .expect("Failed to set SQLite pragmas");

    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run database migrations");

    conn
}

/// AI-generated (GPT-5.2-codex).
fn establish_sql_proxy_connection(app: &App) -> RusqliteConnection {
    let database_path = ensure_database_path(app);

    let connection = RusqliteConnection::open(database_path)
        .expect("Failed to open SQL proxy database connection");

    connection
        .execute_batch(SQLITE_PRAGMAS)
        .expect("Failed to set SQL proxy pragmas");

    connection
}

fn ensure_database_path(app: &App) -> std::path::PathBuf {
    let app_data = app
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");

    if !app_data.exists() {
        std::fs::create_dir_all(&app_data).expect("Failed to create app data directory");
    }

    let database_path = app_data.join("app_database.sqlite");

    if !database_path.exists() {
        std::fs::File::create(&database_path).expect("Failed to create database file");
    }

    database_path
}
