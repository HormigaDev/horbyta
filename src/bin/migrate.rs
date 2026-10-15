use sea_orm::Database;

#[tokio::main(worker_threads = 1)]
async fn main() {
    dotenvy::dotenv().ok();
    let result = async {
        let url = std::env::var("DATABASE_URL").map_err(|_| "Falta DATABASE_URL".to_owned())?;
        let db = Database::connect(url)
            .await
            .map_err(|_| "No se pudo conectar a PostgreSQL".to_owned())?;
        horbyta::migrations::migrate(&db)
            .await
            .map_err(|e| format!("Error de migración: {e:?}"))?;
        db.close()
            .await
            .map_err(|_| "No se pudo cerrar la conexión".to_owned())?;
        Ok::<_, String>(())
    }
    .await;
    match result {
        Ok(()) => {
            println!("Esquema actualizado; migraciones aplicadas correctamente.")
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
