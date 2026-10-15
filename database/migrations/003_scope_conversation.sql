-- Historial de conversación: ventana por revisión y adjuntos conservados.
create index scope_project_events_window_idx on scope_project_events(project_id,revision,recorded_at,id);
create table scope_project_event_attachments (
    event_id uuid not null references scope_project_events(id) on delete restrict,
    attachment_id uuid not null references attachments(id) on delete restrict,
    metadata jsonb not null,
    primary key(event_id,attachment_id)
);
create trigger immutable_project_event_attachments before update or delete on scope_project_event_attachments
for each row execute function scope_immutable();
create function scope_event_attachment_guard() returns trigger language plpgsql as $$
declare e scope_project_events%rowtype; a attachments%rowtype;
begin
    select * into e from scope_project_events where id=new.event_id for update;
    select * into a from attachments where id=new.attachment_id for update;
    if not found or a.project_id is distinct from e.project_id
       or not exists(select 1 from projects p where p.id=e.project_id and p.user_id=a.user_id) then
        raise exception 'scope_conflict: Attachment does not belong to this conversation.';
    end if;
    if (select count(*) from scope_project_event_attachments where event_id=new.event_id)>=10 then
        raise exception 'scope_conflict: At most 10 attachments per message.';
    end if;
    if lower(coalesce(a.mime_type,'')) like 'audio/%' or lower(a.original_name) ~ '\.(mp3|wav|ogg|oga|opus|m4a|aac|flac|aiff|aif|amr|wma)$' then
        raise exception 'scope_conflict: Audio attachments are not supported in conversations.';
    end if;
    new.metadata:=to_jsonb(a);
    insert into scope_preserved_attachments(attachment_id,metadata)
        values(a.id,new.metadata) on conflict do nothing;
    return new;
end; $$;
create trigger scope_event_attachment_guard before insert on scope_project_event_attachments
for each row execute function scope_event_attachment_guard();
