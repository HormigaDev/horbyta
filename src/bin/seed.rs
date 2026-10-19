//! CLI explícito para reconstruir datos de prueba locales.
use horbyta::development_seed;
use object_store::local::LocalFileSystem;
use sea_orm::{ConnectOptions, Database};
use std::{env, path::Path, process::ExitCode, time::Duration};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!(
            "Datos ficticios solo para APP_ENV=development y horbyta_development.\n\nUso: cargo run --locked --bin seed [-- --reset]\n\nSin --reset requiere una base sin usuarios y aplica la migración inicial.\n--reset reemplaza el esquema public y los datos de desarrollo en una transacción.\nAdmin: admin@horbyta.test / Admin@123\nPortal: cliente1@horbyta.test / Cliente@123\nNo se ejecuta automáticamente al instalar ni al arrancar. Requiere almacenamiento local."
        );
        return Ok(());
    }
    if !(args.is_empty() || args == ["--reset"]) {
        return Err("Argumentos desconocidos; consulte --help.".into());
    }
    if let Err(error) = dotenvy::dotenv() {
        if !error.not_found() {
            return Err("No se pudo leer .env.".into());
        }
    }
    let mode = env::var("APP_ENV").unwrap_or_default();
    development_seed::require_development(&mode)?;
    if env::var("S3_BUCKET").is_ok_and(|value| !value.trim().is_empty()) {
        return Err("El seed usa archivos locales; quite S3_BUCKET de la configuración de desarrollo.".into());
    }
    let url = env::var("DATABASE_URL").map_err(|_| "Falta DATABASE_URL")?;
    let mut options = ConnectOptions::new(url);
    options
        .max_connections(1)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(10))
        .acquire_timeout(Duration::from_secs(10))
        .sqlx_logging(false);
    let db = Database::connect(options)
        .await
        .map_err(|_| "No se pudo conectar a PostgreSQL")?;
    let directory = env::var("UPLOAD_DIR")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "./data/storage".into());
    std::fs::create_dir_all(&directory)?;
    let storage = LocalFileSystem::new_with_prefix(Path::new(&directory).canonicalize()?)?;
    let result = development_seed::seed(&db, &storage, &mode, !args.is_empty()).await;
    db.close().await?;
    let report = result?;
    println!(
        "Seed de desarrollo completado.\nAdmin: {} / Admin@123\nPortal: {} / Cliente@123",
        development_seed::ADMIN_EMAIL,
        report["client_email"].as_str().unwrap_or_default()
    );
    println!(
        "ID del administrador para el portal: {}",
        report["admin_id"].as_str().unwrap_or_default()
    );
    println!("{}", serde_json::to_string_pretty(&report["tables"])?);
    Ok(())
}

#[tokio::main(worker_threads = 1)]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("seed: {error}");
            ExitCode::FAILURE
        }
    }
}
