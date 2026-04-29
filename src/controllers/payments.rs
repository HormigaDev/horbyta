use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
};
use sea_orm::TransactionTrait;
use serde_json::json;
use uuid::Uuid;

use crate::{
    dtos::{
        payment::{CreatePaymentDto, CreateRefundDto, UpdatePaymentDto, payment_filter_schema},
        query::QueryPayload,
        user::UserDto,
        utils::AppResult,
    },
    middlewares::validator::{ValidatedJson, ValidatedSearchQuery},
    services::{notifications::NotificationService, payments::PaymentService},
    state::AppState,
};

async fn search_payments(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedSearchQuery(payload): ValidatedSearchQuery<QueryPayload>,
) -> AppResult<impl IntoResponse> {
    payment_filter_schema().validate_payload(&payload)?;

    let (items, total) = PaymentService::search(state.db.as_ref(), user.id, payload.clone()).await?;

    Ok(Json(json!({
        "payload": payload,
        "items": items,
        "total": total,
    })))
}

async fn get_payment(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let payment = PaymentService::get_by_id(state.db.as_ref(), user.id, id).await?;
    Ok(Json(payment))
}

async fn create_payment(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<CreatePaymentDto>,
) -> AppResult<impl IntoResponse> {
    let txn = state.db.begin().await?;
    let payment = PaymentService::create(&txn, user.id, payload).await?;

    txn.commit().await?;

    if let Err(error) = NotificationService::notify_payment_received(&state, user.id, &payment).await {
        tracing::error!("Failed to create payment notification: {}", error);
    }

    Ok(Json(payment))
}

async fn create_refund(
    State(state): State<AppState>,
    user: UserDto,
    ValidatedJson(payload): ValidatedJson<CreateRefundDto>,
) -> AppResult<impl IntoResponse> {
    let payment = PaymentService::create_refund(state.db.as_ref(), user.id, payload).await?;
    Ok(Json(payment))
}

async fn update_payment(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(payload): ValidatedJson<UpdatePaymentDto>,
) -> AppResult<impl IntoResponse> {
    let payment = PaymentService::update(state.db.as_ref(), user.id, id, payload).await?;
    Ok(Json(payment))
}

async fn delete_payment(
    State(state): State<AppState>,
    user: UserDto,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    PaymentService::delete(state.db.as_ref(), user.id, id).await?;
    Ok(Json(json!({ "message": "Pago eliminado correctamente" })))
}

pub fn payments_controller() -> Router<AppState> {
    Router::new()
        .route("/search", get(search_payments))
        .route("/", post(create_payment))
        .route("/refund", post(create_refund))
        .route("/{id}", get(get_payment).put(update_payment).delete(delete_payment))
}
