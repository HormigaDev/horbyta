use horbyta::migrations;
use sea_orm::{Database, DatabaseConnection};

const USAGE: &str = "Uso:
  migrate                      Aplica todas las migraciones pendientes.
  migrate up [VERSIÓN]         Aplica las pendientes hasta VERSIÓN inclusive (por defecto, todas).
  migrate down [VERSIÓN] --yes Revierte VERSIÓN y todas las posteriores (por defecto, solo la última).
  migrate status               Muestra qué migraciones están aplicadas.

Las migraciones son lineales: aplicar N aplica antes las anteriores pendientes y
revertir N revierte antes las posteriores, porque cada una depende de las previas.
Revertir elimina los datos de las tablas y columnas que la migración creó.";

enum Command {
    Up(i32),
    Down { from: Option<i32>, confirmed: bool },
    Status,
}

fn parse(args: &[String]) -> Result<Command, String> {
    let confirmed = args.iter().any(|a| a == "--yes");
    let rest: Vec<&str> = args.iter().map(String::as_str).filter(|a| *a != "--yes").collect();
    let version = |arg: Option<&&str>| -> Result<Option<i32>, String> {
        arg.map(|v| v.parse::<i32>().map_err(|_| format!("Versión inválida: {v}")))
            .transpose()
    };
    match rest.as_slice() {
        [] => Ok(Command::Up(migrations::latest_version())),
        ["up", tail @ ..] if tail.len() <= 1 => {
            Ok(Command::Up(version(tail.first())?.unwrap_or(migrations::latest_version())))
        }
        ["down", tail @ ..] if tail.len() <= 1 => Ok(Command::Down { from: version(tail.first())?, confirmed }),
        ["status"] => Ok(Command::Status),
        _ => Err(USAGE.to_owned()),
    }
}

async fn run(db: &DatabaseConnection, command: Command) -> Result<String, String> {
    let fail = |e: horbyta::error::AppError| format!("Error de migración: {e:?}");
    let applied = migrations::applied_versions(db).await.map_err(fail)?;
    match command {
        Command::Up(target) => {
            let done = migrations::migrate_to(db, target).await.map_err(fail)?;
            Ok(if done.is_empty() {
                format!("Nada que aplicar hasta la versión {target}.")
            } else {
                format!("Migraciones aplicadas: {done:?}.")
            })
        }
        Command::Down { from, confirmed } => {
            let Some(&last) = applied.last() else {
                return Ok("No hay migraciones aplicadas.".to_owned());
            };
            let from = from.unwrap_or(last);
            if from < 1 {
                return Err("La versión a revertir debe ser 1 o mayor.".to_owned());
            }
            let pending: Vec<i32> = applied.iter().rev().copied().filter(|v| *v >= from).collect();
            if pending.is_empty() {
                return Ok(format!("La migración {from} no está aplicada."));
            }
            if !confirmed {
                return Err(format!(
                    "Se revertirían, en este orden, las migraciones {pending:?} y se perderían los datos que contienen.\n\
                     Repita el comando con --yes para confirmar."
                ));
            }
            let done = migrations::rollback_to(db, from - 1).await.map_err(fail)?;
            Ok(format!(
                "Migraciones revertidas: {done:?}. El backend no arrancará hasta volver a aplicarlas."
            ))
        }
        Command::Status => Ok((1..=migrations::latest_version())
            .map(|v| format!("{v:>3}  {}", if applied.contains(&v) { "aplicada" } else { "pendiente" }))
            .collect::<Vec<_>>()
            .join("\n")),
    }
}

#[tokio::main(worker_threads = 1)]
async fn main() {
    dotenvy::dotenv().ok();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{USAGE}");
        return;
    }
    let result = async {
        let command = parse(&args)?;
        let url = std::env::var("DATABASE_URL").map_err(|_| "Falta DATABASE_URL".to_owned())?;
        let db = Database::connect(url)
            .await
            .map_err(|_| "No se pudo conectar a PostgreSQL".to_owned())?;
        let output = run(&db, command).await;
        db.close()
            .await
            .map_err(|_| "No se pudo cerrar la conexión".to_owned())?;
        output
    }
    .await;
    match result {
        Ok(message) => println!("{message}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
