use crate::error_codes as codes;
use crate::{dtos::scope::*, error::AppError};
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::{Value, json};
use uuid::Uuid;

async fn json_query(db: &impl ConnectionTrait, sql: &str, values: Vec<sea_orm::Value>) -> Result<Value, AppError> {
    let row = db
        .query_one(Statement::from_sql_and_values(DbBackend::Postgres, sql, values))
        .await?
        .ok_or(AppError::InternalError)?;
    Ok(row.try_get("", "data")?)
}
pub async fn authorize(
    db: &impl ConnectionTrait,
    owner: Uuid,
    client: Option<Uuid>,
    project: Uuid,
) -> Result<Value, AppError> {
    let row=db.query_one(Statement::from_sql_and_values(DbBackend::Postgres,
        "SELECT jsonb_build_object('id',p.id,'name',p.name,'client_id',p.client_id) AS data FROM projects p WHERE p.id=$1 AND p.user_id=$2 AND ($3::uuid IS NULL OR p.client_id=$3)",
        [project.into(),owner.into(),client.into()])).await?.ok_or_else(||AppError::NotFound(codes::CONVERSATION_PROJECT_NOT_FOUND.into()))?;
    Ok(row.try_get("", "data")?)
}
pub async fn resources(
    db: &impl ConnectionTrait,
    owner: Uuid,
    client: Option<Uuid>,
    project: Uuid,
    search: &str,
) -> Result<Value, AppError> {
    authorize(db, owner, client, project).await?;
    json_query(db,r#"SELECT coalesce(jsonb_agg(to_jsonb(r)),'[]') AS data FROM (SELECT * FROM (
      SELECT 'project'::text kind,p.id,p.name label FROM projects p WHERE p.user_id=$1 AND ($2::uuid IS NULL OR p.client_id=$2) AND (p.client_id=(SELECT client_id FROM projects WHERE id=$3) OR p.id=$3) AND p.name ILIKE $4
      UNION ALL SELECT 'milestone',m.id,m.name FROM milestones m JOIN projects p ON p.id=m.project_id WHERE (p.client_id=(SELECT client_id FROM projects WHERE id=$3) OR p.id=$3) AND p.user_id=$1 AND ($2::uuid IS NULL OR p.client_id=$2) AND m.name ILIKE $4
      UNION ALL SELECT 'change',c.id,c.title FROM change_requests c JOIN projects p ON p.id=c.project_id WHERE (p.client_id=(SELECT client_id FROM projects WHERE id=$3) OR p.id=$3) AND p.user_id=$1 AND ($2::uuid IS NULL OR (p.client_id=$2 AND c.client_id=$2 AND c.status<>'draft')) AND c.client_id=p.client_id AND c.status<>'draft' AND c.title ILIKE $4
    ) candidates ORDER BY label,id LIMIT 50) r"#,vec![owner.into(),client.into(),project.into(),format!("%{search}%").into()]).await
}
// Clients can discover files already shared in published scope or conversation.
const FILE_REFERENCE_SQL: &str = "SELECT a.original_name AS label,a.mime_type,a.size_bytes FROM attachments a WHERE a.id=$1 AND a.user_id=$2 AND a.project_id=$4 AND ($3::uuid IS NULL OR EXISTS(SELECT 1 FROM scope_revisions r WHERE r.project_id=$4 AND r.published AND EXISTS(SELECT 1 FROM jsonb_array_elements(r.attachment_snapshot) f WHERE f->>'id'=a.id::text)) OR EXISTS(SELECT 1 FROM project_message_attachments f JOIN project_messages m ON m.id=f.event_id WHERE f.attachment_id=a.id AND m.project_id=$4) OR EXISTS(SELECT 1 FROM scope_project_event_attachments f JOIN scope_project_events e ON e.id=f.event_id WHERE f.attachment_id=a.id AND e.project_id=$4) OR EXISTS(SELECT 1 FROM scope_change_event_attachments f JOIN scope_events e ON e.id=f.event_id JOIN change_requests c ON c.id=e.change_request_id JOIN scope_change_revisions r ON r.change_request_id=c.id AND r.version=e.version WHERE f.attachment_id=a.id AND c.project_id=$4 AND r.snapshot->>'status'<>'draft'))";
pub async fn file_resources(
    db: &impl ConnectionTrait,
    owner: Uuid,
    client: Option<Uuid>,
    project: Uuid,
    search: &str,
) -> Result<Value, AppError> {
    authorize(db, owner, client, project).await?;
    let sql = FILE_REFERENCE_SQL
        .replace(
            "SELECT a.original_name AS label,a.mime_type,a.size_bytes",
            "SELECT 'file' AS kind,a.id,a.original_name AS label,a.mime_type,a.size_bytes",
        )
        .replace("a.id=$1 AND ", "")
        .replace("$2", "$1")
        .replace("$3", "$2")
        .replace("$4", "$3");
    json_query(db,&format!("SELECT coalesce(jsonb_agg(to_jsonb(f)),'[]') AS data FROM ({sql} AND a.original_name ILIKE $4 AND a.mime_type NOT LIKE 'audio/%' ORDER BY a.original_name,a.id LIMIT 50) f"),vec![owner.into(),client.into(),project.into(),format!("%{search}%").into()]).await
}
pub async fn window(
    db: &impl ConnectionTrait,
    owner: Uuid,
    client: Option<Uuid>,
    project: Uuid,
    query: ConversationWindowQuery,
) -> Result<Value, AppError> {
    authorize(db, owner, client, project).await?;
    if [query.before, query.after, query.around, query.notification]
        .iter()
        .filter(|x| x.is_some())
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
        let value=json_query(db,"SELECT coalesce((SELECT to_jsonb(coalesce((n.payload->>'message_id')::uuid,(SELECT CASE WHEN count(*)=1 THEN min(e.id::text)::uuid END FROM project_conversation_timeline e WHERE e.project_id=$3 AND e.kind='message' AND e.actor_kind='client' AND e.text=n.body AND (e.context->>'scope_revision'=n.payload->>'revision' OR e.context->>'change_request_id'=n.payload->>'change_request_id')))) FROM notifications n WHERE n.id=$1 AND n.user_id=$2 AND n.payload->>'project_id'=$3::text),'null') AS data",vec![notice.into(),owner.into(),project.into()]).await?;
        around = value.as_str().and_then(|s| Uuid::parse_str(s).ok());
    }
    for id in [query.before, query.after, around].into_iter().flatten() {
        if db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT 1 FROM project_conversation_timeline WHERE id=$1 AND project_id=$2",
                [id.into(), project.into()],
            ))
            .await?
            .is_none()
        {
            return Err(AppError::NotFound(codes::CONVERSATION_MESSAGE_NOT_FOUND.into()));
        }
    }
    let order = if query.after.is_some() { "ASC" } else { "DESC" };
    let segment = if around.is_some() {
        "SELECT * FROM (SELECT e.* FROM project_conversation_timeline e WHERE e.project_id=$1 AND (e.recorded_at,e.id)<=(SELECT recorded_at,id FROM project_conversation_timeline WHERE id=$4 AND project_id=$1) ORDER BY e.recorded_at DESC,e.id DESC LIMIT 50) a UNION ALL SELECT * FROM (SELECT e.* FROM project_conversation_timeline e WHERE e.project_id=$1 AND (e.recorded_at,e.id)>(SELECT recorded_at,id FROM project_conversation_timeline WHERE id=$4 AND project_id=$1) ORDER BY e.recorded_at,e.id LIMIT 50) b".to_owned()
    } else {
        format!(
            "SELECT e.* FROM project_conversation_timeline e WHERE e.project_id=$1 AND ($2::uuid IS NULL OR (e.recorded_at,e.id)<(SELECT recorded_at,id FROM project_conversation_timeline WHERE id=$2 AND project_id=$1)) AND ($3::uuid IS NULL OR (e.recorded_at,e.id)>(SELECT recorded_at,id FROM project_conversation_timeline WHERE id=$3 AND project_id=$1)) ORDER BY e.recorded_at {order},e.id {order} LIMIT 100"
        )
    };
    json_query(db,&format!(r#"WITH segment AS ({segment}) SELECT jsonb_build_object(
      'focus_id',$4::uuid,'items',coalesce((SELECT jsonb_agg(to_jsonb(e)||jsonb_build_object('revision',coalesce((context->>'scope_revision')::integer,(context->>'change_version')::integer,0),'document_id',context->'document_id','actor_name',CASE WHEN e.actor_kind='professional' THEN (SELECT name FROM users WHERE id=e.actor_id) ELSE (SELECT c.name FROM client_auth a JOIN clients c ON c.id=a.client_id WHERE a.id=e.actor_id AND a.user_id=$5) END) ORDER BY recorded_at,id) FROM segment e),'[]'),
      'has_older',EXISTS(SELECT 1 FROM project_conversation_timeline e WHERE project_id=$1 AND (recorded_at,id)<(SELECT recorded_at,id FROM segment ORDER BY recorded_at,id LIMIT 1)),
      'has_newer',EXISTS(SELECT 1 FROM project_conversation_timeline e WHERE project_id=$1 AND (recorded_at,id)>(SELECT recorded_at,id FROM segment ORDER BY recorded_at DESC,id DESC LIMIT 1))) AS data"#),vec![project.into(),query.before.into(),query.after.into(),around.into(),owner.into()]).await
}
pub async fn send(
    db: &impl ConnectionTrait,
    owner: Uuid,
    client: Option<Uuid>,
    actor: Uuid,
    project: Uuid,
    dto: ProjectMessageDto,
) -> Result<Uuid, AppError> {
    authorize(db, owner, client, project).await?;
    let text = dto.text.trim();
    if text.is_empty() && dto.attachment_ids.is_empty() {
        return Err(AppError::BadRequest(codes::CONVERSATION_MESSAGE_EMPTY.into()));
    }
    let mut context = json!({});
    let revision = match dto.scope_revision {
        Some(revision) => Some(revision),
        None => db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT max(revision) AS revision FROM scope_revisions WHERE project_id=$1 AND published",
                [project.into()],
            ))
            .await?
            .ok_or(AppError::InternalError)?
            .try_get::<Option<i32>>("", "revision")?,
    };
    if let Some(revision) = revision {
        crate::services::scope_conversation::authorize(db, owner, client, project, revision).await?;
        context["scope_revision"] = json!(revision);
        if let Some(document) = dto.document_id {
            let row=db.query_one(Statement::from_sql_and_values(DbBackend::Postgres,"SELECT 1 FROM scope_revisions r WHERE r.project_id=$1 AND r.revision=$2 AND EXISTS(SELECT 1 FROM jsonb_array_elements(coalesce(r.document->'documents','[]')) d WHERE d->>'id'=$3::text)",[project.into(),revision.into(),document.into()])).await?;
            if row.is_none() {
                return Err(AppError::BadRequest(
                    codes::CONVERSATION_DOCUMENT_REFERENCE_INVALID.into(),
                ));
            }
            context["document_id"] = json!(document);
        }
    } else if dto.document_id.is_some() {
        return Err(AppError::BadRequest(
            codes::CONVERSATION_DOCUMENT_REFERENCE_INVALID.into(),
        ));
    }
    if let Some(change) = dto.change_request_id {
        let row=db.query_one(Statement::from_sql_and_values(DbBackend::Postgres,"SELECT version,title FROM change_requests WHERE id=$1 AND project_id=$2 AND user_id=$3 AND status<>'draft' AND ($4::uuid IS NULL OR client_id=$4)",[change.into(),project.into(),owner.into(),client.into()])).await?.ok_or_else(||AppError::NotFound(codes::CONVERSATION_CHANGE_REQUEST_NOT_FOUND.into()))?;
        context["change_request_id"] = json!(change);
        context["change_version"] = json!(row.try_get::<i32>("", "version")?);
        context["change_title"] = json!(row.try_get::<String>("", "title")?);
    }
    let mut mentions = vec![];
    let mut attachment_ids = dto.attachment_ids;
    let characters: Vec<char> = text.chars().collect();
    let mut ranges = Vec::new();
    for mention in dto.mentions {
        let sql = match mention.kind.as_str() {
            "project" => {
                "SELECT name AS label FROM projects WHERE id=$1 AND user_id=$2 AND ($3::uuid IS NULL OR client_id=$3) AND (client_id=(SELECT client_id FROM projects WHERE id=$4) OR id=$4)"
            }
            "milestone" => {
                "SELECT m.name AS label FROM milestones m JOIN projects p ON p.id=m.project_id WHERE m.id=$1 AND p.user_id=$2 AND ($3::uuid IS NULL OR p.client_id=$3) AND (p.client_id=(SELECT client_id FROM projects WHERE id=$4) OR p.id=$4)"
            }
            "change" => {
                "SELECT c.title AS label FROM change_requests c JOIN projects p ON p.id=c.project_id WHERE c.id=$1 AND p.user_id=$2 AND ($3::uuid IS NULL OR (p.client_id=$3 AND c.client_id=$3 AND c.status<>'draft')) AND c.status<>'draft' AND c.client_id=p.client_id AND (p.client_id=(SELECT client_id FROM projects WHERE id=$4) OR p.id=$4)"
            }
            "file" => FILE_REFERENCE_SQL,
            _ => {
                return Err(AppError::BadRequest(
                    codes::CONVERSATION_RESOURCE_REFERENCE_INVALID.into(),
                ));
            }
        };
        let mut values = vec![mention.id.into(), owner.into(), client.into()];
        values.push(project.into());
        let row = db
            .query_one(Statement::from_sql_and_values(DbBackend::Postgres, sql, values))
            .await?
            .ok_or_else(|| AppError::NotFound(codes::CONVERSATION_RESOURCE_NOT_FOUND.into()))?;
        let label: String = row.try_get("", "label")?;
        match (mention.start, mention.end) {
            (Some(start), Some(end)) if start < end && end <= characters.len() => {
                let selected: String = characters[start..end].iter().collect();
                if selected != format!("{}{label}", if mention.kind == "file" { "#" } else { "@" })
                    || ranges.iter().any(|&(a, b)| start < b && end > a)
                {
                    return Err(AppError::BadRequest(codes::CONVERSATION_MENTION_INVALID.into()));
                }
                ranges.push((start, end));
            }
            (None, None) => {}
            _ => return Err(AppError::BadRequest(codes::CONVERSATION_MENTION_INVALID.into())),
        }
        let mut reference =
            json!({"kind":mention.kind,"id":mention.id,"label":label,"start":mention.start,"end":mention.end});
        if mention.kind == "file" {
            reference["mime_type"] = json!(row.try_get::<String>("", "mime_type")?);
            reference["size_bytes"] = json!(row.try_get::<i64>("", "size_bytes")?);
            if !attachment_ids.contains(&mention.id) {
                attachment_ids.push(mention.id);
            }
        }
        mentions.push(reference);
    }
    attachment_ids.sort_unstable();
    attachment_ids.dedup();
    if attachment_ids.len() > 10 {
        return Err(AppError::BadRequest(codes::CONVERSATION_TOO_MANY_FILES.into()));
    }
    let kind = if client.is_some() { "client" } else { "professional" };
    let row=db.query_one(Statement::from_sql_and_values(DbBackend::Postgres,"INSERT INTO project_messages(project_id,actor_id,actor_kind,text,context,mentions) VALUES($1,$2,$3,$4,$5,$6) RETURNING id",vec![project.into(),actor.into(),kind.into(),text.into(),context.into(),json!(mentions).into()])).await?.ok_or(AppError::InternalError)?;
    let id: Uuid = row.try_get("", "id")?;
    for file in attachment_ids {
        let row=db.query_one(Statement::from_sql_and_values(DbBackend::Postgres,"SELECT jsonb_build_object('original_name',original_name,'mime_type',mime_type,'size_bytes',size_bytes) AS data FROM attachments WHERE id=$1 AND user_id=$2 AND project_id=$3",[file.into(),owner.into(),project.into()])).await?.ok_or_else(||AppError::NotFound(codes::CONVERSATION_ATTACHMENT_NOT_FOUND.into()))?;
        let metadata: Value = row.try_get("", "data")?;
        let mime = metadata["mime_type"].as_str().unwrap_or("");
        let name = metadata["original_name"].as_str().unwrap_or("").to_lowercase();
        if mime.starts_with("audio/")
            || [
                "mp3", "wav", "ogg", "oga", "opus", "m4a", "aac", "flac", "aiff", "aif", "amr", "wma",
            ]
            .iter()
            .any(|ext| name.ends_with(&format!(".{ext}")))
        {
            return Err(AppError::BadRequest(codes::CONVERSATION_AUDIO_NOT_SUPPORTED.into()));
        }
        db.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO project_message_attachments(event_id,attachment_id,metadata) VALUES($1,$2,$3)",
            [id.into(), file.into(), metadata.into()],
        ))
        .await?;
    }
    Ok(id)
}
pub async fn inbox(
    db: &impl ConnectionTrait,
    owner: Uuid,
    client: Option<Uuid>,
    actor: Uuid,
) -> Result<Value, AppError> {
    let kind = if client.is_some() { "client" } else { "professional" };
    json_query(db,r#"WITH unread AS (
      SELECT e.*,p.name project_name FROM projects p JOIN project_conversation_timeline e ON e.project_id=p.id LEFT JOIN project_conversation_reads r ON r.project_id=p.id AND r.actor_id=$3 AND r.actor_kind=$4
      WHERE p.user_id=$1 AND ($2::uuid IS NULL OR p.client_id=$2) AND e.kind='message' AND e.actor_kind<>$4 AND (r.last_at IS NULL OR (e.recorded_at,e.id)>(r.last_at,r.last_id))
    ), groups AS (SELECT project_id,project_name,count(*) unread_count FROM unread GROUP BY project_id,project_name)
    SELECT coalesce(jsonb_agg(to_jsonb(g)||jsonb_build_object('message_id',e.id,'recorded_at',e.recorded_at,'preview',CASE WHEN length(e.text)<=60 THEN e.text ELSE NULL END) ORDER BY e.recorded_at DESC),'[]') AS data
    FROM groups g JOIN LATERAL (SELECT * FROM unread WHERE project_id=g.project_id ORDER BY recorded_at DESC,id DESC LIMIT 1) e ON true"#,vec![owner.into(),client.into(),actor.into(),kind.into()]).await
}
pub async fn read(
    db: &impl ConnectionTrait,
    owner: Uuid,
    client: Option<Uuid>,
    actor: Uuid,
    project: Uuid,
    id: Uuid,
) -> Result<(), AppError> {
    authorize(db, owner, client, project).await?;
    let kind = if client.is_some() { "client" } else { "professional" };
    let result=db.execute(Statement::from_sql_and_values(DbBackend::Postgres,
      "INSERT INTO project_conversation_reads(project_id,actor_id,actor_kind,last_at,last_id) SELECT project_id,$3,$4,recorded_at,id FROM project_conversation_timeline WHERE id=$1 AND project_id=$2 ON CONFLICT(project_id,actor_kind,actor_id) DO UPDATE SET last_at=excluded.last_at,last_id=excluded.last_id WHERE (excluded.last_at,excluded.last_id)>(project_conversation_reads.last_at,project_conversation_reads.last_id)",vec![id.into(),project.into(),actor.into(),kind.into()])).await?;
    if result.rows_affected() == 0
        && db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT 1 FROM project_conversation_timeline WHERE id=$1 AND project_id=$2",
                [id.into(), project.into()],
            ))
            .await?
            .is_none()
    {
        return Err(AppError::NotFound(codes::CONVERSATION_MESSAGE_NOT_FOUND.into()));
    }
    // Conversation notices and message read state must agree; unrelated notices remain untouched.
    if client.is_none() {
        db.execute(Statement::from_sql_and_values(DbBackend::Postgres,"UPDATE notifications SET is_read=true, read_at=now() WHERE user_id=$1 AND payload->>'scope_conversation'='true' AND payload->>'project_id'=$2::text AND (payload->>'message_id') IN (SELECT e.id::text FROM project_conversation_timeline e JOIN project_conversation_reads r ON r.project_id=e.project_id AND r.actor_id=$1 AND r.actor_kind='professional' WHERE e.project_id=$2 AND (e.recorded_at,e.id)<=(r.last_at,r.last_id))",[owner.into(),project.into()])).await?;
    }
    Ok(())
}
pub async fn latest_event(
    db: &impl ConnectionTrait,
    owner: Uuid,
    project: Uuid,
    event: Option<Uuid>,
) -> Result<Value, AppError> {
    let query = if event.is_some() {
        "SELECT to_jsonb(e)||jsonb_build_object('event_id',e.id,'revision',0,'project_name',p.name,'client_id',p.client_id) AS data FROM project_conversation_timeline e JOIN projects p ON p.id=e.project_id WHERE e.project_id=$1 AND p.user_id=$2 AND e.id=$3"
    } else {
        "SELECT to_jsonb(e)||jsonb_build_object('event_id',e.id,'revision',0,'project_name',p.name,'client_id',p.client_id) AS data FROM project_conversation_timeline e JOIN projects p ON p.id=e.project_id WHERE e.project_id=$1 AND p.user_id=$2 AND ($3::uuid IS NULL) ORDER BY e.recorded_at DESC,e.id DESC LIMIT 1"
    };
    json_query(db, query, vec![project.into(), owner.into(), event.into()]).await
}
pub async fn publish_latest(s: &crate::state::AppState, owner: Uuid, project: Uuid, event: Option<Uuid>) {
    match latest_event(s.db.as_ref(), owner, project, event).await {
        Ok(mut data) => {
            data["kind"] = json!(if data["kind"] == "message" {
                "comment"
            } else {
                data["kind"].as_str().unwrap_or("event")
            });
            let client = data["client_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
            let _ = s.conversation_bus.send(ConversationEvent {
                user_id: owner,
                client_id: client,
                project_id: project,
                data,
            });
        }
        Err(e) => tracing::warn!("Conversation refresh event failed: {e}"),
    }
}
