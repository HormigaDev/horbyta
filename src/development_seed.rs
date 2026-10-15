//! Datos ficticios para desarrollo; nunca se ejecutan durante migraciones ni al arrancar.
use object_store::{ObjectStore, ObjectStoreExt, path::Path as ObjectPath};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbBackend, Statement, TransactionTrait, Value,
};
use serde_json::json;
use uuid::Uuid;

use crate::{dtos::scope::*, error::AppError, migrations, services::scope::ScopeService, utils::crypto::hash_password};

pub const ADMIN_EMAIL: &str = "admin@horbyta.test";
pub const ADMIN_PASSWORD: &str = "Admin@123";
pub const CLIENT_PASSWORD: &str = "Cliente@123";

fn conflict(message: &str) -> AppError {
    AppError::Conflict(message.into())
}

pub fn require_development(mode: &str) -> Result<(), AppError> {
    if mode != "development" {
        return Err(conflict("El seed solo admite APP_ENV=development."));
    }
    Ok(())
}

async fn execute(db: &impl ConnectionTrait, sql: &str, values: Vec<Value>) -> Result<(), AppError> {
    db.execute(Statement::from_sql_and_values(DbBackend::Postgres, sql, values))
        .await?;
    Ok(())
}

async fn scalar<T: sea_orm::TryGetable>(db: &impl ConnectionTrait, sql: &str) -> Result<T, AppError> {
    let row = db
        .query_one(Statement::from_string(DbBackend::Postgres, sql.to_owned()))
        .await?
        .ok_or(AppError::InternalError)?;
    Ok(row.try_get("", "value")?)
}

/// El reset y el seed son atómicos en PostgreSQL; los archivos nuevos se limpian si fallan.
pub async fn seed(
    db: &DatabaseConnection,
    storage: &dyn ObjectStore,
    mode: &str,
    reset: bool,
) -> Result<serde_json::Value, AppError> {
    require_development(mode)?;
    let database: String = scalar(db, "SELECT current_database() AS value").await?;
    if database != "hormigapm_development" {
        return Err(conflict("El seed está limitado a la base hormigapm_development."));
    }
    let tx = db.begin().await?;
    tx.execute_unprepared("SET LOCAL lock_timeout='5s'; SELECT pg_advisory_xact_lock(719327401)")
        .await?;
    let mut previous = Vec::<String>::new();
    if reset {
        let schema: String = scalar(&tx, "SELECT current_schema() AS value").await?;
        if schema != "public" {
            return Err(conflict("--reset solo reconstruye el esquema public de desarrollo."));
        }
        let exists: bool = scalar(&tx, "SELECT to_regclass('attachments') IS NOT NULL AS value").await?;
        if exists {
            for row in tx
                .query_all(Statement::from_string(
                    DbBackend::Postgres,
                    "SELECT storage_key FROM attachments WHERE storage_key LIKE 'development-seed/%'".to_owned(),
                ))
                .await?
            {
                previous.push(row.try_get("", "storage_key")?);
            }
        }
        tx.execute_unprepared(
            "DROP SCHEMA public CASCADE; CREATE SCHEMA public; GRANT USAGE ON SCHEMA public TO PUBLIC",
        )
        .await?;
    }
    migrations::apply(&tx).await?;
    let users: i64 = scalar(&tx, "SELECT count(*) AS value FROM users").await?;
    if users != 0 {
        return Err(conflict(
            "La base ya tiene usuarios. No se sobrescriben: use --reset únicamente si desea reemplazar los datos de desarrollo.",
        ));
    }
    let mut files = Vec::new();
    let result = populate(&tx, storage, &mut files).await;
    match result {
        Ok(report) => {
            if let Err(error) = tx.commit().await {
                for key in &files {
                    let _ = storage.delete(&ObjectPath::from(key.as_str())).await;
                }
                return Err(error.into());
            }
            for key in previous {
                let _ = storage.delete(&ObjectPath::from(key)).await;
            }
            Ok(report)
        }
        Err(error) => {
            tx.rollback().await?;
            for key in &files {
                let _ = storage.delete(&ObjectPath::from(key.as_str())).await;
            }
            Err(error)
        }
    }
}

async fn file(
    tx: &DatabaseTransaction,
    storage: &dyn ObjectStore,
    files: &mut Vec<String>,
    owner: Uuid,
    project: Uuid,
    name: &str,
    mime: &str,
    content: &str,
) -> Result<Uuid, AppError> {
    let id = Uuid::now_v7();
    let key = format!("development-seed/{owner}/{id}/{name}");
    storage
        .put(&ObjectPath::from(key.as_str()), content.as_bytes().to_vec().into())
        .await
        .map_err(|_| AppError::Internal("No se pudo guardar un archivo del seed.".into()))?;
    files.push(key.clone());
    execute(tx,"INSERT INTO attachments(id,user_id,project_id,original_name,storage_key,mime_type,size_bytes,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,now()-interval '20 days')",
        vec![id.into(),owner.into(),project.into(),name.into(),key.into(),mime.into(),(content.len() as i64).into()]).await?;
    Ok(id)
}

async fn populate(
    tx: &DatabaseTransaction,
    storage: &dyn ObjectStore,
    files: &mut Vec<String>,
) -> Result<serde_json::Value, AppError> {
    let owner = Uuid::now_v7();
    let hash = hash_password(ADMIN_PASSWORD.into())?;
    let client_hash = hash_password(CLIENT_PASSWORD.into())?;
    execute(tx,"INSERT INTO users(id,name,email,password_hash,role,preferences,created_at) VALUES($1,'Estudio Horizonte',$2,$3,'admin',$4,now()-interval '180 days')",
        vec![owner.into(),ADMIN_EMAIL.into(),hash.clone().into(),json!({"language":"es","timezone":"America/Sao_Paulo","theme":"system","max_daily_hours":8}).into()]).await?;
    for (name, email, status) in [
        ("Lucía Méndez", "lucia@horbyta.test", "active"),
        ("Diego Campos", "diego@horbyta.test", "inactive"),
        ("Cuenta de prueba bloqueada", "bloqueada@horbyta.test", "blocked"),
    ] {
        execute(tx,"INSERT INTO users(name,email,password_hash,status,created_at) VALUES($1,$2,$3,$4,now()-interval '90 days')",vec![name.into(),email.into(),hash.clone().into(),status.into()]).await?;
    }
    let category = Uuid::now_v7();
    execute(
        tx,
        "INSERT INTO categories(id,user_id,name) VALUES($1,$2,'Clientes del estudio')",
        vec![category.into(), owner.into()],
    )
    .await?;
    execute(
        tx,
        "INSERT INTO categories(user_id,name) VALUES($1,'Productos digitales'),($1,'Operaciones internas')",
        vec![owner.into()],
    )
    .await?;
    execute(tx,"INSERT INTO tags(user_id,name,color) VALUES($1,'Cliente','#6366f1'),($1,'Accesibilidad','#10b981'),($1,'Documentación','#f59e0b')",vec![owner.into()]).await?;
    let activity_rows = tx
        .query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT id FROM activities WHERE user_id=$1 ORDER BY name",
            [owner.into()],
        ))
        .await?;
    let activities: Vec<Uuid> = activity_rows
        .iter()
        .map(|row| row.try_get("", "id"))
        .collect::<Result<_, _>>()?;
    let activity: Uuid = scalar(
        tx,
        &format!("SELECT id AS value FROM activities WHERE user_id='{owner}' AND name='Development'"),
    )
    .await?;
    let mut clients = Vec::new();
    for (i, (name, company)) in [
        ("Clara Vidal", "Lumen Editorial"),
        ("Mateo Salas", "Norte Café"),
        ("Inés Duarte", "Verde Vivo"),
        ("Bruno Costa", "Atlas Consultoría"),
        ("Elena Ríos", "Aula Abierta"),
        ("Tomás Herrera", "Estación Creativa"),
        ("Sofía Luna", "Marea Hotel"),
        ("Pablo Soto", "Archivo Taller"),
    ]
    .iter()
    .enumerate()
    {
        let id = Uuid::now_v7();
        let actor = Uuid::now_v7();
        let email = format!("cliente{}@horbyta.test", i + 1);
        let status = if i == 6 {
            "inactive"
        } else if i == 7 {
            "blocked"
        } else {
            "active"
        };
        execute(tx,"INSERT INTO clients(id,user_id,name,email,phone,address_line,company,notes,status,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,now()-($10::int*interval '1 day'))",
            vec![id.into(),owner.into(),(*name).into(),email.clone().into(),format!("+550000000{:04}",i).into(),format!("Avenida Ejemplo {}, ciudad ficticia",i+10).into(),(*company).into(),"Datos ficticios. Contacto habitual: reunión los martes y revisiones por el portal.".into(),status.into(),(150-i as i32*12).into()]).await?;
        execute(
            tx,
            "INSERT INTO client_auth(id,client_id,user_id,email,password_hash) VALUES($1,$2,$3,$4,$5)",
            vec![
                actor.into(),
                id.into(),
                owner.into(),
                email.into(),
                client_hash.clone().into(),
            ],
        )
        .await?;
        clients.push((id, actor));
    }
    let names = [
        "Portal editorial Lumen",
        "Web de Norte Café",
        "Catálogo Verde Vivo",
        "Panel de Atlas",
        "Campus Aula Abierta",
        "Identidad Estación Creativa",
        "Reserva Marea Hotel",
        "Archivo digital del taller",
        "Mantenimiento editorial",
        "Integración de analítica",
        "Propuesta de tienda online",
        "Automatización del estudio",
    ];
    let titles = [
        "Arquitectura de contenidos",
        "Diseñar navegación accesible",
        "Preparar componentes visuales",
        "Implementar vista principal",
        "Integrar formularios",
        "Pruebas de aceptación",
        "Documentación de entrega",
        "Revisión con el cliente",
    ];
    let mut accepted = Vec::new();
    for (i, name) in names.iter().enumerate() {
        let (client, actor) = clients[i % clients.len()];
        let project = Uuid::now_v7();
        let currency = if i % 3 == 0 {
            "EUR"
        } else if i % 3 == 1 {
            "USD"
        } else {
            "BRL"
        };
        execute(tx,"INSERT INTO projects(id,user_id,client_id,category_id,name,description,billable,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,now()-($8::int*interval '1 day'))",
            vec![project.into(),owner.into(),client.into(),category.into(),(*name).into(),format!("## Objetivo\n\n{name}: una experiencia clara y accesible.\n\n## Seguimiento\n\nRevisiones semanales, entregas por hitos y decisiones documentadas en el portal.").into(),(i!=11).into(),(130-i as i32*8).into()]).await?;
        if i != 11 {
            let sql = match i % 3 {
                0 => {
                    "INSERT INTO project_billing(user_id,project_id,currency_type,billing_type,fixed_price) VALUES($1,$2,$3,'fixed',4200)"
                }
                1 => {
                    "INSERT INTO project_billing(user_id,project_id,currency_type,billing_type,hourly_rate) VALUES($1,$2,$3,'hourly',75)"
                }
                _ => {
                    "INSERT INTO project_billing(user_id,project_id,currency_type,billing_type,retainer_amount,payment_day) VALUES($1,$2,$3,'retainer',1800,10)"
                }
            };
            execute(tx, sql, vec![owner.into(), project.into(), currency.into()]).await?;
        }
        let mut milestones = Vec::new();
        for (m, title) in [
            "Descubrimiento y propuesta",
            "Primera entrega funcional",
            "Validación y publicación",
        ]
        .iter()
        .enumerate()
        {
            let id = Uuid::now_v7();
            execute(tx,"INSERT INTO milestones(id,project_id,name,description,due_date,created_at) VALUES($1,$2,$3,'Validar entregables y registrar conformidad del cliente.',current_date+($4::int*interval '1 day'),now()-interval '30 days')",
                vec![id.into(),project.into(),(*title).into(),((m as i32*14)+i as i32-8).into()]).await?;
            milestones.push(id);
        }
        let preparation = Uuid::now_v7();
        execute(tx,"INSERT INTO tasks(id,user_id,project_id,title,description,created_at) VALUES($1,$2,$3,'Preparar propuesta y recopilar contexto','Conversación previa y presupuesto; no incluye desarrollo de entregables.',now()-interval '40 days')",vec![preparation.into(),owner.into(),project.into()]).await?;
        ScopeService::preparation(tx, owner, preparation, "Preparación comercial previa al acuerdo").await?;
        execute(
            tx,
            "UPDATE tasks SET status='done' WHERE id=$1",
            vec![preparation.into()],
        )
        .await?;
        execute(tx,"INSERT INTO time_entries(user_id,task_id,activity,duration_minutes,billable,work_date,notes) VALUES($1,$2,$3,20,false,current_date-$4::int,'Preparación no facturable del presupuesto')",vec![owner.into(),preparation.into(),activity.into(),(60+i as i32).into()]).await?;
        let mut tasks = Vec::new();
        for (t, title) in titles.iter().enumerate() {
            let id = Uuid::now_v7();
            execute(tx,"INSERT INTO tasks(id,user_id,project_id,milestone_id,title,description,priority,complexity,due_date,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,current_date+($9::int*interval '1 day'),now()-interval '35 days')",
                vec![id.into(),owner.into(),project.into(),milestones[t%3].into(),(*title).into(),format!("### Resultado esperado\n\n{title} para {name}.\n\n- Validar estados vacíos y errores.\n- Documentar decisiones.\n- Revisar en móvil y escritorio.").into(),((t%4+1) as i16).into(),((t%4+1) as i16).into(),(t as i32*3+i as i32-8).into()]).await?;
            execute(
                tx,
                "INSERT INTO task_tags(task_id,tag_id) SELECT $1,id FROM tags WHERE user_id=$2 AND name=$3",
                vec![
                    id.into(),
                    owner.into(),
                    if t % 2 == 0 { "feature" } else { "Documentación" }.into(),
                ],
            )
            .await?;
            tasks.push(id);
        }
        for t in 0..2 {
            execute(tx,"INSERT INTO tasks(user_id,project_id,parent_id,milestone_id,title,description,priority) VALUES($1,$2,$3,$4,$5,'Comprobación concreta incluida en la tarea principal.',2)",vec![owner.into(),project.into(),tasks[t].into(),milestones[t%3].into(),format!("{} · revisar estados de error",titles[t]).into()]).await?;
        }
        let annex=file(tx,storage,files,owner,project,"criterios-aceptacion.md","text/markdown","# Criterios de aceptación\n\n- Navegación por teclado.\n- Estados de carga y error comprensibles.\n- Lectura en móvil sin desbordamiento.\n").await?;
        let reference=file(tx,storage,files,owner,project,"referencia-visual.svg","image/svg+xml","<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"640\" height=\"360\" viewBox=\"0 0 640 360\"><rect width=\"640\" height=\"360\" fill=\"#f1f5f9\"/><rect x=\"32\" y=\"32\" width=\"576\" height=\"60\" rx=\"12\" fill=\"#6366f1\"/><rect x=\"32\" y=\"120\" width=\"180\" height=\"208\" rx=\"12\" fill=\"#cbd5e1\"/><text x=\"240\" y=\"164\" font-family=\"sans-serif\" font-size=\"22\" fill=\"#334155\">Referencia ficticia</text></svg>").await?;
        let doc_id = Uuid::now_v7();
        let content = format!(
            "# Alcance de {name}\n\n## Entregables\nOcho tareas de diseño, implementación y validación descritas en esta propuesta.\n\n## Requisitos\nAcceso autenticado, navegación accesible y vista adaptable.\n\n## Restricciones\nDos rondas de revisión de contenido.\n\n## Exclusiones\nNo incluye tienda, campañas ni mantenimiento adicional.\n\n## Criterios de aceptación\n[Ver criterios incluidos](/api/v1/attachments/{annex}/raw).\n\n![Referencia visual de contexto](/api/v1/attachments/{reference}/raw)\n"
        );
        let requirement_id = Uuid::now_v7();
        let reference_id = Uuid::now_v7();
        let make_document = |based_on: Option<i32>| {
            ScopeDocument {
            documents: vec![
                ScopeContentDocument {
                    id: doc_id, title: format!("Especificación de {name}"), role: "main".into(), section: None,
                    content: if based_on.is_some() {content.replace("vista adaptable.", "vista adaptable. La navegación móvil se verifica con teclado y lector de pantalla.")} else {content.clone()},
                    source_name: Some("alcance.md".into()), source_content: Some(content.clone()), requirement_id: Some(format!("REQ-{:02}", i+1)),
                },
                ScopeContentDocument {
                    id: requirement_id, title: "Acceso y validación de formularios".into(), role: "annex".into(), section: Some("requirements".into()),
                    content: "# Requisito de acceso\n\n## Resultado verificable\n\n- El usuario autenticado accede a su información.\n- Los errores de formulario explican cómo corregirlos.\n- El cierre de sesión impide volver a consultar datos privados.\n\n## Exclusiones\n\nNo incluye integración con proveedores externos de identidad.\n".into(),
                    source_name: None, source_content: None, requirement_id: Some(format!("AUTH-{:02}",i+1)),
                },
                ScopeContentDocument {
                    id: reference_id, title: "Orientación visual del cliente".into(), role: "reference".into(), section: None,
                    content: format!("# Referencia de contexto\n\nPaleta y distribución orientativas; no incorporan pantallas ni funcionalidades adicionales.\n\n![Ejemplo visual ficticio](/api/v1/attachments/{reference}/raw)\n"),
                    source_name: None, source_content: None, requirement_id: None,
                },
            ],
            attachment_ids: vec![annex], reference_attachment_ids: vec![reference], based_on,
            precedence: "Las exclusiones prevalecen; el documento principal define entregables y el anexo detalla sus criterios. Las referencias visuales no añaden trabajo.".into(),
            ..Default::default()
        }
        };
        ScopeService::actor(tx, owner, "professional").await?;
        ScopeService::publish_with_storage(
            tx,
            owner,
            project,
            PublishScopeDto {
                expected_revision: 0,
                publish: i != 11,
                document: make_document(None),
            },
            Some(storage),
        )
        .await?;
        if i == 9 || i == 10 {
            ScopeService::feedback(
                tx,
                owner,
                Some(client),
                actor,
                project,
                ScopeFeedbackDto {
                    revision: 1,
                    kind: if i == 9 { "request_changes" } else { "reject" }.into(),
                    text: if i == 9 {
                        "Necesitamos reducir la primera entrega para respetar el presupuesto."
                    } else {
                        "No podemos aprobar la propuesta con las fechas actuales."
                    }
                    .into(),
                    document_id: Some(doc_id),
                    attachment_ids: vec![],
                },
            )
            .await?;
            ScopeService::feedback(
                tx,
                owner,
                None,
                owner,
                project,
                ScopeFeedbackDto {
                    revision: 1,
                    kind: "comment".into(),
                    text:
                        "Gracias por el contexto. Revisaremos alcance y condiciones antes de publicar otra propuesta."
                            .into(),
                    document_id: Some(doc_id),
                    attachment_ids: vec![],
                },
            )
            .await?;
            continue;
        }
        if i >= 8 {
            continue;
        }
        if i == 0 {
            ScopeService::feedback(
                tx,
                owner,
                Some(client),
                actor,
                project,
                ScopeFeedbackDto {
                    revision: 1,
                    kind: "request_changes".into(),
                    text: "Precisemos la navegación móvil y conservemos el resto del presupuesto.".into(),
                    document_id: Some(doc_id),
                    attachment_ids: vec![],
                },
            )
            .await?;
            ScopeService::publish_with_storage(
                tx,
                owner,
                project,
                PublishScopeDto {
                    expected_revision: 1,
                    publish: true,
                    document: make_document(Some(1)),
                },
                Some(storage),
            )
            .await?;
        }
        let revision = if i == 0 { 2 } else { 1 };
        execute(
            tx,
            "SELECT set_config('horbyta.accepted_ip',$1,true),set_config('horbyta.ip_source','development_seed',true)",
            vec![format!("203.0.113.{}", i + 10).into()],
        )
        .await?;
        ScopeService::accept(tx, owner, client, actor, project, revision).await?;
        ScopeService::feedback(tx,owner,None,owner,project,ScopeFeedbackDto{attachment_ids:vec![],revision,kind:"comment".into(),text:"Acuerdo registrado. Compartiremos avances por cada hito y gestionaremos cualquier extra como cambio.".into(),document_id:Some(doc_id)}).await?;
        for (t, id) in tasks.iter().enumerate() {
            execute(
                tx,
                "INSERT INTO scope_task_requirements(task_id,project_id,revision,document_id) VALUES($1,$2,$3,$4)",
                vec![
                    (*id).into(),
                    project.into(),
                    revision.into(),
                    if t % 2 == 0 { requirement_id } else { doc_id }.into(),
                ],
            )
            .await?;
            let status = [
                "done",
                "done",
                "review",
                "in_progress",
                "todo",
                "todo",
                "done",
                "review",
            ][t];
            execute(
                tx,
                "UPDATE tasks SET status=$2 WHERE id=$1",
                vec![(*id).into(), status.into()],
            )
            .await?;
            for entry in 0..3 {
                let day = ((i * 24 + t * 3 + entry) % 90 + 1) as i32;
                execute(tx,"INSERT INTO time_entries(user_id,task_id,activity,duration_minutes,work_date,billable,applied_rate,notes,created_at) VALUES($1,$2,$3,$4,current_date-$5::int,$6,75,$7,now()-($5::int*interval '1 day'))",
                    vec![owner.into(),(*id).into(),activities[(i+t+entry)%activities.len()].into(),(25+entry as i32*10).into(),day.into(),(i%3==1 && t!=7).into(),format!("{}: {}",titles[t],["Investigación y preparación","Implementación y validación","Revisión y documentación"][entry]).into()]).await?;
            }
        }
        execute(
            tx,
            "UPDATE tasks SET status='done' WHERE parent_id IN($1,$2)",
            vec![tasks[0].into(), tasks[1].into()],
        )
        .await?;
        execute(
            tx,
            "UPDATE milestones SET status='released' WHERE id=$1",
            vec![milestones[0].into()],
        )
        .await?;
        if i == 2 {
            execute(
                tx,
                "UPDATE milestones SET status='review' WHERE id=$1",
                vec![milestones[1].into()],
            )
            .await?;
        }
        accepted.push((project, client, actor, milestones[2], currency));
        for n in 0..3 {
            let invoice = Uuid::now_v7();
            let status = if n == 0 {
                "paid"
            } else if n == 1 {
                "open"
            } else {
                "draft"
            };
            execute(tx,"INSERT INTO invoices(id,user_id,project_id,amount,currency_type,status,due_date,issued_at,notes,created_at) VALUES($1,$2,$3,1200,$4,$5,current_date+$6::int,CASE WHEN $5='draft' THEN NULL ELSE now()-interval '25 days' END,$7,now()-($8::int*interval '1 day'))",
                vec![invoice.into(),owner.into(),project.into(),currency.into(),status.into(),(n*15-20).into(),format!("{} · {}",name,["Primera entrega","Segunda entrega con anticipo parcial","Borrador de cierre"][n as usize]).into(),(100-i as i32*9-n*8).into()]).await?;
            if n == 0 && i % 3 == 1 {
                execute(tx,"UPDATE time_entries SET invoice_id=$1 WHERE id IN(SELECT e.id FROM time_entries e JOIN tasks t ON t.id=e.task_id WHERE t.project_id=$2 AND e.billable ORDER BY e.work_date LIMIT 2)",vec![invoice.into(),project.into()]).await?;
            }
            if n < 2 {
                let payment = Uuid::now_v7();
                execute(tx,"INSERT INTO payments(id,user_id,project_id,invoice_id,amount,currency_type,payment_date,payment_method,reference_code,notes,created_at) VALUES($1,$2,$3,$4,$5::numeric,$6,current_date-$7::int,'bank_transfer',$8,'Pago ficticio registrado para validar el seguimiento financiero.',now()-($7::int*interval '1 day'))",
                    vec![payment.into(),owner.into(),project.into(),invoice.into(),if n==0 {"1200"} else {"450"}.into(),currency.into(),(90-i as i32*9-n*8).into(),format!("DEMO-{:02}-{}",i+1,n+1).into()]).await?;
                if n == 1 && i % 2 == 0 {
                    execute(tx,"INSERT INTO payments(user_id,project_id,invoice_id,parent_payment_id,amount,currency_type,transaction_type,payment_date,payment_method,notes) VALUES($1,$2,$3,$4,75,$5,'refund',current_date-5,'bank_transfer','Reembolso parcial ficticio por ajuste de anticipo.')",vec![owner.into(),project.into(),invoice.into(),payment.into(),currency.into()]).await?;
                }
            }
        }
    }
    // Solicitudes recorren las mismas transiciones y disparadores que usa la aplicación.
    for (i, status) in [
        "draft",
        "submitted",
        "assessed",
        "approved",
        "declined",
        "implemented",
        "cancelled",
    ]
    .iter()
    .enumerate()
    {
        let (project, client, actor, milestone, currency) = accepted[i % accepted.len()];
        let id = Uuid::now_v7();
        let task = Uuid::now_v7();
        ScopeService::actor(tx, owner, "professional").await?;
        execute(tx,"INSERT INTO change_requests(id,user_id,project_id,client_id,title,client_request_description,origin,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,now()-($8::int*interval '1 day'))",
            vec![id.into(),owner.into(),project.into(),client.into(),["Explorar exportación de informes","Añadir buscador de contenidos","Configurar segunda lengua","Exportar resultados a CSV","Añadir pagos en línea","Ampliar filtros del catálogo","Campaña temporal de lanzamiento"][i].into(),"Solicitud adicional fuera del alcance aceptado. Revisar costo, plazo y trabajo antes de ejecutar.".into(),if i%2==0 {"client_portal"} else {"freelancer_dashboard"}.into(),(25-i as i32*2).into()]).await?;
        execute(tx,"INSERT INTO tasks(id,user_id,project_id,title,description) VALUES($1,$2,$3,$4,'Extra condicionado a la decisión del cliente; no amplía el acuerdo automáticamente.')",vec![task.into(),owner.into(),project.into(),format!("Trabajo adicional · {}",status).into()]).await?;
        ScopeService::classify(
            tx,
            owner,
            owner,
            task,
            ClassifyTaskDto {
                change_request_id: Some(id),
                reason: "Solicitud explícita fuera de los entregables originales".into(),
            },
        )
        .await?;
        if *status != "draft" {
            execute(
                tx,
                "UPDATE change_requests SET status='submitted' WHERE id=$1",
                vec![id.into()],
            )
            .await?;
        }
        if !["draft", "submitted", "cancelled"].contains(status) {
            execute(tx,"UPDATE change_requests SET technical_assessment='Un día de implementación y validación; suplemento fijo sin doble facturación del tiempo.',impact_hours=8,impact_cost=360,schedule_impact_days=3,target_milestone_id=$2,status='assessed' WHERE id=$1",vec![id.into(),milestone.into()]).await?;
            ScopeService::comment(tx,owner,None,owner,id,ScopeCommentDto{version:3,attachment_ids:vec![],text:"Propuesta revisada: el resto del acuerdo permanece vigente. Confirma estas condiciones para habilitar el extra.".into()}).await?;
        }
        if ["approved", "implemented", "declined"].contains(status) {
            ScopeService::actor(tx, actor, "client").await?;
            execute(tx,"SELECT set_config('horbyta.accepted_ip','198.51.100.25',true),set_config('horbyta.ip_source','development_seed',true)",vec![]).await?;
            if *status == "declined" {
                execute(
                    tx,
                    "UPDATE change_requests SET status='declined',decline_reason='postponed_to_next_phase' WHERE id=$1",
                    vec![id.into()],
                )
                .await?;
            } else {
                execute(
                    tx,
                    "UPDATE change_requests SET status='approved' WHERE id=$1",
                    vec![id.into()],
                )
                .await?;
                execute(
                    tx,
                    "UPDATE tasks SET status=$2 WHERE id=$1",
                    vec![
                        task.into(),
                        if *status == "implemented" {
                            "done"
                        } else {
                            "in_progress"
                        }
                        .into(),
                    ],
                )
                .await?;
                execute(tx,"INSERT INTO time_entries(user_id,task_id,activity,duration_minutes,billable,work_date,notes) VALUES($1,$2,$3,40,false,current_date-$4::int,'Tiempo del suplemento fijo, sin facturarlo otra vez')",vec![owner.into(),task.into(),activity.into(),(i as i32+1).into()]).await?;
                if *status == "implemented" {
                    ScopeService::actor(tx, owner, "professional").await?;
                    execute(
                        tx,
                        "UPDATE change_requests SET status='implemented' WHERE id=$1",
                        vec![id.into()],
                    )
                    .await?;
                }
            }
        }
        if *status == "cancelled" {
            execute(
                tx,
                "UPDATE change_requests SET status='cancelled' WHERE id=$1",
                vec![id.into()],
            )
            .await?;
        }
        let _ = currency;
    }
    for sql in [
        "UPDATE projects SET status='closed' WHERE id=$1",
        "UPDATE tasks SET status='done' WHERE project_id=$1 AND NOT EXISTS(SELECT 1 FROM scope_task_links l WHERE l.task_id=tasks.id AND l.change_request_id IS NOT NULL)",
        "UPDATE milestones SET status='approved' WHERE project_id=$1 AND status='open'",
    ] {
        execute(tx, sql, vec![accepted[7].0.into()]).await?;
    }
    for sql in [
        "UPDATE projects SET status='cancelled' WHERE id=$1",
        "UPDATE milestones SET status='cancelled' WHERE project_id=$1 AND status NOT IN('approved','released')",
    ] {
        execute(tx, sql, vec![accepted[6].0.into()]).await?;
    }
    execute(tx,"INSERT INTO invoices(user_id,project_id,amount,currency_type,status,notes) VALUES($1,$2,500,$3,'void','Borrador sustituido por una propuesta más precisa.'),($1,$2,300,$3,'uncollectible','Caso ficticio para probar seguimiento de cobros.')",vec![owner.into(),accepted[0].0.into(),accepted[0].4.into()]).await?;
    // Tareas de bandeja, sin proyecto, para probar captura rápida y clasificación posterior.
    for title in [
        "Responder consulta del próximo cliente",
        "Revisar licencias de tipografías",
        "Preparar agenda de la semana",
        "Actualizar portafolio",
    ] {
        execute(tx,"INSERT INTO tasks(user_id,title,description,priority) VALUES($1,$2,'Pendiente personal del estudio sin proyecto asignado.',2)",vec![owner.into(),title.into()]).await?;
    }
    execute(tx,"INSERT INTO conversation_history(user_id,session_id,title,messages,created_at) VALUES($1,$2,'Plan de la próxima entrega',$3,now()-interval '2 days')",
        vec![owner.into(),Uuid::now_v7().into(),json!([{"role":"user","content":"¿Cómo organizo las próximas entregas?"},{"role":"assistant","content":"Prioriza los acuerdos aceptados y deja los extras pendientes hasta recibir la decisión del cliente."}]).into()]).await?;
    execute(
        tx,
        "INSERT INTO user_ui_state(user_id,scope,key,value) VALUES($1,'board','filters',$2)",
        vec![
            owner.into(),
            json!({"projectId":null,"milestoneId":null,"parentId":null,"createdFrom":null,"createdTo":null}).into(),
        ],
    )
    .await?;
    for (i, (key, title, body, severity)) in [
        (
            "billing.invoice_paid",
            "Primer hito cobrado",
            "El cliente registró el pago completo de la primera entrega.",
            "positive",
        ),
        (
            "billing.payment_received",
            "Anticipo recibido",
            "Se registró un pago parcial; queda saldo pendiente.",
            "info",
        ),
        (
            "client_portal.change_request_created",
            "Nueva solicitud del cliente",
            "Revisa la solicitud adicional antes de estimar su impacto.",
            "warning",
        ),
        (
            "work.milestone_released",
            "Hito publicado",
            "La primera entrega está disponible para revisión.",
            "positive",
        ),
    ]
    .iter()
    .enumerate()
    {
        execute(tx,"INSERT INTO notifications(user_id,preference_key,event_type,title,body,severity,source,entity_type,entity_id,payload,is_read,read_at,created_at) VALUES($1,$2,$2,$3,$4,$5,'system','project',$6,$7,$8,CASE WHEN $8 THEN now()-interval '1 day' ELSE NULL END,now()-($9::int*interval '1 hour'))",
            vec![owner.into(),(*key).into(),(*title).into(),(*body).into(),(*severity).into(),accepted[i].0.into(),json!({"project_id":accepted[i].0}).into(),(i%2==0).into(),(i as i32*8+1).into()]).await?;
    }
    let mut counts = serde_json::Map::new();
    for table in [
        "users",
        "clients",
        "client_auth",
        "categories",
        "activities",
        "tags",
        "projects",
        "project_billing",
        "milestones",
        "tasks",
        "task_tags",
        "time_entries",
        "attachments",
        "invoices",
        "payments",
        "change_requests",
        "conversation_history",
        "user_ui_state",
        "notifications",
        "scope_projects",
        "scope_revisions",
        "scope_task_links",
        "scope_task_requirements",
        "scope_project_events",
        "scope_change_revisions",
        "scope_events",
        "scope_effects",
        "scope_notices",
        "scope_delivery",
        "scope_preserved_attachments",
        "seq_by_user",
    ] {
        let count: i64 = scalar(tx, &format!("SELECT count(*) AS value FROM {table}")).await?;
        if count == 0 {
            return Err(conflict(&format!("El seed no cubrió la tabla {table}.")));
        }
        counts.insert(table.into(), json!(count));
    }
    Ok(json!({"admin_id":owner,"admin_email":ADMIN_EMAIL,"client_email":"cliente1@horbyta.test","tables":counts}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn rechaza_otra_base_sin_iniciar_transacciones() {
        use sea_orm::{DatabaseBackend, MockDatabase};
        use std::collections::BTreeMap;
        let row = BTreeMap::from([("value".to_owned(), Value::from("hormigapm_production"))]);
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results([vec![row]])
            .into_connection();
        let storage = object_store::memory::InMemory::new();
        assert!(seed(&db, &storage, "development", true).await.is_err());
        assert_eq!(db.into_transaction_log().len(), 1);
    }

    #[test]
    fn rechaza_produccion_y_modos_ambiguos() {
        for mode in ["production", "staging", "", "Development"] {
            assert!(require_development(mode).is_err());
        }
        assert!(require_development("development").is_ok());
    }
}
