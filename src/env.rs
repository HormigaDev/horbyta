use std::{env, io::ErrorKind, path::PathBuf, time::Duration};

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_secret_client: String,
    pub app_uri: String,
    pub frontend_origin: String,
    pub is_production: bool,
    pub time_to_exp: Duration,

    // Storage
    pub s3_bucket: Option<String>,
    pub s3_endpoint: Option<String>,
    pub s3_region: Option<String>,
    pub s3_access_key: Option<String>,
    pub s3_secret_key: Option<String>,

    pub upload_dir: Option<String>,

    pub allow_registration: bool,
}

impl Config {
    pub fn init() -> Self {
        let environment = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        Self {
            database_url: env::var("DATABASE_URL")
                .expect("CRITICAL: DATABASE_URL is missing from environment variables."),
            jwt_secret: env::var("JWT_SECRET").expect("CRITICAL: JWT_SECRET is missing from environment variables."),
            jwt_secret_client: env::var("JWT_SECRET_CLIENT")
                .expect("CRITICAL: JWT_SECRET_CLIENT is missing from environment variables."),
            app_uri: env::var("APP_URI").unwrap_or_else(|_| "127.0.0.1:8890".to_string()),
            frontend_origin: env::var("FRONTEND_ORIGIN").unwrap_or_else(|_| "http://localhost:9000".to_string()),
            is_production: environment == "production",
            time_to_exp: duration_str::parse(
                env::var("TIME_TO_EXP_SESSION")
                    .expect("CRITICAL: TIME_TO_EXP_SESSION is missing from environment variables."),
            )
            .expect("CRITICAL: Invalid format for environment variable TIME_TO_EXP_SESSION."),

            s3_bucket: optional_env("S3_BUCKET"),
            s3_endpoint: optional_env("S3_ENDPOINT"),
            s3_region: optional_env("S3_REGION"),
            s3_access_key: optional_env("S3_ACCESS_KEY"),
            s3_secret_key: optional_env("S3_SECRET_KEY"),
            upload_dir: optional_env("UPLOAD_DIR"),
            allow_registration: env::var("ALLOW_REGISTRATION")
                .unwrap_or_else(|_| "false".to_string())
                .to_lowercase()
                == "true",
        }
    }
}

fn optional_env(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}

/// Archivo de entorno del servicio (`EnvironmentFile` en deploy/horbyta.service).
pub const SERVICE_ENV_FILE: &str = "/etc/horbyta/horbyta.env";

/// Carga el entorno de las herramientas de línea de comandos (`migrate`, `create-user`).
///
/// systemd solo aplica `EnvironmentFile` al proceso del servicio; ejecutadas a mano, estas
/// herramientas no lo verían. Precedencia: variables exportadas, `.env` del directorio actual
/// y, por último, `HORBYTA_ENV_FILE` o el archivo del servicio si existe. Ninguna fuente
/// sobrescribe una variable ya definida. Devuelve las fuentes leídas, para los mensajes.
pub fn load_cli_env() -> Result<Vec<String>, String> {
    let mut sources = Vec::new();
    match dotenvy::dotenv() {
        Ok(path) => sources.push(path.display().to_string()),
        Err(error) if error.not_found() => {}
        Err(error) => return Err(format!("No se pudo leer .env: {error}")),
    }
    let explicit = env::var("HORBYTA_ENV_FILE").ok().filter(|value| !value.trim().is_empty());
    let path = PathBuf::from(explicit.as_deref().unwrap_or(SERVICE_ENV_FILE));
    match dotenvy::from_path(&path) {
        Ok(()) => sources.push(path.display().to_string()),
        Err(error) if error.not_found() && explicit.is_none() => {}
        Err(dotenvy::Error::Io(error)) if error.kind() == ErrorKind::PermissionDenied => {
            return Err(format!(
                "Sin permiso para leer {}. Ejecute la herramienta como el usuario del servicio (horbyta) o como root.",
                path.display()
            ));
        }
        Err(error) => return Err(format!("No se pudo leer {}: {error}", path.display())),
    }
    Ok(sources)
}

/// Mensaje de error cuando falta una variable, indicando dónde se buscó.
pub fn missing_env(name: &str, sources: &[String]) -> String {
    let searched = if sources.is_empty() {
        format!(".env del directorio actual y {SERVICE_ENV_FILE} (no encontrados)")
    } else {
        sources.join(", ")
    };
    format!("Falta {name}. Se buscó en: variables exportadas, {searched}. Use HORBYTA_ENV_FILE=/ruta/al/archivo para indicar otro.")
}
