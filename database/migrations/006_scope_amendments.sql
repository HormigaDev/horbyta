-- El acuerdo vigente permanece activo mientras se negocia una corrección.
alter table scope_revisions add column revision_kind text not null default 'proposal'
    check (revision_kind in ('proposal','consolidation'));
alter table scope_revisions add column previous_accepted_revision integer;
alter table scope_revisions add foreign key(project_id,previous_accepted_revision)
    references scope_revisions(project_id,revision) on delete restrict;
alter table scope_revisions add column source_change_id uuid references change_requests(id) on delete restrict;
alter table scope_revisions add column approved_changes jsonb not null default '[]'
    check (jsonb_typeof(approved_changes)='array');
create unique index scope_consolidation_change_idx on scope_revisions(source_change_id) where source_change_id is not null;

create or replace function scope_baseline_guard() returns trigger language plpgsql as $$
declare state scope_projects%rowtype; item jsonb; t tasks%rowtype; prior scope_revisions%rowtype;
begin
    perform 1 from projects where id=new.project_id for update;
    select * into state from scope_projects where project_id=new.project_id for update;
    if tg_op='INSERT' then
        if new.revision<>state.last_revision+1 or new.previous_accepted_revision is distinct from state.accepted_revision then
            raise exception 'scope_conflict: El acuerdo vigente cambió; vuelva a leerlo.';
        end if;
        if new.accepted_at is not null or new.accepted_by is not null or new.accepted_ip is not null then
            raise exception 'scope_conflict: No se permite fabricar una aceptación.';
        end if;
        if state.accepted_revision is not null then
            select * into prior from scope_revisions where project_id=new.project_id and revision=state.accepted_revision;
        end if;
        if new.revision_kind='consolidation' then
            if state.accepted_revision is null or not new.published or
               current_setting('horbyta.consolidating_change',true) is distinct from new.source_change_id::text or
               not exists(select 1 from change_requests c join scope_effects e on e.change_request_id=c.id
                   where c.id=new.source_change_id and c.project_id=new.project_id and c.user_id=state.user_id
                   and c.client_id=state.client_id and c.status in ('approved','implemented') and not c.legacy) then
                raise exception 'scope_conflict: La consolidación requiere un cambio aprobado verificable.';
            end if;
            if new.document is distinct from prior.document or new.task_snapshot is distinct from prior.task_snapshot
               or new.attachment_snapshot is distinct from prior.attachment_snapshot or new.billing_snapshot is distinct from prior.billing_snapshot then
                raise exception 'scope_conflict: Consolidar no permite reescribir el acuerdo original.';
            end if;
        else
            if new.source_change_id is not null or
               new.approved_changes is distinct from coalesce(prior.approved_changes,'[]'::jsonb) then
                raise exception 'scope_conflict: Los suplementos aprobados deben conservarse.';
            end if;
            if state.accepted_revision is not null and (
                nullif(trim(new.document->>'amendment_reason'),'') is null or
                (new.document->>'agreement_revision')::integer is distinct from state.accepted_revision) then
                raise exception 'scope_conflict: Declare el motivo de la corrección y el acuerdo vigente.';
            end if;
        end if;
        update scope_projects set last_revision=new.revision where project_id=new.project_id;
        if new.published then
            update projects set enforce_scope=true where id=new.project_id;
            for item in select * from jsonb_array_elements(new.attachment_snapshot) loop
                insert into scope_preserved_attachments(attachment_id,metadata)
                values((item->>'id')::uuid,item) on conflict do nothing;
            end loop;
            update scope_notices set resolved=true where kind='baseline' and record_id=new.project_id;
            if new.revision_kind='proposal' then
                insert into scope_notices(user_id,client_id,kind,record_id,revision,title)
                values(state.user_id,state.client_id,'baseline',new.project_id,new.revision,'Revisar propuesta de alcance');
            else
                -- No hay otra firma: el consentimiento está en el cambio vinculado.
                update scope_projects set accepted_revision=new.revision where project_id=new.project_id;
            end if;
        end if;
        return new;
    end if;
    if (to_jsonb(new)-'accepted_at'-'accepted_by'-'accepted_ip'-'accepted_ip_source'-'acceptance_statement') is distinct from
       (to_jsonb(old)-'accepted_at'-'accepted_by'-'accepted_ip'-'accepted_ip_source'-'acceptance_statement')
       or old.accepted_at is not null or new.accepted_at is null or new.accepted_by is null
       or not old.published or old.revision_kind<>'proposal' then
        raise exception 'scope_conflict: La revisión de alcance es inmutable.';
    end if;
    if new.revision<>(select max(revision) from scope_revisions where project_id=new.project_id and published)
       or new.previous_accepted_revision is distinct from state.accepted_revision
       or exists(select 1 from scope_project_events where project_id=new.project_id and revision=new.revision and kind in ('reject','request_changes')) then
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
        values(t.id,new.project_id,new.revision,'Incluida en el alcance aceptado',new.accepted_by) on conflict do nothing;
    end loop;
    update scope_notices set resolved=true where kind='baseline' and record_id=new.project_id;
    return new;
end; $$;

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
    if new.kind in ('reject','request_changes') then
        if new.revision<>(select max(revision) from scope_revisions where project_id=new.project_id and published)
           or r.accepted_at is not null or r.revision_kind<>'proposal'
           or r.previous_accepted_revision is distinct from s.accepted_revision
           or exists(select 1 from scope_project_events where project_id=new.project_id and revision=new.revision and kind in ('reject','request_changes')) then
            raise exception 'scope_conflict: La propuesta cambió o ya recibió una decisión.';
        end if;
        update scope_notices set resolved=true where kind='baseline' and record_id=new.project_id;
    end if;
    new.recorded_at:=clock_timestamp();
    return new;
end; $$;

-- Dos suplementos independientes pueden aprobarse sin invalidarse entre sí.
-- Una corrección del documento sí requiere volver a evaluar las ofertas antiguas.
create function scope_contract_root(p_project uuid,p_revision integer) returns integer language plpgsql stable as $$
declare r scope_revisions%rowtype;
begin
    loop
        select * into r from scope_revisions where project_id=p_project and revision=p_revision;
        if not found then return null; end if;
        if r.revision_kind='proposal' then return r.revision; end if;
        p_revision:=r.previous_accepted_revision;
    end loop;
end; $$;

create function scope_consolidate_change(p_change uuid) returns void language plpgsql as $$
declare c change_requests%rowtype; e scope_effects%rowtype; s scope_projects%rowtype;
    prior scope_revisions%rowtype; changes jsonb; dates jsonb; offer jsonb; decision jsonb; bundle jsonb;
begin
    select * into c from change_requests where id=p_change;
    if not found or c.status not in ('approved','implemented') or c.legacy then return; end if;
    perform 1 from projects where id=c.project_id for update;
    select * into s from scope_projects where project_id=c.project_id for update;
    if exists(select 1 from scope_revisions where source_change_id=c.id) then return; end if;
    select * into e from scope_effects where change_request_id=c.id;
    if not found then return; end if;
    if scope_contract_root(c.project_id,c.scope_revision) is distinct from scope_contract_root(c.project_id,s.accepted_revision) then
        raise exception 'scope_conflict: El acuerdo cambió; vuelva a evaluar esta solicitud.';
    end if;
    select * into prior from scope_revisions where project_id=c.project_id and revision=s.accepted_revision;
    select snapshot into offer from scope_change_revisions where change_request_id=c.id and version=e.offer_version;
    select jsonb_build_object('actor_id',actor_id,'recorded_at',recorded_at) into decision
        from scope_events where change_request_id=c.id and kind='approved' order by recorded_at limit 1;
    changes:=prior.approved_changes||jsonb_build_array(jsonb_build_object(
        'id',c.id,'title',offer->>'title','offer_version',e.offer_version,
        'description',offer->>'client_request_description','assessment',offer->>'technical_assessment',
        'amount',e.amount::text,'currency_type',e.currency_type,'invoice_id',e.invoice_id,
        'milestone_id',e.milestone_id,'schedule_before',e.schedule_before,'schedule_after',e.schedule_after,
        'approved_at',c.client_approved_at,'approved_by',decision->>'actor_id',
        'accepted_ip',c.client_signature_ip,'accepted_ip_source',c.client_signature_ip_source));
    select coalesce(jsonb_agg(case when item->>'id'=e.milestone_id::text
        then item||jsonb_build_object('due_date',e.schedule_after) else item end),'[]') into dates
        from jsonb_array_elements(prior.milestone_snapshot) item;
    bundle:=jsonb_build_object('document',prior.document,'tasks',prior.task_snapshot,
        'attachments',prior.attachment_snapshot,'billing',prior.billing_snapshot,'milestones',dates,
        'approved_changes',changes,'previous_accepted_revision',s.accepted_revision);
    perform set_config('horbyta.consolidating_change',c.id::text,true);
    insert into scope_revisions(project_id,revision,document,task_snapshot,attachment_snapshot,billing_snapshot,
        milestone_snapshot,published,created_by,bundle_hash,revision_kind,previous_accepted_revision,source_change_id,approved_changes,created_at)
    values(c.project_id,s.last_revision+1,prior.document,prior.task_snapshot,prior.attachment_snapshot,
        prior.billing_snapshot,dates,true,c.user_id,encode(sha256(convert_to(bundle::text,'UTF8')),'hex'),
        'consolidation',s.accepted_revision,c.id,changes,clock_timestamp());
    perform set_config('horbyta.consolidating_change','',true);
end; $$;

create function scope_change_project_lock() returns trigger language plpgsql as $$
begin
    if new.status='approved' and old.status='assessed' then
        perform 1 from projects where id=new.project_id for update;
    end if;
    return new;
end; $$;
create trigger a_scope_change_project_lock before update on change_requests for each row execute function scope_change_project_lock();
create function scope_change_consolidation() returns trigger language plpgsql as $$
begin
    if new.status='approved' and old.status='assessed' then perform scope_consolidate_change(new.id); end if;
    return new;
end; $$;
-- Después de scope_change_record: decisión, oferta e IP ya están conservadas.
create trigger zz_scope_change_consolidation after update on change_requests for each row execute function scope_change_consolidation();

-- Incorpora únicamente acuerdos históricos con efectos verificables, sin facturar otra vez.
do $$ declare c record; begin
    for c in select cr.id from change_requests cr join scope_effects e on e.change_request_id=cr.id
        where cr.status in ('approved','implemented') and not cr.legacy order by cr.client_approved_at,cr.id
    loop perform scope_consolidate_change(c.id); end loop;
end; $$;

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
select md5(r.project_id::text||':'||r.revision||':published')::uuid,r.project_id,
 case when r.revision_kind='consolidation' then 'scope_consolidated' else 'scope_published' end,
 r.created_by,'professional',r.created_at,'',jsonb_build_object('scope_revision',r.revision,'change_request_id',r.source_change_id,'change_title',r.approved_changes->-1->>'title'),'[]'::jsonb,'[]'::jsonb
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
