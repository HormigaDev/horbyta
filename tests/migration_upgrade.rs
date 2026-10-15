//! La base antigua se conserva y recibe solo la ampliación consolidada.
use futures::FutureExt;
use horbyta::migrations;
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};
use serde_json::Value;
use std::panic::AssertUnwindSafe;
use uuid::Uuid;

async fn query(db: &DatabaseConnection, sql: &str) -> Value {
    db.query_one(Statement::from_string(DbBackend::Postgres, sql.to_owned()))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "data")
        .unwrap()
}

#[tokio::test]
#[ignore = "Requiere PostgreSQL; verifica actualización no destructiva en dos esquemas aislados"]
async fn conserva_base_antigua_y_rechaza_estructura_incompatible() {
    dotenvy::dotenv().ok();
    let url = std::env::var("DATABASE_URL").expect("Falta conexión de pruebas");
    let mut options = ConnectOptions::new(url);
    options.max_connections(1).min_connections(1).sqlx_logging(false);
    let db = Database::connect(options).await.unwrap();
    for compatible in [true, false] {
        let schema = format!("upgrade_test_{}", Uuid::now_v7().simple());
        db.execute_unprepared(&format!("CREATE SCHEMA {schema}; SET search_path TO {schema}, public"))
            .await
            .unwrap();
        let result = AssertUnwindSafe(check(&db, &schema, compatible)).catch_unwind().await;
        db.execute_unprepared(&format!("SET search_path TO public; DROP SCHEMA {schema} CASCADE"))
            .await
            .unwrap();
        if let Err(error) = result {
            db.close().await.unwrap();
            std::panic::resume_unwind(error);
        }
    }
    db.close().await.unwrap();
}

async fn check(db: &DatabaseConnection, schema: &str, compatible: bool) {
    db.execute_unprepared(migrations::UUID_COMPATIBILITY).await.unwrap();
    db.execute_unprepared(migrations::INITIAL_MIGRATION).await.unwrap();
    let owner = Uuid::now_v7();
    let client = Uuid::now_v7();
    let project = Uuid::now_v7();
    let task = Uuid::now_v7();
    let invoice = Uuid::now_v7();
    let payment = Uuid::now_v7();
    let change = Uuid::now_v7();
    db.execute_unprepared(&format!("INSERT INTO users(id,name,email,password_hash,role) VALUES('{owner}','Cuenta anterior','upgrade@horbyta.test','hash anterior intacto','admin'); INSERT INTO clients(id,user_id,name) VALUES('{client}','{owner}','Cliente anterior'); INSERT INTO projects(id,user_id,client_id,name) VALUES('{project}','{owner}','{client}','Proyecto anterior'); INSERT INTO project_billing(user_id,project_id,currency_type,billing_type,fixed_price) VALUES('{owner}','{project}','EUR','fixed',150.25); INSERT INTO tasks(id,user_id,project_id,title,status) VALUES('{task}','{owner}','{project}','Trabajo anterior','in_progress'); INSERT INTO invoices(id,user_id,project_id,amount,currency_type,status) VALUES('{invoice}','{owner}','{project}',150.25,'EUR','paid'); INSERT INTO payments(id,user_id,project_id,invoice_id,amount,currency_type) VALUES('{payment}','{owner}','{project}','{invoice}',150.25,'EUR'); INSERT INTO change_requests(id,user_id,project_id,client_id,title,client_request_description,status,impact_cost) VALUES('{change}','{owner}','{project}','{client}','Extra anterior','Condiciones históricas','approved',35.50)")).await.unwrap();
    let snapshot =
        |table: &str| format!("SELECT coalesce(jsonb_agg(to_jsonb(t) ORDER BY id),'[]') AS data FROM {table} t");
    let before_users = query(db, &snapshot("users")).await;
    let before_invoices = query(db, &snapshot("invoices")).await;
    let before_payments = query(db, &snapshot("payments")).await;
    if compatible {
        assert!(
            db.execute_unprepared(migrations::INITIAL_MIGRATION).await.is_err(),
            "Reaplicar schema.sql no debe interpretarse como actualización incremental"
        );
        db.execute_unprepared("ROLLBACK").await.unwrap();
        assert_eq!(query(db, &snapshot("users")).await, before_users);
    }
    if !compatible {
        db.execute_unprepared("DROP INDEX uq_users_email").await.unwrap();
        let error = migrations::migrate(db).await.unwrap_err().to_string();
        assert!(error.contains("index:uq_users_email"), "{error}");
        assert_eq!(
            query(
                db,
                &format!("SELECT to_jsonb(to_regclass('{schema}.horbyta_migrations') IS NULL) AS data")
            )
            .await,
            true
        );
        assert_eq!(query(db, &snapshot("users")).await, before_users);
        return;
    }
    migrations::migrate(db).await.unwrap();
    migrations::migrate(db).await.unwrap();
    migrations::require_current(db).await.unwrap();
    assert_eq!(query(db, &snapshot("users")).await, before_users);
    assert_eq!(query(db, &snapshot("invoices")).await, before_invoices);
    assert_eq!(query(db, &snapshot("payments")).await, before_payments);
    assert_eq!(
        query(db, "SELECT to_jsonb(count(*)) AS data FROM horbyta_migrations").await,
        6
    );
    assert_eq!(
        query(db, "SELECT to_jsonb(count(*)) AS data FROM scope_effects").await,
        0
    );
    assert_eq!(query(db,&format!("SELECT to_jsonb(status='approved' AND legacy AND impact_cost=35.50) AS data FROM change_requests WHERE id='{change}'")).await,true);
    assert_eq!(
        query(
            db,
            &format!(
                "SELECT to_jsonb(status='in_progress' AND scope_task_allowed(id)) AS data FROM tasks WHERE id='{task}'"
            )
        )
        .await,
        true
    );
    assert_eq!(
        query(
            db,
            &format!("SELECT to_jsonb(NOT enforce_scope) AS data FROM projects WHERE id='{project}'")
        )
        .await,
        true
    );
    assert_eq!(
        query(
            db,
            "SELECT to_jsonb(count(*)) AS data FROM pg_namespace WHERE nspname LIKE 'horbyta_base_%'"
        )
        .await,
        0
    );
}
