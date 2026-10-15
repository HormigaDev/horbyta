//! Correcciones y consolidaciones conservan consentimiento, trabajo y suplementos.
use futures::FutureExt;
use horbyta::{dtos::scope::*, migrations, services::scope::ScopeService};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement, TransactionTrait};
use serde_json::{Value, json};
use uuid::Uuid;

async fn sql(db: &impl ConnectionTrait, query: &str) {
    db.execute_unprepared(query).await.unwrap();
}
fn document(agreement: Option<i32>) -> ScopeDocument {
    ScopeDocument {
        deliverables: "Sitio accesible".into(),
        criteria: "Navegación operativa".into(),
        requirements: "Tres páginas".into(),
        restrictions: "Una entrega".into(),
        exclusions: "Sin tienda".into(),
        agreement_revision: agreement,
        amendment_reason: if agreement.is_some() {
            "Corregir una interpretación del cliente".into()
        } else {
            String::new()
        },
        ..Default::default()
    }
}
async fn publish(
    db: &DatabaseConnection,
    owner: Uuid,
    project: Uuid,
    expected: i32,
    agreement: Option<i32>,
    published: bool,
) -> Value {
    let tx = db.begin().await.unwrap();
    let result = ScopeService::publish(
        &tx,
        owner,
        project,
        PublishScopeDto {
            expected_revision: expected,
            publish: published,
            document: document(agreement),
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    result
}
async fn accept(db: &DatabaseConnection, owner: Uuid, client: Uuid, actor: Uuid, project: Uuid, revision: i32) {
    let tx = db.begin().await.unwrap();
    sql(
        &tx,
        "SELECT set_config('horbyta.accepted_ip','198.51.100.25',true),set_config('horbyta.ip_source','direct',true)",
    )
    .await;
    ScopeService::accept(&tx, owner, client, actor, project, revision)
        .await
        .unwrap();
    tx.commit().await.unwrap();
}
async fn offer(db: &DatabaseConnection, owner: Uuid, client: Uuid, project: Uuid) -> Uuid {
    let id = Uuid::now_v7();
    sql(db,&format!("SET horbyta.actor='{owner}'; SET horbyta.actor_kind='professional'; INSERT INTO change_requests(id,user_id,project_id,client_id,title,client_request_description) VALUES('{id}','{owner}','{project}','{client}','Cambio autorizado','Una página adicional'); UPDATE change_requests SET status='submitted' WHERE id='{id}'; SET horbyta.publish_offer='true'; UPDATE change_requests SET status='assessed',technical_assessment='Implementación y validación',impact_hours=2,impact_cost=25 WHERE id='{id}'; SET horbyta.publish_offer='false'")).await;
    id
}
async fn approve(db: &DatabaseConnection, actor: Uuid, change: Uuid) {
    sql(db,&format!("SET horbyta.actor='{actor}'; SET horbyta.actor_kind='client'; SET horbyta.accepted_ip='198.51.100.25'; SET horbyta.ip_source='direct'; UPDATE change_requests SET status='approved' WHERE id='{change}'")).await;
}
async fn invoices(db: &DatabaseConnection) -> i64 {
    db.query_one(Statement::from_string(
        DbBackend::Postgres,
        "SELECT count(*) AS n FROM invoices".to_owned(),
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get("", "n")
    .unwrap()
}

#[tokio::test]
#[ignore = "PostgreSQL; esquema desechable, un hilo y un job"]
async fn corrections_and_approved_changes_preserve_agreements() {
    dotenvy::dotenv().ok();
    let url = std::env::var("SCOPE_TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap();
    let mut options = ConnectOptions::new(url);
    options.max_connections(1).min_connections(1).sqlx_logging(false);
    let db = Database::connect(options).await.unwrap();
    let schema = format!("amendment_test_{}", Uuid::now_v7().simple());
    sql(
        &db,
        &format!("CREATE SCHEMA {schema}; SET search_path TO {schema},public"),
    )
    .await;
    let result = std::panic::AssertUnwindSafe(run(&db)).catch_unwind().await;
    sql(&db, &format!("SET search_path TO public; DROP SCHEMA {schema} CASCADE")).await;
    db.close().await.unwrap();
    if let Err(error) = result {
        std::panic::resume_unwind(error)
    }
}
async fn run(db: &DatabaseConnection) {
    migrations::migrate(db).await.unwrap();
    let owner = Uuid::now_v7();
    let client = Uuid::now_v7();
    let actor = Uuid::now_v7();
    let project = Uuid::now_v7();
    let task = Uuid::now_v7();
    sql(db,&format!("INSERT INTO users(id,name,email,password_hash,role) VALUES('{owner}','Profesional','amendment@horbyta.test','hash','admin'); INSERT INTO clients(id,user_id,name) VALUES('{client}','{owner}','Cliente'); INSERT INTO client_auth(id,user_id,client_id,email,password_hash) VALUES('{actor}','{owner}','{client}','client@horbyta.test','hash'); INSERT INTO projects(id,user_id,client_id,name,billable) VALUES('{project}','{owner}','{client}','Proyecto',true); INSERT INTO project_billing(user_id,project_id,currency_type,billing_type,fixed_price) VALUES('{owner}','{project}','EUR','fixed',100); INSERT INTO tasks(id,user_id,project_id,title) VALUES('{task}','{owner}','{project}','Trabajo original')")).await;
    publish(db, owner, project, 0, None, true).await;
    accept(db, owner, client, actor, project, 1).await;
    let initial = ScopeService::project(db, owner, None, project).await.unwrap()["revisions"][0].clone();
    let tx = db.begin().await.unwrap();
    assert!(
        ScopeService::publish(
            &tx,
            owner,
            project,
            PublishScopeDto {
                expected_revision: 1,
                publish: true,
                document: document(None)
            }
        )
        .await
        .is_err()
    );
    tx.rollback().await.unwrap();
    publish(db, owner, project, 1, Some(1), false).await;
    assert_eq!(
        ScopeService::project(db, owner, Some(client), project).await.unwrap()["revisions"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    publish(db, owner, project, 2, Some(1), true).await;
    let tx = db.begin().await.unwrap();
    ScopeService::feedback(
        &tx,
        owner,
        Some(client),
        actor,
        project,
        ScopeFeedbackDto {
            revision: 3,
            kind: "reject".into(),
            text: "Conservar el acuerdo anterior".into(),
            document_id: None,
            attachment_ids: vec![],
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(
        ScopeService::project(db, owner, Some(client), project).await.unwrap()["accepted_revision"],
        1
    );
    publish(db, owner, project, 3, Some(1), true).await;
    let tx = db.begin().await.unwrap();
    ScopeService::feedback(
        &tx,
        owner,
        Some(client),
        actor,
        project,
        ScopeFeedbackDto {
            revision: 4,
            kind: "request_changes".into(),
            text: "Ajustar el texto".into(),
            document_id: None,
            attachment_ids: vec![],
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    publish(db, owner, project, 4, Some(1), true).await;
    accept(db, owner, client, actor, project, 5).await;
    let stale = offer(db, owner, client, project).await;
    publish(db, owner, project, 5, Some(5), true).await;
    accept(db, owner, client, actor, project, 6).await;
    let tx = db.begin().await.unwrap();
    ScopeService::actor(&tx, actor, "client").await.unwrap();
    let error = tx
        .execute_unprepared(&format!(
            "UPDATE change_requests SET status='approved' WHERE id='{stale}'"
        ))
        .await
        .unwrap_err();
    assert!(error.to_string().contains("vuelva a evaluar"), "{error}");
    tx.rollback().await.unwrap();
    assert_eq!(invoices(db).await, 0);
    let first = offer(db, owner, client, project).await;
    let second = offer(db, owner, client, project).await;
    approve(db, actor, first).await;
    let state = ScopeService::project(db, owner, Some(client), project).await.unwrap();
    assert_eq!(state["accepted_revision"], 7);
    assert_eq!(state["revisions"][0]["status"], "consolidated");
    assert_eq!(
        state["revisions"][0]["accepted_at"],
        Value::Null,
        "No se fabrica una aceptación del documento entero"
    );
    assert_eq!(state["revisions"][0]["source_change_id"], first.to_string());
    assert_eq!(state["revisions"][0]["document"], state["revisions"][1]["document"]);
    assert_eq!(state["revisions"][0]["approved_changes"][0]["amount"], "25.0000");
    assert_eq!(invoices(db).await, 1);
    approve(db, actor, first).await;
    assert_eq!(invoices(db).await, 1);
    assert_eq!(
        ScopeService::project(db, owner, None, project).await.unwrap()["last_revision"],
        7
    );
    approve(db, actor, second).await;
    assert_eq!(invoices(db).await, 2);
    let tx = db.begin().await.unwrap();
    assert!(
        ScopeService::publish(
            &tx,
            owner,
            project,
            PublishScopeDto {
                expected_revision: 8,
                publish: true,
                document: document(Some(7))
            }
        )
        .await
        .is_err()
    );
    tx.rollback().await.unwrap();
    let revised = publish(db, owner, project, 8, Some(8), true).await;
    assert_eq!(revised["accepted_revision"], 8);
    assert_eq!(revised["revisions"][0]["approved_changes"].as_array().unwrap().len(), 2);
    accept(db, owner, client, actor, project, 9).await;
    assert_eq!(invoices(db).await, 2);
    let third = offer(db, owner, client, project).await;
    publish(db, owner, project, 9, Some(9), true).await;
    approve(db, actor, third).await;
    let tx = db.begin().await.unwrap();
    assert!(
        ScopeService::accept(&tx, owner, client, actor, project, 10)
            .await
            .is_err()
    );
    tx.rollback().await.unwrap();
    let current = ScopeService::project(db, owner, None, project).await.unwrap();
    assert_eq!(current["accepted_revision"], 11);
    assert_eq!(current["revisions"][1]["status"], "superseded");
    assert_eq!(current["revisions"].as_array().unwrap().last().unwrap(), &initial);
    assert_eq!(current["revisions"][0]["approved_changes"].as_array().unwrap().len(), 3);
    let tx = db.begin().await.unwrap();
    assert!(
        tx.execute_unprepared(&format!(
            "UPDATE scope_revisions SET document='{{}}' WHERE project_id='{project}' AND revision=11"
        ))
        .await
        .is_err()
    );
    tx.rollback().await.unwrap();
    let allowed = db
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            format!("SELECT scope_task_allowed('{task}') AS allowed"),
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get::<bool>("", "allowed")
        .unwrap();
    assert!(allowed);
    assert_eq!(invoices(db).await, 3);
    assert_eq!(
        current["revisions"][0]["approved_changes"][0]["accepted_ip"],
        "198.51.100.25/32"
    );
    assert!(current["revisions"][0]["bundle_hash"].as_str().unwrap().len() == 64);
    let timeline =
        horbyta::services::project_conversation::window(db, owner, Some(client), project, Default::default())
            .await
            .unwrap();
    assert!(
        timeline["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["kind"] == json!("scope_consolidated"))
    );
    let entries = timeline["items"].as_array().unwrap();
    let last_consolidation = entries
        .iter()
        .rposition(|item| item["kind"] == "scope_consolidated")
        .unwrap();
    let last_approval = entries
        .iter()
        .rposition(|item| item["kind"] == "change_approved")
        .unwrap();
    assert!(
        last_consolidation > last_approval,
        "La consolidación debe seguir a la aprobación"
    );
    let event = horbyta::services::project_conversation::latest_event(db, owner, project, None)
        .await
        .unwrap();
    assert_eq!(event["kind"], "scope_consolidated");
    assert_eq!(event["context"]["change_request_id"], third.to_string());

    // Revertir la migración 6 borraría consolidaciones aceptadas: se rechaza sin cambios.
    let error = horbyta::migrations::rollback_to(db, 5).await.unwrap_err();
    assert!(format!("{error:?}").contains("evidencia contractual"), "{error:?}");
    horbyta::migrations::require_current(db).await.unwrap();
}
