//! Pruebas reales en un esquema desechable; no escriben en las tablas habituales.
use futures::FutureExt;
use horbyta::{dtos::scope::*, migrations, services::scope::ScopeService};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement, TransactionTrait};
use serde_json::{Value, json};
use std::panic::AssertUnwindSafe;
use uuid::Uuid;

async fn exec(db: &DatabaseConnection, sql: &str) {
    db.execute_unprepared(sql).await.unwrap();
}
async fn value(db: &DatabaseConnection, sql: &str) -> Value {
    db.query_one(Statement::from_string(
        DbBackend::Postgres,
        format!("SELECT to_jsonb(v.data) AS data FROM ({sql}) v"),
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get("", "data")
    .unwrap()
}
async fn fails(db: &DatabaseConnection, sql: &str) {
    let error = db
        .execute_unprepared(sql)
        .await
        .expect_err("Se permitió una operación prohibida")
        .to_string();
    assert!(
        error.contains("scope_conflict:")
            || error.contains("foreign key constraint")
            || error.contains("fallo de prueba"),
        "La operación falló por una causa ajena al control de alcance: {error}; SQL: {sql}"
    );
}
fn document() -> ScopeDocument {
    ScopeDocument {
        deliverables: "Sitio de tres páginas".into(),
        criteria: "Navegación operativa y diseño adaptable".into(),
        restrictions: "Una revisión".into(),
        exclusions: "Sin tienda".into(),
        requirements: "Accesibilidad y acceso autenticado".into(),
        ..Default::default()
    }
}

#[tokio::test]
#[ignore = "Requiere PostgreSQL; crea y elimina un esquema aislado. Ejecutar con --ignored --test-threads=1"]
async fn flujo_real_y_migracion() {
    dotenvy::dotenv().ok();
    let url = std::env::var("SCOPE_TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("Falta conexión de pruebas");
    let mut options = ConnectOptions::new(url);
    options.max_connections(1).min_connections(1).sqlx_logging(false);
    let db = Database::connect(options)
        .await
        .expect("No se pudo conectar a PostgreSQL para las pruebas aisladas");
    let schema = format!("scope_test_{}", Uuid::now_v7().simple());
    exec(
        &db,
        &format!("CREATE SCHEMA {schema}; SET search_path TO {schema}, public"),
    )
    .await;
    let result = AssertUnwindSafe(run(&db, &schema)).catch_unwind().await;
    if std::env::var("KEEP_SCOPE_TEST_SCHEMA").as_deref() != Ok("1") || result.is_err() {
        exec(&db, &format!("SET search_path TO public; DROP SCHEMA {schema} CASCADE")).await;
    }
    db.close().await.unwrap();
    if let Err(error) = result {
        std::panic::resume_unwind(error)
    }
}
async fn run(db: &DatabaseConnection, schema: &str) {
    // Compatibilidad del laboratorio PostgreSQL 16: UUIDv7 conserva tiempo y aleatoriedad.
    exec(db,"CREATE FUNCTION uuidv7() RETURNS uuid LANGUAGE sql VOLATILE AS $$ WITH r AS (SELECT replace(gen_random_uuid()::text,'-','') AS v) SELECT (lpad(to_hex((extract(epoch from clock_timestamp())*1000)::bigint),12,'0')||'7'||substr(v,14,3)||'8'||substr(v,18,15))::uuid FROM r $$").await;
    migrations::migrate(db).await.unwrap();
    migrations::migrate(db).await.unwrap();
    migrations::require_current(db).await.unwrap();
    let owner = Uuid::now_v7();
    let client = Uuid::now_v7();
    let actor = Uuid::now_v7();
    let project = Uuid::now_v7();
    let base_task = Uuid::now_v7();
    let milestone = Uuid::now_v7();
    let activity = Uuid::now_v7();
    let password = Uuid::now_v7().to_string();
    let hash = horbyta::utils::crypto::hash_password(password.clone()).unwrap();
    db.execute(Statement::from_sql_and_values(DbBackend::Postgres,
        "INSERT INTO users(id,name,email,password_hash,role) VALUES($1,'Pruebas de alcance','scope-tests@horbyta.test',$2,'admin')",[owner.into(),hash.clone().into()])).await.unwrap();
    db.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO clients(id,user_id,name,email) VALUES($1,$2,'Cliente de pruebas','scope-client@horbyta.test')",
        [client.into(), owner.into()],
    ))
    .await
    .unwrap();
    db.execute(Statement::from_sql_and_values(DbBackend::Postgres,
        "INSERT INTO client_auth(id,user_id,client_id,email,password_hash) VALUES($1,$2,$3,'scope-client@horbyta.test',$4)",[actor.into(),owner.into(),client.into(),hash.into()])).await.unwrap();
    exec(db,&format!("INSERT INTO projects(id,user_id,client_id,name) VALUES('{project}','{owner}','{client}','Proyecto de prueba'); INSERT INTO project_billing(user_id,project_id,currency_type,billing_type,fixed_price) VALUES('{owner}','{project}','USD','fixed',100); INSERT INTO tasks(id,user_id,project_id,title) VALUES('{base_task}','{owner}','{project}','Trabajo original'); INSERT INTO milestones(id,project_id,name,due_date) VALUES('{milestone}','{project}','Entrega','2026-11-01T12:00:00Z'); INSERT INTO activities(id,user_id,name) VALUES('{activity}','{owner}','Desarrollo')")).await;
    assert_eq!(
        value(db, "SELECT count(*)::int AS data FROM horbyta_migrations").await,
        7
    );
    assert_eq!(
        value(db, &format!("SELECT scope_task_allowed('{base_task}') AS data")).await,
        false
    );
    let tx = db.begin().await.unwrap();
    ScopeService::publish(
        &tx,
        owner,
        project,
        PublishScopeDto {
            expected_revision: 0,
            publish: true,
            document: document(),
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(
        ScopeService::pending(db, owner, client)
            .await
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let tx = db.begin().await.unwrap();
    assert!(
        ScopeService::publish(
            &tx,
            owner,
            project,
            PublishScopeDto {
                expected_revision: 0,
                publish: true,
                document: document()
            }
        )
        .await
        .is_err()
    );
    tx.rollback().await.unwrap();
    // Guardar un borrador privado no retira la última oferta publicada al cliente.
    let tx = db.begin().await.unwrap();
    ScopeService::publish(
        &tx,
        owner,
        project,
        PublishScopeDto {
            expected_revision: 1,
            publish: false,
            document: document(),
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(
        ScopeService::project(db, owner, Some(client), project).await.unwrap()["published_revision"],
        1
    );
    let tx = db.begin().await.unwrap();
    ScopeService::accept(&tx, owner, client, actor, project, 1)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let tx = db.begin().await.unwrap();
    ScopeService::accept(&tx, owner, client, actor, project, 1)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(
        value(db, &format!("SELECT scope_task_allowed('{base_task}') AS data")).await,
        true
    );
    fails(
        db,
        &format!("UPDATE scope_revisions SET document='{{}}' WHERE project_id='{project}'"),
    )
    .await;
    fails(db, &format!("DELETE FROM projects WHERE id='{project}'")).await;
    fails(db, &format!("DELETE FROM clients WHERE id='{client}'")).await;
    fails(db, &format!("DELETE FROM users WHERE id='{owner}'")).await;
    fails(db, &format!("DELETE FROM tasks WHERE id='{base_task}'")).await;
    assert!(
        ScopeService::project(db, owner, Some(Uuid::now_v7()), project)
            .await
            .is_err()
    );
    assert!(ScopeService::project(db, Uuid::now_v7(), None, project).await.is_err());
    exec(
        db,
        &format!("UPDATE tasks SET status='in_progress' WHERE id='{base_task}'"),
    )
    .await;
    let cr = Uuid::now_v7();
    let extra = Uuid::now_v7();
    exec(db,&format!("SET horbyta.actor='{owner}';SET horbyta.actor_kind='professional';INSERT INTO change_requests(id,user_id,project_id,client_id,title,client_request_description,status) VALUES('{cr}','{owner}','{project}','{client}','Tienda','Añadir tienda','submitted');INSERT INTO tasks(id,user_id,project_id,title) VALUES('{extra}','{owner}','{project}','Extra bloqueado')")).await;
    let tx = db.begin().await.unwrap();
    ScopeService::classify(
        &tx,
        owner,
        owner,
        extra,
        ClassifyTaskDto {
            change_request_id: Some(cr),
            reason: "Tienda excluida del alcance".into(),
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    fails(db, &format!("UPDATE tasks SET status='in_progress' WHERE id='{extra}'")).await;
    fails(db, &format!("UPDATE tasks SET project_id=NULL WHERE id='{extra}'")).await;
    fails(db, &format!("DELETE FROM scope_task_links WHERE task_id='{extra}'")).await;
    fails(db,&format!("INSERT INTO time_entries(user_id,task_id,activity,duration_minutes,billable) VALUES('{owner}','{extra}','{activity}',10,false)")).await;
    exec(db,&format!("UPDATE change_requests SET technical_assessment='Una jornada',impact_hours=8,impact_cost=50,schedule_impact_days=2,target_milestone_id='{milestone}',status='assessed' WHERE id='{cr}'")).await;
    assert_eq!(
        value(
            db,
            &format!("SELECT version AS data FROM change_requests WHERE id='{cr}'")
        )
        .await,
        2
    );
    fails(db,&format!("INSERT INTO time_entries(user_id,task_id,activity,start_time,duration_minutes,billable) VALUES('{owner}','{extra}','{activity}',now(),0,false)")).await;
    // Cambiar únicamente el importe invalida la oferta y conserva la anterior.
    exec(
        db,
        &format!("UPDATE change_requests SET impact_cost=60 WHERE id='{cr}'"),
    )
    .await;
    assert_eq!(
        value(
            db,
            &format!("SELECT status AS data FROM change_requests WHERE id='{cr}'")
        )
        .await,
        "submitted"
    );
    exec(
        db,
        &format!("UPDATE change_requests SET status='assessed' WHERE id='{cr}'"),
    )
    .await;
    assert_eq!(value(db,&format!("SELECT (snapshot->>'impact_cost')::numeric AS data FROM scope_change_revisions WHERE change_request_id='{cr}' AND version=2")).await.as_f64(),Some(50.0));
    // La condición de versión obsoleta no afecta ninguna fila.
    let stale = db
        .execute_unprepared(&format!(
            "UPDATE change_requests SET status='approved' WHERE id='{cr}' AND version=2"
        ))
        .await
        .unwrap();
    assert_eq!(stale.rows_affected(), 0);
    fails(
        db,
        &format!("UPDATE change_requests SET status='approved' WHERE id='{cr}'"),
    )
    .await;
    exec(db,&format!("SET horbyta.actor='{actor}'; SET horbyta.actor_kind='client'; UPDATE milestones SET due_date=due_date+interval '1 day' WHERE id='{milestone}'")).await;
    fails(
        db,
        &format!("UPDATE change_requests SET status='approved' WHERE id='{cr}'"),
    )
    .await;
    assert_eq!(value(db, "SELECT count(*)::int AS data FROM invoices").await, 0);
    exec(db,&format!("SET horbyta.actor='{owner}'; SET horbyta.actor_kind='professional'; SET horbyta.publish_offer='true'; UPDATE change_requests SET status='assessed' WHERE id='{cr}'; SET horbyta.publish_offer='false'; SET horbyta.actor='{actor}'; SET horbyta.actor_kind='client'; UPDATE change_requests SET status='approved' WHERE id='{cr}'")).await;
    assert_eq!(
        value(db, &format!("SELECT scope_task_allowed('{extra}') AS data")).await,
        true
    );
    assert_eq!(value(db, "SELECT count(*)::int AS data FROM invoices").await, 1);
    assert_eq!(
        value(
            db,
            &format!("SELECT extract(day from due_date)::int AS data FROM milestones WHERE id='{milestone}'")
        )
        .await,
        4
    );
    exec(
        db,
        &format!("UPDATE change_requests SET status='approved' WHERE id='{cr}'"),
    )
    .await;
    assert_eq!(value(db, "SELECT count(*)::int AS data FROM invoices").await, 1);
    fails(
        db,
        &format!("UPDATE change_requests SET impact_cost=99 WHERE id='{cr}'"),
    )
    .await;
    fails(
        db,
        &format!("UPDATE scope_change_revisions SET snapshot='{{}}' WHERE change_request_id='{cr}'"),
    )
    .await;
    fails(db, &format!("DELETE FROM change_requests WHERE id='{cr}'")).await;
    fails(db, "UPDATE invoices SET amount=999").await;
    fails(db,&format!("INSERT INTO time_entries(user_id,task_id,activity,duration_minutes,billable) VALUES('{owner}','{extra}','{activity}',10,true)")).await;
    exec(db,&format!("INSERT INTO time_entries(user_id,task_id,activity,duration_minutes,billable) VALUES('{owner}','{extra}','{activity}',10,false); UPDATE tasks SET status='in_progress' WHERE id='{extra}'")).await;
    fails(
        db,
        &format!("UPDATE change_requests SET status='implemented' WHERE id='{cr}'"),
    )
    .await;
    exec(db,&format!("UPDATE tasks SET status='done' WHERE id='{extra}';UPDATE change_requests SET status='implemented' WHERE id='{cr}'")).await;
    // Moneda histórica y pagos reales siguen separados.
    exec(
        db,
        &format!("UPDATE project_billing SET currency_type='EUR' WHERE project_id='{project}'"),
    )
    .await;
    assert_eq!(
        value(
            db,
            &format!("SELECT currency_type AS data FROM scope_effects WHERE change_request_id='{cr}'")
        )
        .await,
        "USD"
    );
    assert_eq!(
        ScopeService::change(db, owner, Some(client), cr).await.unwrap()["effect"]["paid_amount"],
        "0"
    );
    assert_eq!(
        ScopeService::project(db, owner, Some(client), project).await.unwrap()["revisions"][0]["billing_snapshot"]["currency_type"],
        "USD"
    );
    assert_eq!(
        ScopeService::project(db, owner, Some(client), project).await.unwrap()["revisions"][0]["billing_snapshot"]["fixed_price"],
        "100.0000"
    );
    // Rechazo y costo/días cero: ningún efecto sobre presupuesto o plazo.
    let rejected = Uuid::now_v7();
    let zero = Uuid::now_v7();
    exec(db,&format!("SET horbyta.actor='{owner}'; SET horbyta.actor_kind='professional';INSERT INTO change_requests(id,user_id,project_id,client_id,title,client_request_description,status) VALUES('{rejected}','{owner}','{project}','{client}','Rechazar','Extra','submitted'),('{zero}','{owner}','{project}','{client}','Gratis','Extra sin costo','submitted');UPDATE change_requests SET technical_assessment='Sin costo ni plazo',status='assessed' WHERE id IN ('{rejected}','{zero}');SET horbyta.actor='{actor}';SET horbyta.actor_kind='client';UPDATE change_requests SET status='declined',decline_reason='not_necessary_anymore' WHERE id='{rejected}';UPDATE change_requests SET status='approved' WHERE id='{zero}'")).await;
    assert_eq!(value(db, "SELECT count(*)::int AS data FROM invoices").await, 1);
    assert_eq!(
        value(
            db,
            &format!("SELECT count(*)::int AS data FROM scope_effects WHERE change_request_id='{rejected}'")
        )
        .await,
        0
    );
    // Fallo dentro de la aplicación de efectos: todo revierte.
    let failed = Uuid::now_v7();
    exec(db,&format!("SET horbyta.actor='{owner}';SET horbyta.actor_kind='professional';INSERT INTO change_requests(id,user_id,project_id,client_id,title,client_request_description,status) VALUES('{failed}','{owner}','{project}','{client}','Fallo','Prueba de rollback','submitted');UPDATE change_requests SET technical_assessment='Dos días',impact_cost=10,schedule_impact_days=2,target_milestone_id='{milestone}',status='assessed' WHERE id='{failed}';CREATE FUNCTION reject_test_invoice() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'fallo de prueba'; END; $$;CREATE TRIGGER reject_test_invoice BEFORE INSERT ON invoices FOR EACH ROW EXECUTE FUNCTION reject_test_invoice();SET horbyta.actor='{actor}';SET horbyta.actor_kind='client'")).await;
    fails(
        db,
        &format!("UPDATE change_requests SET status='approved' WHERE id='{failed}'"),
    )
    .await;
    assert_eq!(
        value(
            db,
            &format!("SELECT status AS data FROM change_requests WHERE id='{failed}'")
        )
        .await,
        "assessed"
    );
    assert_eq!(
        value(
            db,
            &format!("SELECT extract(day from due_date)::int AS data FROM milestones WHERE id='{milestone}'")
        )
        .await,
        4
    );
    exec(
        db,
        "DROP TRIGGER reject_test_invoice ON invoices; DROP FUNCTION reject_test_invoice()",
    )
    .await;
    assert_eq!(
        value(
            db,
            &format!("SELECT count(*)::int AS data FROM scope_effects WHERE change_request_id='{failed}'")
        )
        .await,
        0
    );
    // Las dos transacciones ven el mismo estado: solo una decisión puede persistir.
    let race = Uuid::now_v7();
    exec(db,&format!("SET horbyta.actor='{owner}';SET horbyta.actor_kind='professional';INSERT INTO change_requests(id,user_id,project_id,client_id,title,client_request_description,status) VALUES('{race}','{owner}','{project}','{client}','Carrera','Decisiones simultáneas','submitted');UPDATE change_requests SET technical_assessment='Sin costo',status='assessed' WHERE id='{race}'")).await;
    let url = std::env::var("SCOPE_TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap();
    let mut options = ConnectOptions::new(url);
    options.max_connections(1).sqlx_logging(false);
    let a = Database::connect(options.clone()).await.unwrap();
    let b = Database::connect(options).await.unwrap();
    exec(&a, &format!("SET search_path TO {schema}, public")).await;
    exec(&b, &format!("SET search_path TO {schema}, public")).await;
    let barrier = std::sync::Arc::new(tokio::sync::Barrier::new(2));
    let vote_a = async {
        let tx = a.begin().await.unwrap();
        ScopeService::actor(&tx, actor, "client").await.unwrap();
        barrier.wait().await;
        let result = horbyta::services::change_requests::ChangeRequestService::client_approve(
            &tx,
            owner,
            client,
            race,
            horbyta::dtos::change_request::ClientApproveChangeRequestDto {
                version: 2,
                client_signature_ip: None,
            },
        )
        .await;
        if result.is_ok() {
            tx.commit().await.unwrap();
        } else {
            tx.rollback().await.unwrap();
        }
        result
    };
    let vote_b = async {
        let tx = b.begin().await.unwrap();
        ScopeService::actor(&tx, actor, "client").await.unwrap();
        barrier.wait().await;
        let result = horbyta::services::change_requests::ChangeRequestService::client_decline(
            &tx,
            owner,
            client,
            race,
            horbyta::dtos::change_request::ClientDeclineChangeRequestDto {
                version: 2,
                decline_reason: horbyta::dtos::change_request::ChangeRequestDeclineReason::NotNecessaryAnymore,
            },
        )
        .await;
        if result.is_ok() {
            tx.commit().await.unwrap();
        } else {
            tx.rollback().await.unwrap();
        }
        result
    };
    let (r1, r2) = tokio::join!(vote_a, vote_b);
    assert_ne!(r1.is_ok(), r2.is_ok());
    assert_eq!(value(db,&format!("SELECT count(*)::int AS data FROM scope_events WHERE change_request_id='{race}' AND kind IN ('approved','declined')")).await,1);
    assert_eq!(
        value(
            db,
            &format!("SELECT version AS data FROM change_requests WHERE id='{race}'")
        )
        .await,
        3
    );
    // Publicar una sustitución y aceptar la oferta vista compiten por la misma versión.
    let replacement = Uuid::now_v7();
    exec(db,&format!("SET horbyta.actor='{owner}';SET horbyta.actor_kind='professional';INSERT INTO change_requests(id,user_id,project_id,client_id,title,client_request_description,status) VALUES('{replacement}','{owner}','{project}','{client}','Sustitución concurrente','Oferta versionada','submitted');UPDATE change_requests SET technical_assessment='Primera oferta',status='assessed' WHERE id='{replacement}'")).await;
    let publish = async {
        let tx = a.begin().await.unwrap();
        ScopeService::actor(&tx, owner, "professional").await.unwrap();
        barrier.wait().await;
        let result = horbyta::services::change_requests::ChangeRequestService::assess(
            &tx,
            owner,
            replacement,
            horbyta::dtos::change_request::AssessChangeRequestDto {
                version: 2,
                target_milestone_id: None,
                technical_assessment: "Oferta sustituida".into(),
                impact_hours: sea_orm::prelude::Decimal::ZERO,
                impact_cost: sea_orm::prelude::Decimal::ZERO,
                schedule_impact_days: 0,
            },
        )
        .await;
        if result.is_ok() {
            tx.commit().await.unwrap();
        } else {
            tx.rollback().await.unwrap();
        }
        result
    };
    let accept = async {
        let tx = b.begin().await.unwrap();
        ScopeService::actor(&tx, actor, "client").await.unwrap();
        barrier.wait().await;
        let result = horbyta::services::change_requests::ChangeRequestService::client_approve(
            &tx,
            owner,
            client,
            replacement,
            horbyta::dtos::change_request::ClientApproveChangeRequestDto {
                version: 2,
                client_signature_ip: None,
            },
        )
        .await;
        if result.is_ok() {
            tx.commit().await.unwrap();
        } else {
            tx.rollback().await.unwrap();
        }
        result
    };
    let (published, accepted) = tokio::join!(publish, accept);
    assert_ne!(published.is_ok(), accepted.is_ok());
    assert_eq!(
        value(
            db,
            &format!("SELECT version AS data FROM change_requests WHERE id='{replacement}'")
        )
        .await,
        3
    );
    exec(db, &format!("SET horbyta.actor='{owner}';SET horbyta.actor_kind='professional';UPDATE change_requests SET status='cancelled' WHERE id='{replacement}' AND status='assessed'")).await;
    a.close().await.unwrap();
    b.close().await.unwrap();
    // Las subtareas heredan bloqueo y origen. Un alta con estado avanzado tampoco elude el control.
    let pending_task = Uuid::now_v7();
    let child = Uuid::now_v7();
    exec(db,&format!("INSERT INTO tasks(id,user_id,project_id,title) VALUES('{pending_task}','{owner}','{project}','Extra pendiente');INSERT INTO scope_task_links(task_id,project_id,change_request_id,reason) VALUES('{pending_task}','{project}','{failed}','Extra pendiente de la oferta');INSERT INTO tasks(id,user_id,project_id,parent_id,title) VALUES('{child}','{owner}','{project}','{pending_task}','Subtarea')")).await;
    assert_eq!(
        value(db, &format!("SELECT scope_task_allowed('{child}') AS data")).await,
        false
    );
    fails(
        db,
        &format!("INSERT INTO tasks(user_id,project_id,title,status) VALUES('{owner}','{project}','Evasión','done')"),
    )
    .await;
    fails(db, &format!("UPDATE tasks SET status='review' WHERE id='{child}'")).await;
    // Un adjunto conservado no puede borrarse por metadata ni por cascada.
    let attachment = Uuid::now_v7();
    exec(db,&format!("INSERT INTO attachments(id,user_id,project_id,change_request_id,original_name,mime_type,storage_key,size_bytes) VALUES('{attachment}','{owner}','{project}','{failed}','acuerdo.txt','text/plain','scope-test.txt',10);SELECT scope_capture_attachments('{project}','{failed}')")).await;
    fails(db, &format!("DELETE FROM attachments WHERE id='{attachment}'")).await;
    assert_eq!(
        ScopeService::pending(db, owner, client)
            .await
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    fails(
        db,
        &format!("UPDATE attachments SET storage_key='reemplazo.txt' WHERE id='{attachment}'"),
    )
    .await;
    // Precio fijo del extra en las tres modalidades, sin modificar la modalidad original.
    for mode in ["hourly", "retainer"] {
        let supplemental = Uuid::now_v7();
        exec(db,&format!("UPDATE project_billing SET billing_type='{mode}',fixed_price=NULL,hourly_rate=CASE WHEN '{mode}'='hourly' THEN 25 ELSE NULL END,retainer_amount=CASE WHEN '{mode}'='retainer' THEN 200 ELSE NULL END,payment_day=CASE WHEN '{mode}'='retainer' THEN 10 ELSE NULL END WHERE project_id='{project}';SET horbyta.actor='{owner}';SET horbyta.actor_kind='professional';INSERT INTO change_requests(id,user_id,project_id,client_id,title,client_request_description,status) VALUES('{supplemental}','{owner}','{project}','{client}','Suplemento por modalidad','Extra independiente','submitted');UPDATE change_requests SET technical_assessment='Precio fijo del extra',impact_hours=1,impact_cost=12.5,status='assessed' WHERE id='{supplemental}';SET horbyta.actor='{actor}';SET horbyta.actor_kind='client';UPDATE change_requests SET status='approved' WHERE id='{supplemental}'")).await;
        assert_eq!(
            value(
                db,
                &format!("SELECT amount::text AS data FROM scope_effects WHERE change_request_id='{supplemental}'")
            )
            .await,
            "12.5000"
        );
        assert_eq!(
            value(
                db,
                &format!("SELECT billing_type AS data FROM project_billing WHERE project_id='{project}'")
            )
            .await,
            mode
        );
    }
    // La entrega fallida no altera el acuerdo y se puede reintentar sin duplicados.
    exec(db,"CREATE FUNCTION reject_test_notice() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'fallo de entrega de prueba'; END; $$;CREATE TRIGGER reject_test_notice BEFORE INSERT ON notifications FOR EACH ROW EXECUTE FUNCTION reject_test_notice()").await;
    assert!(horbyta::services::scope_delivery::deliver_one(db).await.is_err());
    assert_eq!(
        value(
            db,
            &format!("SELECT status AS data FROM change_requests WHERE id='{cr}'")
        )
        .await,
        "implemented"
    );
    exec(db,"DROP TRIGGER reject_test_notice ON notifications;DROP FUNCTION reject_test_notice();UPDATE scope_delivery SET next_attempt_at=now()").await;
    while horbyta::services::scope_delivery::deliver_one(db)
        .await
        .unwrap()
        .is_some()
    {}
    assert_eq!(
        value(db, "SELECT count(*)::int AS data FROM notifications").await,
        value(
            db,
            "SELECT count(*)::int AS data FROM scope_delivery WHERE delivered_at IS NOT NULL"
        )
        .await
    );
    assert!(
        horbyta::services::scope_delivery::deliver_one(db)
            .await
            .unwrap()
            .is_none()
    );
    // Facturas y cobros anteriores permanecen intactos; aprobar no equivale a cobrar.
    let previous_invoice = Uuid::now_v7();
    let previous_payment = Uuid::now_v7();
    let financial = Uuid::now_v7();
    exec(db, &format!("INSERT INTO invoices(id,user_id,project_id,amount,currency_type,status) VALUES('{previous_invoice}','{owner}','{project}',100,'USD','paid');INSERT INTO payments(id,user_id,project_id,invoice_id,amount,currency_type) VALUES('{previous_payment}','{owner}','{project}','{previous_invoice}',100,'USD')")).await;
    let before_invoice = value(
        db,
        &format!("SELECT to_jsonb(i) AS data FROM invoices i WHERE id='{previous_invoice}'"),
    )
    .await;
    let before_payment = value(
        db,
        &format!("SELECT to_jsonb(p) AS data FROM payments p WHERE id='{previous_payment}'"),
    )
    .await;
    exec(db, &format!("SET horbyta.actor='{owner}';SET horbyta.actor_kind='professional';INSERT INTO change_requests(id,user_id,project_id,client_id,title,client_request_description,status) VALUES('{financial}','{owner}','{project}','{client}','Finanzas conservadas','Suplemento separado','submitted');UPDATE change_requests SET technical_assessment='Suplemento',impact_cost=11,status='assessed' WHERE id='{financial}';SET horbyta.actor='{actor}';SET horbyta.actor_kind='client';UPDATE change_requests SET status='approved' WHERE id='{financial}'")).await;
    assert_eq!(
        value(
            db,
            &format!("SELECT to_jsonb(i) AS data FROM invoices i WHERE id='{previous_invoice}'")
        )
        .await,
        before_invoice
    );
    assert_eq!(
        value(
            db,
            &format!("SELECT to_jsonb(p) AS data FROM payments p WHERE id='{previous_payment}'")
        )
        .await,
        before_payment
    );
    assert_eq!(
        ScopeService::change(db, owner, Some(client), financial).await.unwrap()["effect"]["paid_amount"],
        "0"
    );
    negotiation(db, owner, client, actor, activity).await;
    documentary_assets(db, owner, client, actor).await;
    let conversation_project = conversation_windows(db, owner, client, actor).await;
    change_conversation(db, owner, client, actor, project, cr).await;
    if std::env::var("KEEP_SCOPE_TEST_SCHEMA").as_deref() == Ok("1") {
        std::fs::create_dir_all("target/scope-tests").unwrap();
        std::fs::write("target/scope-tests/state.json",serde_json::to_vec_pretty(&json!({"conversation_project":conversation_project,"schema":schema,"owner":owner,"client":client,"actor":actor,"project":project,"base_task":base_task,"extra":extra,"change":cr,"pending_change":failed,"milestone":milestone,"email":"scope-tests@horbyta.test","client_email":"scope-client@horbyta.test","password":password})).unwrap()).unwrap();
    }
}

async fn negotiation(db: &DatabaseConnection, owner: Uuid, client: Uuid, actor: Uuid, activity: Uuid) {
    let project = Uuid::now_v7();
    let work = Uuid::now_v7();
    let prep = Uuid::now_v7();
    exec(db,&format!("INSERT INTO projects(id,user_id,client_id,name) VALUES('{project}','{owner}','{client}','Negociación documental');INSERT INTO tasks(id,user_id,project_id,title) VALUES('{work}','{owner}','{project}','Ejecución pendiente'),('{prep}','{owner}','{project}','Preparar propuesta')")).await;
    assert_eq!(
        value(db, &format!("SELECT scope_task_allowed('{work}') AS data")).await,
        false
    );
    fails(db, &format!("UPDATE tasks SET status='in_progress' WHERE id='{work}'")).await;
    fails(db,&format!("INSERT INTO time_entries(user_id,task_id,activity,duration_minutes,billable) VALUES('{owner}','{work}','{activity}',1,false)")).await;
    ScopeService::preparation(
        db,
        owner,
        prep,
        "Estimar y redactar propuesta; no desarrollar entregables",
    )
    .await
    .unwrap();
    exec(db, &format!("UPDATE tasks SET status='in_progress' WHERE id='{prep}'")).await;
    assert_eq!(
        value(db, &format!("SELECT scope_task_allowed('{prep}') AS data")).await,
        true
    );
    fails(
        db,
        &format!("UPDATE tasks SET scope_preparation_reason=NULL WHERE id='{prep}'"),
    )
    .await;
    fails(
        db,
        &format!("UPDATE projects SET enforce_scope=false WHERE id='{project}'"),
    )
    .await;
    fails(db,&format!("INSERT INTO time_entries(user_id,task_id,activity,duration_minutes,billable) VALUES('{owner}','{prep}','{activity}',1,true)")).await;
    exec(db,&format!("INSERT INTO time_entries(user_id,task_id,activity,duration_minutes,billable) VALUES('{owner}','{prep}','{activity}',1,false)")).await;
    let doc_id = Uuid::now_v7();
    let full = |content: &str| ScopeDocument {
        documents: vec![ScopeContentDocument {
            id: doc_id,
            title: "Especificación completa".into(),
            role: "main".into(),
            section: None,
            content: content.into(),
            source_name: Some("alcance.md".into()),
            source_content: Some("# Alcance original\nSin tienda".into()),
            requirement_id: Some("REQ-001".into()),
        }],
        precedence: "Las exclusiones del documento principal prevalecen; anexos solo detallan lo incluido.".into(),
        ..Default::default()
    };
    let tx = db.begin().await.unwrap();
    ScopeService::publish(
        &tx,
        owner,
        project,
        PublishScopeDto {
            expected_revision: 0,
            publish: true,
            document: full("# Alcance original\nSin tienda"),
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    let tx = db.begin().await.unwrap();
    ScopeService::feedback(
        &tx,
        owner,
        Some(client),
        actor,
        project,
        ScopeFeedbackDto {
            revision: 1,
            kind: "request_changes".into(),
            text: "Aclarar el inicio de sesión".into(),
            document_id: Some(doc_id),
            attachment_ids: vec![],
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    let tx = db.begin().await.unwrap();
    assert!(
        ScopeService::accept(&tx, owner, client, actor, project, 1)
            .await
            .is_err()
    );
    tx.rollback().await.unwrap();
    let tx = db.begin().await.unwrap();
    ScopeService::publish(
        &tx,
        owner,
        project,
        PublishScopeDto {
            expected_revision: 1,
            publish: true,
            document: full("# Alcance revisado\nAcceso y tienda"),
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    let tx = db.begin().await.unwrap();
    ScopeService::feedback(
        &tx,
        owner,
        Some(client),
        actor,
        project,
        ScopeFeedbackDto {
            revision: 1,
            kind: "prefer".into(),
            text: "Prefiero volver al alcance sin tienda".into(),
            document_id: None,
            attachment_ids: vec![],
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    let mut restored = full("# Alcance original\nSin tienda");
    restored.based_on = Some(1);
    let tx = db.begin().await.unwrap();
    ScopeService::publish(
        &tx,
        owner,
        project,
        PublishScopeDto {
            expected_revision: 2,
            publish: true,
            document: restored,
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    let tx = db.begin().await.unwrap();
    assert!(
        ScopeService::accept(&tx, owner, client, actor, project, 2)
            .await
            .is_err()
    );
    tx.rollback().await.unwrap();
    let tx = db.begin().await.unwrap();
    exec_tx_ip(&tx).await;
    ScopeService::accept(&tx, owner, client, actor, project, 3)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let state = ScopeService::project(db, owner, Some(client), project).await.unwrap();
    assert_eq!(state["accepted_revision"], 3);
    assert_eq!(state["revisions"][0]["status"], "accepted");
    assert_eq!(state["revisions"][1]["status"], "superseded");
    assert_eq!(state["revisions"][2]["status"], "changes_requested");
    assert_eq!(state["revisions"][0]["document"]["documents"][0]["version"], 3);
    assert_eq!(
        state["revisions"][0]["document"]["documents"][0]["source_content"],
        "# Alcance original\nSin tienda"
    );
    assert!(
        state["revisions"][0]["bundle_hash"]
            .as_str()
            .is_some_and(|s| s.len() == 64)
    );
    assert_eq!(state["revisions"][0]["accepted_ip"], "203.0.113.7");
    assert_eq!(
        value(db, &format!("SELECT scope_task_allowed('{work}') AS data")).await,
        true
    );
    ScopeService::requirement(db, owner, work, doc_id, 3).await.unwrap();
    fails(
        db,
        &format!("DELETE FROM scope_task_requirements WHERE task_id='{work}'"),
    )
    .await;
    fails(
        db,
        &format!("UPDATE scope_revisions SET accepted_ip='203.0.113.8' WHERE project_id='{project}' AND revision=3"),
    )
    .await;
    fails(
        db,
        &format!("DELETE FROM scope_project_events WHERE project_id='{project}'"),
    )
    .await;
    let invalid = full("![Diagrama](imagenes/diagrama.png)");
    let tx = db.begin().await.unwrap();
    assert!(
        ScopeService::publish(
            &tx,
            owner,
            project,
            PublishScopeDto {
                expected_revision: 3,
                publish: true,
                document: invalid
            }
        )
        .await
        .is_err()
    );
    tx.rollback().await.unwrap();
}
async fn documentary_assets(db: &DatabaseConnection, owner: Uuid, client: Uuid, actor: Uuid) {
    use object_store::{ObjectStoreExt, memory::InMemory, path::Path};
    let storage = InMemory::new();
    let project = Uuid::now_v7();
    let attachment = Uuid::now_v7();
    let unselected = Uuid::now_v7();
    exec(db,&format!("INSERT INTO projects(id,user_id,client_id,name) VALUES('{project}','{owner}','{client}','Anexos elegidos');INSERT INTO attachments(id,user_id,project_id,original_name,mime_type,storage_key,size_bytes) VALUES('{attachment}','{owner}','{project}','figura.txt','text/plain','figura.txt',8),('{unselected}','{owner}','{project}','boceto.txt','text/plain','boceto.txt',3)")).await;
    storage
        .put(&Path::from("figura.txt"), b"original".to_vec().into())
        .await
        .unwrap();
    let mut doc = document();
    doc.attachment_ids = vec![attachment];
    doc.requirements = format!("Requisito [Figura](/api/v1/attachments/{attachment}/raw)");
    let tx = db.begin().await.unwrap();
    ScopeService::publish_with_storage(
        &tx,
        owner,
        project,
        PublishScopeDto {
            expected_revision: 0,
            publish: true,
            document: doc,
        },
        Some(&storage),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    let state = ScopeService::project(db, owner, Some(client), project).await.unwrap();
    assert_eq!(
        state["revisions"][0]["attachment_snapshot"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        value(
            db,
            &format!(
                "SELECT count(*)::int AS data FROM scope_preserved_attachments WHERE attachment_id='{unselected}'"
            )
        )
        .await,
        0
    );
    ScopeService::verify_bundle_assets(db, &storage, owner, client, project, 1)
        .await
        .unwrap();
    storage
        .put(&Path::from("figura.txt"), b"alterado".to_vec().into())
        .await
        .unwrap();
    assert!(
        ScopeService::verify_bundle_assets(db, &storage, owner, client, project, 1)
            .await
            .is_err()
    );
    storage
        .put(&Path::from("figura.txt"), b"original".to_vec().into())
        .await
        .unwrap();
    let tx = db.begin().await.unwrap();
    ScopeService::feedback(
        &tx,
        owner,
        Some(client),
        actor,
        project,
        ScopeFeedbackDto {
            revision: 1,
            kind: "reject".into(),
            text: "No deseo continuar con esta propuesta".into(),
            document_id: None,
            attachment_ids: vec![],
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    let tx = db.begin().await.unwrap();
    assert!(
        ScopeService::accept(&tx, owner, client, actor, project, 1)
            .await
            .is_err()
    );
    tx.rollback().await.unwrap();
    assert_eq!(
        ScopeService::project(db, owner, Some(client), project).await.unwrap()["revisions"][0]["status"],
        "rejected"
    );
}
async fn exec_tx_ip(tx: &sea_orm::DatabaseTransaction) {
    tx.execute_unprepared(
        "SELECT set_config('horbyta.accepted_ip','203.0.113.7',true),set_config('horbyta.ip_source','socket',true)",
    )
    .await
    .unwrap();
}

async fn conversation_windows(db: &DatabaseConnection, owner: Uuid, client: Uuid, actor: Uuid) -> Uuid {
    use horbyta::services::scope_conversation::{authorize, window};
    let project = Uuid::now_v7();
    exec(db, &format!("INSERT INTO projects(id,user_id,client_id,name) VALUES('{project}','{owner}','{client}','Ventanas de conversación')")).await;
    let tx = db.begin().await.unwrap();
    ScopeService::publish(
        &tx,
        owner,
        project,
        PublishScopeDto {
            expected_revision: 0,
            publish: true,
            document: document(),
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    exec(db, &format!("INSERT INTO scope_project_events(project_id,revision,kind,text,actor_id,actor_kind) SELECT '{project}',1,'comment','Mensaje '||n,'{owner}','professional' FROM generate_series(1,250) n")).await;
    let latest = window(db, owner, None, project, 1, ConversationWindowQuery::default())
        .await
        .unwrap();
    assert_eq!(latest["items"].as_array().unwrap().len(), 100);
    assert_eq!(latest["items"][0]["text"], "Mensaje 151");
    assert_eq!(latest["items"][99]["text"], "Mensaje 250");
    assert_eq!(latest["has_older"], true);
    assert_eq!(latest["has_newer"], false);
    let before = Uuid::parse_str(latest["items"][0]["id"].as_str().unwrap()).unwrap();
    let older = window(
        db,
        owner,
        Some(client),
        project,
        1,
        ConversationWindowQuery {
            around: None,
            notification: None,
            before: Some(before),
            after: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(older["items"].as_array().unwrap().len(), 100);
    let anchor: Uuid = older["items"][49]["id"].as_str().unwrap().parse().unwrap();
    let focused = window(
        db,
        owner,
        Some(client),
        project,
        1,
        ConversationWindowQuery {
            around: Some(anchor),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(focused["focus_id"], anchor.to_string());
    assert_eq!(focused["items"].as_array().unwrap().len(), 100);
    assert!(
        focused["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["id"] == anchor.to_string())
    );
    assert!(
        window(
            db,
            owner,
            None,
            project,
            1,
            ConversationWindowQuery {
                around: Some(Uuid::now_v7()),
                ..Default::default()
            }
        )
        .await
        .is_err()
    );
    assert_eq!(older["items"][0]["text"], "Mensaje 51");
    assert_eq!(older["items"][99]["text"], "Mensaje 150");
    assert_eq!(older["has_newer"], true);
    let after = Uuid::parse_str(older["items"][99]["id"].as_str().unwrap()).unwrap();
    let newer = window(
        db,
        owner,
        None,
        project,
        1,
        ConversationWindowQuery {
            around: None,
            notification: None,
            before: None,
            after: Some(after),
        },
    )
    .await
    .unwrap();
    assert_eq!(newer["items"], latest["items"]);
    let first = window(
        db,
        owner,
        None,
        project,
        1,
        ConversationWindowQuery {
            around: None,
            notification: None,
            before: Some(Uuid::parse_str(older["items"][0]["id"].as_str().unwrap()).unwrap()),
            after: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(first["items"].as_array().unwrap().len(), 50);
    assert_eq!(first["has_older"], false);
    assert!(authorize(db, Uuid::now_v7(), None, project, 1).await.is_err());
    assert!(authorize(db, owner, Some(Uuid::now_v7()), project, 1).await.is_err());
    assert!(
        window(
            db,
            owner,
            None,
            project,
            1,
            ConversationWindowQuery {
                around: None,
                notification: None,
                before: Some(before),
                after: Some(after)
            }
        )
        .await
        .is_err()
    );
    assert_eq!(
        ScopeService::project(db, owner, None, project).await.unwrap()["events"]
            .as_array()
            .unwrap()
            .len(),
        100
    );
    let file = Uuid::now_v7();
    exec(db,&format!("INSERT INTO attachments(id,user_id,project_id,original_name,storage_key,mime_type,size_bytes) VALUES('{file}','{owner}','{project}','nota.md','nota-{file}','text/markdown',10)")).await;
    let tx = db.begin().await.unwrap();
    ScopeService::feedback(
        &tx,
        owner,
        Some(client),
        actor,
        project,
        ScopeFeedbackDto {
            revision: 1,
            kind: "comment".into(),
            text: String::new(),
            document_id: None,
            attachment_ids: vec![file],
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    let result = window(db, owner, None, project, 1, ConversationWindowQuery::default())
        .await
        .unwrap();
    assert_eq!(result["items"][99]["text"], "nota.md");
    assert_eq!(result["items"][99]["actor_name"], "Cliente de pruebas");
    assert_eq!(result["items"][99]["attachments"][0]["id"], file.to_string());
    let legacy = Uuid::now_v7();
    exec(db, &format!("INSERT INTO notifications(id,user_id,preference_key,event_type,title,body,payload) VALUES('{legacy}','{owner}','client_portal.change_request_updated','client_portal.change_request_updated','Aviso anterior','nota.md',jsonb_build_object('project_id','{project}','revision',1))")).await;
    let legacy_window = window(
        db,
        owner,
        None,
        project,
        1,
        ConversationWindowQuery {
            notification: Some(legacy),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(legacy_window["focus_id"], result["items"][99]["id"]);
    assert!(legacy_window["items"].as_array().unwrap().len() <= 100);
    let invalid_notice = window(
        db,
        owner,
        None,
        project,
        1,
        ConversationWindowQuery {
            notification: Some(Uuid::now_v7()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert!(invalid_notice["focus_id"].is_null());
    fails(db, &format!("DELETE FROM attachments WHERE id='{file}'")).await;
    let foreign_file = Uuid::now_v7();
    let foreign_project = Uuid::now_v7();
    exec(db,&format!("INSERT INTO projects(id,user_id,client_id,name) VALUES('{foreign_project}','{owner}','{client}','Otro proyecto del laboratorio')")).await;
    exec(db,&format!("INSERT INTO attachments(id,user_id,project_id,original_name,storage_key,mime_type,size_bytes) VALUES('{foreign_file}','{owner}','{foreign_project}','externo.txt','ext-{foreign_file}','text/plain',1)")).await;
    let tx = db.begin().await.unwrap();
    assert!(
        ScopeService::feedback(
            &tx,
            owner,
            None,
            owner,
            project,
            ScopeFeedbackDto {
                revision: 1,
                kind: "comment".into(),
                text: "Archivo ajeno".into(),
                document_id: None,
                attachment_ids: vec![foreign_file]
            }
        )
        .await
        .is_err()
    );
    tx.rollback().await.unwrap();
    let audio = Uuid::now_v7();
    exec(db,&format!("INSERT INTO attachments(id,user_id,project_id,original_name,storage_key,mime_type,size_bytes) VALUES('{audio}','{owner}','{project}','voz.mp3','audio-{audio}','audio/mpeg',1)")).await;
    let tx = db.begin().await.unwrap();
    assert!(
        ScopeService::feedback(
            &tx,
            owner,
            None,
            owner,
            project,
            ScopeFeedbackDto {
                revision: 1,
                kind: "comment".into(),
                text: "Audio".into(),
                document_id: None,
                attachment_ids: vec![audio]
            }
        )
        .await
        .is_err()
    );
    tx.rollback().await.unwrap();
    project
}

async fn change_conversation(
    db: &DatabaseConnection,
    owner: Uuid,
    client: Uuid,
    actor: Uuid,
    project: Uuid,
    change: Uuid,
) {
    use horbyta::services::scope_conversation::change_window;
    let version = value(
        db,
        &format!("SELECT version AS data FROM change_requests WHERE id='{change}'"),
    )
    .await
    .as_i64()
    .unwrap() as i32;
    exec(db,&format!("INSERT INTO scope_events(change_request_id,version,kind,actor_id,actor_kind,body) SELECT '{change}',{version},'comment','{owner}','professional',jsonb_build_object('text','Cambio '||n) FROM generate_series(1,250) n")).await;
    let latest = change_window(db, owner, None, change, version, ConversationWindowQuery::default())
        .await
        .unwrap();
    assert_eq!(latest["items"].as_array().unwrap().len(), 100);
    assert_eq!(latest["items"][0]["text"], "Cambio 151");
    assert_eq!(latest["items"][99]["actor_name"], "Pruebas de alcance");
    assert_eq!(latest["has_older"], true);
    assert_eq!(latest["has_newer"], false);
    let first = latest["items"][0]["id"].as_str().unwrap().parse().unwrap();
    let older = change_window(
        db,
        owner,
        Some(client),
        change,
        version,
        ConversationWindowQuery {
            before: Some(first),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(older["items"].as_array().unwrap().len(), 100);
    assert_eq!(older["items"][0]["text"], "Cambio 51");
    let anchor = older["items"][25]["id"].as_str().unwrap().parse().unwrap();
    let around = change_window(
        db,
        owner,
        None,
        change,
        version,
        ConversationWindowQuery {
            around: Some(anchor),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(around["focus_id"], anchor.to_string());
    assert_eq!(around["items"].as_array().unwrap().len(), 100);
    assert!(
        change_window(
            db,
            Uuid::now_v7(),
            None,
            change,
            version,
            ConversationWindowQuery::default()
        )
        .await
        .is_err()
    );
    assert!(
        change_window(
            db,
            owner,
            Some(Uuid::now_v7()),
            change,
            version,
            ConversationWindowQuery::default()
        )
        .await
        .is_err()
    );
    assert!(
        change_window(
            db,
            owner,
            None,
            change,
            version,
            ConversationWindowQuery {
                around: Some(Uuid::now_v7()),
                ..Default::default()
            }
        )
        .await
        .is_err()
    );
    let file = Uuid::now_v7();
    exec(db,&format!("INSERT INTO attachments(id,user_id,project_id,original_name,storage_key,mime_type,size_bytes) VALUES('{file}','{owner}','{project}','cambio.md','cambio-{file}','text/markdown',12)")).await;
    let tx = db.begin().await.unwrap();
    let sent = ScopeService::comment(
        &tx,
        owner,
        Some(client),
        actor,
        change,
        ScopeCommentDto {
            version,
            text: String::new(),
            attachment_ids: vec![file],
        },
    )
    .await
    .unwrap();
    assert_eq!(sent["_feedback_event"]["text"], "cambio.md");
    assert_eq!(sent["_feedback_event"]["actor_name"], "Cliente de pruebas");
    tx.commit().await.unwrap();
    let last = change_window(db, owner, None, change, version, ConversationWindowQuery::default())
        .await
        .unwrap();
    assert_eq!(last["items"][99]["attachments"][0]["id"], file.to_string());
    assert!(
        ScopeService::change(db, owner, None, change).await.unwrap()["events"]
            .as_array()
            .unwrap()
            .iter()
            .all(|e| e["kind"] != "comment")
    );
    fails(db, &format!("DELETE FROM attachments WHERE id='{file}'")).await;
    let audio = Uuid::now_v7();
    exec(db,&format!("INSERT INTO attachments(id,user_id,project_id,original_name,storage_key,mime_type,size_bytes) VALUES('{audio}','{owner}','{project}','voz.mp3','voz-{audio}','audio/mpeg',1)")).await;
    let tx = db.begin().await.unwrap();
    assert!(
        ScopeService::comment(
            &tx,
            owner,
            None,
            owner,
            change,
            ScopeCommentDto {
                version,
                text: "Audio".into(),
                attachment_ids: vec![audio]
            }
        )
        .await
        .is_err()
    );
    tx.rollback().await.unwrap();
    let tx = db.begin().await.unwrap();
    assert!(
        ScopeService::comment(
            &tx,
            owner,
            None,
            owner,
            change,
            ScopeCommentDto {
                version: version - 1,
                text: "Revisión antigua".into(),
                attachment_ids: vec![]
            }
        )
        .await
        .is_err()
    );
    tx.rollback().await.unwrap();
}
