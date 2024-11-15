use log::info;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use serde::Deserialize;
use sqlx::migrate::Migrator;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::SqlitePool;
use std::fs::{create_dir_all, File};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use tauri::async_runtime::Mutex;
use tauri::path::BaseDirectory;
use tauri::plugin::TauriPlugin;
use tauri::{plugin, AppHandle, Manager, Runtime};

pub struct SqliteConnection(pub Mutex<Pool<SqliteConnectionManager>>);

#[derive(Default, Deserialize)]
pub struct PluginConfig {
    // Nothing yet (will be configurable in tauri.conf.json
}

#[derive(Default)]
pub struct Builder {
    // Nothing Yet
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build<R: Runtime>(&self) -> TauriPlugin<R, Option<PluginConfig>> {
        plugin::Builder::<R, Option<PluginConfig>>::new("yaak_models")
            .setup(|app, _api| {
                let app_path = app.path().app_data_dir().unwrap();
                create_dir_all(app_path.clone()).expect("Problem creating App directory!");

                let db_file_path = app_path.join("db.sqlite");

                {
                    let db_file_path = db_file_path.clone();
                    tauri::async_runtime::block_on(async move {
                        must_migrate_db(app.app_handle(), &db_file_path).await;
                    });
                };

                let manager = SqliteConnectionManager::file(db_file_path);
                let pool = Pool::builder()
                    .max_size(100) // Up from 10 (just in case)
                    .connection_timeout(Duration::from_secs(10)) // Down from 30
                    .build(manager)
                    .unwrap();

                app.manage(SqliteConnection(Mutex::new(pool)));

                Ok(())
            })
            .build()
    }
}

async fn must_migrate_db<R: Runtime>(app_handle: &AppHandle<R>, path: &PathBuf) {
    let app_data_dir = app_handle.path().app_data_dir().unwrap();
    let sqlite_file_path = app_data_dir.join("db.sqlite");

    info!("Creating database file at {:?}", sqlite_file_path);
    File::options()
        .write(true)
        .create(true)
        .open(&sqlite_file_path)
        .expect("Problem creating database file!");

    let p_string = sqlite_file_path.to_string_lossy().replace(' ', "%20");
    let url = format!("sqlite://{}?mode=rwc", p_string);

    info!("Connecting to database at {}", url);
    let opts = SqliteConnectOptions::from_str(path.to_string_lossy().to_string().as_str()).unwrap();
    let pool = SqlitePool::connect_with(opts)
        .await
        .expect("Failed to connect to database");
    let p = app_handle
        .path()
        .resolve("migrations", BaseDirectory::Resource)
        .expect("failed to resolve resource");

    info!("Running database migrations from: {}", p.to_string_lossy());
    let mut m = Migrator::new(p).await.expect("Failed to load migrations");
    m.set_ignore_missing(true); // So we can roll back versions and not crash
    m.run(&pool).await.expect("Failed to run migrations");

    info!("Database migrations complete");
}
