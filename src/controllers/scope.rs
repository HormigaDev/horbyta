use crate::{
    dtos::{client::ClientAuthDto, scope::*, user::UserDto},
    error::AppError,
    middlewares::validator::ValidatedJson,
    services::scope::ScopeService,
    state::AppState,
};
use axum::{
    Json, Router,
    extract::{ConnectInfo, Multipart, Path, Query, State},
    http::HeaderMap,
    routing::{get, post},
};
use sea_orm::{ConnectionTrait, TransactionTrait};
use serde_json::Value;
use uuid::Uuid;

async fn project(State(s): State<AppState>, u: UserDto, Path(id): Path<Uuid>) -> Result<Json<Value>, AppError> {
    Ok(Json(ScopeService::project(s.db.as_ref(), u.id, None, id).await?))
}
async fn client_project(
    State(s): State<AppState>,
    u: ClientAuthDto,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(
        ScopeService::project(s.db.as_ref(), u.user_id, Some(u.client_id), id).await?,
    ))
}
async fn publish(
    State(s): State<AppState>,
    u: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(dto): ValidatedJson<PublishScopeDto>,
) -> Result<Json<Value>, AppError> {
    let published = dto.publish;
    let tx = s.db.begin().await?;
    let data = ScopeService::publish_with_storage(&tx, u.id, id, dto, Some(s.storage.as_ref())).await?;
    tx.commit().await?;
    if published {
        crate::services::project_conversation::publish_latest(&s, u.id, id, None).await;
    }
    Ok(Json(data))
}
async fn accept(
    ConnectInfo(peer): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    State(s): State<AppState>,
    u: ClientAuthDto,
    Path(id): Path<Uuid>,
    ValidatedJson(dto): ValidatedJson<ScopeDecisionDto>,
) -> Result<Json<Value>, AppError> {
    let tx = s.db.begin().await?;
    let trusted = std::env::var("TRUSTED_PROXY_IPS").unwrap_or_default();
    let (ip, source) = crate::middlewares::acceptance_ip::acceptance_ip(peer.ip(), &headers, &trusted);
    tx.execute(sea_orm::Statement::from_sql_and_values(
        sea_orm::DbBackend::Postgres,
        "SELECT set_config('horbyta.accepted_ip',$1,true),set_config('horbyta.ip_source',$2,true)",
        [ip.to_string().into(), source.into()],
    ))
    .await?;
    ScopeService::verify_bundle_assets(&tx, s.storage.as_ref(), u.user_id, u.client_id, id, dto.revision).await?;
    let data = ScopeService::accept(&tx, u.user_id, u.client_id, u.id, id, dto.revision).await?;
    tx.commit().await?;
    crate::services::project_conversation::publish_latest(&s, u.user_id, id, None).await;
    Ok(Json(data))
}
async fn change(State(s): State<AppState>, u: UserDto, Path(id): Path<Uuid>) -> Result<Json<Value>, AppError> {
    Ok(Json(ScopeService::change(s.db.as_ref(), u.id, None, id).await?))
}
async fn client_change(
    State(s): State<AppState>,
    u: ClientAuthDto,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(
        ScopeService::change(s.db.as_ref(), u.user_id, Some(u.client_id), id).await?,
    ))
}
async fn comment(
    State(s): State<AppState>,
    u: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(dto): ValidatedJson<ScopeCommentDto>,
) -> Result<Json<Value>, AppError> {
    change_comment_for(s, u.id, None, u.id, id, dto).await
}
async fn client_comment(
    State(s): State<AppState>,
    u: ClientAuthDto,
    Path(id): Path<Uuid>,
    ValidatedJson(dto): ValidatedJson<ScopeCommentDto>,
) -> Result<Json<Value>, AppError> {
    change_comment_for(s, u.user_id, Some(u.client_id), u.id, id, dto).await
}
async fn change_comment_for(
    s: AppState,
    owner: Uuid,
    client: Option<Uuid>,
    actor: Uuid,
    id: Uuid,
    dto: ScopeCommentDto,
) -> Result<Json<Value>, AppError> {
    let tx = s.db.begin().await?;
    let mut result = ScopeService::comment(&tx, owner, client, actor, id, dto).await?;
    let notice = if client.is_some() {
        crate::services::notifications::NotificationService::create(&tx,crate::dtos::notification::CreateNotificationInput {
            user_id:owner,preference_key:"client_portal.change_request_updated",title:"notifications.scope.message.title".into(),
            body:result["_feedback_event"]["text"].as_str().and_then(conversation_preview),
            payload:Some(serde_json::json!({"project_id":result["current"]["project_id"],"change_request_id":id,"revision":result["_feedback_event"]["version"],"message_id":result["_feedback_event"]["id"],"scope_conversation":true,"translation":{"titleKey":"notifications.scope.message.title","params":{"project":result["current"]["project_name"]}}})),
            severity:"info",source:"client",entity_type:Some("change_request"),entity_id:Some(id)
        }).await?
    } else {
        None
    };
    tx.commit().await?;
    crate::services::scope_conversation::publish_change(&s, owner, &result);
    crate::utils::notifications::publish_notification(&s, owner, notice);
    result.as_object_mut().unwrap().remove("_feedback_event");
    Ok(Json(result))
}
async fn task(State(s): State<AppState>, u: UserDto, Path(id): Path<Uuid>) -> Result<Json<Value>, AppError> {
    Ok(Json(ScopeService::task(s.db.as_ref(), u.id, id).await?))
}
async fn classify(
    State(s): State<AppState>,
    u: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(dto): ValidatedJson<ClassifyTaskDto>,
) -> Result<Json<Value>, AppError> {
    let tx = s.db.begin().await?;
    let data = ScopeService::classify(&tx, u.id, u.id, id, dto).await?;
    tx.commit().await?;
    Ok(Json(data))
}
async fn notices(State(s): State<AppState>, u: ClientAuthDto) -> Result<Json<Value>, AppError> {
    Ok(Json(
        ScopeService::notices(s.db.as_ref(), u.user_id, u.client_id).await?,
    ))
}
async fn pending(State(s): State<AppState>, u: ClientAuthDto) -> Result<Json<Value>, AppError> {
    Ok(Json(
        ScopeService::pending(s.db.as_ref(), u.user_id, u.client_id).await?,
    ))
}
async fn feedback(
    State(s): State<AppState>,
    u: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(dto): ValidatedJson<ScopeFeedbackDto>,
) -> Result<Json<Value>, AppError> {
    let tx = s.db.begin().await?;
    let mut result = ScopeService::feedback(&tx, u.id, None, u.id, id, dto).await?;
    tx.commit().await?;
    crate::services::scope_conversation::publish(&s, u.id, id, &result);
    result.as_object_mut().unwrap().remove("_feedback_event");
    Ok(Json(result))
}
async fn client_feedback(
    State(s): State<AppState>,
    u: ClientAuthDto,
    Path(id): Path<Uuid>,
    ValidatedJson(dto): ValidatedJson<ScopeFeedbackDto>,
) -> Result<Json<Value>, AppError> {
    let tx = s.db.begin().await?;
    let revision = dto.revision;
    let mut result = ScopeService::feedback(&tx, u.user_id, Some(u.client_id), u.id, id, dto).await?;
    let notice = crate::services::notifications::NotificationService::create(
        &tx,
        crate::dtos::notification::CreateNotificationInput {
            user_id: u.user_id,
            preference_key: "client_portal.change_request_updated",
            title: "notifications.scope.message.title".into(),
            body: result["_feedback_event"]["text"].as_str().and_then(conversation_preview),
            payload: Some(serde_json::json!({"project_id":id,"revision":revision,"scope_conversation":true,"message_id":result["_feedback_event"]["id"],"translation":{"titleKey":"notifications.scope.message.title","params":{"project":result["project_name"]}}})),
            severity: "info",
            source: "client",
            entity_type: Some("project"),
            entity_id: Some(id),
        },
    )
    .await?;
    tx.commit().await?;
    crate::services::scope_conversation::publish(&s, u.user_id, id, &result);
    crate::utils::notifications::publish_notification(&s, u.user_id, notice);
    result.as_object_mut().unwrap().remove("_feedback_event");
    Ok(Json(result))
}
async fn preparation(
    State(s): State<AppState>,
    u: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(dto): ValidatedJson<ScopePreparationDto>,
) -> Result<Json<Value>, AppError> {
    let tx = s.db.begin().await?;
    let result = ScopeService::preparation(&tx, u.id, id, &dto.reason).await?;
    tx.commit().await?;
    Ok(Json(result))
}
async fn requirement(
    State(s): State<AppState>,
    u: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(dto): ValidatedJson<ScopeRequirementDto>,
) -> Result<Json<Value>, AppError> {
    let tx = s.db.begin().await?;
    let result = ScopeService::requirement(&tx, u.id, id, dto.document_id, dto.revision).await?;
    tx.commit().await?;
    Ok(Json(result))
}
async fn conversation_window(
    State(s): State<AppState>,
    u: UserDto,
    Path((id, revision)): Path<(Uuid, i32)>,
    Query(q): Query<ConversationWindowQuery>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(
        crate::services::scope_conversation::window(s.db.as_ref(), u.id, None, id, revision, q).await?,
    ))
}
async fn client_conversation_window(
    State(s): State<AppState>,
    u: ClientAuthDto,
    Path((id, revision)): Path<(Uuid, i32)>,
    Query(q): Query<ConversationWindowQuery>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(
        crate::services::scope_conversation::window(s.db.as_ref(), u.user_id, Some(u.client_id), id, revision, q)
            .await?,
    ))
}
async fn upload_conversation(
    State(s): State<AppState>,
    u: UserDto,
    Path((id, revision)): Path<(Uuid, i32)>,
    multipart: Multipart,
) -> Result<Json<crate::dtos::attachment::AttachmentUploadResponseDto>, AppError> {
    upload_conversation_for(s, u.id, None, id, revision, multipart).await
}
async fn client_upload_conversation(
    State(s): State<AppState>,
    u: ClientAuthDto,
    Path((id, revision)): Path<(Uuid, i32)>,
    multipart: Multipart,
) -> Result<Json<crate::dtos::attachment::AttachmentUploadResponseDto>, AppError> {
    upload_conversation_for(s, u.user_id, Some(u.client_id), id, revision, multipart).await
}
async fn upload_conversation_for(
    s: AppState,
    owner: Uuid,
    client: Option<Uuid>,
    id: Uuid,
    revision: i32,
    multipart: Multipart,
) -> Result<Json<crate::dtos::attachment::AttachmentUploadResponseDto>, AppError> {
    crate::middlewares::rate_limit::check_user_rate_limit(
        &s.rate_limit_uploads,
        owner,
        crate::middlewares::rate_limit::UPLOADS_RATE_LIMIT,
    )?;
    let tx = s.db.begin().await?;
    crate::services::scope_conversation::authorize(&tx, owner, client, id, revision).await?;
    let result = crate::services::attachments::AttachmentService::upload_conversation_files(
        &tx,
        s.storage.as_ref(),
        owner,
        id,
        multipart,
    )
    .await?;
    if let Err(error) = tx.commit().await {
        crate::services::attachments::AttachmentService::cleanup_storage_objects(
            s.storage.as_ref(),
            &result.storage_keys,
        )
        .await;
        return Err(AppError::Database(error));
    }
    Ok(Json(result.value))
}
async fn upload_change_files_for(
    s: AppState,
    owner: Uuid,
    client: Option<Uuid>,
    id: Uuid,
    revision: i32,
    multipart: Multipart,
) -> Result<Json<crate::dtos::attachment::AttachmentUploadResponseDto>, AppError> {
    crate::middlewares::rate_limit::check_user_rate_limit(
        &s.rate_limit_uploads,
        owner,
        crate::middlewares::rate_limit::UPLOADS_RATE_LIMIT,
    )?;
    let tx = s.db.begin().await?;
    let project = crate::services::scope_conversation::authorize_change(&tx, owner, client, id, revision).await?;
    let row = tx
        .query_one(sea_orm::Statement::from_sql_and_values(
            sea_orm::DbBackend::Postgres,
            "SELECT version FROM change_requests WHERE id=$1 FOR UPDATE",
            [id.into()],
        ))
        .await?
        .ok_or(AppError::InternalError)?;
    if row.try_get::<i32>("", "version")? != revision {
        return Err(AppError::Conflict(
            "La revisión cambió o no es accesible; vuelva a leerla.".into(),
        ));
    }
    let result = crate::services::attachments::AttachmentService::upload_conversation_files(
        &tx,
        s.storage.as_ref(),
        owner,
        project,
        multipart,
    )
    .await?;
    if let Err(error) = tx.commit().await {
        crate::services::attachments::AttachmentService::cleanup_storage_objects(
            s.storage.as_ref(),
            &result.storage_keys,
        )
        .await;
        return Err(AppError::Database(error));
    }
    Ok(Json(result.value))
}
async fn change_messages(
    State(s): State<AppState>,
    u: UserDto,
    Path((id, version)): Path<(Uuid, i32)>,
    Query(q): Query<ConversationWindowQuery>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(
        crate::services::scope_conversation::change_window(s.db.as_ref(), u.id, None, id, version, q).await?,
    ))
}
async fn client_change_messages(
    State(s): State<AppState>,
    u: ClientAuthDto,
    Path((id, version)): Path<(Uuid, i32)>,
    Query(q): Query<ConversationWindowQuery>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(
        crate::services::scope_conversation::change_window(s.db.as_ref(), u.user_id, Some(u.client_id), id, version, q)
            .await?,
    ))
}
async fn change_files(
    State(s): State<AppState>,
    u: UserDto,
    Path((id, version)): Path<(Uuid, i32)>,
    multipart: Multipart,
) -> Result<Json<crate::dtos::attachment::AttachmentUploadResponseDto>, AppError> {
    upload_change_files_for(s, u.id, None, id, version, multipart).await
}
async fn client_change_files(
    State(s): State<AppState>,
    u: ClientAuthDto,
    Path((id, version)): Path<(Uuid, i32)>,
    multipart: Multipart,
) -> Result<Json<crate::dtos::attachment::AttachmentUploadResponseDto>, AppError> {
    upload_change_files_for(s, u.user_id, Some(u.client_id), id, version, multipart).await
}

pub fn scope_controller() -> Router<AppState> {
    Router::new()
        .merge(crate::controllers::project_conversation::router())
        .route("/projects/{id}/conversation/{revision}", get(conversation_window))
        .route(
            "/client/projects/{id}/conversation/{revision}",
            get(client_conversation_window),
        )
        .route(
            "/projects/{id}/conversation/{revision}/files",
            post(upload_conversation),
        )
        .route(
            "/client/projects/{id}/conversation/{revision}/files",
            post(client_upload_conversation),
        )
        .route("/projects/{id}", get(project).post(publish))
        .route("/client/projects/{id}", get(client_project))
        .route("/client/projects/{id}/accept", post(accept))
        .route("/projects/{id}/feedback", post(feedback))
        .route("/client/projects/{id}/feedback", post(client_feedback))
        .route("/tasks/{id}/preparation", post(preparation))
        .route("/tasks/{id}/requirements", post(requirement))
        .route("/changes/{id}/conversation/{revision}", get(change_messages))
        .route(
            "/client/changes/{id}/conversation/{revision}",
            get(client_change_messages),
        )
        .route("/changes/{id}/conversation/{revision}/files", post(change_files))
        .route(
            "/client/changes/{id}/conversation/{revision}/files",
            post(client_change_files),
        )
        .route("/changes/{id}", get(change))
        .route("/client/changes/{id}", get(client_change))
        .route("/changes/{id}/comments", post(comment))
        .route("/client/changes/{id}/comments", post(client_comment))
        .route("/tasks/{id}", get(task).post(classify))
        .route("/client/pending", get(pending))
        .route("/client/notices", get(notices))
}

fn conversation_preview(text: &str) -> Option<String> {
    let preview = text.split_whitespace().collect::<Vec<_>>().join(" ");
    (!preview.is_empty() && preview.chars().count() <= 60).then_some(preview)
}

#[cfg(test)]
mod notification_tests {
    #[test]
    fn short_conversation_preview_only() {
        use super::conversation_preview;
        assert_eq!(
            conversation_preview("  prueba\n corta ").as_deref(),
            Some("prueba corta")
        );
        assert!(conversation_preview(&"x".repeat(61)).is_none());
        assert!(conversation_preview(&"😀".repeat(61)).is_none());
        assert_eq!(conversation_preview(&"😀".repeat(60)).unwrap().chars().count(), 60);
        assert!(conversation_preview(" \n ").is_none());
    }
}
