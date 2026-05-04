use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, patch, post},
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    dtos::{
        invoice::{CreateInvoiceDto, InvoiceStatus, UpdateInvoiceDto, UpdateInvoiceStatusDto, invoice_filter_schema},
        query::QueryPayload,
        user::UserDto,
        utils::AppResult,
    },
    middlewares::validator::{ValidatedJson, ValidatedSearchQuery},
    services::{invoices::InvoiceService, notifications::NotificationService},
    state::AppState,
};

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub(crate) struct BillableTasksQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub invoice_id: Option<Uuid>,
}

#[utoipa::path(
    get,
    path = "/api/v1/invoices/search",
    tag = "Invoices",
    params(QueryPayload),
    responses(
        (status = 200, description = "Paginated list of invoices"),
        (status = 400, description = "Invalid query"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn search_invoices(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedSearchQuery(payload): ValidatedSearchQuery<QueryPayload>,
) -> AppResult<impl IntoResponse> {
    invoice_filter_schema().validate_payload(&payload)?;

    let (items, total) = InvoiceService::search(state.db.as_ref(), user.id, payload.clone()).await?;

    Ok(Json(json!({
        "payload": payload,
        "items": items,
        "total": total,
    })))
}

#[utoipa::path(
    get,
    path = "/api/v1/invoices/{id}",
    tag = "Invoices",
    params(("id" = Uuid, Path, description = "Invoice id")),
    responses(
        (status = 200, description = "Invoice detail", body = crate::dtos::invoice::InvoiceDetailDto),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Invoice not found")
    )
)]
pub(crate) async fn get_invoice(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let invoice = InvoiceService::get_by_id(state.db.as_ref(), user.id, id).await?;
    Ok(Json(invoice))
}

#[utoipa::path(
    post,
    path = "/api/v1/invoices/",
    tag = "Invoices",
    request_body = CreateInvoiceDto,
    responses(
        (status = 200, description = "Invoice created", body = crate::dtos::invoice::InvoiceDetailDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(crate) async fn create_invoice(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<CreateInvoiceDto>,
) -> AppResult<impl IntoResponse> {
    let invoice = InvoiceService::create(state.db.as_ref(), user.id, payload).await?;
    Ok(Json(invoice))
}

#[utoipa::path(
    put,
    path = "/api/v1/invoices/{id}",
    tag = "Invoices",
    params(("id" = Uuid, Path, description = "Invoice id")),
    request_body = UpdateInvoiceDto,
    responses(
        (status = 200, description = "Invoice updated", body = crate::dtos::invoice::InvoiceDetailDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Invoice not found")
    )
)]
pub(crate) async fn update_invoice(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateInvoiceDto>,
) -> AppResult<impl IntoResponse> {
    let invoice = InvoiceService::update(state.db.as_ref(), user.id, id, payload).await?;
    Ok(Json(invoice))
}

#[utoipa::path(
    patch,
    path = "/api/v1/invoices/{id}/status",
    tag = "Invoices",
    params(("id" = Uuid, Path, description = "Invoice id")),
    request_body = UpdateInvoiceStatusDto,
    responses(
        (status = 200, description = "Invoice status updated", body = crate::dtos::invoice::InvoiceDetailDto),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Invoice not found")
    )
)]
pub(crate) async fn update_invoice_status(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdateInvoiceStatusDto>,
) -> AppResult<impl IntoResponse> {
    let requested_status = payload.status.clone();
    let invoice = InvoiceService::update_status(state.db.as_ref(), user.id, id, payload).await?;

    let notification_result = match requested_status {
        InvoiceStatus::Open => NotificationService::notify_invoice_opened(&state, &invoice).await,
        InvoiceStatus::Paid => NotificationService::notify_invoice_paid(&state, &invoice).await,
        _ => Ok(()),
    };

    if let Err(error) = notification_result {
        tracing::error!("Failed to create invoice status notification: {}", error);
    }

    Ok(Json(invoice))
}

#[utoipa::path(
    delete,
    path = "/api/v1/invoices/{id}",
    tag = "Invoices",
    params(("id" = Uuid, Path, description = "Invoice id")),
    responses(
        (status = 200, description = "Invoice deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Invoice not found")
    )
)]
pub(crate) async fn delete_invoice(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    InvoiceService::delete(state.db.as_ref(), user.id, id).await?;
    Ok(Json(json!({ "message": "Factura eliminada correctamente" })))
}

#[utoipa::path(
    get,
    path = "/api/v1/invoices/project/{project_id}/billable-tasks",
    tag = "Invoices",
    params(
        ("project_id" = Uuid, Path, description = "Project id"),
        BillableTasksQuery
    ),
    responses(
        (status = 200, description = "Billable tasks for invoice creation", body = crate::dtos::invoice::BillableTasksResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Project not found")
    )
)]
pub(crate) async fn get_billable_tasks(
    State(state): State<AppState>,
    user: UserDto,
    Path(project_id): Path<Uuid>,
    Query(query): Query<BillableTasksQuery>,
) -> AppResult<impl IntoResponse> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(100);

    let result =
        InvoiceService::get_billable_tasks(state.db.as_ref(), user.id, project_id, query.invoice_id, page, per_page)
            .await?;

    Ok(Json(result))
}

pub fn invoices_controller() -> Router<AppState> {
    Router::new()
        .route("/search", get(search_invoices))
        .route("/", post(create_invoice))
        .route("/project/{project_id}/billable-tasks", get(get_billable_tasks))
        .route("/{id}", get(get_invoice).put(update_invoice).delete(delete_invoice))
        .route("/{id}/status", patch(update_invoice_status))
}
