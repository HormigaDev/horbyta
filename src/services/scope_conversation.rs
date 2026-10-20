use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::error_codes as codes;
use crate::{dtos::scope::ConversationWindowQuery, error::AppError};

pub async fn authorize(
    db: &impl ConnectionTrait,
    owner: Uuid,
    client: Option<Uuid>,
    project: Uuid,
    revision: i32,
) -> Result<(), AppError> {
    let row = db.query_one(Statement::from_sql_and_values(DbBackend::Postgres,
        "SELECT 1 AS allowed FROM projects p JOIN scope_revisions r ON r.project_id=p.id WHERE p.id=$1 AND p.user_id=$2 AND ($3::uuid IS NULL OR p.client_id=$3) AND r.revision=$4 AND r.published",
        vec![project.into(),owner.into(),client.into(),revision.into()])).await?;
    if row.is_none() {
        return Err(AppError::NotFound(codes::SCOPE_RECORD_NOT_FOUND.into()));
    }
    Ok(())
}

pub async fn window(
    db: &impl ConnectionTrait,
    owner: Uuid,
    client: Option<Uuid>,
    project: Uuid,
    revision: i32,
    query: ConversationWindowQuery,
) -> Result<Value, AppError> {
    authorize(db, owner, client, project, revision).await?;
    if [
        query.before.is_some(),
        query.after.is_some(),
        query.around.is_some(),
        query.notification.is_some(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count()
        > 1
    {
        return Err(AppError::BadRequest(codes::CONVERSATION_CURSOR_INVALID.into()));
    }
    let mut around = query.around;
    if let Some(notice) = query.notification {
        if client.is_some() {
            return Err(AppError::BadRequest(codes::CONVERSATION_CURSOR_INVALID.into()));
        }
        // Recuperar avisos anteriores únicamente cuando el texto identifica un mensaje único.
        let row = db.query_one(Statement::from_sql_and_values(DbBackend::Postgres,
            "SELECT CASE WHEN count(*)=1 THEN min(e.id::text)::uuid END AS id FROM notifications n JOIN scope_project_events e ON e.project_id=$2 AND e.revision=$3 AND e.actor_kind='client' AND e.text=n.body WHERE n.id=$1 AND n.user_id=$4 AND n.payload->>'project_id'=$2::text AND n.payload->>'revision'=$3::text",
            [notice.into(),project.into(),revision.into(),owner.into()])).await?.ok_or(AppError::InternalError)?;
        around = row.try_get("", "id")?;
    }
    if let Some(anchor) = around {
        if db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT 1 FROM scope_project_events WHERE id=$1 AND project_id=$2 AND revision=$3",
                [anchor.into(), project.into(), revision.into()],
            ))
            .await?
            .is_none()
        {
            return Err(AppError::NotFound(codes::SCOPE_RECORD_NOT_FOUND.into()));
        }
    }
    let order = if query.after.is_some() { "ASC" } else { "DESC" };
    let segment = if around.is_some() {
        "SELECT * FROM (SELECT e.* FROM scope_project_events e WHERE e.project_id=$1 AND e.revision=$2 AND $3::uuid IS NULL AND $4::uuid IS NULL AND (e.recorded_at,e.id)<=(SELECT recorded_at,id FROM scope_project_events WHERE id=$6) ORDER BY e.recorded_at DESC,e.id DESC LIMIT 50) older UNION ALL SELECT * FROM (SELECT e.* FROM scope_project_events e WHERE e.project_id=$1 AND e.revision=$2 AND (e.recorded_at,e.id)>(SELECT recorded_at,id FROM scope_project_events WHERE id=$6) ORDER BY e.recorded_at,e.id LIMIT 50) newer".to_owned()
    } else {
        format!(
            "SELECT e.* FROM scope_project_events e WHERE e.project_id=$1 AND e.revision=$2 AND ($3::uuid IS NULL OR (e.recorded_at,e.id)<(SELECT recorded_at,id FROM scope_project_events WHERE id=$3 AND project_id=$1 AND revision=$2)) AND ($4::uuid IS NULL OR (e.recorded_at,e.id)>(SELECT recorded_at,id FROM scope_project_events WHERE id=$4 AND project_id=$1 AND revision=$2)) ORDER BY e.recorded_at {order},e.id {order} LIMIT 100"
        )
    };
    let sql = format!(
        r#"WITH segment AS (
        {segment}
    ) SELECT jsonb_build_object(
        'focus_id',$6::uuid,
        'items',coalesce((SELECT jsonb_agg(to_jsonb(e)||jsonb_build_object(
            'actor_name',CASE WHEN e.actor_kind='professional' THEN (SELECT name FROM users WHERE id=e.actor_id)
                ELSE (SELECT c.name FROM client_auth a JOIN clients c ON c.id=a.client_id WHERE a.id=e.actor_id AND a.user_id=$5) END,
            'attachments',coalesce((SELECT jsonb_agg(jsonb_build_object('id',a.attachment_id,'original_name',a.metadata->>'original_name','mime_type',a.metadata->>'mime_type','size_bytes',a.metadata->'size_bytes') ORDER BY a.attachment_id) FROM scope_project_event_attachments a WHERE a.event_id=e.id),'[]')
        ) ORDER BY e.recorded_at,e.id) FROM segment e),'[]'),
        'has_older',EXISTS(SELECT 1 FROM scope_project_events e WHERE e.project_id=$1 AND e.revision=$2 AND (e.recorded_at,e.id)<(SELECT recorded_at,id FROM segment ORDER BY recorded_at,id LIMIT 1)),
        'has_newer',EXISTS(SELECT 1 FROM scope_project_events e WHERE e.project_id=$1 AND e.revision=$2 AND (e.recorded_at,e.id)>(SELECT recorded_at,id FROM segment ORDER BY recorded_at DESC,id DESC LIMIT 1))
    ) AS data"#
    );
    let row = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            vec![
                project.into(),
                revision.into(),
                query.before.into(),
                query.after.into(),
                owner.into(),
                around.into(),
            ],
        ))
        .await?
        .ok_or(AppError::InternalError)?;
    Ok(row.try_get("", "data")?)
}

pub fn publish(state: &crate::state::AppState, owner: Uuid, project: Uuid, result: &Value) {
    let Some(event) = result.get("_feedback_event") else {
        return;
    };
    let client = result["client_id"].as_str().and_then(|id| Uuid::parse_str(id).ok());
    let _ = state.conversation_bus.send(crate::dtos::scope::ConversationEvent {
        user_id: owner, client_id: client, project_id: project,
        data: json!({"project_id":project,"revision":event["revision"],"event_id":event["id"],
            "kind":event["kind"],"actor_id":event["actor_id"],"actor_kind":event["actor_kind"],"actor_name":event["actor_name"],
            "project_name":result["project_name"]}),
    });
}

pub async fn authorize_change(
    db: &impl ConnectionTrait,
    owner: Uuid,
    client: Option<Uuid>,
    id: Uuid,
    version: i32,
) -> Result<Uuid, AppError> {
    let row=db.query_one(Statement::from_sql_and_values(DbBackend::Postgres,
        "SELECT c.project_id FROM change_requests c JOIN scope_change_revisions r ON r.change_request_id=c.id WHERE c.id=$1 AND c.user_id=$2 AND ($3::uuid IS NULL OR c.client_id=$3) AND r.version=$4 AND ($3::uuid IS NULL OR r.snapshot->>'status'<>'draft')",
        [id.into(),owner.into(),client.into(),version.into()])).await?.ok_or_else(||AppError::NotFound(codes::SCOPE_RECORD_NOT_FOUND.into()))?;
    Ok(row.try_get("", "project_id")?)
}
pub async fn change_window(
    db: &impl ConnectionTrait,
    owner: Uuid,
    client: Option<Uuid>,
    project: Uuid,
    revision: i32,
    query: ConversationWindowQuery,
) -> Result<Value, AppError> {
    authorize_change(db, owner, client, project, revision).await?;
    if query.notification.is_some() {
        return Err(AppError::BadRequest(codes::CONVERSATION_CURSOR_INVALID.into()));
    }
    if [
        query.before.is_some(),
        query.after.is_some(),
        query.around.is_some(),
        query.notification.is_some(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count()
        > 1
    {
        return Err(AppError::BadRequest(codes::CONVERSATION_CURSOR_INVALID.into()));
    }
    let around = query.around;
    if let Some(anchor) = around {
        if db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT 1 FROM scope_events WHERE id=$1 AND change_request_id=$2 AND version=$3 AND kind='comment'",
                [anchor.into(), project.into(), revision.into()],
            ))
            .await?
            .is_none()
        {
            return Err(AppError::NotFound(codes::SCOPE_RECORD_NOT_FOUND.into()));
        }
    }
    let order = if query.after.is_some() { "ASC" } else { "DESC" };
    let segment = if around.is_some() {
        "SELECT * FROM (SELECT e.* FROM scope_events e WHERE e.change_request_id=$1 AND e.version=$2 AND e.kind='comment' AND $3::uuid IS NULL AND $4::uuid IS NULL AND (e.recorded_at,e.id)<=(SELECT recorded_at,id FROM scope_events WHERE id=$6) ORDER BY e.recorded_at DESC,e.id DESC LIMIT 50) older UNION ALL SELECT * FROM (SELECT e.* FROM scope_events e WHERE e.change_request_id=$1 AND e.version=$2 AND e.kind='comment' AND (e.recorded_at,e.id)>(SELECT recorded_at,id FROM scope_events WHERE id=$6) ORDER BY e.recorded_at,e.id LIMIT 50) newer".to_owned()
    } else {
        format!(
            "SELECT e.* FROM scope_events e WHERE e.change_request_id=$1 AND e.version=$2 AND e.kind='comment' AND ($3::uuid IS NULL OR (e.recorded_at,e.id)<(SELECT recorded_at,id FROM scope_events WHERE id=$3 AND change_request_id=$1 AND version=$2 AND kind='comment')) AND ($4::uuid IS NULL OR (e.recorded_at,e.id)>(SELECT recorded_at,id FROM scope_events WHERE id=$4 AND change_request_id=$1 AND version=$2 AND kind='comment')) ORDER BY e.recorded_at {order},e.id {order} LIMIT 100"
        )
    };
    let sql = format!(
        r#"WITH segment AS (
        {segment}
    ) SELECT jsonb_build_object(
        'focus_id',$6::uuid,
        'items',coalesce((SELECT jsonb_agg(to_jsonb(e)||jsonb_build_object('revision',e.version,'text',e.body->>'text','document_id',null,
            'actor_name',CASE WHEN e.actor_kind='professional' THEN (SELECT name FROM users WHERE id=e.actor_id)
                ELSE (SELECT c.name FROM client_auth a JOIN clients c ON c.id=a.client_id WHERE a.id=e.actor_id AND a.user_id=$5) END,
            'attachments',coalesce((SELECT jsonb_agg(jsonb_build_object('id',a.attachment_id,'original_name',a.metadata->>'original_name','mime_type',a.metadata->>'mime_type','size_bytes',a.metadata->'size_bytes') ORDER BY a.attachment_id) FROM scope_change_event_attachments a WHERE a.event_id=e.id),'[]')
        ) ORDER BY e.recorded_at,e.id) FROM segment e),'[]'),
        'has_older',EXISTS(SELECT 1 FROM scope_events e WHERE e.change_request_id=$1 AND e.version=$2 AND e.kind='comment' AND (e.recorded_at,e.id)<(SELECT recorded_at,id FROM segment ORDER BY recorded_at,id LIMIT 1)),
        'has_newer',EXISTS(SELECT 1 FROM scope_events e WHERE e.change_request_id=$1 AND e.version=$2 AND e.kind='comment' AND (e.recorded_at,e.id)>(SELECT recorded_at,id FROM segment ORDER BY recorded_at DESC,id DESC LIMIT 1))
    ) AS data"#
    );
    let row = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            vec![
                project.into(),
                revision.into(),
                query.before.into(),
                query.after.into(),
                owner.into(),
                around.into(),
            ],
        ))
        .await?
        .ok_or(AppError::InternalError)?;
    Ok(row.try_get("", "data")?)
}

pub fn publish_change(state: &crate::state::AppState, owner: Uuid, result: &Value) {
    let current = &result["current"];
    let Some(project) = current["project_id"].as_str().and_then(|id| Uuid::parse_str(id).ok()) else {
        return;
    };
    let event = &result["_feedback_event"];
    let client = current["client_id"].as_str().and_then(|id| Uuid::parse_str(id).ok());
    let client = if current["status"] == "draft" { None } else { client };
    let _=state.conversation_bus.send(crate::dtos::scope::ConversationEvent {
        user_id:owner,client_id:client,project_id:project,
        data:json!({"project_id":project,"change_request_id":current["id"],"revision":event["version"],"event_id":event["id"],"kind":event["kind"],"actor_kind":event["actor_kind"],"actor_name":event["actor_name"],"project_name":current["project_name"]})
    });
}
