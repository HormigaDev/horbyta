use sea_orm::{ConnectionTrait, DbBackend, Statement, Value};
use uuid::Uuid;

use crate::error_codes as codes;
use crate::{
    dtos::task_status::{ReorderTaskStatusesDto, SaveTaskStatusDto, TaskStatusDto},
    error::AppError,
};

const SELECT: &str = "SELECT s.id, s.key, s.name, s.category, s.color, s.position, \
    (SELECT count(*) FROM tasks t WHERE t.status_id = s.id) AS task_count \
    FROM task_statuses s";

pub struct TaskStatusService;

impl TaskStatusService {
    /// Normaliza #abc a #aabbcc (la base guarda siempre seis dígitos).
    fn color(raw: &str) -> Result<String, AppError> {
        let hex = raw.trim().trim_start_matches('#');
        let full = match hex.len() {
            3 => hex.chars().flat_map(|c| [c, c]).collect::<String>(),
            6 => hex.to_owned(),
            _ => return Err(AppError::BadRequest(codes::TASK_STATUS_COLOR_INVALID.into())),
        };
        if !full.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(AppError::BadRequest(codes::TASK_STATUS_COLOR_INVALID.into()));
        }
        Ok(format!("#{}", full.to_uppercase()))
    }

    async fn many(db: &impl ConnectionTrait, sql: String, values: Vec<Value>) -> Result<Vec<TaskStatusDto>, AppError> {
        let rows = db
            .query_all(Statement::from_sql_and_values(DbBackend::Postgres, sql, values))
            .await?;
        rows.iter()
            .map(|row| <TaskStatusDto as sea_orm::FromQueryResult>::from_query_result(row, "").map_err(AppError::from))
            .collect()
    }

    pub async fn list(db: &impl ConnectionTrait, user_id: Uuid) -> Result<Vec<TaskStatusDto>, AppError> {
        Self::many(
            db,
            format!("{SELECT} WHERE s.user_id = $1 ORDER BY s.position, s.created_at"),
            vec![user_id.into()],
        )
        .await
    }

    async fn one(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<TaskStatusDto, AppError> {
        Self::many(
            db,
            format!("{SELECT} WHERE s.user_id = $1 AND s.id = $2"),
            vec![user_id.into(), id.into()],
        )
        .await?
        .pop()
        .ok_or_else(|| AppError::NotFound(codes::TASK_STATUS_NOT_FOUND.into()))
    }

    pub async fn create(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        dto: SaveTaskStatusDto,
    ) -> Result<TaskStatusDto, AppError> {
        let row = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "INSERT INTO task_statuses (user_id, name, category, color, position) \
                 VALUES ($1, $2, $3, $4, (SELECT coalesce(max(position), 0) + 1 FROM task_statuses WHERE user_id = $1)) \
                 RETURNING id",
                vec![
                    user_id.into(),
                    dto.name.trim().into(),
                    dto.category.to_string().into(),
                    Self::color(&dto.color)?.into(),
                ],
            ))
            .await?
            .ok_or(AppError::InternalError)?;
        Self::one(db, user_id, row.try_get("", "id")?).await
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        dto: SaveTaskStatusDto,
    ) -> Result<TaskStatusDto, AppError> {
        let result = db
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "UPDATE task_statuses SET name = $3, category = $4, color = $5 WHERE id = $2 AND user_id = $1",
                vec![
                    user_id.into(),
                    id.into(),
                    dto.name.trim().into(),
                    dto.category.to_string().into(),
                    Self::color(&dto.color)?.into(),
                ],
            ))
            .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(codes::TASK_STATUS_NOT_FOUND.into()));
        }
        Self::one(db, user_id, id).await
    }

    pub async fn delete(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let result = db
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "DELETE FROM task_statuses WHERE id = $2 AND user_id = $1",
                vec![user_id.into(), id.into()],
            ))
            .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(codes::TASK_STATUS_NOT_FOUND.into()));
        }
        Ok(())
    }

    /// Reordena según la lista recibida; debe contener todos los estados del usuario.
    pub async fn reorder(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        dto: ReorderTaskStatusesDto,
    ) -> Result<Vec<TaskStatusDto>, AppError> {
        let current = Self::list(db, user_id).await?;
        let mut expected: Vec<Uuid> = current.iter().map(|s| s.id).collect();
        let mut received = dto.ids.clone();
        expected.sort();
        received.sort();
        if expected != received {
            return Err(AppError::Conflict(codes::TASK_STATUS_ORDER_CHANGED.into()));
        }
        for (index, id) in dto.ids.iter().enumerate() {
            db.execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "UPDATE task_statuses SET position = $3 WHERE id = $2 AND user_id = $1",
                vec![user_id.into(), (*id).into(), (index as i32 + 1).into()],
            ))
            .await?;
        }
        Self::list(db, user_id).await
    }
}
