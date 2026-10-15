use sea_orm::{ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbBackend, Statement, TransactionTrait};
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::AppError;

// La primera migración apunta al esquema base, en su ubicación original.
pub const INITIAL_MIGRATION: &str = include_str!("../database/schema.sql");
pub const UUID_COMPATIBILITY: &str = include_str!("../database/uuidv7.sql");
// Las dos ampliaciones anteriores forman una única migración adicional.
pub const SCOPE_MIGRATION: &str = include_str!("../database/migrations/002_scope_control.sql");
pub const CHANGE_CONVERSATION_MIGRATION: &str = include_str!("../database/migrations/004_change_conversation.sql");
pub const CONVERSATION_MIGRATION: &str = include_str!("../database/migrations/003_scope_conversation.sql");
pub const PROJECT_CONVERSATION_MIGRATION: &str = include_str!("../database/migrations/005_project_conversation.sql");
pub const SCOPE_AMENDMENTS_MIGRATION: &str = include_str!("../database/migrations/006_scope_amendments.sql");

pub const SCOPE_DELIVERY_SIGNALS_MIGRATION: &str =
    include_str!("../database/migrations/007_scope_delivery_signals.sql");

const MIGRATIONS: &[(i32, &[&str])] = &[
    (1, &[UUID_COMPATIBILITY, INITIAL_MIGRATION]),
    (2, &[SCOPE_MIGRATION]),
    (3, &[CONVERSATION_MIGRATION]),
    (4, &[CHANGE_CONVERSATION_MIGRATION]),
    (5, &[PROJECT_CONVERSATION_MIGRATION]),
    (6, &[SCOPE_AMENDMENTS_MIGRATION]),
    (7, &[SCOPE_DELIVERY_SIGNALS_MIGRATION]),
];

pub fn migration_checksums() -> Vec<(i32, String)> {
    MIGRATIONS
        .iter()
        .map(|(version, scripts)| {
            let mut digest = Sha256::new();
            for script in *scripts {
                digest.update((script.len() as u64).to_be_bytes());
                digest.update(script.as_bytes());
            }
            (*version, hex::encode(digest.finalize()))
        })
        .collect()
}

// schema.sql conserva su transacción para uso independiente; el ejecutor controla la suya.
fn transaction_body(sql: &str) -> &str {
    let sql = sql.trim();
    if let Some(body) = sql.strip_prefix("begin;") {
        body.trim_end()
            .strip_suffix("commit;")
            .expect("El esquema base debe cerrar su transacción")
            .trim()
    } else {
        sql
    }
}

async fn execute_scripts(tx: &DatabaseTransaction, scripts: &[&str]) -> Result<(), AppError> {
    for script in scripts {
        tx.execute_unprepared(transaction_body(script)).await?;
    }
    Ok(())
}

pub async fn migrate(db: &DatabaseConnection) -> Result<(), AppError> {
    let tx = db.begin().await?;
    apply(&tx).await?;
    tx.commit().await?;
    Ok(())
}

pub(crate) async fn apply(tx: &DatabaseTransaction) -> Result<(), AppError> {
    tx.execute_unprepared("SELECT pg_advisory_xact_lock(719327401)").await?;
    let existing = tx.query_one(Statement::from_string(DbBackend::Postgres,
        "SELECT to_regclass(format('%I.horbyta_migrations',current_schema())) IS NOT NULL AS tracked, to_regclass(format('%I.users',current_schema())) IS NOT NULL AS populated".to_owned())).await?.ok_or(AppError::InternalError)?;
    let adopt_base = !existing.try_get::<bool>("", "tracked")? && existing.try_get::<bool>("", "populated")?;
    if adopt_base {
        verify_existing_base(tx).await?;
    }
    tx.execute_unprepared("CREATE TABLE IF NOT EXISTS horbyta_migrations (version integer PRIMARY KEY, checksum text NOT NULL, applied_at timestamptz NOT NULL DEFAULT now())").await?;
    let supported = MIGRATIONS
        .iter()
        .map(|(version, _)| version.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let unknown = tx
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            format!("SELECT EXISTS(SELECT 1 FROM horbyta_migrations WHERE version NOT IN ({supported})) AS present"),
        ))
        .await?
        .ok_or(AppError::InternalError)?;
    if unknown.try_get::<bool>("", "present")? {
        return Err(AppError::Conflict(
            "El historial pertenece a otra versión del esquema; no se altera automáticamente.".into(),
        ));
    }
    for ((version, scripts), (_, checksum)) in MIGRATIONS.iter().zip(migration_checksums()) {
        let existing = tx
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT checksum FROM horbyta_migrations WHERE version=$1",
                [(*version).into()],
            ))
            .await?;
        if let Some(row) = existing {
            if row.try_get::<String>("", "checksum")? != checksum {
                return Err(AppError::Conflict("La migración aplicada tiene otro checksum. No se reetiquetan historiales ni se modifican migraciones publicadas; la consolidación experimental de desarrollo requiere --reset.".into()));
            }
        } else {
            if !(*version == 1 && adopt_base) {
                execute_scripts(tx, scripts).await?;
            }
            tx.execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "INSERT INTO horbyta_migrations(version,checksum) VALUES($1,$2)",
                [(*version).into(), checksum.into()],
            ))
            .await?;
        }
    }
    Ok(())
}

// Compara estructura, restricciones, índices, vistas, funciones y disparadores con
// un esquema temporal construido desde el SQL base. No interpreta "hay users" como
// prueba suficiente de compatibilidad ni borra los datos para hacerlos encajar.
async fn verify_existing_base(tx: &DatabaseTransaction) -> Result<(), AppError> {
    let row = tx
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            "SELECT current_schema() AS schema,current_setting('search_path') AS path".to_owned(),
        ))
        .await?
        .ok_or(AppError::InternalError)?;
    let actual_schema: String = row.try_get("", "schema")?;
    let original_path: String = row.try_get("", "path")?;
    let expected_schema = format!("horbyta_base_{}", Uuid::now_v7().simple());
    tx.execute_unprepared(&format!("CREATE SCHEMA {expected_schema}"))
        .await?;
    let escaped = actual_schema.replace('"', "\"\"");
    tx.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT set_config('search_path',$1,true)",
        [format!("{expected_schema},\"{escaped}\",pg_catalog").into()],
    ))
    .await?;
    tx.execute_unprepared(transaction_body(INITIAL_MIGRATION)).await?;
    let expected = base_catalog(tx, &expected_schema).await?;
    let actual = base_catalog(tx, &actual_schema).await?;
    let expected = normalize_catalog(expected, &[&expected_schema, &actual_schema]);
    let actual = normalize_catalog(actual, &[&expected_schema, &actual_schema]);
    for (name, definition) in expected.as_object().ok_or(AppError::InternalError)? {
        if actual.get(name) != Some(definition) {
            return Err(AppError::Conflict(format!(
                "El esquema existente no coincide con el base en {name}. No se aplicaron cambios; revise la estructura antes de migrar."
            )));
        }
    }
    tx.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "SELECT set_config('search_path',$1,true)",
        [original_path.into()],
    ))
    .await?;
    tx.execute_unprepared(&format!("DROP SCHEMA {expected_schema} CASCADE"))
        .await?;
    Ok(())
}

fn normalize_catalog(value: Value, schemas: &[&str]) -> Value {
    match value {
        Value::String(mut text) => {
            for schema in schemas {
                text = text
                    .replace(&format!("\"{schema}\"."), "")
                    .replace(&format!("{schema}."), "");
            }
            Value::String(text)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(|v| normalize_catalog(v, schemas)).collect()),
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(k, v)| (k, normalize_catalog(v, schemas)))
                .collect(),
        ),
        other => other,
    }
}

async fn base_catalog(tx: &DatabaseTransaction, schema: &str) -> Result<Value, AppError> {
    let row=tx.query_one(Statement::from_sql_and_values(DbBackend::Postgres,r#"
      SELECT coalesce(jsonb_object_agg(key,definition),'{}') AS data FROM (
        SELECT 'table:'||c.relname AS key,
          (SELECT jsonb_agg(jsonb_build_object('name',a.attname,'type',format_type(a.atttypid,a.atttypmod),'not_null',a.attnotnull,'default',pg_get_expr(d.adbin,d.adrelid)) ORDER BY a.attnum)
           FROM pg_attribute a LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum WHERE a.attrelid=c.oid AND a.attnum>0 AND NOT a.attisdropped) AS definition
          FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname=$1 AND c.relkind='r'
        UNION ALL SELECT 'constraint:'||c.relname||':'||k.conname,to_jsonb(pg_get_constraintdef(k.oid)) FROM pg_constraint k JOIN pg_class c ON c.oid=k.conrelid JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname=$1
        UNION ALL SELECT 'index:'||c.relname,to_jsonb(pg_get_indexdef(c.oid)) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname=$1 AND c.relkind='i'
        UNION ALL SELECT 'view:'||c.relname,to_jsonb(pg_get_viewdef(c.oid)) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname=$1 AND c.relkind='v'
        UNION ALL SELECT 'trigger:'||c.relname||':'||t.tgname,to_jsonb(pg_get_triggerdef(t.oid)) FROM pg_trigger t JOIN pg_class c ON c.oid=t.tgrelid JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname=$1 AND NOT t.tgisinternal
        UNION ALL SELECT 'function:'||p.proname||':'||pg_get_function_identity_arguments(p.oid),to_jsonb(pg_get_functiondef(p.oid)) FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace WHERE n.nspname=$1 AND p.prokind='f'
      ) definitions
    "#,[schema.into()])).await?.ok_or(AppError::InternalError)?;
    Ok(row.try_get("", "data")?)
}

pub async fn require_current(db: &DatabaseConnection) -> Result<(), AppError> {
    let row = db
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            "SELECT to_regclass(format('%I.horbyta_migrations',current_schema())) IS NOT NULL AS present".to_owned(),
        ))
        .await?
        .ok_or(AppError::InternalError)?;
    if !row.try_get::<bool>("", "present")? {
        return Err(AppError::Unavailable(
            "Ejecute cargo run --locked --bin migrate antes de iniciar el backend.".into(),
        ));
    }
    let supported = MIGRATIONS
        .iter()
        .map(|(version, _)| version.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let unknown = db
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            format!("SELECT EXISTS(SELECT 1 FROM horbyta_migrations WHERE version NOT IN ({supported})) AS present"),
        ))
        .await?
        .ok_or(AppError::InternalError)?;
    if unknown.try_get::<bool>("", "present")? {
        return Err(AppError::Unavailable(
            "La base requiere otra versión del backend.".into(),
        ));
    }
    for (version, checksum) in migration_checksums() {
        let row = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT checksum FROM horbyta_migrations WHERE version=$1",
                [version.into()],
            ))
            .await?;
        if row.as_ref().map(|r| r.try_get::<String>("", "checksum")).transpose()? != Some(checksum) {
            return Err(AppError::Unavailable(
                "El esquema no corresponde a esta versión; ejecute el binario migrate.".into(),
            ));
        }
    }
    Ok(())
}
