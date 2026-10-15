//! Cada reverso devuelve el esquema exactamente al estado previo a su migración.
use futures::FutureExt;
use horbyta::migrations;
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};
use serde_json::Value;
use std::panic::AssertUnwindSafe;
use uuid::Uuid;

// Estructura del esquema actual, sin el propio historial de migraciones.
async fn catalog(db: &DatabaseConnection) -> Value {
    db.query_one(Statement::from_string(DbBackend::Postgres, r#"
      SELECT coalesce(jsonb_object_agg(key,definition),'{}') AS data FROM (
        SELECT 'table:'||c.relname AS key,
          (SELECT jsonb_agg(jsonb_build_object('name',a.attname,'type',format_type(a.atttypid,a.atttypmod),'not_null',a.attnotnull,'default',pg_get_expr(d.adbin,d.adrelid)) ORDER BY a.attnum)
           FROM pg_attribute a LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum WHERE a.attrelid=c.oid AND a.attnum>0 AND NOT a.attisdropped) AS definition
          FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname=current_schema() AND c.relkind='r' AND c.relname<>'horbyta_migrations'
        UNION ALL SELECT 'constraint:'||c.relname||':'||k.conname,to_jsonb(pg_get_constraintdef(k.oid)) FROM pg_constraint k JOIN pg_class c ON c.oid=k.conrelid JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname=current_schema() AND c.relname<>'horbyta_migrations'
        UNION ALL SELECT 'index:'||c.relname,to_jsonb(pg_get_indexdef(c.oid)) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname=current_schema() AND c.relkind='i' AND c.relname<>'horbyta_migrations_pkey'
        UNION ALL SELECT 'view:'||c.relname,to_jsonb(pg_get_viewdef(c.oid)) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname=current_schema() AND c.relkind='v'
        UNION ALL SELECT 'trigger:'||c.relname||':'||t.tgname,to_jsonb(pg_get_triggerdef(t.oid)) FROM pg_trigger t JOIN pg_class c ON c.oid=t.tgrelid JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname=current_schema() AND NOT t.tgisinternal
        UNION ALL SELECT 'function:'||p.proname||':'||pg_get_function_identity_arguments(p.oid),to_jsonb(pg_get_functiondef(p.oid)) FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace WHERE n.nspname=current_schema()
      ) definitions
    "#.to_owned()))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "data")
        .unwrap()
}

#[tokio::test]
#[ignore = "Requiere PostgreSQL; aplica y revierte cada migración en un esquema aislado"]
async fn cada_rollback_restaura_el_esquema_anterior() {
    dotenvy::dotenv().ok();
    let url = std::env::var("DATABASE_URL").expect("Falta conexión de pruebas");
    let mut options = ConnectOptions::new(url);
    options.max_connections(1).min_connections(1).sqlx_logging(false);
    let db = Database::connect(options).await.unwrap();
    let schema = format!("rollback_test_{}", Uuid::now_v7().simple());
    db.execute_unprepared(&format!("CREATE SCHEMA {schema}; SET search_path TO {schema}, public"))
        .await
        .unwrap();
    let result = AssertUnwindSafe(check(&db)).catch_unwind().await;
    db.execute_unprepared(&format!("SET search_path TO public; DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
    db.close().await.unwrap();
    if let Err(error) = result {
        std::panic::resume_unwind(error);
    }
}

async fn check(db: &DatabaseConnection) {
    let latest = migrations::latest_version();
    let mut states = vec![catalog(db).await];
    for version in 1..=latest {
        assert_eq!(migrations::migrate_to(db, version).await.unwrap(), vec![version]);
        states.push(catalog(db).await);
    }
    migrations::require_current(db).await.unwrap();

    // Ida y vuelta de cada migración por separado, de la última a la primera.
    for version in (1..=latest).rev() {
        assert_eq!(migrations::rollback_to(db, version - 1).await.unwrap(), vec![version]);
        assert_eq!(catalog(db).await, states[version as usize - 1], "rollback de {version}");
        assert_eq!(migrations::migrate_to(db, version).await.unwrap(), vec![version]);
        assert_eq!(catalog(db).await, states[version as usize], "reaplicar {version}");
        migrations::rollback_to(db, version - 1).await.unwrap();
    }
    assert!(migrations::applied_versions(db).await.unwrap().is_empty());
    assert!(migrations::require_current(db).await.is_err());

    // Revertir varias de una vez y volver a aplicar todo.
    migrations::migrate(db).await.unwrap();
    assert_eq!(migrations::rollback_to(db, 3).await.unwrap(), (4..=latest).rev().collect::<Vec<_>>());
    assert_eq!(catalog(db).await, states[3]);
    assert!(migrations::rollback_to(db, latest + 1).await.is_err());
    migrations::migrate(db).await.unwrap();
    assert_eq!(catalog(db).await, states[latest as usize]);
    migrations::require_current(db).await.unwrap();
}
