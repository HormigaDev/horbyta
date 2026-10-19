-- Aceptar una corrección no vuelve a vincular las tareas que ya pertenecen al acuerdo.
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
        -- El trabajo ya vinculado conserva su origen. "on conflict do nothing" no basta:
        -- scope_link_guard se ejecuta antes de detectar el conflicto y rechazaría
        -- las subtareas ya vinculadas a la revisión anterior.
        if not exists(select 1 from scope_task_links where task_id=t.id) then
            insert into scope_task_links(task_id,project_id,baseline_revision,reason,created_by)
            values(t.id,new.project_id,new.revision,'Incluida en el alcance aceptado',new.accepted_by) on conflict do nothing;
        end if;
    end loop;
    update scope_notices set resolved=true where kind='baseline' and record_id=new.project_id;
    return new;
end; $$;
