use std::collections::HashMap;

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, JoinType, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, RelationTrait, Set, TransactionTrait, prelude::Decimal, sea_query::Expr,
};
use uuid::Uuid;

use crate::{
    dtos::{
        invoice::{
            BillableTaskDto, BillableTaskRow, BillableTasksResponse, BillableTimeEntryDto, BillableTimeEntryWithTask,
            CreateInvoiceDto, InvoiceDetailDto, InvoiceDetailRow, InvoiceSearchField, InvoiceStatus, InvoiceSummaryDto,
            InvoiceTaskGroupDto, InvoiceTimeEntryDto, UpdateInvoiceDto, UpdateInvoiceStatusDto,
        },
        query::QueryPayload,
    },
    entities::{
        activities, clients,
        invoices::{self, Model as InvoiceModel},
        prelude::{Invoices, Projects, Tasks, TimeEntries},
        project_billing, projects, tasks, time_entries,
    },
    error::AppError,
    utils::search::{apply_filters, apply_ordering, apply_pagination},
};

const FREELANCER_NOTES_SEPARATOR: &str = "\n\n---\n";
const MAX_TASKS_PER_INVOICE: usize = 500;
const MAX_TASKS_PER_PAGE: u64 = 100;

pub struct InvoiceService;

impl InvoiceService {
    pub async fn search(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        payload: QueryPayload,
    ) -> Result<(Vec<InvoiceSummaryDto>, u64), AppError> {
        let base_condition = Condition::all().add(invoices::Column::UserId.eq(user_id));
        let condition = apply_filters(base_condition, &payload.filters, |column| {
            column
                .parse::<InvoiceSearchField>()
                .ok()
                .map(InvoiceSearchField::filter_field)
        })?;

        let query = Invoices::find()
            .filter(condition)
            .join(JoinType::LeftJoin, invoices::Relation::Projects.def())
            .join(JoinType::LeftJoin, projects::Relation::Clients.def());

        let query = apply_ordering(
            query,
            &payload,
            InvoiceSearchField::CreatedAt.order_column(),
            false,
            |column| {
                column
                    .parse::<InvoiceSearchField>()
                    .ok()
                    .map(InvoiceSearchField::order_column)
            },
        )?;

        let total_items = query.clone().count(db).await?;

        let items = apply_pagination(
            query
                .select_only()
                .column(invoices::Column::Id)
                .column(invoices::Column::RecordNumber)
                .column(invoices::Column::ProjectId)
                .column(invoices::Column::Amount)
                .column(invoices::Column::CurrencyType)
                .column(invoices::Column::Status)
                .column(invoices::Column::DueDate)
                .column(invoices::Column::IssuedAt)
                .column(invoices::Column::CreatedAt)
                .column(invoices::Column::UpdatedAt)
                .column_as(projects::Column::Name, "project_name")
                .column_as(clients::Column::Name, "client_name"),
            &payload,
        )
        .into_model::<InvoiceSummaryDto>()
        .all(db)
        .await?;

        Ok((items, total_items))
    }

    pub async fn get_by_id(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<InvoiceDetailDto, AppError> {
        let row: InvoiceDetailRow = Invoices::find()
            .filter(invoices::Column::Id.eq(id))
            .filter(invoices::Column::UserId.eq(user_id))
            .join(JoinType::LeftJoin, invoices::Relation::Projects.def())
            .join(JoinType::LeftJoin, projects::Relation::Clients.def())
            .select_only()
            .column(invoices::Column::Id)
            .column(invoices::Column::RecordNumber)
            .column(invoices::Column::UserId)
            .column(invoices::Column::ProjectId)
            .column(invoices::Column::Amount)
            .column(invoices::Column::CurrencyType)
            .column(invoices::Column::Status)
            .column(invoices::Column::DueDate)
            .column(invoices::Column::IssuedAt)
            .column(invoices::Column::Notes)
            .column(invoices::Column::CreatedAt)
            .column(invoices::Column::UpdatedAt)
            .column_as(projects::Column::Name, "project_name")
            .column_as(projects::Column::Status, "project_status")
            .column_as(projects::Column::ClientId, "client_id")
            .column_as(clients::Column::Name, "client_name")
            .column_as(clients::Column::Email, "client_email")
            .into_model::<InvoiceDetailRow>()
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("Factura no encontrada".to_string()))?;

        let time_entries = Self::get_invoice_time_entries(db, id).await?;
        let task_groups = Self::group_time_entries_by_task(time_entries);
        let freelancer_notes = Self::extract_freelancer_notes(row.notes.as_deref());
        let status: InvoiceStatus = row
            .status
            .try_into()
            .map_err(|_| AppError::Internal("Status inválido".to_string()))?;
        Ok(InvoiceDetailDto {
            id: row.id,
            record_number: row.record_number,
            user_id: row.user_id,
            project_id: row.project_id,
            project_name: row.project_name.unwrap_or_default(),
            project_status: row.project_status.unwrap_or_default(),
            client_id: row.client_id,
            client_name: row.client_name,
            client_email: row.client_email,
            amount: row.amount,
            currency_type: row.currency_type,
            status,
            due_date: row.due_date,
            issued_at: row.issued_at,
            notes: row.notes,
            freelancer_notes,
            task_groups,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    pub async fn create(
        db: &(impl ConnectionTrait + TransactionTrait),
        user_id: Uuid,
        dto: CreateInvoiceDto,
    ) -> Result<InvoiceDetailDto, AppError> {
        Self::ensure_project_ownership(db, user_id, dto.project_id).await?;

        let currency_type = project_billing::Entity::find()
            .filter(project_billing::Column::ProjectId.eq(dto.project_id))
            .filter(project_billing::Column::UserId.eq(user_id))
            .one(db)
            .await
            .map_err(AppError::Database)?
            .map(|b| b.currency_type)
            .unwrap_or_else(|| "USD".to_string());

        let time_entries =
            Self::validate_and_fetch_time_entries(db, user_id, dto.project_id, &dto.time_entry_ids, None).await?;

        Self::validate_task_count(&time_entries)?;

        let notes = Self::build_invoice_notes(&time_entries, dto.freelancer_notes.as_deref());

        let txn = db.begin().await.map_err(AppError::Database)?;

        let invoice = invoices::ActiveModel {
            user_id: Set(user_id),
            project_id: Set(dto.project_id),
            amount: Set(dto.amount),
            currency_type: Set(currency_type),
            status: Set("draft".to_string()),
            due_date: Set(dto.due_date.map(|dt| dt.date_naive())),
            issued_at: Set(None),
            provider: Set("manual".to_string()),
            provider_external_id: Set(None),
            payment_link: Set(None),
            notes: Set(Some(notes)),
            ..Default::default()
        };

        let created = invoice.insert(&txn).await.map_err(AppError::Database)?;

        Self::link_time_entries(&txn, &dto.time_entry_ids, created.id).await?;

        txn.commit().await.map_err(AppError::Database)?;

        Self::get_by_id(db, user_id, created.id).await
    }

    pub async fn update(
        db: &(impl ConnectionTrait + TransactionTrait),
        user_id: Uuid,
        id: Uuid,
        dto: UpdateInvoiceDto,
    ) -> Result<InvoiceDetailDto, AppError> {
        let invoice = Self::find_by_id(db, user_id, id).await?;
        Self::ensure_draft(&invoice)?;

        let time_entries =
            Self::validate_and_fetch_time_entries(db, user_id, invoice.project_id, &dto.time_entry_ids, Some(id))
                .await?;

        Self::validate_task_count(&time_entries)?;

        let notes = Self::build_invoice_notes(&time_entries, dto.freelancer_notes.as_deref());

        let txn = db.begin().await.map_err(AppError::Database)?;

        Self::unlink_time_entries(&txn, id).await?;
        Self::link_time_entries(&txn, &dto.time_entry_ids, id).await?;

        let mut active: invoices::ActiveModel = invoice.into();
        active.amount = Set(dto.amount);
        active.due_date = Set(dto.due_date.map(|dt| dt.date_naive()));
        active.notes = Set(Some(notes));
        active.update(&txn).await.map_err(AppError::Database)?;

        txn.commit().await.map_err(AppError::Database)?;

        Self::get_by_id(db, user_id, id).await
    }

    pub async fn update_status(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        dto: UpdateInvoiceStatusDto,
    ) -> Result<InvoiceDetailDto, AppError> {
        let invoice = Self::find_by_id(db, user_id, id).await?;

        Self::validate_status_transition(&invoice.status, &dto.status.to_string())?;

        let mut active: invoices::ActiveModel = invoice.into();
        active.status = Set(dto.status.clone().to_string());

        if dto.status.to_string() == "open" {
            active.issued_at = Set(Some(Utc::now().fixed_offset()));
        }

        active.update(db).await.map_err(AppError::Database)?;

        Self::get_by_id(db, user_id, id).await
    }

    pub async fn delete(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let invoice = Self::find_by_id(db, user_id, id).await?;
        Self::ensure_draft(&invoice)?;

        invoices::ActiveModel {
            id: Set(invoice.id),
            ..Default::default()
        }
        .delete(db)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    pub async fn get_billable_tasks(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        project_id: Uuid,
        include_invoice_id: Option<Uuid>,
        page: u64,
        per_page: u64,
    ) -> Result<BillableTasksResponse, AppError> {
        Self::ensure_project_ownership(db, user_id, project_id).await?;

        let per_page = per_page.min(MAX_TASKS_PER_PAGE).max(1);
        let page = page.max(1);

        let billable_condition = Self::build_billable_condition(user_id, project_id, include_invoice_id);

        let total_tasks = Tasks::find()
            .filter(tasks::Column::UserId.eq(user_id))
            .filter(tasks::Column::ProjectId.eq(project_id))
            .join(JoinType::InnerJoin, tasks::Relation::TimeEntries.def())
            .filter(billable_condition.clone())
            .select_only()
            .column(tasks::Column::Id)
            .into_tuple::<Uuid>()
            .all(db)
            .await?;

        let total_count = {
            let mut seen = std::collections::HashSet::new();
            total_tasks.into_iter().filter(|id| seen.insert(*id)).count() as u64
        };

        let offset = (page - 1) * per_page;

        let task_rows: Vec<BillableTaskRow> = Tasks::find()
            .filter(tasks::Column::UserId.eq(user_id))
            .filter(tasks::Column::ProjectId.eq(project_id))
            .join(JoinType::InnerJoin, tasks::Relation::TimeEntries.def())
            .filter(billable_condition.clone())
            .select_only()
            .column_as(tasks::Column::Id, "task_id")
            .column_as(tasks::Column::RecordNumber, "task_record_number")
            .column_as(tasks::Column::Title, "task_title")
            .group_by(tasks::Column::Id)
            .group_by(tasks::Column::RecordNumber)
            .group_by(tasks::Column::Title)
            .order_by_asc(tasks::Column::RecordNumber)
            .limit(per_page)
            .offset(offset)
            .into_model::<BillableTaskRow>()
            .all(db)
            .await?;

        if task_rows.is_empty() {
            return Ok(BillableTasksResponse {
                tasks: vec![],
                total_tasks: total_count,
                page,
                per_page,
            });
        }

        let task_ids: Vec<Uuid> = task_rows.iter().map(|r| r.task_id).collect();

        let entries: Vec<BillableTimeEntryDto> = TimeEntries::find()
            .filter(time_entries::Column::UserId.eq(user_id))
            .filter(time_entries::Column::TaskId.is_in(task_ids))
            .filter(time_entries::Column::Billable.eq(true))
            .filter(time_entries::Column::DurationMinutes.gt(0))
            .filter(
                Condition::any()
                    .add(time_entries::Column::InvoiceId.is_null())
                    .add_option(include_invoice_id.map(|inv_id| time_entries::Column::InvoiceId.eq(inv_id))),
            )
            .left_join(activities::Entity)
            .select_only()
            .column(time_entries::Column::Id)
            .column(time_entries::Column::TaskId)
            .column(time_entries::Column::DurationMinutes)
            .column(time_entries::Column::WorkDate)
            .column(time_entries::Column::Billable)
            .column(time_entries::Column::AppliedRate)
            .column(time_entries::Column::Notes)
            .column_as(activities::Column::Name, "activity_name")
            .order_by_asc(time_entries::Column::WorkDate)
            .into_model::<BillableTimeEntryDto>()
            .all(db)
            .await?;

        let mut entries_by_task: HashMap<Uuid, Vec<BillableTimeEntryDto>> = HashMap::new();
        for entry in entries {
            entries_by_task.entry(entry.task_id).or_default().push(entry);
        }

        let tasks = task_rows
            .into_iter()
            .filter_map(|row| {
                let task_entries = entries_by_task.remove(&row.task_id).unwrap_or_default();
                if task_entries.is_empty() {
                    return None;
                }
                let total_duration = task_entries.iter().map(|e| i64::from(e.duration_minutes)).sum();
                Some(BillableTaskDto {
                    task_id: row.task_id,
                    task_record_number: row.task_record_number,
                    task_title: row.task_title,
                    total_duration_minutes: total_duration,
                    time_entries: task_entries,
                })
            })
            .collect();

        Ok(BillableTasksResponse {
            tasks,
            total_tasks: total_count,
            page,
            per_page,
        })
    }

    async fn find_by_id(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<InvoiceModel, AppError> {
        Invoices::find()
            .filter(invoices::Column::Id.eq(id))
            .filter(invoices::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("Factura no encontrada".to_string()))
    }

    async fn ensure_project_ownership(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        project_id: Uuid,
    ) -> Result<(), AppError> {
        Projects::find()
            .filter(crate::entities::projects::Column::Id.eq(project_id))
            .filter(crate::entities::projects::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::BadRequest("Proyecto inválido o sin permisos".to_string()))?;

        Ok(())
    }

    fn ensure_draft(invoice: &InvoiceModel) -> Result<(), AppError> {
        if invoice.status != "draft" {
            return Err(AppError::Conflict(
                "Solo se pueden modificar o eliminar facturas en estado borrador".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_status_transition(current: &str, next: &str) -> Result<(), AppError> {
        let allowed = match current {
            "draft" => ["open", "void", "", ""],
            "open" => ["paid", "void", "uncollectible", ""],
            _ => {
                return Err(AppError::Conflict(format!(
                    "No se puede cambiar el estado de una factura en estado '{current}'"
                )));
            }
        };

        if !allowed.contains(&next) {
            return Err(AppError::BadRequest(format!(
                "Transición de estado inválida: '{current}' → '{next}'"
            )));
        }

        Ok(())
    }

    async fn validate_and_fetch_time_entries(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        project_id: Uuid,
        ids: &[Uuid],
        current_invoice_id: Option<Uuid>,
    ) -> Result<Vec<BillableTimeEntryWithTask>, AppError> {
        if ids.is_empty() {
            return Err(AppError::BadRequest(
                "Debes seleccionar al menos una entrada de tiempo".to_string(),
            ));
        }

        let rows: Vec<BillableTimeEntryWithTask> = TimeEntries::find()
            .filter(time_entries::Column::UserId.eq(user_id))
            .filter(time_entries::Column::Id.is_in(ids.to_vec()))
            .filter(time_entries::Column::Billable.eq(true))
            .filter(time_entries::Column::DurationMinutes.gt(0))
            .filter(
                Condition::any()
                    .add(time_entries::Column::InvoiceId.is_null())
                    .add_option(current_invoice_id.map(|inv_id| time_entries::Column::InvoiceId.eq(inv_id))),
            )
            .join(JoinType::InnerJoin, time_entries::Relation::Tasks.def())
            .filter(tasks::Column::ProjectId.eq(project_id))
            .select_only()
            .column(time_entries::Column::Id)
            .column(time_entries::Column::TaskId)
            .column(time_entries::Column::DurationMinutes)
            .column(time_entries::Column::AppliedRate)
            .column_as(tasks::Column::RecordNumber, "task_record_number")
            .column_as(tasks::Column::Title, "task_title")
            .into_model::<BillableTimeEntryWithTask>()
            .all(db)
            .await?;

        if rows.len() != ids.len() {
            return Err(AppError::BadRequest(
                "Una o más entradas de tiempo son inválidas, no son facturables, o ya están vinculadas a otra factura"
                    .to_string(),
            ));
        }

        Ok(rows)
    }

    fn validate_task_count(entries: &[BillableTimeEntryWithTask]) -> Result<(), AppError> {
        let unique_tasks: std::collections::HashSet<Uuid> = entries.iter().map(|e| e.task_id).collect();
        if unique_tasks.len() > MAX_TASKS_PER_INVOICE {
            return Err(AppError::BadRequest(format!(
                "Una factura puede contener como máximo {MAX_TASKS_PER_INVOICE} tareas"
            )));
        }
        Ok(())
    }

    async fn link_time_entries(db: &impl ConnectionTrait, ids: &[Uuid], invoice_id: Uuid) -> Result<(), AppError> {
        if ids.is_empty() {
            return Ok(());
        }

        TimeEntries::update_many()
            .col_expr(time_entries::Column::InvoiceId, Expr::value(invoice_id))
            .filter(time_entries::Column::Id.is_in(ids.to_vec()))
            .exec(db)
            .await
            .map_err(AppError::Database)?;

        Ok(())
    }

    async fn unlink_time_entries(db: &impl ConnectionTrait, invoice_id: Uuid) -> Result<(), AppError> {
        TimeEntries::update_many()
            .col_expr(time_entries::Column::InvoiceId, Expr::value(sea_orm::Value::Uuid(None)))
            .filter(time_entries::Column::InvoiceId.eq(invoice_id))
            .exec(db)
            .await
            .map_err(AppError::Database)?;

        Ok(())
    }

    fn build_billable_condition(user_id: Uuid, _project_id: Uuid, include_invoice_id: Option<Uuid>) -> Condition {
        let base = Condition::all()
            .add(time_entries::Column::UserId.eq(user_id))
            .add(time_entries::Column::Billable.eq(true))
            .add(time_entries::Column::DurationMinutes.gt(0));

        match include_invoice_id {
            Some(inv_id) => base.add(
                Condition::any()
                    .add(time_entries::Column::InvoiceId.is_null())
                    .add(time_entries::Column::InvoiceId.eq(inv_id)),
            ),
            None => base.add(time_entries::Column::InvoiceId.is_null()),
        }
    }

    async fn get_invoice_time_entries(
        db: &impl ConnectionTrait,
        invoice_id: Uuid,
    ) -> Result<Vec<InvoiceTimeEntryDto>, AppError> {
        TimeEntries::find()
            .filter(time_entries::Column::InvoiceId.eq(invoice_id))
            .left_join(activities::Entity)
            .select_only()
            .column(time_entries::Column::Id)
            .column(time_entries::Column::TaskId)
            .column(time_entries::Column::DurationMinutes)
            .column(time_entries::Column::WorkDate)
            .column(time_entries::Column::Billable)
            .column(time_entries::Column::AppliedRate)
            .column(time_entries::Column::Notes)
            .column_as(activities::Column::Name, "activity_name")
            .order_by_asc(time_entries::Column::WorkDate)
            .into_model::<InvoiceTimeEntryDto>()
            .all(db)
            .await
            .map_err(AppError::Database)
    }

    fn group_time_entries_by_task(entries: Vec<InvoiceTimeEntryDto>) -> Vec<InvoiceTaskGroupDto> {
        let mut task_order: Vec<Uuid> = Vec::new();
        let mut groups: HashMap<Uuid, InvoiceTaskGroupDto> = HashMap::new();

        for entry in entries {
            if !groups.contains_key(&entry.task_id) {
                task_order.push(entry.task_id);
                groups.insert(
                    entry.task_id,
                    InvoiceTaskGroupDto {
                        task_id: entry.task_id,
                        task_record_number: 0,
                        task_title: String::new(),
                        total_duration_minutes: 0,
                        time_entries: Vec::new(),
                    },
                );
            }
            if let Some(group) = groups.get_mut(&entry.task_id) {
                group.total_duration_minutes += i64::from(entry.duration_minutes);
                group.time_entries.push(entry);
            }
        }

        task_order.into_iter().filter_map(|id| groups.remove(&id)).collect()
    }

    fn build_invoice_notes(entries: &[BillableTimeEntryWithTask], freelancer_notes: Option<&str>) -> String {
        let mut task_order: Vec<Uuid> = Vec::new();
        let mut task_data: HashMap<Uuid, (i64, i64, Option<Decimal>, String)> = HashMap::new();

        for entry in entries {
            let e = task_data.entry(entry.task_id).or_insert_with(|| {
                task_order.push(entry.task_id);
                (entry.task_record_number, 0, None, entry.task_title.clone())
            });
            e.1 += i64::from(entry.duration_minutes);
            if let Some(rate) = entry.applied_rate {
                let earned = rate * Decimal::from(entry.duration_minutes) / Decimal::from(60);
                e.2 = Some(e.2.map_or(earned, |acc| acc + earned));
            }
        }

        let mut lines: Vec<String> = Vec::new();
        for task_id in &task_order {
            if let Some((record_number, total_minutes, total_amount, _title)) = task_data.get(task_id) {
                let hours = total_minutes / 60;
                let mins = total_minutes % 60;
                let duration_str = if mins > 0 {
                    format!("{hours}h {mins}min")
                } else {
                    format!("{hours}h")
                };

                let line = match total_amount {
                    Some(amount) => {
                        let rounded = amount.round_dp(2);
                        format!("TASK-#{record_number:05} {duration_str} · {rounded}")
                    }
                    None => format!("TASK-#{record_number:05} {duration_str}"),
                };
                lines.push(line);
            }
        }

        let task_section = lines.join("\n");

        match freelancer_notes {
            Some(notes) if !notes.trim().is_empty() => {
                format!("{task_section}{FREELANCER_NOTES_SEPARATOR}{}", notes.trim())
            }
            _ => task_section,
        }
    }

    fn extract_freelancer_notes(notes: Option<&str>) -> Option<String> {
        notes.and_then(|n| {
            n.find(FREELANCER_NOTES_SEPARATOR)
                .map(|pos| n[pos + FREELANCER_NOTES_SEPARATOR.len()..].to_string())
        })
    }
}
