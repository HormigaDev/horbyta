-- Conversación de solicitudes: ventanas por revisión y archivos preservados.
create index scope_change_messages_window_idx on scope_events(change_request_id,version,recorded_at,id) where kind='comment';
create table scope_change_event_attachments (
    event_id uuid not null references scope_events(id) on delete restrict,
    attachment_id uuid not null references attachments(id) on delete restrict,
    metadata jsonb not null,
    primary key(event_id,attachment_id)
);
create trigger immutable_change_event_attachments before update or delete on scope_change_event_attachments
for each row execute function scope_immutable();
create function scope_change_attachment_guard() returns trigger language plpgsql as $$
declare project uuid; owner uuid; a attachments%rowtype;
begin
    select c.project_id,c.user_id into project,owner from scope_events e join change_requests c on c.id=e.change_request_id where e.id=new.event_id and e.kind='comment' for update of e;
    select * into a from attachments where id=new.attachment_id for update;
    if project is null or not found or a.project_id is distinct from project or a.user_id is distinct from owner then
        raise exception 'scope_conflict: Attachment does not belong to this conversation.';
    end if;
    if (select count(*) from scope_change_event_attachments where event_id=new.event_id)>=10 then
        raise exception 'scope_conflict: At most 10 attachments per message.';
    end if;
    if lower(coalesce(a.mime_type,'')) like 'audio/%' or lower(a.original_name) ~ '\.(mp3|wav|ogg|oga|opus|m4a|aac|flac|aiff|aif|amr|wma)$' then
        raise exception 'scope_conflict: Audio attachments are not supported in conversations.';
    end if;
    new.metadata:=to_jsonb(a);
    insert into scope_preserved_attachments(attachment_id,metadata) values(a.id,new.metadata) on conflict do nothing;
    return new;
end; $$;
create trigger scope_change_attachment_guard before insert on scope_change_event_attachments
for each row execute function scope_change_attachment_guard();
