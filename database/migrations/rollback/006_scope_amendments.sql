-- Revierte las correcciones y consolidaciones del acuerdo de alcance.
-- El esquema anterior no puede representar consolidaciones ni correcciones de un
-- acuerdo vigente; si existen, se rechaza el rollback en lugar de borrar evidencia.
do $$ begin
    if exists(select 1 from scope_revisions where revision_kind='consolidation' or previous_accepted_revision is not null) then
        raise exception 'La migración 6 ya registró consolidaciones o correcciones de acuerdos; revertirla destruiría evidencia contractual. Restaure un respaldo previo.';
    end if;
end; $$;

drop trigger zz_scope_change_consolidation on change_requests;
drop trigger a_scope_change_project_lock on change_requests;
drop function scope_change_consolidation(), scope_change_project_lock(),
    scope_consolidate_change(uuid), scope_contract_root(uuid, integer);

-- Vista de la migración 5.
create or replace view project_conversation_timeline as
select m.id,m.project_id,'message'::text kind,m.actor_id,m.actor_kind,m.recorded_at,m.text,m.context,m.mentions,
 coalesce((select jsonb_agg(a.metadata||jsonb_build_object('id',a.attachment_id) order by a.attachment_id) from project_message_attachments a where a.event_id=m.id),'[]') attachments
from project_messages m
union all
select e.id,e.project_id,case e.kind when 'comment' then 'message' when 'reject' then 'scope_rejected' when 'request_changes' then 'scope_changes_requested' else 'scope_preferred' end,
 e.actor_id,e.actor_kind,e.recorded_at,e.text,jsonb_build_object('scope_revision',e.revision,'document_id',e.document_id),'[]'::jsonb,
 coalesce((select jsonb_agg(a.metadata||jsonb_build_object('id',a.attachment_id) order by a.attachment_id) from scope_project_event_attachments a where a.event_id=e.id),'[]')
from scope_project_events e
union all
select md5(r.project_id::text||':'||r.revision||':published')::uuid,r.project_id,'scope_published',r.created_by,'professional',r.created_at,'',jsonb_build_object('scope_revision',r.revision),'[]'::jsonb,'[]'::jsonb
from scope_revisions r where r.published
union all
select md5(r.project_id::text||':'||r.revision||':accepted')::uuid,r.project_id,'scope_accepted',r.accepted_by,'client',r.accepted_at,'',jsonb_build_object('scope_revision',r.revision),'[]'::jsonb,'[]'::jsonb
from scope_revisions r where r.accepted_at is not null
union all
select e.id,c.project_id,case e.kind when 'comment' then 'message' else 'change_'||e.kind end,
 e.actor_id,e.actor_kind,e.recorded_at,coalesce(e.body->>'text',''),
 jsonb_build_object('change_request_id',c.id,'change_version',e.version,'change_title',r.snapshot->>'title','source_message_id',s.message_id),'[]'::jsonb,
 coalesce((select jsonb_agg(a.metadata||jsonb_build_object('id',a.attachment_id) order by a.attachment_id) from scope_change_event_attachments a where a.event_id=e.id),'[]')
from scope_events e join change_requests c on c.id=e.change_request_id
join scope_change_revisions r on r.change_request_id=c.id and r.version=e.version
left join conversation_change_sources s on s.change_request_id=c.id
where r.snapshot->>'status'<>'draft' and e.kind<>'draft';

-- Guardas de la migración 2.
create or replace function scope_project_event_guard() returns trigger language plpgsql as $$
declare s scope_projects%rowtype; r scope_revisions%rowtype;
begin
    perform 1 from projects where id=new.project_id for update;
    select * into s from scope_projects where project_id=new.project_id for update;
    select * into r from scope_revisions where project_id=new.project_id and revision=new.revision;
    if not found or not r.published then raise exception 'scope_conflict: Esta revisión no está publicada.'; end if;
    if new.actor_kind='client' and not exists(select 1 from client_auth where id=new.actor_id and user_id=s.user_id and client_id=s.client_id) then
        raise exception 'scope_conflict: La identidad no pertenece al cliente.';
    end if;
    if new.actor_kind='professional' and (new.actor_id<>s.user_id or new.kind<>'comment') then
        raise exception 'scope_conflict: Acción de negociación no permitida.';
    end if;
    if new.document_id is not null and not exists(select 1 from jsonb_array_elements(coalesce(r.document->'documents','[]')) d where d->>'id'=new.document_id::text) then
        raise exception 'scope_conflict: El documento no pertenece a esta revisión.';
    end if;
    if new.kind<>'comment' and s.accepted_revision is not null then raise exception 'scope_conflict: El acuerdo ya está activo; solicite un cambio.'; end if;
    if new.kind in ('reject','request_changes') then
        if new.revision<>(select max(revision) from scope_revisions where project_id=new.project_id and published)
           or exists(select 1 from scope_project_events where project_id=new.project_id and revision=new.revision and kind in ('reject','request_changes')) then
            raise exception 'scope_conflict: La propuesta cambió o ya recibió una decisión.';
        end if;
        update scope_notices set resolved=true where kind='baseline' and record_id=new.project_id;
    end if;
    new.recorded_at:=clock_timestamp();
    return new;
end; $$;

create or replace function scope_baseline_guard() returns trigger language plpgsql as $$
declare state scope_projects%rowtype; item jsonb; t tasks%rowtype;
begin
    perform 1 from projects where id=new.project_id for update;
    select * into state from scope_projects where project_id=new.project_id for update;
    if tg_op='INSERT' then
        if state.accepted_revision is not null or new.revision<>state.last_revision+1 then
            raise exception 'scope_conflict: El alcance cambió o ya fue aceptado.';
        end if;
        if new.accepted_at is not null or new.accepted_by is not null or new.accepted_ip is not null then
            raise exception 'scope_conflict: No se permite fabricar una aceptación.';
        end if;
        update scope_projects set last_revision=new.revision where project_id=new.project_id;
        if new.published then
            update projects set enforce_scope=true where id=new.project_id;
            -- Solo los archivos seleccionados explícitamente forman parte del acuerdo.
            for item in select * from jsonb_array_elements(new.attachment_snapshot) loop
                insert into scope_preserved_attachments(attachment_id,metadata)
                values((item->>'id')::uuid,item) on conflict do nothing;
            end loop;
            update scope_notices set resolved=true where kind='baseline' and record_id=new.project_id;
            insert into scope_notices(user_id,client_id,kind,record_id,revision,title)
            values(state.user_id,state.client_id,'baseline',new.project_id,new.revision,'Revisar propuesta de alcance');
        end if;
        return new;
    end if;
    if (to_jsonb(new)-'accepted_at'-'accepted_by'-'accepted_ip'-'accepted_ip_source'-'acceptance_statement') is distinct from
       (to_jsonb(old)-'accepted_at'-'accepted_by'-'accepted_ip'-'accepted_ip_source'-'acceptance_statement')
       or old.accepted_at is not null or new.accepted_at is null or new.accepted_by is null or not old.published then
        raise exception 'scope_conflict: La revisión de alcance es inmutable.';
    end if;
    if new.revision<>(select max(revision) from scope_revisions where project_id=new.project_id and published)
       or state.accepted_revision is not null or exists(select 1 from scope_project_events where project_id=new.project_id and revision=new.revision and kind in ('reject','request_changes')) then
        raise exception 'scope_conflict: Esta revisión ya no está pendiente de aceptación.';
    end if;
    if not exists(select 1 from client_auth where id=new.accepted_by and user_id=state.user_id and client_id=state.client_id) then
        raise exception 'scope_conflict: El actor no pertenece al cliente del proyecto.';
    end if;
    new.accepted_at:=clock_timestamp();
    update scope_projects set accepted_revision=new.revision,activated_at=new.accepted_at where project_id=new.project_id;
    for item in select * from jsonb_array_elements(new.task_snapshot) loop
        select * into t from tasks where id=(item->>'id')::uuid;
        if not found or t.project_id is distinct from new.project_id or t.user_id<>state.user_id then
            raise exception 'scope_conflict: Las tareas de la propuesta cambiaron; publique otra revisión.';
        end if;
        insert into scope_task_links(task_id,project_id,baseline_revision,reason,created_by)
        values(t.id,new.project_id,new.revision,'Incluida en el alcance original aceptado',new.accepted_by) on conflict do nothing;
    end loop;
    update scope_notices set resolved=true where kind='baseline' and record_id=new.project_id;
    return new;
end; $$;


drop index scope_consolidation_change_idx;
alter table scope_revisions
    drop column approved_changes,
    drop column source_change_id,
    drop column previous_accepted_revision,
    drop column revision_kind;
