//! Estados de tarea editables: valores por defecto, sincronización de la categoría y reglas.
use futures::FutureExt;
use horbyta::{
    dtos::{
        task::{CreateTaskDto, TaskStatus, UpdateTaskStatusDto},
        task_status::{ReorderTaskStatusesDto, SaveTaskStatusDto},
    },
    error::AppError,
    migrations,
    services::{task_statuses::TaskStatusService, tasks::TaskService},
};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};
use std::panic::AssertUnwindSafe;
use uuid::Uuid;

async fn sql(db: &DatabaseConnection, query: &str) {
    db.execute_unprepared(query).await.unwrap();
}

async fn text(db: &DatabaseConnection, query: &str) -> String {
    db.query_one(Statement::from_string(DbBackend::Postgres, query.to_owned()))
        .await
        .unwrap()
        .unwrap()
        .try_get("", "value")
        .unwrap()
}

fn task(title: &str, status_id: Option<Uuid>) -> CreateTaskDto {
    CreateTaskDto {
        project_id: None,
        parent_id: None,
        milestone_id: None,
        title: title.into(),
        description: None,
        status_id,
        status: None,
        priority: 2,
        complexity: 2,
        block_parent: false,
        due_date: None,
        tag_ids: vec![],
        scope_origin: None,
    }
}

fn save(name: &str, category: TaskStatus) -> SaveTaskStatusDto {
    SaveTaskStatusDto { name: name.into(), category, color: "#f59e0b".into() }
}

#[tokio::test]
#[ignore = "Requiere PostgreSQL; esquema aislado"]
async fn estados_editables_por_usuario() {
    dotenvy::dotenv().ok();
    let url = std::env::var("DATABASE_URL").expect("Falta conexión de pruebas");
    let mut options = ConnectOptions::new(url);
    options.max_connections(1).min_connections(1).sqlx_logging(false);
    let db = Database::connect(options).await.unwrap();
    let schema = format!("task_status_test_{}", Uuid::now_v7().simple());
    sql(&db, &format!("CREATE SCHEMA {schema}; SET search_path TO {schema}, public")).await;
    let result = AssertUnwindSafe(run(&db)).catch_unwind().await;
    sql(&db, &format!("SET search_path TO public; DROP SCHEMA {schema} CASCADE")).await;
    db.close().await.unwrap();
    if let Err(error) = result {
        std::panic::resume_unwind(error);
    }
}

async fn run(db: &DatabaseConnection) {
    migrations::migrate(db).await.unwrap();
    let (owner, other) = (Uuid::now_v7(), Uuid::now_v7());
    sql(db, &format!("INSERT INTO users(id,name,email,password_hash,role) VALUES('{owner}','Dueña','estados@horbyta.test','hash','admin'),('{other}','Otra','otra@horbyta.test','hash','user')")).await;

    // Un usuario nuevo recibe los cinco estados por defecto, en orden.
    let defaults = TaskStatusService::list(db, owner).await.unwrap();
    let keys: Vec<_> = defaults.iter().map(|s| s.key.clone().unwrap_or_default()).collect();
    assert_eq!(keys, ["todo", "in_progress", "review", "done", "cancelled"]);

    // Sin estado elegido: el primero de la categoría "todo".
    let plain = TaskService::create(db, owner, task("Sin estado", None)).await.unwrap();
    assert_eq!(plain.status_key.as_deref(), Some("todo"));

    // Estado propio: la tarea toma su categoría.
    let blocked = TaskStatusService::create(db, owner, save("Bloqueada", TaskStatus::InProgress)).await.unwrap();
    assert_eq!(blocked.position, 6);
    let custom = TaskService::create(db, owner, task("Bloqueada", Some(blocked.id))).await.unwrap();
    assert_eq!(custom.status, TaskStatus::InProgress);
    assert_eq!(custom.status_name.as_deref(), Some("Bloqueada"));

    // Cambiar solo la categoría (clientes antiguos) elige el primer estado de esa categoría.
    TaskService::change_status(db, owner, custom.id, UpdateTaskStatusDto { status_id: None, status: Some(TaskStatus::Cancelled) })
        .await
        .unwrap();
    assert_eq!(text(db, &format!("SELECT s.key AS value FROM tasks t JOIN task_statuses s ON s.id=t.status_id WHERE t.id='{}'", custom.id)).await, "cancelled");
    TaskService::change_status(db, owner, custom.id, UpdateTaskStatusDto { status_id: Some(blocked.id), status: None }).await.unwrap();

    // Reglas: en uso, último "todo", categoría de un estado en uso, nombre repetido, otro usuario.
    assert!(matches!(TaskStatusService::delete(db, owner, blocked.id).await, Err(AppError::Conflict(_))));
    let todo = defaults.iter().find(|s| s.key.as_deref() == Some("todo")).unwrap().id;
    TaskService::change_status(db, owner, plain.id, UpdateTaskStatusDto { status_id: Some(blocked.id), status: None }).await.unwrap();
    assert!(matches!(TaskStatusService::delete(db, owner, todo).await, Err(AppError::Conflict(_))), "último de la categoría todo");
    assert!(matches!(
        TaskStatusService::update(db, owner, blocked.id, save("Bloqueada", TaskStatus::Review)).await,
        Err(AppError::Conflict(_))
    ));
    assert!(matches!(
        TaskStatusService::create(db, owner, save("bloqueada", TaskStatus::Todo)).await,
        Err(AppError::Conflict(_))
    ));
    let foreign = TaskStatusService::list(db, other).await.unwrap()[0].id;
    assert!(TaskService::create(db, owner, task("Ajena", Some(foreign))).await.is_err());

    // Renombrar un estado por defecto conserva su clave y su uso.
    let renamed = TaskStatusService::update(db, owner, todo, save("Pendiente", TaskStatus::Todo)).await.unwrap();
    assert_eq!((renamed.key.as_deref(), renamed.name.as_str()), (Some("todo"), "Pendiente"));

    // Reordenar exige la lista completa.
    let mut ids: Vec<Uuid> = TaskStatusService::list(db, owner).await.unwrap().iter().map(|s| s.id).collect();
    ids.reverse();
    let reordered = TaskStatusService::reorder(db, owner, ReorderTaskStatusesDto { ids: ids.clone() }).await.unwrap();
    assert_eq!(reordered.iter().map(|s| s.id).collect::<Vec<_>>(), ids);
    assert!(TaskStatusService::reorder(db, owner, ReorderTaskStatusesDto { ids: ids[1..].to_vec() }).await.is_err());

    // Borrar al usuario elimina sus estados aunque tenga tareas.
    sql(db, &format!("DELETE FROM users WHERE id='{owner}'")).await;
    assert_eq!(text(db, &format!("SELECT count(*)::text AS value FROM task_statuses WHERE user_id='{owner}'")).await, "0");
}
