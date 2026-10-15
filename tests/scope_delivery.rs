//! LISTEN/NOTIFY y outbox reales; esquema desechable y un único hilo.
use futures::FutureExt;
use horbyta::{env::Config, migrations, services::scope_delivery, state::AppState};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement, TransactionTrait};
use std::{panic::AssertUnwindSafe, time::Duration};
use tokio::time::{sleep, timeout};
use uuid::Uuid;

async fn sql(db: &impl ConnectionTrait, query: &str) {
    db.execute_unprepared(query).await.unwrap();
}
async fn number(db: &impl ConnectionTrait, query: &str) -> i64 {
    db.query_one(Statement::from_string(DbBackend::Postgres, query.to_owned()))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "n")
        .unwrap()
}
async fn enqueue(db: &impl ConnectionTrait, change: Uuid, delay: &str) -> Uuid {
    let event = Uuid::now_v7();
    sql(db, &format!("INSERT INTO scope_events(id,change_request_id,version,kind,actor_kind) VALUES('{event}','{change}',1,'declined','client'); INSERT INTO scope_delivery(event_id,next_attempt_at) VALUES('{event}',clock_timestamp()+interval '{delay}')")).await;
    event
}
fn config(url: String, dir: String) -> Config {
    Config {
        database_url: url,
        jwt_secret: "test".into(),
        jwt_secret_client: "test-client".into(),
        app_uri: "127.0.0.1:0".into(),
        frontend_origin: "http://localhost".into(),
        is_production: false,
        time_to_exp: Duration::from_secs(300),
        s3_bucket: None,
        s3_endpoint: None,
        s3_region: None,
        s3_access_key: None,
        s3_secret_key: None,
        upload_dir: Some(dir),
        allow_registration: false,
    }
}
#[tokio::test]
#[ignore = "PostgreSQL; crea y elimina un esquema aislado, ejecutar con un job/hilo"]
async fn signals_recovery_retries_and_no_idle_queries() {
    dotenvy::dotenv().ok();
    let url = std::env::var("SCOPE_TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap();
    let mut options = ConnectOptions::new(url.clone());
    options.max_connections(1).min_connections(1).sqlx_logging(false);
    let db = Database::connect(options.clone()).await.unwrap();
    let observer = Database::connect(options).await.unwrap();
    let schema = format!("delivery_test_{}", Uuid::now_v7().simple());
    let dir = format!("/tmp/horbyta-{schema}");
    sql(
        &db,
        &format!("CREATE SCHEMA {schema};SET search_path TO {schema},public"),
    )
    .await;
    let mut worker = None;
    let result = AssertUnwindSafe(async {
        migrations::migrate(&db).await.unwrap();
        let owner=Uuid::now_v7(); let client=Uuid::now_v7(); let project=Uuid::now_v7(); let change=Uuid::now_v7();
        sql(&db, &format!("INSERT INTO users(id,name,email,password_hash,role) VALUES('{owner}','Prueba','delivery@horbyta.test','hash','admin'); INSERT INTO clients(id,user_id,name) VALUES('{client}','{owner}','Cliente'); INSERT INTO projects(id,user_id,client_id,name) VALUES('{project}','{owner}','{client}','Proyecto'); INSERT INTO change_requests(id,user_id,project_id,client_id,title,client_request_description) VALUES('{change}','{owner}','{project}','{client}','Cambio','Pedido conservado')")).await;
        let mut listener=scope_delivery::listen(&db).await.unwrap();
        let tx=db.begin().await.unwrap();
        enqueue(&tx,change,"0 seconds").await;
        assert!(timeout(Duration::from_millis(150),listener.recv()).await.is_err(),"Sin aviso antes de commit");
        tx.rollback().await.unwrap();
        assert!(timeout(Duration::from_millis(150),listener.recv()).await.is_err(),"Rollback no publica");
        assert!(scope_delivery::next_attempt_delay(&db).await.unwrap().is_none());
        let tx=db.begin().await.unwrap();
        enqueue(&tx,change,"0 seconds").await;
        tx.commit().await.unwrap();
        timeout(Duration::from_secs(2),listener.recv()).await.unwrap().unwrap();
        // Sin avisos de otros esquemas.
        observer.execute_unprepared("SELECT pg_notify('horbyta_scope_delivery_otro','')").await.unwrap();
        assert!(timeout(Duration::from_millis(150),listener.recv()).await.is_err());
        let state_db: DatabaseConnection=db.get_postgres_connection_pool().clone().into();
        let state=AppState::new(state_db,config(url.clone(),dir.clone())).await.unwrap();
        let mut events=state.notification_bus.subscribe();
        worker=Some(tokio::spawn(scope_delivery::run(state.clone())));
        timeout(Duration::from_secs(5),events.recv()).await.unwrap().unwrap();
        // Comparar query_start desde otra conexión demuestra ausencia de SQL en
        // reposo, no simplemente que el log haya sido silenciado.
        sleep(Duration::from_millis(250)).await;
        let pid=number(&db,"SELECT pg_backend_pid()::bigint AS n").await;
        // Esta consulta explícita pasa a ser la última de la conexión compartida.
        let before: String=observer.query_one(Statement::from_string(DbBackend::Postgres,format!("SELECT query_start::text AS stamp FROM pg_stat_activity WHERE pid={pid}"))).await.unwrap().unwrap().try_get("","stamp").unwrap();
        sleep(Duration::from_secs(6)).await;
        let after: String=observer.query_one(Statement::from_string(DbBackend::Postgres,format!("SELECT query_start::text AS stamp FROM pg_stat_activity WHERE pid={pid}"))).await.unwrap().unwrap().try_get("","stamp").unwrap();
        assert_eq!(before,after,"La cola vacía no ejecuta consultas cada cinco segundos");
        enqueue(&db,change,"200 milliseconds").await;
        assert!(timeout(Duration::from_millis(80),events.recv()).await.is_err());
        timeout(Duration::from_secs(3),events.recv()).await.unwrap().unwrap();
        assert_eq!(number(&db,"SELECT count(*) AS n FROM scope_delivery WHERE delivered_at IS NOT NULL").await,2);
        // Fallo real: se conserva la fila y se programa un reintento.
        sql(&db,"CREATE FUNCTION fail_delivery() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'fallo de prueba'; END; $$; CREATE TRIGGER fail_delivery BEFORE INSERT ON notifications FOR EACH ROW EXECUTE FUNCTION fail_delivery()").await;
        let failed=enqueue(&db,change,"0 seconds").await;
        timeout(Duration::from_secs(3),async { loop { if number(&db,&format!("SELECT attempts::bigint AS n FROM scope_delivery WHERE event_id='{failed}'")).await>0 {break;} sleep(Duration::from_millis(50)).await; }}).await.unwrap();
        assert!(timeout(Duration::from_millis(100),events.recv()).await.is_err());
        sql(&db,&format!("DROP TRIGGER fail_delivery ON notifications; DROP FUNCTION fail_delivery(); UPDATE scope_delivery SET next_attempt_at=clock_timestamp()+interval '150 milliseconds' WHERE event_id='{failed}'")).await;
        timeout(Duration::from_secs(3),events.recv()).await.unwrap().unwrap();
        assert_eq!(number(&db,&format!("SELECT attempts::bigint AS n FROM scope_delivery WHERE event_id='{failed}'")).await,2);
        worker.take().unwrap().abort();
        sleep(Duration::from_millis(100)).await;
        // La señal puede perderse sin consumidor; el registro durable se recupera.
        enqueue(&db,change,"0 seconds").await;
        worker=Some(tokio::spawn(scope_delivery::run(state.clone())));
        timeout(Duration::from_secs(5),events.recv()).await.unwrap().unwrap();
        assert_eq!(number(&db,"SELECT count(*) AS n FROM scope_delivery WHERE delivered_at IS NOT NULL").await,4);
        assert!(timeout(Duration::from_millis(150),events.recv()).await.is_err(),"Sin duplicar entregas");
        // Una desconexión real de LISTEN también obliga a releer la cola durable.
        observer.execute_unprepared(&format!("SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE application_name='horbyta-scope-delivery' AND query LIKE '%horbyta_scope_delivery_'||md5('{schema}')||'%'" )).await.unwrap();
        sleep(Duration::from_millis(200)).await;
        enqueue(&db,change,"0 seconds").await;
        timeout(Duration::from_secs(10),events.recv()).await.unwrap().unwrap();
        assert_eq!(number(&db,"SELECT count(*) AS n FROM scope_delivery WHERE delivered_at IS NOT NULL").await,5);
        drop(listener);
    }).catch_unwind().await;
    if let Some(worker) = worker {
        worker.abort();
        let _ = worker.await;
    }
    sql(&db, &format!("SET search_path TO public;DROP SCHEMA {schema} CASCADE")).await;
    db.close().await.unwrap();
    observer.close().await.unwrap();
    let _ = std::fs::remove_dir_all(dir);
    if let Err(error) = result {
        std::panic::resume_unwind(error);
    }
}
