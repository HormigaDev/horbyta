use sea_orm::DatabaseConnection;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    dtos::{
        project::project_filter_schema,
        query::{Filter, QueryPayload},
        task::{CreateTaskDto, task_filter_schema},
    },
    error::AppError,
    services::{projects::ProjectService, tasks::TaskService},
};

pub struct AiToolSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: Value,
}

#[derive(Debug, Deserialize)]
struct IdArgs {
    id: Uuid,
}

#[derive(Debug, Deserialize)]
struct PaginationArgs {
    page: u64,
    limit: u64,
}

#[derive(Debug, Deserialize)]
struct TasksListArgs {
    page: u64,
    limit: u64,
    order_by: Option<String>,
    order_asc: Option<bool>,
    filters: Option<Vec<Filter>>,
}

fn ensure_valid_pagination(page: u64, limit: u64) -> Result<(), AppError> {
    if page < 1 {
        return Err(AppError::BadRequest("'page' must be greater than or equal to 1".to_string()));
    }
    if !(1..=100).contains(&limit) {
        return Err(AppError::BadRequest("'limit' must be between 1 and 100".to_string()));
    }
    Ok(())
}

pub fn tool_specs() -> Vec<AiToolSpec> {
    vec![
        AiToolSpec {
            name: "project_read",
            description: "Lee toda la informacion de un proyecto por su id.",
            parameters: json!({
              "type": "object",
              "additionalProperties": false,
              "required": ["id"],
              "properties": {
                "id": {
                  "type": "string",
                  "format": "uuid",
                  "description": "UUID del proyecto"
                }
              }
            }),
        },
        AiToolSpec {
            name: "projects_list",
            description: "Lista proyectos retornando id y titulo (name), con paginacion.",
            parameters: json!({
              "type": "object",
              "additionalProperties": false,
              "required": ["page", "limit"],
              "properties": {
                "page": {
                  "type": "integer",
                  "minimum": 1,
                  "description": "Numero de pagina (>= 1)"
                },
                "limit": {
                  "type": "integer",
                  "minimum": 1,
                  "maximum": 100,
                  "description": "Cantidad de elementos por pagina"
                }
              }
            }),
        },
        AiToolSpec {
            name: "task_read",
            description: "Lee toda la informacion de una tarea por su id.",
            parameters: json!({
              "type": "object",
              "additionalProperties": false,
              "required": ["id"],
              "properties": {
                "id": {
                  "type": "string",
                  "format": "uuid",
                  "description": "UUID de la tarea"
                }
              }
            }),
        },
        AiToolSpec {
            name: "tasks_list",
            description: "Lista tareas con paginacion y filtros del backend (id, titulo, estado, complejidad, prioridad, project_id).",
            parameters: json!({
              "type": "object",
              "additionalProperties": false,
              "required": ["page", "limit"],
              "properties": {
                "page": {
                  "type": "integer",
                  "minimum": 1
                },
                "limit": {
                  "type": "integer",
                  "minimum": 1,
                  "maximum": 100
                },
                "order_by": {
                  "type": "string",
                  "enum": ["id", "project_id", "parent_id", "milestone_id", "title", "status", "priority", "complexity", "block_parent", "due_date", "created_at", "updated_at"]
                },
                "order_asc": {
                  "type": "boolean"
                },
                "filters": {
                  "type": "array",
                  "maxItems": 5,
                  "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["column", "operator", "value"],
                    "properties": {
                      "column": {
                        "type": "string",
                        "enum": ["id", "project_id", "parent_id", "milestone_id", "title", "status", "priority", "complexity", "block_parent", "due_date", "created_at", "updated_at"]
                      },
                      "operator": {
                        "type": "string",
                        "enum": ["=", "!=", ">", "<", ">=", "<=", "><", "<>", "()"],
                        "description": "Operador >< significa contiene (ilike), el operador <> significa no contiene, y el operador '()' significa entre, se aplica a fechas únicamente"
                      },
                      "value": {
                        "description": "Valor del filtro, dependiente de column/operator"
                      }
                    }
                  }
                }
              }
            }),
        },
        AiToolSpec {
            name: "task_create",
            description: "Crea una tarea con los mismos parametros de CreateTaskDto (incluyendo opcionales).",
            parameters: json!({
              "type": "object",
              "additionalProperties": false,
              "required": ["title"],
              "properties": {
                "project_id": { "type": "string", "format": "uuid", "description": "UUID opcional del proyecto" },
                "parent_id": { "type": "string", "format": "uuid", "description": "UUID opcional de la tarea padre" },
                "milestone_id": { "type": "string", "format": "uuid", "description": "UUID opcional del milestone" },
                "title": { "type": "string", "minLength": 1, "maxLength": 255 },
                "description": { "type": "string", "maxLength": 15000, "description": "Descripcion opcional" },
                "status": { "type": "string", "enum": ["todo", "in_progress", "review", "done"], "default": "todo" },
                "priority": { "type": "integer", "minimum": 1, "maximum": 4, "default": 1 },
                "complexity": { "type": "integer", "minimum": 1, "maximum": 4, "default": 1 },
                "block_parent": { "type": "boolean", "default": true },
                "due_date": { "type": "string", "format": "date-time", "description": "Fecha opcional RFC3339" },
                "tag_ids": { "type": "array", "items": { "type": "string", "format": "uuid" }, "default": [] }
              }
            }),
        },
    ]
}

pub async fn execute_tool(
    db: Arc<DatabaseConnection>,
    user_id: Uuid,
    tool_name: &str,
    args: Value,
) -> Result<Value, AppError> {
    match tool_name {
        "project_read" => {
            let parsed: IdArgs = serde_json::from_value(args)
                .map_err(|e| AppError::BadRequest(format!("invalid arguments for project_read: {}", e)))?;

            let project = ProjectService::get_by_id(db.as_ref(), user_id, parsed.id).await?;
            Ok(json!({ "project": project }))
        }
        "projects_list" => {
            let parsed: PaginationArgs = serde_json::from_value(args)
                .map_err(|e| AppError::BadRequest(format!("invalid arguments for projects_list: {}", e)))?;

            ensure_valid_pagination(parsed.page, parsed.limit)?;

            let payload = QueryPayload {
                page: parsed.page,
                limit: parsed.limit,
                order_by: Some("created_at".to_string()),
                order_asc: false,
                filters: vec![],
            };

            let (items, total) = ProjectService::search(db.as_ref(), user_id, payload).await?;
            let projects = items
                .into_iter()
                .map(|p| {
                    json!({
                        "id": p.id,
                        "title": p.name,
                        "name": p.name
                    })
                })
                .collect::<Vec<Value>>();

            Ok(json!({
                "items": projects,
                "total": total,
                "page": parsed.page,
                "limit": parsed.limit
            }))
        }
        "task_read" => {
            let parsed: IdArgs = serde_json::from_value(args)
                .map_err(|e| AppError::BadRequest(format!("invalid arguments for task_read: {}", e)))?;

            let task = TaskService::get_by_id(db.as_ref(), user_id, parsed.id).await?;
            Ok(json!({ "task": task }))
        }
        "tasks_list" => {
            let parsed: TasksListArgs = serde_json::from_value(args)
                .map_err(|e| AppError::BadRequest(format!("invalid arguments for tasks_list: {}", e)))?;

            ensure_valid_pagination(parsed.page, parsed.limit)?;

            let payload = QueryPayload {
                page: parsed.page,
                limit: parsed.limit,
                order_by: parsed.order_by,
                order_asc: parsed.order_asc.unwrap_or(false),
                filters: parsed.filters.unwrap_or_default(),
            };

            payload
                .validate()
                .map_err(|e| AppError::BadRequest(format!("invalid payload in tasks_list: {}", e)))?;
            task_filter_schema().validate_payload(&payload)?;

            let (items, total) = TaskService::search(db.as_ref(), user_id, payload.clone()).await?;

            let tasks = items
                .into_iter()
                .map(|t| {
                    json!({
                        "id": t.id,
                        "title": t.title,
                        "status": t.status,
                        "complexity": t.complexity,
                        "priority": t.priority,
                        "project_id": t.project_id
                    })
                })
                .collect::<Vec<Value>>();

            Ok(json!({
                "items": tasks,
                "total": total,
                "page": parsed.page,
                "limit": parsed.limit,
                "filters": payload.filters
            }))
        }
        "task_create" => {
            let dto: CreateTaskDto = serde_json::from_value(args)
                .map_err(|e| AppError::BadRequest(format!("invalid arguments for task_create: {}", e)))?;

            dto.validate()
                .map_err(|e| AppError::BadRequest(format!("invalid payload in task_create: {}", e)))?;

            let created = TaskService::create(db.as_ref(), user_id, dto).await?;
            Ok(json!({ "task": created }))
        }
        _ => Err(AppError::BadRequest(format!("tool not supported: {}", tool_name))),
    }
}

pub fn list_tool_names() -> Vec<&'static str> {
    tool_specs().into_iter().map(|spec| spec.name).collect()
}

pub fn projects_filter_columns() -> Vec<&'static str> {
    project_filter_schema().allowed_columns.keys().copied().collect()
}
