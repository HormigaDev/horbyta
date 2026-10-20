use std::collections::HashMap;

use chrono::{NaiveDate, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
    prelude::{Decimal, Expr},
};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::error_codes as codes;
use crate::{
    dtos::{
        query::QueryPayload,
        time_entries::{
            CreateTimeEntryDto, StartTimeEntryTimerDto, StopTimeEntryTimerDto, TimeEntryDetailDto, TimeEntryDetailRow,
            TimeEntrySearchField, TimeEntrySummaryDto, UpdateTimeEntryDto,
        },
        user::UserPreferences,
    },
    entities::{
        activities,
        prelude::{Activities, ProjectBilling, Projects, Tasks, TimeEntries, Users},
        project_billing, projects, tasks,
        time_entries::{self, Model as TimeEntryModel},
    },
    error::AppError,
    utils::search::{apply_filters, apply_ordering, apply_pagination},
};

pub const MAX_DAILY_DURATION_LIMIT_HOURS: i32 = 14;

pub struct TimeEntryService;

impl TimeEntryService {
    pub async fn create_manual(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        dto: CreateTimeEntryDto,
    ) -> Result<TimeEntryDetailDto, AppError> {
        let task = Self::find_owned_task(db, user_id, dto.task_id).await?;
        Self::ensure_activity_ownership(db, user_id, dto.activity).await?;

        let work_date = dto.work_date.unwrap_or_else(|| Utc::now().date_naive());
        let daily_limit_minutes = Self::get_effective_daily_limit_minutes(db, user_id).await?;
        let applied_rate = Self::resolve_applied_rate(db, user_id, &task, dto.billable).await?;

        let total_duration = Self::get_daily_duration(db, user_id, dto.task_id, work_date, None).await?;
        Self::ensure_daily_duration_limit(total_duration, i64::from(dto.duration_minutes), daily_limit_minutes)?;

        let entry = time_entries::ActiveModel {
            user_id: Set(user_id),
            task_id: Set(dto.task_id),
            activity: Set(dto.activity),
            start_time: Set(None),
            end_time: Set(None),
            duration_minutes: Set(dto.duration_minutes),
            work_date: Set(work_date),
            billable: Set(dto.billable),
            applied_rate: Set(applied_rate),
            notes: Set(dto.notes),
            ..Default::default()
        };

        let created = entry.insert(db).await.map_err(Self::map_time_entry_write_error)?;
        Self::get_by_id(db, user_id, created.id).await
    }

    pub async fn start_timer(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        dto: StartTimeEntryTimerDto,
    ) -> Result<TimeEntryDetailDto, AppError> {
        Self::ensure_no_active_timer(db, user_id).await?;
        let task = Self::find_owned_task(db, user_id, dto.task_id).await?;
        Self::ensure_activity_ownership(db, user_id, dto.activity).await?;

        let start_time = dto.start_time.unwrap_or_else(|| Utc::now().fixed_offset());
        let work_date = dto.work_date.unwrap_or_else(|| start_time.date_naive());
        let daily_limit_minutes = Self::get_effective_daily_limit_minutes(db, user_id).await?;
        let applied_rate = Self::resolve_applied_rate(db, user_id, &task, dto.billable).await?;
        let total_duration = Self::get_daily_duration(db, user_id, dto.task_id, work_date, None).await?;
        Self::ensure_daily_duration_limit(total_duration, 0, daily_limit_minutes)?;

        let entry = time_entries::ActiveModel {
            user_id: Set(user_id),
            task_id: Set(dto.task_id),
            activity: Set(dto.activity),
            start_time: Set(Some(start_time)),
            end_time: Set(None),
            duration_minutes: Set(0),
            work_date: Set(work_date),
            billable: Set(dto.billable),
            applied_rate: Set(applied_rate),
            notes: Set(dto.notes),
            ..Default::default()
        };

        let created = entry.insert(db).await.map_err(Self::map_time_entry_write_error)?;
        Self::get_by_id(db, user_id, created.id).await
    }

    pub async fn stop_timer(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        dto: StopTimeEntryTimerDto,
    ) -> Result<TimeEntryDetailDto, AppError> {
        let entry = Self::find_by_id(db, user_id, id).await?;

        let entry_id = entry.id;
        let entry_work_date = entry.work_date;

        let start_time = entry
            .start_time
            .ok_or_else(|| AppError::Conflict(codes::TIME_ENTRY_NOT_TIMER.into()))?;

        if entry.end_time.is_some() {
            return Err(AppError::Conflict(codes::TIME_ENTRY_TIMER_ALREADY_STOPPED.into()));
        }

        let end_time = dto.end_time.unwrap_or_else(|| Utc::now().fixed_offset());

        if end_time <= start_time {
            return Err(AppError::BadRequest(codes::TIME_ENTRY_END_BEFORE_START.into()));
        }

        let duration_minutes_i64 = end_time.signed_duration_since(start_time).num_minutes();
        let duration_minutes = i32::try_from(duration_minutes_i64)
            .map_err(|_| AppError::BadRequest(codes::TIME_ENTRY_DURATION_EXCEEDED.into()))?;

        if duration_minutes <= 0 {
            return Err(AppError::BadRequest(codes::TIME_ENTRY_TIMER_TOO_SHORT.into()));
        }

        let daily_limit_minutes = Self::get_effective_daily_limit_minutes(db, user_id).await?;
        let total_duration =
            Self::get_daily_duration(db, user_id, entry.task_id, entry_work_date, Some(entry_id)).await?;
        Self::ensure_daily_duration_limit(total_duration, i64::from(duration_minutes), daily_limit_minutes)?;

        let mut active_entry: time_entries::ActiveModel = entry.into();
        active_entry.end_time = Set(Some(end_time));
        active_entry.duration_minutes = Set(duration_minutes);

        let updated = active_entry
            .update(db)
            .await
            .map_err(Self::map_time_entry_write_error)?;
        Self::get_by_id(db, user_id, updated.id).await
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        dto: UpdateTimeEntryDto,
    ) -> Result<TimeEntryDetailDto, AppError> {
        let entry = Self::find_by_id(db, user_id, id).await?;

        if entry.start_time.is_some() && entry.end_time.is_none() {
            return Err(AppError::Conflict(codes::TIME_ENTRY_TIMER_ACTIVE.into()));
        }

        let task = Self::find_owned_task(db, user_id, dto.task_id).await?;
        Self::ensure_activity_ownership(db, user_id, dto.activity).await?;

        let daily_limit_minutes = Self::get_effective_daily_limit_minutes(db, user_id).await?;
        let total_duration = Self::get_daily_duration(db, user_id, dto.task_id, dto.work_date, Some(entry.id)).await?;
        Self::ensure_daily_duration_limit(total_duration, i64::from(dto.duration_minutes), daily_limit_minutes)?;
        let applied_rate = Self::resolve_applied_rate(db, user_id, &task, dto.billable).await?;

        let mut active_entry: time_entries::ActiveModel = entry.into();
        active_entry.task_id = Set(dto.task_id);
        active_entry.activity = Set(dto.activity);
        active_entry.duration_minutes = Set(dto.duration_minutes);
        active_entry.work_date = Set(dto.work_date);
        active_entry.billable = Set(dto.billable);
        active_entry.applied_rate = Set(applied_rate);
        active_entry.notes = Set(dto.notes);

        let updated = active_entry
            .update(db)
            .await
            .map_err(Self::map_time_entry_write_error)?;
        Self::get_by_id(db, user_id, updated.id).await
    }

    pub async fn get_active_timer(
        db: &impl ConnectionTrait,
        user_id: Uuid,
    ) -> Result<Option<TimeEntryDetailDto>, AppError> {
        let active = Self::find_active_timer(db, user_id).await?;

        match active {
            Some(model) => Ok(Some(Self::get_by_id(db, user_id, model.id).await?)),
            None => Ok(None),
        }
    }

    pub async fn get_by_id(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<TimeEntryDetailDto, AppError> {
        let row: TimeEntryDetailRow = TimeEntries::find()
            .filter(time_entries::Column::Id.eq(id))
            .filter(time_entries::Column::UserId.eq(user_id))
            .left_join(tasks::Entity)
            .left_join(activities::Entity)
            .select_only()
            .column(time_entries::Column::Id)
            .column(time_entries::Column::UserId)
            .column(time_entries::Column::TaskId)
            .column(time_entries::Column::Activity)
            .column(time_entries::Column::InvoiceId)
            .column(time_entries::Column::StartTime)
            .column(time_entries::Column::EndTime)
            .column(time_entries::Column::DurationMinutes)
            .column(time_entries::Column::WorkDate)
            .column(time_entries::Column::Billable)
            .column(time_entries::Column::AppliedRate)
            .column(time_entries::Column::Notes)
            .column(time_entries::Column::CreatedAt)
            .column(time_entries::Column::UpdatedAt)
            .column_as(tasks::Column::Title, "task_title")
            .column_as(activities::Column::Name, "activity_name")
            .into_model::<TimeEntryDetailRow>()
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound(codes::TIME_ENTRY_NOT_FOUND.into()))?;

        Ok(Self::map_detail_row(row))
    }

    pub async fn delete(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let entry = Self::find_by_id(db, user_id, id).await?;

        time_entries::ActiveModel {
            id: Set(entry.id),
            ..Default::default()
        }
        .delete(db)
        .await
        .map_err(AppError::from)?;

        Ok(())
    }

    pub async fn search(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        payload: QueryPayload,
    ) -> Result<(Vec<TimeEntrySummaryDto>, u64, HashMap<String, i64>, i64), AppError> {
        let base_condition = Condition::all().add(time_entries::Column::UserId.eq(user_id));
        let condition = apply_filters(base_condition, &payload.filters, |column| {
            column
                .parse::<TimeEntrySearchField>()
                .ok()
                .map(TimeEntrySearchField::filter_field)
        })?;

        let query = TimeEntries::find()
            .filter(condition)
            .left_join(tasks::Entity)
            .left_join(activities::Entity);

        let query = apply_ordering(
            query,
            &payload,
            TimeEntrySearchField::CreatedAt.order_column(),
            false,
            |column| {
                column
                    .parse::<TimeEntrySearchField>()
                    .ok()
                    .map(TimeEntrySearchField::order_column)
            },
        )?;

        let total_items = query.clone().count(db).await?;
        let items = apply_pagination(
            query
                .select_only()
                .column(time_entries::Column::Id)
                .column(time_entries::Column::TaskId)
                .column(time_entries::Column::Activity)
                .column(time_entries::Column::InvoiceId)
                .column(time_entries::Column::StartTime)
                .column(time_entries::Column::EndTime)
                .column(time_entries::Column::DurationMinutes)
                .column(time_entries::Column::WorkDate)
                .column(time_entries::Column::Billable)
                .column(time_entries::Column::AppliedRate)
                .column(time_entries::Column::Notes)
                .column(time_entries::Column::CreatedAt)
                .column(time_entries::Column::UpdatedAt)
                .column_as(tasks::Column::Title, "task_title")
                .column_as(activities::Column::Name, "activity_name"),
            &payload,
        )
        .into_model::<TimeEntrySummaryDto>()
        .all(db)
        .await?;

        let task_id = Self::extract_task_filter(&payload);
        let daily_totals_by_date = Self::get_daily_totals_by_date(db, user_id, task_id).await?;
        let daily_limit_minutes = Self::get_effective_daily_limit_minutes(db, user_id).await?;

        Ok((items, total_items, daily_totals_by_date, daily_limit_minutes))
    }

    async fn get_effective_daily_limit_minutes(db: &impl ConnectionTrait, user_id: Uuid) -> Result<i64, AppError> {
        let user = Users::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or_else(|| AppError::Unauthorized(codes::AUTH_NOT_AUTHENTICATED.into()))?;

        let preferences = UserPreferences::from_json_value(user.preferences.as_ref());
        let limited_hours = preferences.max_daily_hours.clamp(1, MAX_DAILY_DURATION_LIMIT_HOURS);

        Ok(i64::from(limited_hours) * 60)
    }

    async fn find_by_id(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<TimeEntryModel, AppError> {
        TimeEntries::find()
            .filter(time_entries::Column::Id.eq(id))
            .filter(time_entries::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound(codes::TIME_ENTRY_NOT_FOUND.into()))
    }

    async fn find_active_timer(db: &impl ConnectionTrait, user_id: Uuid) -> Result<Option<TimeEntryModel>, AppError> {
        TimeEntries::find()
            .filter(time_entries::Column::UserId.eq(user_id))
            .filter(time_entries::Column::StartTime.is_not_null())
            .filter(time_entries::Column::EndTime.is_null())
            .order_by_desc(time_entries::Column::StartTime)
            .one(db)
            .await
            .map_err(AppError::from)
    }

    async fn ensure_no_active_timer(db: &impl ConnectionTrait, user_id: Uuid) -> Result<(), AppError> {
        let active_timer = Self::find_active_timer(db, user_id).await?;

        if active_timer.is_some() {
            return Err(AppError::Conflict(codes::TIME_ENTRY_TIMER_ALREADY_RUNNING.into()));
        }

        Ok(())
    }

    async fn find_owned_task(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        task_id: Uuid,
    ) -> Result<tasks::Model, AppError> {
        Tasks::find()
            .filter(tasks::Column::Id.eq(task_id))
            .filter(tasks::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::BadRequest(codes::ATTACHMENT_TASK_FORBIDDEN.into()))
    }

    async fn resolve_applied_rate(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        task: &tasks::Model,
        time_entry_billable: bool,
    ) -> Result<Option<Decimal>, AppError> {
        if !time_entry_billable {
            return Ok(None);
        }

        let Some(project_id) = task.project_id else {
            return Ok(None);
        };

        let project = Projects::find()
            .filter(projects::Column::Id.eq(project_id))
            .filter(projects::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::BadRequest(codes::ATTACHMENT_PROJECT_FORBIDDEN.into()))?;

        if !project.billable {
            return Ok(None);
        }

        let billing = ProjectBilling::find()
            .filter(project_billing::Column::ProjectId.eq(project_id))
            .filter(project_billing::Column::UserId.eq(user_id))
            .one(db)
            .await?;

        let Some(billing) = billing else {
            return Ok(None);
        };

        if !billing.billing_type.eq_ignore_ascii_case("hourly") {
            return Ok(None);
        }

        Ok(billing.hourly_rate)
    }

    async fn ensure_activity_ownership(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        activity_id: Uuid,
    ) -> Result<(), AppError> {
        let exists = Activities::find()
            .filter(activities::Column::Id.eq(activity_id))
            .filter(activities::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .is_some();

        if !exists {
            return Err(AppError::BadRequest(codes::TIME_ENTRY_ACTIVITY_FORBIDDEN.into()));
        }

        Ok(())
    }

    async fn get_daily_duration(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        task_id: Uuid,
        work_date: NaiveDate,
        exclude_entry_id: Option<Uuid>,
    ) -> Result<i64, AppError> {
        let mut query = TimeEntries::find()
            .filter(time_entries::Column::UserId.eq(user_id))
            .filter(time_entries::Column::TaskId.eq(task_id))
            .filter(time_entries::Column::WorkDate.eq(work_date));

        if let Some(entry_id) = exclude_entry_id {
            query = query.filter(time_entries::Column::Id.ne(entry_id));
        }

        let result = query
            .select_only()
            .column_as(Expr::col(time_entries::Column::DurationMinutes).sum(), "total_duration")
            .into_tuple::<Option<i64>>()
            .one(db)
            .await?;

        Ok(result.flatten().unwrap_or(0))
    }

    async fn get_daily_totals_by_date(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        task_id: Option<Uuid>,
    ) -> Result<HashMap<String, i64>, AppError> {
        let mut query = TimeEntries::find().filter(time_entries::Column::UserId.eq(user_id));

        if let Some(task_id) = task_id {
            query = query.filter(time_entries::Column::TaskId.eq(task_id));
        }

        let rows = query
            .select_only()
            .column(time_entries::Column::WorkDate)
            .column_as(Expr::col(time_entries::Column::DurationMinutes).sum(), "total_duration")
            .group_by(time_entries::Column::WorkDate)
            .into_tuple::<(NaiveDate, Option<i64>)>()
            .all(db)
            .await?;

        let mut totals = HashMap::with_capacity(rows.len());
        for (work_date, total) in rows {
            totals.insert(work_date.to_string(), total.unwrap_or(0));
        }

        Ok(totals)
    }

    fn extract_task_filter(payload: &QueryPayload) -> Option<Uuid> {
        payload.filters.iter().find_map(|filter| {
            if filter.column != "task_id" {
                return None;
            }

            match filter.operator {
                crate::dtos::query::FilterOperator::Eq => match &filter.value {
                    JsonValue::String(raw) => Uuid::parse_str(raw).ok(),
                    _ => None,
                },
                _ => None,
            }
        })
    }

    fn ensure_daily_duration_limit(
        current_total: i64,
        new_duration: i64,
        daily_limit_minutes: i64,
    ) -> Result<(), AppError> {
        if current_total >= daily_limit_minutes {
            return Err(AppError::Conflict(codes::TIME_ENTRY_DAY_FULL.with([hours(daily_limit_minutes)])));
        }

        if current_total + new_duration > daily_limit_minutes {
            // Máximo, horas que se quieren añadir y horas ya registradas ese día.
            return Err(AppError::Conflict(codes::TIME_ENTRY_DAILY_LIMIT_EXCEEDED.with([
                hours(daily_limit_minutes),
                hours(new_duration),
                hours(current_total),
            ])));
        }

        Ok(())
    }

    fn map_time_entry_write_error(error: sea_orm::DbErr) -> AppError {
        let message = error.to_string();

        // El trigger informa máximo, horas a añadir y horas ya registradas: viajan como params.
        if let Some((_, detail)) = message.split_once("daily_limit_exceeded: ") {
            let text = detail.lines().next().unwrap_or_default();
            return AppError::Conflict(
                crate::error::match_catalog(text).unwrap_or_else(|| codes::TIME_ENTRY_DAILY_LIMIT.into()),
            );
        }

        if message.contains("idx_uq_time_entries_active_timer_per_user") {
            return AppError::Conflict(codes::TIME_ENTRY_TIMER_ALREADY_RUNNING.into());
        }

        AppError::Database(error)
    }

    fn map_detail_row(row: TimeEntryDetailRow) -> TimeEntryDetailDto {
        TimeEntryDetailDto {
            id: row.id,
            user_id: row.user_id,
            task_id: row.task_id,
            activity: row.activity,
            invoice_id: row.invoice_id,
            start_time: row.start_time,
            end_time: row.end_time,
            duration_minutes: row.duration_minutes,
            work_date: row.work_date,
            billable: row.billable,
            applied_rate: row.applied_rate,
            notes: row.notes,
            task_title: row.task_title,
            activity_name: row.activity_name,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// Minutos como horas legibles: 480 → "8", 90 → "1.5".
fn hours(minutes: i64) -> String {
    let value = minutes as f64 / 60.0;
    if value.fract() == 0.0 { format!("{value:.0}") } else { format!("{value:.1}") }
}
