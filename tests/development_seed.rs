//! Instalación limpia y seed en un esquema aislado de hormigapm_development.
use futures::FutureExt;
use horbyta::{development_seed, migrations, utils::crypto::compare_password};
use object_store::{ObjectStoreExt, memory::InMemory, path::Path};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};
use std::panic::AssertUnwindSafe;
use uuid::Uuid;

async fn count(db: &DatabaseConnection, query: &str) -> i64 {
    db.query_one(Statement::from_string(DbBackend::Postgres, query.to_owned()))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "value")
        .unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[ignore = "Requiere hormigapm_development; solo crea y elimina su esquema aislado"]
async fn instalacion_limpia_y_seed_completo() {
    dotenvy::dotenv().ok();
    let url = std::env::var("SEED_TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("Falta conexión de pruebas");
    let mut options = ConnectOptions::new(url);
    options.max_connections(1).min_connections(1).sqlx_logging(false);
    let db = Database::connect(options).await.unwrap();
    let schema = format!("seed_test_{}", Uuid::now_v7().simple());
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
    let storage = InMemory::new();
    assert!(development_seed::seed(db, &storage, "production", true).await.is_err());
    let report = development_seed::seed(db, &storage, "development", false)
        .await
        .unwrap();
    assert!(
        report["tables"]
            .as_object()
            .unwrap()
            .values()
            .all(|n| n.as_i64().unwrap() > 0)
    );
    assert_eq!(
        count(db, "SELECT count(*) AS value FROM horbyta_migrations").await,
        i64::from(migrations::latest_version())
    );
    migrations::migrate(db).await.unwrap();
    migrations::migrate(db).await.unwrap();
    migrations::require_current(db).await.unwrap();
    assert_eq!(count(db, "SELECT count(*) AS value FROM projects").await, 12);
    assert_eq!(
        count(db, "SELECT count(DISTINCT status) AS value FROM projects").await,
        3
    );
    assert_eq!(
        count(db, "SELECT count(DISTINCT status) AS value FROM change_requests").await,
        7
    );
    assert_eq!(
        count(db, "SELECT count(DISTINCT status) AS value FROM invoices").await,
        5
    );
    assert_eq!(count(db, "SELECT count(DISTINCT status) AS value FROM tasks").await, 4);
    assert_eq!(
        count(db, "SELECT count(DISTINCT activity) AS value FROM time_entries").await,
        9
    );
    assert_eq!(
        count(db, "SELECT count(DISTINCT currency_type) AS value FROM payments").await,
        3
    );
    assert_eq!(
        count(
            db,
            "SELECT count(*) AS value FROM scope_projects WHERE accepted_revision IS NOT NULL"
        )
        .await,
        8
    );
    assert_eq!(count(db,"SELECT count(*) AS value FROM tasks t JOIN scope_task_links l ON l.task_id=t.id JOIN change_requests c ON c.id=l.change_request_id WHERE c.status NOT IN('approved','implemented') AND scope_task_allowed(t.id)").await,0);
    assert_eq!(count(db,"SELECT count(*) AS value FROM time_entries e JOIN tasks t ON t.id=e.task_id WHERE e.billable AND (t.scope_preparation_reason IS NOT NULL OR EXISTS(SELECT 1 FROM scope_task_links l WHERE l.task_id=t.id AND l.change_request_id IS NOT NULL))").await,0);
    assert_eq!(count(db,"SELECT count(*) AS value FROM (SELECT user_id,work_date,sum(duration_minutes) AS minutes FROM time_entries GROUP BY user_id,work_date HAVING sum(duration_minutes)>480) d").await,0);
    assert_eq!(count(db,"SELECT count(*) AS value FROM payments p JOIN invoices i ON i.id=p.invoice_id WHERE p.project_id<>i.project_id OR p.currency_type<>i.currency_type OR p.user_id<>i.user_id").await,0);
    assert_eq!(
        count(db, "SELECT count(DISTINCT status) AS value FROM milestones").await,
        5
    );
    assert!(
        count(
            db,
            "SELECT count(*) AS value FROM time_entries WHERE invoice_id IS NOT NULL"
        )
        .await
            > 0
    );
    assert_eq!(count(db,"SELECT count(DISTINCT d->>'role') AS value FROM scope_revisions r, jsonb_array_elements(r.document->'documents') d").await,3);
    assert!(count(db,"SELECT count(*) AS value FROM scope_revisions r,jsonb_array_elements(r.document->'documents') d WHERE d->>'version'='2'").await>0);
    let row = db
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            "SELECT password_hash FROM users WHERE email='admin@horbyta.test'".to_owned(),
        ))
        .await
        .unwrap()
        .unwrap();
    let hash: String = row.try_get("", "password_hash").unwrap();
    assert!(compare_password("Admin@123".into(), hash).unwrap());
    for row in db
        .query_all(Statement::from_string(
            DbBackend::Postgres,
            "SELECT storage_key,size_bytes FROM attachments".to_owned(),
        ))
        .await
        .unwrap()
    {
        let key: String = row.try_get("", "storage_key").unwrap();
        let size: i64 = row.try_get("", "size_bytes").unwrap();
        assert_eq!(
            storage
                .get(&Path::from(key))
                .await
                .unwrap()
                .bytes()
                .await
                .unwrap()
                .len() as i64,
            size
        );
    }
    let users = count(db, "SELECT count(*) AS value FROM users").await;
    assert!(
        development_seed::seed(db, &storage, "development", false)
            .await
            .is_err()
    );
    assert!(development_seed::seed(db, &storage, "development", true).await.is_err()); // --reset nunca toca public desde otro search_path.
    assert_eq!(count(db, "SELECT count(*) AS value FROM users").await, users);
    let future = migrations::latest_version() + 1;
    db.execute_unprepared(&format!(
        "INSERT INTO horbyta_migrations(version,checksum) VALUES({future},'otra-version')"
    ))
    .await
    .unwrap();
    assert!(migrations::migrate(db).await.is_err());
    assert_eq!(count(db, "SELECT count(*) AS value FROM users").await, users);
    db.execute_unprepared(&format!(
        "DELETE FROM horbyta_migrations WHERE version={future}; UPDATE horbyta_migrations SET checksum='alterado' WHERE version=1"
    ))
    .await
    .unwrap();
    assert!(migrations::migrate(db).await.is_err());
    assert!(migrations::require_current(db).await.is_err());
    assert_eq!(count(db, "SELECT count(*) AS value FROM users").await, users);
}
