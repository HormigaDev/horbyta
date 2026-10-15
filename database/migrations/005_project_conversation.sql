-- Conversation is project-wide; historical scope records remain untouched.
create table project_messages (
    id uuid primary key default uuidv7(),
    project_id uuid not null references projects(id) on delete restrict,
    actor_id uuid not null,
    actor_kind text not null check(actor_kind in ('client','professional')),
    text text not null check(length(text)<=4000),
    context jsonb not null default '{}',
    mentions jsonb not null default '[]',
    recorded_at timestamptz not null default clock_timestamp()
);
create index project_messages_window on project_messages(project_id,recorded_at,id);
create trigger immutable_project_messages before update or delete on project_messages for each row execute function scope_immutable();
create table project_message_attachments (
    event_id uuid not null references project_messages(id) on delete restrict,
    attachment_id uuid not null references attachments(id) on delete restrict,
    metadata jsonb not null,
    primary key(event_id,attachment_id)
);
create trigger immutable_project_message_attachments before update or delete on project_message_attachments for each row execute function scope_immutable();
create table project_conversation_reads (
    project_id uuid not null references projects(id) on delete cascade,
    actor_id uuid not null,
    actor_kind text not null check(actor_kind in ('client','professional')),
    last_at timestamptz not null,
    last_id uuid not null,
    primary key(project_id,actor_kind,actor_id)
);
create table conversation_change_sources (
    change_request_id uuid primary key references change_requests(id) on delete restrict,
    project_id uuid not null references projects(id) on delete restrict,
    message_id uuid not null,
    snapshot jsonb not null
);
create trigger immutable_conversation_change_sources before update or delete on conversation_change_sources for each row execute function scope_immutable();
-- A single ordered timeline, without copying or rewriting accepted evidence.
create view project_conversation_timeline as
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
