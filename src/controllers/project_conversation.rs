use crate::error_codes as codes;
use crate::{
    dtos::{client::ClientAuthDto, scope::*, user::UserDto},
    error::AppError,
    middlewares::validator::ValidatedJson,
    services::project_conversation as chat,
    state::AppState,
};
use axum::{
    Json, Router,
    extract::{Multipart, Path, Query, State},
    routing::{get, post},
};
use sea_orm::{ConnectionTrait, DbBackend, Statement, TransactionTrait};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;
#[derive(Deserialize, Default)]
struct ResourceQuery {
    #[serde(default)]
    search: String,
    #[serde(default = "resource_kind")]
    kind: String,
}
fn resource_kind() -> String {
    "resources".into()
}
async fn resource_list(
    db: &impl ConnectionTrait,
    owner: Uuid,
    client: Option<Uuid>,
    project: Uuid,
    search: &str,
    kind: &str,
) -> Result<Value, AppError> {
    match kind {
        "resources" => chat::resources(db, owner, client, project, search).await,
        "files" => chat::file_resources(db, owner, client, project, search).await,
        _ => Err(AppError::BadRequest(
            codes::CONVERSATION_REFERENCE_CATEGORY_INVALID.into(),
        )),
    }
}
async fn messages(
    State(s): State<AppState>,
    u: UserDto,
    Path(id): Path<Uuid>,
    Query(q): Query<ConversationWindowQuery>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(chat::window(s.db.as_ref(), u.id, None, id, q).await?))
}
async fn client_messages(
    State(s): State<AppState>,
    u: ClientAuthDto,
    Path(id): Path<Uuid>,
    Query(q): Query<ConversationWindowQuery>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(
        chat::window(s.db.as_ref(), u.user_id, Some(u.client_id), id, q).await?,
    ))
}
async fn inbox(State(s): State<AppState>, u: UserDto) -> Result<Json<Value>, AppError> {
    Ok(Json(chat::inbox(s.db.as_ref(), u.id, None, u.id).await?))
}
async fn client_inbox(State(s): State<AppState>, u: ClientAuthDto) -> Result<Json<Value>, AppError> {
    Ok(Json(
        chat::inbox(s.db.as_ref(), u.user_id, Some(u.client_id), u.id).await?,
    ))
}
async fn resources(
    State(s): State<AppState>,
    u: UserDto,
    Path(id): Path<Uuid>,
    Query(q): Query<ResourceQuery>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(
        resource_list(
            s.db.as_ref(),
            u.id,
            None,
            id,
            &q.search.chars().take(100).collect::<String>(),
            &q.kind,
        )
        .await?,
    ))
}
async fn client_resources(
    State(s): State<AppState>,
    u: ClientAuthDto,
    Path(id): Path<Uuid>,
    Query(q): Query<ResourceQuery>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(
        resource_list(
            s.db.as_ref(),
            u.user_id,
            Some(u.client_id),
            id,
            &q.search.chars().take(100).collect::<String>(),
            &q.kind,
        )
        .await?,
    ))
}
async fn send(
    State(s): State<AppState>,
    u: UserDto,
    Path(id): Path<Uuid>,
    ValidatedJson(dto): ValidatedJson<ProjectMessageDto>,
) -> Result<Json<Value>, AppError> {
    send_for(s, u.id, None, u.id, id, dto).await
}
async fn client_send(
    State(s): State<AppState>,
    u: ClientAuthDto,
    Path(id): Path<Uuid>,
    ValidatedJson(dto): ValidatedJson<ProjectMessageDto>,
) -> Result<Json<Value>, AppError> {
    send_for(s, u.user_id, Some(u.client_id), u.id, id, dto).await
}
async fn send_for(
    s: AppState,
    owner: Uuid,
    client: Option<Uuid>,
    actor: Uuid,
    project: Uuid,
    dto: ProjectMessageDto,
) -> Result<Json<Value>, AppError> {
    let tx = s.db.begin().await?;
    let info = chat::authorize(&tx, owner, client, project).await?;
    let text = dto.text.split_whitespace().collect::<Vec<_>>().join(" ");
    let id = chat::send(&tx, owner, client, actor, project, dto).await?;
    let notice = if client.is_some() {
        Some(crate::services::notifications::NotificationService::create(&tx,crate::dtos::notification::CreateNotificationInput {
        user_id:owner,preference_key:"client_portal.change_request_updated",title:"notifications.scope.message.title".into(),body:(!text.is_empty()&&text.chars().count()<=60).then_some(text),
        payload:Some(json!({"project_id":project,"revision":0,"message_id":id,"scope_conversation":true,"translation":{"titleKey":"notifications.scope.message.title","params":{"project":info["name"]}}})),severity:"info",source:"client",entity_type:Some("project"),entity_id:Some(project),
    }).await?)
    } else {
        None
    };
    tx.commit().await?;
    chat::publish_latest(&s, owner, project, Some(id)).await;
    if let Some(notice) = notice {
        crate::utils::notifications::publish_notification(&s, owner, notice);
    }
    Ok(Json(json!({"id":id})))
}
async fn read(
    State(s): State<AppState>,
    u: UserDto,
    Path(id): Path<Uuid>,
    Json(dto): Json<ConversationReadDto>,
) -> Result<Json<Value>, AppError> {
    read_for(s, u.id, None, u.id, id, dto.message_id).await
}
async fn client_read(
    State(s): State<AppState>,
    u: ClientAuthDto,
    Path(id): Path<Uuid>,
    Json(dto): Json<ConversationReadDto>,
) -> Result<Json<Value>, AppError> {
    read_for(s, u.user_id, Some(u.client_id), u.id, id, dto.message_id).await
}
async fn read_for(
    s: AppState,
    owner: Uuid,
    client: Option<Uuid>,
    actor: Uuid,
    project: Uuid,
    id: Uuid,
) -> Result<Json<Value>, AppError> {
    let tx = s.db.begin().await?;
    chat::read(&tx, owner, client, actor, project, id).await?;
    tx.commit().await?;
    let mut data = json!({"project_id":project,"event_id":id,"revision":0,"kind":"read","actor_kind":if client.is_some(){"client"}else{"professional"},"actor_id":actor});
    if client.is_none() {
        if let Some(row)=s.db.query_one(Statement::from_sql_and_values(DbBackend::Postgres,"SELECT (SELECT count(*) FROM notifications WHERE user_id=$1 AND NOT is_read) AS unread_count, coalesce((SELECT jsonb_agg(id) FROM (SELECT id FROM notifications WHERE user_id=$1 AND is_read AND payload->>'project_id'=$2::text AND payload->>'scope_conversation'='true' ORDER BY created_at DESC LIMIT 100) n),'[]') AS read_ids",[owner.into(),project.into()])).await? {
            data["unread_count"]=json!(row.try_get::<i64>("","unread_count")?);data["read_ids"]=row.try_get("","read_ids")?;
        }
    }
    let _ = s.conversation_bus.send(ConversationEvent {
        user_id: owner,
        client_id: client,
        project_id: project,
        data,
    });
    Ok(Json(json!({"ok":true})))
}
async fn files(
    State(s): State<AppState>,
    u: UserDto,
    Path(id): Path<Uuid>,
    m: Multipart,
) -> Result<Json<crate::dtos::attachment::AttachmentUploadResponseDto>, AppError> {
    files_for(s, u.id, None, id, m).await
}
async fn client_files(
    State(s): State<AppState>,
    u: ClientAuthDto,
    Path(id): Path<Uuid>,
    m: Multipart,
) -> Result<Json<crate::dtos::attachment::AttachmentUploadResponseDto>, AppError> {
    files_for(s, u.user_id, Some(u.client_id), id, m).await
}
async fn files_for(
    s: AppState,
    owner: Uuid,
    client: Option<Uuid>,
    id: Uuid,
    m: Multipart,
) -> Result<Json<crate::dtos::attachment::AttachmentUploadResponseDto>, AppError> {
    crate::middlewares::rate_limit::check_user_rate_limit(
        &s.rate_limit_uploads,
        owner,
        crate::middlewares::rate_limit::UPLOADS_RATE_LIMIT,
    )?;
    let tx = s.db.begin().await?;
    chat::authorize(&tx, owner, client, id).await?;
    let upload = crate::services::attachments::AttachmentService::upload_conversation_files(
        &tx,
        s.storage.as_ref(),
        owner,
        id,
        m,
    )
    .await?;
    if let Err(e) = tx.commit().await {
        crate::services::attachments::AttachmentService::cleanup_storage_objects(
            s.storage.as_ref(),
            &upload.storage_keys,
        )
        .await;
        return Err(AppError::Database(e));
    }
    Ok(Json(upload.value))
}
async fn propose(
    State(s): State<AppState>,
    u: UserDto,
    Path((project, message)): Path<(Uuid, Uuid)>,
    ValidatedJson(dto): ValidatedJson<ConversationChangeDto>,
) -> Result<Json<Value>, AppError> {
    propose_for(s, u.id, None, u.id, project, message, dto).await
}
async fn client_propose(
    State(s): State<AppState>,
    u: ClientAuthDto,
    Path((project, message)): Path<(Uuid, Uuid)>,
    ValidatedJson(dto): ValidatedJson<ConversationChangeDto>,
) -> Result<Json<Value>, AppError> {
    propose_for(s, u.user_id, Some(u.client_id), u.id, project, message, dto).await
}
async fn propose_for(
    s: AppState,
    owner: Uuid,
    client: Option<Uuid>,
    actor: Uuid,
    project: Uuid,
    message: Uuid,
    dto: ConversationChangeDto,
) -> Result<Json<Value>, AppError> {
    let tx = s.db.begin().await?;
    let info = chat::authorize(&tx, owner, client, project).await?;
    let source=tx.query_one(Statement::from_sql_and_values(DbBackend::Postgres,"SELECT to_jsonb(e) AS data FROM project_conversation_timeline e WHERE e.project_id=$1 AND e.id=$2 AND e.kind='message'",[project.into(),message.into()])).await?.ok_or_else(||AppError::NotFound(codes::CONVERSATION_MESSAGE_NOT_FOUND.into()))?;
    let snapshot: Value = source.try_get("", "data")?;
    crate::services::scope::ScopeService::actor(&tx, actor, if client.is_some() { "client" } else { "professional" })
        .await?;
    let mut notice = None;
    let id = if let Some(client) = client {
        let created = crate::services::change_requests::ChangeRequestService::create_by_client(
            &tx,
            owner,
            client,
            crate::dtos::change_request::ClientCreateChangeRequestDto {
                project_id: project,
                title: dto.title,
                client_request_description: dto.description,
            },
        )
        .await?;
        notice = crate::services::notifications::NotificationService::notify_client_change_request_created(
            &tx, owner, client, &created,
        )
        .await?;
        created.id
    } else {
        let client = info["client_id"]
            .as_str()
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| AppError::BadRequest(codes::CONVERSATION_CLIENT_REQUIRED.into()))?;
        crate::services::change_requests::ChangeRequestService::create(
            &tx,
            owner,
            crate::dtos::change_request::CreateChangeRequestDto {
                project_id: project,
                client_id: client,
                task_id: None,
                title: dto.title,
                client_request_description: dto.description,
            },
        )
        .await?
        .id
    };
    tx.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        "INSERT INTO conversation_change_sources(change_request_id,project_id,message_id,snapshot) VALUES($1,$2,$3,$4)",
        [id.into(), project.into(), message.into(), snapshot.into()],
    ))
    .await?;
    tx.commit().await?;
    if let Some(notice) = notice {
        crate::utils::notifications::publish_notification(&s, owner, Some(notice));
    }
    if client.is_some() {
        chat::publish_latest(&s, owner, project, None).await;
    }
    Ok(Json(json!({"id":id})))
}
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/conversation/inbox", get(inbox))
        .route("/client/conversation/inbox", get(client_inbox))
        .route("/projects/{id}/conversation", get(messages).post(send))
        .route(
            "/client/projects/{id}/conversation",
            get(client_messages).post(client_send),
        )
        .route("/projects/{id}/conversation/read", post(read))
        .route("/client/projects/{id}/conversation/read", post(client_read))
        .route("/projects/{id}/conversation/resources", get(resources))
        .route("/client/projects/{id}/conversation/resources", get(client_resources))
        .route("/projects/{id}/conversation/files", post(files))
        .route("/client/projects/{id}/conversation/files", post(client_files))
        .route("/projects/{id}/conversation/{message}/change", post(propose))
        .route(
            "/client/projects/{id}/conversation/{message}/change",
            post(client_propose),
        )
}
