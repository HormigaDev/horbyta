use object_store::{ObjectStore, ObjectStoreExt, path::Path as ObjectPath};
use sea_orm::{ConnectionTrait, DbBackend, Statement, Value};
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    dtos::scope::{ClassifyTaskDto, PublishScopeDto, ScopeCommentDto, ScopeFeedbackDto},
    error::AppError,
};

pub struct ScopeService;

impl ScopeService {
    pub async fn actor(db: &impl ConnectionTrait, id: Uuid, kind: &str) -> Result<(), AppError> {
        db.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT set_config('horbyta.actor',$1,true),set_config('horbyta.actor_kind',$2,true)",
            [id.to_string().into(), kind.into()],
        ))
        .await?;
        Ok(())
    }
    async fn json(db: &impl ConnectionTrait, sql: &str, values: Vec<Value>) -> Result<JsonValue, AppError> {
        let row = db
            .query_one(Statement::from_sql_and_values(DbBackend::Postgres, sql, values))
            .await?
            .ok_or_else(|| AppError::NotFound("Registro de alcance no encontrado.".into()))?;
        Ok(row.try_get("", "data")?)
    }
    pub async fn project(
        db: &impl ConnectionTrait,
        owner: Uuid,
        client: Option<Uuid>,
        id: Uuid,
    ) -> Result<JsonValue, AppError> {
        Self::json(db, r#"SELECT jsonb_build_object('project_id',p.id,'client_id',p.client_id,'project_name',p.name,'last_revision',coalesce(s.last_revision,0),
          'accepted_revision',s.accepted_revision,'published_revision',(SELECT max(revision) FROM scope_revisions WHERE project_id=p.id AND published),'active',s.accepted_revision is not null,'enforce_scope',p.enforce_scope,
          'revisions',coalesce((SELECT jsonb_agg(to_jsonb(r)||jsonb_build_object('accepted_name',(SELECT email FROM client_auth WHERE id=r.accepted_by),'status',CASE WHEN r.revision_kind='consolidation' THEN 'consolidated' WHEN r.accepted_at IS NOT NULL THEN 'accepted' WHEN NOT r.published THEN 'draft' WHEN EXISTS(SELECT 1 FROM scope_project_events e WHERE e.project_id=r.project_id AND e.revision=r.revision AND e.kind='reject') THEN 'rejected' WHEN EXISTS(SELECT 1 FROM scope_project_events e WHERE e.project_id=r.project_id AND e.revision=r.revision AND e.kind='request_changes') THEN 'changes_requested' WHEN r.revision<>(SELECT max(revision) FROM scope_revisions WHERE project_id=r.project_id AND published) THEN 'superseded' ELSE 'pending' END) ORDER BY r.revision DESC) FROM scope_revisions r
              WHERE r.project_id=p.id AND ($3::uuid IS NULL OR r.published)),'[]'),
          'current_billing',(SELECT to_jsonb(pb)||jsonb_build_object('fixed_price',pb.fixed_price::text,'hourly_rate',pb.hourly_rate::text,'retainer_amount',pb.retainer_amount::text) FROM project_billing pb WHERE pb.project_id=p.id),
          'current_work',coalesce((SELECT jsonb_agg(jsonb_build_object('id',t.id,'title',t.title)) FROM tasks t WHERE t.project_id=p.id AND t.scope_preparation_reason IS NULL AND NOT EXISTS(SELECT 1 FROM scope_task_links l WHERE l.task_id=t.id AND l.change_request_id IS NOT NULL)),'[]'),
          'events',coalesce((SELECT jsonb_agg(to_jsonb(e)||jsonb_build_object('actor_name',CASE WHEN e.actor_kind='professional' THEN (SELECT u.name FROM users u WHERE u.id=e.actor_id) ELSE (SELECT c.name FROM client_auth a JOIN clients c ON c.id=a.client_id WHERE a.id=e.actor_id AND a.user_id=p.user_id) END) ORDER BY e.recorded_at,e.id) FROM (SELECT * FROM scope_project_events WHERE project_id=p.id ORDER BY recorded_at DESC,id DESC LIMIT 100) e),'[]'),
          'available_attachments',CASE WHEN $3::uuid IS NULL THEN coalesce((SELECT jsonb_agg(jsonb_build_object('id',a.id,'original_name',a.original_name,'size_bytes',a.size_bytes)) FROM attachments a WHERE a.user_id=p.user_id AND (a.project_id=p.id OR a.task_id IN(SELECT id FROM tasks WHERE project_id=p.id) OR a.milestone_id IN(SELECT id FROM milestones WHERE project_id=p.id))),'[]') ELSE '[]'::jsonb END,
          'work',coalesce((SELECT jsonb_agg(to_jsonb(l)) FROM scope_task_links l WHERE l.project_id=p.id),'[]'),
          'changes',coalesce((SELECT jsonb_agg(jsonb_build_object('id',c.id,'title',c.title,'status',c.status,'version',c.version,'legacy',c.legacy))
              FROM change_requests c WHERE c.project_id=p.id AND ($3::uuid IS NULL OR (c.client_id=$3 AND c.status<>'draft'))),'[]'),
          'milestones',coalesce((SELECT jsonb_agg(jsonb_build_object('id',m.id,'name',m.name,'due_date',m.due_date,'status',m.status))
              FROM milestones m WHERE m.project_id=p.id),'[]')) AS data
          FROM projects p LEFT JOIN scope_projects s ON s.project_id=p.id
          WHERE p.id=$1 AND p.user_id=$2 AND ($3::uuid IS NULL OR p.client_id=$3)"#,
          vec![id.into(),owner.into(),client.into()]).await
    }
    pub async fn publish(
        db: &impl ConnectionTrait,
        owner: Uuid,
        id: Uuid,
        dto: PublishScopeDto,
    ) -> Result<JsonValue, AppError> {
        Self::publish_with_storage(db, owner, id, dto, None).await
    }
    pub async fn publish_with_storage(
        db: &impl ConnectionTrait,
        owner: Uuid,
        id: Uuid,
        dto: PublishScopeDto,
        storage: Option<&dyn ObjectStore>,
    ) -> Result<JsonValue, AppError> {
        let sections = [
            &dto.document.deliverables,
            &dto.document.criteria,
            &dto.document.restrictions,
            &dto.document.exclusions,
            &dto.document.requirements,
        ];
        if dto.document.documents.len() > 50
            || dto.document.attachment_ids.len() + dto.document.reference_attachment_ids.len() > 100
            || dto.document.precedence.len() > 4000
        {
            return Err(AppError::BadRequest(
                "El alcance excede los límites de documentos/anexos.".into(),
            ));
        }
        let main = dto.document.documents.iter().filter(|d| d.role == "main").count();
        if dto.publish && (main > 1 || (main == 0 && sections.iter().any(|v| v.trim().is_empty()))) {
            return Err(AppError::BadRequest(
                "Complete las cinco secciones o incluya un documento principal completo.".into(),
            ));
        }
        if dto.publish && !dto.document.documents.is_empty() && dto.document.precedence.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Indique cómo resolver contradicciones entre secciones y documentos.".into(),
            ));
        }
        let included_ids: Vec<Uuid> = dto
            .document
            .attachment_ids
            .iter()
            .chain(&dto.document.reference_attachment_ids)
            .copied()
            .collect();
        let mut ids = std::collections::HashSet::new();
        let mut requirements = std::collections::HashSet::new();
        let mut total = 0;
        for doc in &dto.document.documents {
            total += doc.content.len() + doc.source_content.as_ref().map_or(0, String::len);
            if !ids.insert(doc.id)
                || !["main", "annex", "reference"].contains(&doc.role.as_str())
                || doc.title.trim().is_empty()
                || doc.title.len() > 255
                || doc.content.trim().is_empty()
                || doc.content.len() > 200_000
                || doc.source_content.as_ref().is_some_and(|s| s.len() > 200_000)
                || doc.source_name.as_ref().is_some_and(|s| s.len() > 255)
                || doc.section.as_ref().is_some_and(|s| {
                    !["deliverables", "criteria", "restrictions", "exclusions", "requirements"].contains(&s.as_str())
                })
                || doc
                    .requirement_id
                    .as_ref()
                    .is_some_and(|s| s.trim().is_empty() || s.len() > 64 || !requirements.insert(s.clone()))
            {
                return Err(AppError::BadRequest(
                    "Revise título, contenido, tipo e identificadores únicos de los documentos.".into(),
                ));
            }
            if dto.publish {
                validate_scope_links(&doc.content, &included_ids)?;
            }
        }
        if total > 1_000_000 {
            return Err(AppError::BadRequest("El conjunto de documentos supera 1 MB.".into()));
        }
        if dto.publish {
            for section in sections {
                validate_scope_links(section, &included_ids)?;
            }
        }
        // El proyecto se bloquea antes que el estado para serializar creación/traslado de tareas.
        let p = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT client_id FROM projects WHERE id=$1 AND user_id=$2 FOR UPDATE",
                [id.into(), owner.into()],
            ))
            .await?
            .ok_or_else(|| AppError::NotFound("Proyecto no encontrado.".into()))?;
        if dto.publish {
            let running=Self::json(db,"SELECT to_jsonb(EXISTS(SELECT 1 FROM time_entries e JOIN tasks t ON t.id=e.task_id WHERE t.project_id=$1 AND t.scope_preparation_reason IS NULL AND e.start_time IS NOT NULL AND e.end_time IS NULL)) AS data",vec![id.into()]).await?;
            if running == true {
                return Err(AppError::Conflict(
                    "Finalice los timers de ejecución antes de publicar el alcance inicial.".into(),
                ));
            }
        }
        let client = p
            .try_get::<Option<Uuid>>("", "client_id")?
            .ok_or_else(|| AppError::Conflict("Asigne un cliente antes de publicar el alcance.".into()))?;
        db.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO scope_projects(project_id,user_id,client_id) VALUES($1,$2,$3) ON CONFLICT DO NOTHING",
            [id.into(), owner.into(), client.into()],
        ))
        .await?;
        let state = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT last_revision,accepted_revision FROM scope_projects WHERE project_id=$1 FOR UPDATE",
                [id.into()],
            ))
            .await?
            .ok_or(AppError::InternalError)?;
        let accepted_revision = state.try_get::<Option<i32>>("", "accepted_revision")?;
        if state.try_get::<i32>("", "last_revision")? != dto.expected_revision {
            return Err(AppError::Conflict("El alcance cambió; vuelva a leerlo.".into()));
        }
        if accepted_revision.is_some()
            && (dto.document.agreement_revision != accepted_revision || dto.document.amendment_reason.trim().is_empty())
        {
            return Err(AppError::Conflict(
                "Indique el motivo de la corrección y la revisión vigente del acuerdo.".into(),
            ));
        }
        if dto.document.amendment_reason.chars().count() > 2000 {
            return Err(AppError::BadRequest("El motivo supera 2000 caracteres.".into()));
        }
        let approved_changes = Self::json(db,
            "SELECT coalesce((SELECT approved_changes FROM scope_revisions WHERE project_id=$1 AND revision=$2),'[]') AS data",
            vec![id.into(),accepted_revision.into()]).await?;
        if let Some(based_on) = dto.document.based_on {
            Self::json(
                db,
                "SELECT document AS data FROM scope_revisions WHERE project_id=$1 AND revision=$2",
                vec![id.into(), based_on.into()],
            )
            .await?;
        }
        let previous = Self::json(db,"SELECT coalesce((SELECT document FROM scope_revisions WHERE project_id=$1 ORDER BY revision DESC LIMIT 1),'{}') AS data",vec![id.into()]).await?;
        let mut document = serde_json::to_value(dto.document).map_err(|_| AppError::InternalError)?;
        if let Some(docs) = document["documents"].as_array_mut() {
            for doc in docs {
                let checksum = hex::encode(Sha256::digest(
                    serde_json::to_vec(doc).map_err(|_| AppError::InternalError)?,
                ));
                let prior = previous["documents"]
                    .as_array()
                    .and_then(|a| a.iter().find(|p| p["id"] == doc["id"]));
                let version = prior.map_or(1, |p| {
                    p["version"].as_i64().unwrap_or(1) + i64::from(p["hash"].as_str() != Some(&checksum))
                });
                doc["version"] = version.into();
                doc["hash"] = checksum.into();
                if let Some(source) = doc["source_content"].as_str() {
                    doc["source_hash"] = hex::encode(Sha256::digest(source.as_bytes())).into();
                }
            }
        }
        let mut attachments = Vec::new();
        let selected = document["attachment_ids"].as_array().ok_or(AppError::InternalError)?;
        let mut selected_ids = std::collections::HashSet::new();
        let referenced = document["reference_attachment_ids"]
            .as_array()
            .ok_or(AppError::InternalError)?;
        for item in selected.iter().chain(referenced) {
            let attachment =
                Uuid::parse_str(item.as_str().ok_or(AppError::InternalError)?).map_err(|_| AppError::InternalError)?;
            if !selected_ids.insert(attachment) {
                return Err(AppError::BadRequest("Hay anexos duplicados.".into()));
            }
            let mut metadata=Self::json(db,"SELECT to_jsonb(a) AS data FROM attachments a WHERE a.id=$1 AND a.user_id=$2 AND (a.project_id=$3 OR a.task_id IN(SELECT id FROM tasks WHERE project_id=$3) OR a.milestone_id IN(SELECT id FROM milestones WHERE project_id=$3)) FOR UPDATE",vec![attachment.into(),owner.into(),id.into()]).await?;
            metadata["scope_role"] = if referenced.contains(item) {
                "reference"
            } else {
                "annex"
            }
            .into();
            if dto.publish {
                let storage = storage.ok_or_else(|| {
                    AppError::BadRequest("La publicación con anexos necesita verificar el almacenamiento.".into())
                })?;
                let key = metadata["storage_key"].as_str().ok_or(AppError::InternalError)?;
                let data = storage
                    .get(&ObjectPath::from(key))
                    .await
                    .map_err(|_| {
                        AppError::Conflict("No se puede leer un anexo; revise los archivos antes de publicar.".into())
                    })?
                    .bytes()
                    .await
                    .map_err(|_| AppError::InternalError)?;
                metadata["sha256"] = hex::encode(Sha256::digest(&data)).into();
            }
            attachments.push(metadata);
        }
        let row=Self::json(db,r#"SELECT jsonb_build_object('tasks',coalesce((SELECT jsonb_agg(to_jsonb(t) ORDER BY t.id) FROM tasks t WHERE t.project_id=$1 AND t.scope_preparation_reason IS NULL AND NOT EXISTS(SELECT 1 FROM scope_task_links l WHERE l.task_id=t.id AND l.change_request_id IS NOT NULL)),'[]'),
            'billing',(SELECT to_jsonb(pb)||jsonb_build_object('fixed_price',pb.fixed_price::text,'hourly_rate',pb.hourly_rate::text,'retainer_amount',pb.retainer_amount::text) FROM project_billing pb WHERE pb.project_id=$1),
            'milestones',coalesce((SELECT jsonb_agg(to_jsonb(m) ORDER BY m.id) FROM milestones m WHERE m.project_id=$1),'[]')) AS data"#,vec![id.into()]).await?;
        let bundle = serde_json::json!({"document":document,"attachments":attachments,"tasks":row["tasks"],"billing":row["billing"],"milestones":row["milestones"],"approved_changes":approved_changes,"previous_accepted_revision":accepted_revision});
        let hash = hex::encode(Sha256::digest(
            serde_json::to_vec(&bundle).map_err(|_| AppError::InternalError)?,
        ));
        db.execute(Statement::from_sql_and_values(DbBackend::Postgres,
            "INSERT INTO scope_revisions(project_id,revision,document,task_snapshot,attachment_snapshot,billing_snapshot,milestone_snapshot,published,created_by,bundle_hash,previous_accepted_revision,approved_changes) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)",
            vec![id.into(),(dto.expected_revision+1).into(),document.into(),row["tasks"].clone().into(),serde_json::json!(attachments).into(),row["billing"].clone().into(),row["milestones"].clone().into(),dto.publish.into(),owner.into(),hash.into(),accepted_revision.into(),approved_changes.into()])).await?;
        Self::project(db, owner, None, id).await
    }
    pub async fn verify_bundle_assets(
        db: &impl ConnectionTrait,
        storage: &dyn ObjectStore,
        owner: Uuid,
        client: Uuid,
        id: Uuid,
        revision: i32,
    ) -> Result<(), AppError> {
        let assets=Self::json(db,"SELECT r.attachment_snapshot AS data FROM scope_revisions r JOIN scope_projects s ON s.project_id=r.project_id WHERE r.project_id=$1 AND r.revision=$2 AND s.user_id=$3 AND s.client_id=$4 AND r.published",vec![id.into(),revision.into(),owner.into(),client.into()]).await?;
        for asset in assets.as_array().ok_or(AppError::InternalError)? {
            if let Some(expected) = asset["sha256"].as_str() {
                let key = asset["storage_key"].as_str().ok_or(AppError::InternalError)?;
                let bytes = storage
                    .get(&ObjectPath::from(key))
                    .await
                    .map_err(|_| {
                        AppError::Conflict("Falta un anexo publicado; recupere el archivo antes de aceptar.".into())
                    })?
                    .bytes()
                    .await
                    .map_err(|_| AppError::InternalError)?;
                if hex::encode(Sha256::digest(&bytes)) != expected {
                    return Err(AppError::Conflict(
                        "Un anexo no coincide con el publicado; revise su integridad antes de aceptar.".into(),
                    ));
                }
            }
        }
        Ok(())
    }
    pub async fn accept(
        db: &impl ConnectionTrait,
        owner: Uuid,
        client: Uuid,
        actor: Uuid,
        id: Uuid,
        revision: i32,
    ) -> Result<JsonValue, AppError> {
        db.query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT id FROM projects WHERE id=$1 AND user_id=$2 AND client_id=$3 FOR UPDATE",
            [id.into(), owner.into(), client.into()],
        ))
        .await?
        .ok_or_else(|| AppError::NotFound("Proyecto no encontrado.".into()))?;
        let state=db.query_one(Statement::from_sql_and_values(DbBackend::Postgres,
            "SELECT accepted_revision FROM scope_projects WHERE project_id=$1 AND user_id=$2 AND client_id=$3 FOR UPDATE",
            [id.into(),owner.into(),client.into()])).await?.ok_or_else(||AppError::NotFound("Alcance no encontrado.".into()))?;
        if state.try_get::<Option<i32>>("", "accepted_revision")? == Some(revision) {
            return Self::project(db, owner, Some(client), id).await;
        }
        let result=db.execute(Statement::from_sql_and_values(DbBackend::Postgres,
            "UPDATE scope_revisions SET accepted_at=now(),accepted_by=$3,accepted_ip=nullif(current_setting('horbyta.accepted_ip',true),'')::inet,accepted_ip_source=coalesce(nullif(current_setting('horbyta.ip_source',true),''),'unavailable'),acceptance_statement='Acepto el alcance y los documentos incluidos de esta revisión, sus condiciones económicas y fechas. Los materiales de referencia no amplían los entregables.' WHERE project_id=$1 AND revision=$2 AND published AND accepted_at IS NULL",
            [id.into(),revision.into(),actor.into()])).await?;
        if result.rows_affected() != 1 {
            return Err(AppError::Conflict(
                "Esta revisión no está disponible para aceptar.".into(),
            ));
        }
        Self::project(db, owner, Some(client), id).await
    }
    pub async fn change(
        db: &impl ConnectionTrait,
        owner: Uuid,
        client: Option<Uuid>,
        id: Uuid,
    ) -> Result<JsonValue, AppError> {
        Self::json(db,r#"SELECT jsonb_build_object('current',to_jsonb(c)||jsonb_build_object('project_name',(SELECT name FROM projects WHERE id=c.project_id),'source_message_id',(SELECT message_id FROM conversation_change_sources WHERE change_request_id=c.id),'impact_cost',c.impact_cost::text,'impact_hours',c.impact_hours::text),
          'revisions',coalesce((SELECT jsonb_agg(to_jsonb(r) ORDER BY version DESC) FROM scope_change_revisions r WHERE r.change_request_id=c.id AND ($3::uuid IS NULL OR r.snapshot->>'status'<>'draft')),'[]'),
          'events',coalesce((SELECT jsonb_agg(to_jsonb(e)||jsonb_build_object('actor_name',CASE WHEN e.actor_kind='professional' THEN (SELECT name FROM users WHERE id=e.actor_id) ELSE (SELECT cl.name FROM client_auth a JOIN clients cl ON cl.id=a.client_id WHERE a.id=e.actor_id AND a.user_id=c.user_id) END) ORDER BY recorded_at,id) FROM (SELECT * FROM scope_events WHERE change_request_id=c.id AND kind<>'comment' AND ($3::uuid IS NULL OR kind<>'draft') ORDER BY recorded_at DESC,id DESC LIMIT 100) e),'[]'),
          'effect',(SELECT to_jsonb(e)||jsonb_build_object('amount',e.amount::text,'invoice_status',i.status,'invoice_number',i.record_number,
            'paid_amount',coalesce((SELECT sum(case when transaction_type='refund' then -amount else amount end)
              FROM payments WHERE invoice_id=e.invoice_id),0)::text)
            FROM scope_effects e LEFT JOIN invoices i ON i.id=e.invoice_id WHERE e.change_request_id=c.id),
          'tasks',coalesce((SELECT jsonb_agg(jsonb_build_object('id',t.id,'title',t.title,'status',t.status,'allowed',scope_task_allowed(t.id)))
            FROM scope_task_links l JOIN tasks t ON t.id=l.task_id WHERE l.change_request_id=c.id),'[]')) AS data
          FROM change_requests c WHERE c.id=$1 AND c.user_id=$2 AND ($3::uuid IS NULL OR (c.client_id=$3 AND c.status<>'draft'))"#,
          vec![id.into(),owner.into(),client.into()]).await
    }
    pub async fn comment(
        db: &impl ConnectionTrait,
        owner: Uuid,
        client: Option<Uuid>,
        actor: Uuid,
        id: Uuid,
        dto: ScopeCommentDto,
    ) -> Result<JsonValue, AppError> {
        crate::services::scope_conversation::authorize_change(db, owner, client, id, dto.version).await?;
        let locked = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT 1 FROM change_requests WHERE id=$1 AND user_id=$2 AND version=$3 FOR UPDATE",
                [id.into(), owner.into(), dto.version.into()],
            ))
            .await?;
        if locked.is_none() {
            return Err(AppError::Conflict(
                "La revisión cambió o no es accesible; vuelva a leerla.".into(),
            ));
        }
        if (dto.text.trim().is_empty() && dto.attachment_ids.is_empty()) || dto.attachment_ids.len() > 10 {
            return Err(AppError::BadRequest("Escriba la aclaración.".into()));
        }
        let text = if dto.text.trim().is_empty() {
            let row=db.query_one(Statement::from_sql_and_values(DbBackend::Postgres,
                "SELECT string_agg(a.original_name, ', ' ORDER BY a.id) AS text FROM attachments a JOIN change_requests c ON c.project_id=a.project_id AND c.user_id=a.user_id WHERE c.id=$1 AND a.id IN(SELECT value::uuid FROM jsonb_array_elements_text($2::jsonb))",
                [id.into(),serde_json::json!(dto.attachment_ids).into()])).await?.ok_or(AppError::InternalError)?;
            row.try_get::<Option<String>>("", "text")?
                .ok_or_else(|| AppError::BadRequest("Escriba la aclaración.".into()))?
        } else {
            dto.text.trim().to_owned()
        };
        let row=db.query_one(Statement::from_sql_and_values(DbBackend::Postgres,
            "INSERT INTO scope_events(change_request_id,version,kind,actor_id,actor_kind,body) SELECT id,version,'comment',$4,$5,jsonb_build_object('text',$6::text) FROM change_requests WHERE id=$1 AND user_id=$2 AND ($3::uuid IS NULL OR client_id=$3) AND version=$7 RETURNING id",
            vec![id.into(),owner.into(),client.into(),actor.into(),if client.is_some(){"client"}else{"professional"}.into(),text.chars().take(4000).collect::<String>().into(),dto.version.into()])).await?;
        let Some(row) = row else {
            return Err(AppError::Conflict(
                "La revisión cambió o no es accesible; vuelva a leerla.".into(),
            ));
        };
        let event: Uuid = row.try_get("", "id")?;
        for file in dto.attachment_ids {
            db.execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "INSERT INTO scope_change_event_attachments(event_id,attachment_id,metadata) VALUES($1,$2,'{}')",
                [event.into(), file.into()],
            ))
            .await?;
        }
        let mut result = Self::change(db, owner, client, id).await?;
        let row=db.query_one(Statement::from_sql_and_values(DbBackend::Postgres,"SELECT to_jsonb(e)||jsonb_build_object('text',e.body->>'text','revision',e.version,'actor_name',CASE WHEN e.actor_kind='professional' THEN (SELECT name FROM users WHERE id=e.actor_id) ELSE (SELECT c.name FROM client_auth a JOIN clients c ON c.id=a.client_id WHERE a.id=e.actor_id) END) AS data FROM scope_events e WHERE e.id=$1",[event.into()])).await?.ok_or(AppError::InternalError)?;
        result["_feedback_event"] = row.try_get("", "data")?;
        Ok(result)
    }
    pub async fn task(db: &impl ConnectionTrait, owner: Uuid, id: Uuid) -> Result<JsonValue, AppError> {
        Self::json(db,r#"SELECT jsonb_build_object('task_id',t.id,'project_id',t.project_id,
          'controlled',coalesce(p.enforce_scope,false) or s.accepted_revision is not null,'active',s.accepted_revision is not null,'preparation_reason',t.scope_preparation_reason,'requirements',coalesce((SELECT jsonb_agg(to_jsonb(q)) FROM scope_task_requirements q WHERE q.task_id=t.id),'[]'),'allowed',scope_task_allowed(t.id),'link',to_jsonb(l)) AS data
          FROM tasks t LEFT JOIN projects p ON p.id=t.project_id LEFT JOIN scope_projects s ON s.project_id=t.project_id LEFT JOIN scope_task_links l ON l.task_id=t.id
          WHERE t.id=$1 AND t.user_id=$2"#,vec![id.into(),owner.into()]).await
    }
    pub async fn classify(
        db: &impl ConnectionTrait,
        owner: Uuid,
        actor: Uuid,
        id: Uuid,
        dto: ClassifyTaskDto,
    ) -> Result<JsonValue, AppError> {
        if dto.reason.trim().is_empty() {
            return Err(AppError::BadRequest("Explique la relación con el alcance.".into()));
        }
        let r = db
            .execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"
          INSERT INTO scope_task_links(task_id,project_id,baseline_revision,change_request_id,reason,created_by)
          SELECT t.id,t.project_id,CASE WHEN $3::uuid IS NULL THEN s.accepted_revision ELSE NULL END,$3,$4,$5
          FROM tasks t JOIN scope_projects s ON s.project_id=t.project_id
          WHERE t.id=$1 AND t.user_id=$2 AND s.accepted_revision IS NOT NULL"#,
                vec![
                    id.into(),
                    owner.into(),
                    dto.change_request_id.into(),
                    dto.reason.trim().into(),
                    actor.into(),
                ],
            ))
            .await?;
        if r.rows_affected() != 1 {
            return Err(AppError::Conflict(
                "La tarea necesita un alcance inicial aceptado.".into(),
            ));
        }
        Self::task(db, owner, id).await
    }
    pub async fn feedback(
        db: &impl ConnectionTrait,
        owner: Uuid,
        client: Option<Uuid>,
        actor: Uuid,
        id: Uuid,
        dto: ScopeFeedbackDto,
    ) -> Result<JsonValue, AppError> {
        crate::services::scope_conversation::authorize(db, owner, client, id, dto.revision).await?;
        if (dto.text.trim().is_empty() && (dto.attachment_ids.is_empty() || dto.kind != "comment"))
            || dto.attachment_ids.len() > 10
            || !["comment", "request_changes", "reject", "prefer"].contains(&dto.kind.as_str())
            || (client.is_none() && dto.kind != "comment")
        {
            return Err(AppError::BadRequest(
                "Indique una acción válida y su explicación.".into(),
            ));
        }
        // El texto de mensajes solo con archivos usa sus nombres, nunca un aviso ficticio.
        let text = if dto.text.trim().is_empty() {
            let row = db.query_one(Statement::from_sql_and_values(DbBackend::Postgres,
                "SELECT string_agg(original_name, ', ' ORDER BY id) AS names FROM attachments WHERE id IN(SELECT value::uuid FROM jsonb_array_elements_text($1::jsonb)) AND user_id=$2 AND project_id=$3",
                [serde_json::json!(dto.attachment_ids).into(), owner.into(), id.into()])).await?.ok_or(AppError::InternalError)?;
            row.try_get::<Option<String>>("", "names")?
                .ok_or_else(|| AppError::BadRequest("Indique una acción válida y su explicación.".into()))?
        } else {
            dto.text.trim().to_owned()
        };
        let text: String = text.chars().take(4000).collect();
        let row = db.query_one(Statement::from_sql_and_values(DbBackend::Postgres,
            "INSERT INTO scope_project_events(project_id,revision,kind,text,document_id,actor_id,actor_kind) VALUES($1,$2,$3,$4,$5,$6,$7) RETURNING id",
            vec![id.into(),dto.revision.into(),dto.kind.into(),text.into(),dto.document_id.into(),actor.into(),if client.is_some(){"client"}else{"professional"}.into()])).await?.ok_or(AppError::InternalError)?;
        let event_id: Uuid = row.try_get("", "id")?;
        for attachment in dto.attachment_ids {
            db.execute(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "INSERT INTO scope_project_event_attachments(event_id,attachment_id,metadata) VALUES($1,$2,'{}')",
                [event_id.into(), attachment.into()],
            ))
            .await?;
        }
        let mut result = Self::project(db, owner, client, id).await?;
        let row = db.query_one(Statement::from_sql_and_values(DbBackend::Postgres,
            "SELECT to_jsonb(e)||jsonb_build_object('actor_name',CASE WHEN e.actor_kind='professional' THEN (SELECT name FROM users WHERE id=e.actor_id) ELSE (SELECT c.name FROM client_auth a JOIN clients c ON c.id=a.client_id WHERE a.id=e.actor_id AND a.user_id=$2) END) AS data FROM scope_project_events e WHERE e.id=$1",
            [event_id.into(), owner.into()])).await?.ok_or(AppError::InternalError)?;
        result["_feedback_event"] = row.try_get("", "data")?;
        Ok(result)
    }
    pub async fn preparation(
        db: &impl ConnectionTrait,
        owner: Uuid,
        id: Uuid,
        reason: &str,
    ) -> Result<JsonValue, AppError> {
        if reason.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Justifique la preparación sin ejecución de entregables.".into(),
            ));
        }
        let r=db.execute(Statement::from_sql_and_values(DbBackend::Postgres,
          "UPDATE tasks SET scope_preparation_reason=$3 WHERE id=$1 AND user_id=$2 AND scope_preparation_reason IS NULL",
          vec![id.into(),owner.into(),reason.trim().into()])).await?;
        if r.rows_affected() != 1 {
            return Err(AppError::Conflict(
                "La tarea ya fue clasificada o no está disponible.".into(),
            ));
        }
        Self::task(db, owner, id).await
    }
    pub async fn requirement(
        db: &impl ConnectionTrait,
        owner: Uuid,
        id: Uuid,
        document_id: Uuid,
        revision: i32,
    ) -> Result<JsonValue, AppError> {
        let r=db.execute(Statement::from_sql_and_values(DbBackend::Postgres,
            "INSERT INTO scope_task_requirements(task_id,project_id,revision,document_id) SELECT id,project_id,$3,$4 FROM tasks WHERE id=$1 AND user_id=$2 ON CONFLICT DO NOTHING",
            vec![id.into(),owner.into(),revision.into(),document_id.into()])).await?;
        if r.rows_affected() == 0 {
            Self::task(db, owner, id).await?;
        }
        Self::task(db, owner, id).await
    }
    /// Avisos del cliente: primero los pendientes y después el historial resuelto reciente.
    pub async fn notices(db: &impl ConnectionTrait, owner: Uuid, client: Uuid) -> Result<JsonValue, AppError> {
        Self::json(db,r#"SELECT coalesce(jsonb_agg(to_jsonb(x) ORDER BY x.resolved, x.created_at DESC),'[]') AS data FROM (
            SELECT n.id,n.kind,n.record_id,n.revision,n.title,n.resolved,n.created_at,
              coalesce(p.id,c.project_id) AS project_id,coalesce(p.name,pc.name) AS project_name
            FROM scope_notices n
            LEFT JOIN projects p ON n.kind='baseline' AND p.id=n.record_id
            LEFT JOIN change_requests c ON n.kind='change' AND c.id=n.record_id
            LEFT JOIN projects pc ON pc.id=c.project_id
            WHERE n.user_id=$1 AND n.client_id=$2
            ORDER BY n.resolved, n.created_at DESC LIMIT 50) x"#,
          vec![owner.into(),client.into()]).await
    }
    pub async fn pending(db: &impl ConnectionTrait, owner: Uuid, client: Uuid) -> Result<JsonValue, AppError> {
        Self::json(db,"SELECT coalesce(jsonb_agg(to_jsonb(n) ORDER BY created_at DESC),'[]') AS data FROM scope_notices n WHERE user_id=$1 AND client_id=$2 AND NOT resolved",
          vec![owner.into(),client.into()]).await
    }
}

// El parser distingue código, enlaces, referencias e imágenes sin reinterpretar el documento.
fn validate_scope_links(content: &str, attachments: &[Uuid]) -> Result<(), AppError> {
    use pulldown_cmark::{Event, Parser, Tag};
    for event in Parser::new(content) {
        let (destination, image) = match event {
            Event::Start(Tag::Link { dest_url, .. }) => (dest_url, false),
            Event::Start(Tag::Image { dest_url, .. }) => (dest_url, true),
            Event::Html(html) | Event::InlineHtml(html) if html.to_lowercase().contains("<img") => {
                return Err(AppError::BadRequest(
                    "Use imágenes Markdown vinculadas a anexos incluidos, para conservar su contenido.".into(),
                ));
            }
            _ => continue,
        };
        let destination = destination.as_ref();
        if !image
            && (destination.starts_with('#')
                || destination.starts_with("https://")
                || destination.starts_with("http://")
                || destination.starts_with("mailto:"))
        {
            continue;
        }
        let included = destination
            .strip_prefix("/api/v1/attachments/")
            .and_then(|s| s.strip_suffix("/raw"))
            .and_then(|s| Uuid::parse_str(s).ok())
            .is_some_and(|id| attachments.contains(&id));
        if !included {
            return Err(AppError::BadRequest("Hay recursos pendientes: suba imágenes/archivos, vincule sus anexos y sustituya los enlaces locales antes de publicar. Las imágenes remotas también deben incorporarse.".into()));
        }
    }
    Ok(())
}
#[cfg(test)]
mod document_tests {
    use super::*;
    #[test]
    fn analiza_referencias_y_no_confunde_ejemplos_de_codigo() {
        assert!(validate_scope_links("![Figura][img]\n\n[img]: imagen.png", &[]).is_err());
        assert!(validate_scope_links("![Figura](https://example.com/cambiante.png)", &[]).is_err());
        assert!(validate_scope_links("```md\n![Ejemplo](ruta.png)\n```", &[]).is_ok());
        assert!(validate_scope_links("[Referencia](https://example.com)", &[]).is_ok());
        let id = Uuid::now_v7();
        assert!(validate_scope_links(&format!("![Figura](/api/v1/attachments/{id}/raw)"), &[id]).is_ok());
    }
}
