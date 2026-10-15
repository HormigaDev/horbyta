-- Migración incremental. El ejecutor proporciona transacción y bloqueo exclusivo.
create table scope_projects (
    project_id uuid primary key references projects(id) on delete restrict,
    user_id uuid not null references users(id) on delete restrict,
    client_id uuid not null references clients(id) on delete restrict,
    last_revision integer not null default 0,
    accepted_revision integer,
    activated_at timestamptz
);
create table scope_revisions (
    project_id uuid not null references scope_projects(project_id) on delete restrict,
    revision integer not null,
    document jsonb not null,
    task_snapshot jsonb not null,
    attachment_snapshot jsonb not null,
    billing_snapshot jsonb,
    milestone_snapshot jsonb not null,
    published boolean not null,
    created_at timestamptz not null default now(),
    created_by uuid not null,
    accepted_at timestamptz,
    accepted_by uuid,
    primary key(project_id, revision)
);
create table scope_task_links (
    task_id uuid primary key references tasks(id) on delete restrict,
    project_id uuid not null references scope_projects(project_id) on delete restrict,
    baseline_revision integer,
    change_request_id uuid references change_requests(id) on delete restrict,
    reason text not null check(length(trim(reason)) > 0),
    created_at timestamptz not null default now(),
    created_by uuid,
    check ((baseline_revision is null) <> (change_request_id is null)),
    foreign key(project_id, baseline_revision) references scope_revisions(project_id, revision) on delete restrict
);
alter table change_requests add column scope_revision integer;
alter table change_requests add column currency_type varchar(3);
alter table change_requests add column billing_terms text;
alter table change_requests add column target_milestone_id uuid references milestones(id) on delete restrict;
alter table change_requests add column schedule_before timestamptz;
alter table change_requests add column schedule_after timestamptz;
alter table change_requests add column legacy boolean not null default false;
alter table change_requests add foreign key(project_id, scope_revision)
    references scope_revisions(project_id, revision) on delete restrict;
-- Procedencia histórica; no representa una oferta publicada/aceptada verificable.
update change_requests set legacy = true;
create index scope_task_links_project_idx on scope_task_links(project_id);
create index scope_task_links_change_idx on scope_task_links(change_request_id);
create table scope_change_revisions (
    change_request_id uuid not null references change_requests(id) on delete restrict,
    version integer not null,
    snapshot jsonb not null,
    actor_id uuid,
    actor_kind text not null,
    recorded_at timestamptz not null default now(),
    provenance text not null default 'live' check(provenance in ('live','legacy')),
    primary key(change_request_id, version)
);
insert into scope_change_revisions(change_request_id,version,snapshot,actor_kind,provenance)
select id,version,to_jsonb(cr)||jsonb_build_object('impact_cost',cr.impact_cost::text,'impact_hours',cr.impact_hours::text),'unknown','legacy' from change_requests cr;
create table scope_events (
    id uuid primary key default uuidv7(),
    change_request_id uuid not null references change_requests(id) on delete restrict,
    version integer not null,
    kind text not null,
    actor_id uuid,
    actor_kind text not null,
    body jsonb not null default '{}',
    recorded_at timestamptz not null default now()
);
create index scope_events_change_idx on scope_events(change_request_id,recorded_at,id);
create table scope_delivery (
    event_id uuid primary key references scope_events(id) on delete restrict,
    attempts integer not null default 0,
    next_attempt_at timestamptz not null default now(),
    delivered_at timestamptz,
    last_error text
);
create index scope_delivery_pending_idx on scope_delivery(next_attempt_at) where delivered_at is null;
create table scope_effects (
    change_request_id uuid primary key references change_requests(id) on delete restrict,
    offer_version integer not null,
    invoice_id uuid unique references invoices(id) on delete restrict,
    amount numeric(15,4) not null,
    currency_type varchar(3),
    milestone_id uuid references milestones(id) on delete restrict,
    schedule_before timestamptz,
    schedule_after timestamptz,
    created_at timestamptz not null default now()
);
create table scope_notices (
    id uuid primary key default uuidv7(),
    user_id uuid not null references users(id) on delete restrict,
    client_id uuid not null references clients(id) on delete restrict,
    kind text not null check(kind in ('baseline','change')),
    record_id uuid not null,
    revision integer not null,
    title text not null,
    resolved boolean not null default false,
    created_at timestamptz not null default now(),
    unique(kind,record_id,revision)
);
create index scope_notices_client_idx on scope_notices(user_id,client_id,resolved,created_at);
create table scope_preserved_attachments (
    attachment_id uuid primary key references attachments(id) on delete restrict,
    metadata jsonb not null,
    preserved_at timestamptz not null default now()
);

create function scope_immutable() returns trigger language plpgsql as $$
begin
    raise exception 'scope_conflict: Este registro conserva un acuerdo; debe archivarse o corregirse con una nueva revisión.';
end;
$$;
create trigger immutable_scope_revisions_delete before delete on scope_revisions for each row execute function scope_immutable();
create trigger immutable_change_revisions before update or delete on scope_change_revisions for each row execute function scope_immutable();
create trigger immutable_scope_events before update or delete on scope_events for each row execute function scope_immutable();
create trigger immutable_scope_effects before update or delete on scope_effects for each row execute function scope_immutable();
create trigger immutable_task_links before update or delete on scope_task_links for each row execute function scope_immutable();
create trigger immutable_preserved_attachments before update or delete on scope_preserved_attachments for each row execute function scope_immutable();

create function scope_capture_attachments(p_project uuid, p_change uuid, p_text text default null) returns void language sql as $$
    insert into scope_preserved_attachments(attachment_id,metadata)
    select a.id,to_jsonb(a) from attachments a
    where a.user_id=(select user_id from projects where id=p_project) and (
      (p_change is not null and a.change_request_id=p_change)
      or (p_change is null and (a.project_id=p_project or a.task_id in (select id from tasks where project_id=p_project)
                               or a.milestone_id in (select id from milestones where project_id=p_project)))
      or a.id::text in (
        select lower(matches[1]) from regexp_matches(
          coalesce(p_text,(select client_request_description||' '||coalesce(technical_assessment,'') from change_requests where id=p_change),''),
          '/api/v1/attachments/([0-9a-fA-F-]{36})/raw','g') as matches
      ))
    on conflict do nothing;
$$;
-- Conservar los adjuntos todavía disponibles de los antecedentes importados.
select scope_capture_attachments(project_id,id) from change_requests;
create function scope_attachment_guard() returns trigger language plpgsql as $$
begin
    if exists(select 1 from scope_preserved_attachments where attachment_id=old.id) then
        raise exception 'scope_conflict: Este adjunto forma parte de un registro conservado; no se reemplaza ni elimina.';
    end if;
    return case when tg_op='DELETE' then old else new end;
end;
$$;
create trigger scope_attachment_guard before update or delete on attachments for each row execute function scope_attachment_guard();

create function scope_baseline_guard() returns trigger language plpgsql as $$
declare state scope_projects%rowtype; item jsonb; t tasks%rowtype;
begin
    if tg_op='INSERT' then
        select * into state from scope_projects where project_id=new.project_id for update;
        if state.accepted_revision is not null then
            raise exception 'scope_conflict: El alcance original ya está aceptado; documente una solicitud de cambio.';
        end if;
        if new.revision <> state.last_revision+1 then
            raise exception 'scope_conflict: El alcance cambió; vuelva a leerlo.';
        end if;
        if new.accepted_at is not null or new.accepted_by is not null then
            raise exception 'scope_conflict: No se permite fabricar una aceptación.';
        end if;
        update scope_projects set last_revision=new.revision where project_id=new.project_id;
        if new.published then
            perform scope_capture_attachments(new.project_id,null,new.document::text);
            update scope_notices set resolved=true where kind='baseline' and record_id=new.project_id;
            insert into scope_notices(user_id,client_id,kind,record_id,revision,title)
            values(state.user_id,state.client_id,'baseline',new.project_id,new.revision,'Aceptar alcance inicial');
        end if;
        return new;
    end if;
    if (to_jsonb(new)-'accepted_at'-'accepted_by') is distinct from (to_jsonb(old)-'accepted_at'-'accepted_by')
       or old.accepted_at is not null or new.accepted_at is null or new.accepted_by is null or not old.published then
        raise exception 'scope_conflict: La revisión de alcance es inmutable.';
    end if;
    select * into state from scope_projects where project_id=new.project_id for update;
    if new.revision<>(select max(revision) from scope_revisions where project_id=new.project_id and published) or state.accepted_revision is not null then
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
            raise exception 'scope_conflict: Las tareas de la propuesta cambiaron; publique un alcance nuevo.';
        end if;
        insert into scope_task_links(task_id,project_id,baseline_revision,reason,created_by)
        values(t.id,new.project_id,new.revision,'Incluida en el alcance original aceptado',new.accepted_by) on conflict do nothing;
    end loop;
    update scope_notices set resolved=true where kind='baseline' and record_id=new.project_id;
    return new;
end;
$$;
create trigger baseline_guard before insert or update on scope_revisions for each row execute function scope_baseline_guard();

create function scope_link_guard() returns trigger language plpgsql as $$
declare t tasks%rowtype; p scope_projects%rowtype; c change_requests%rowtype;
begin
    select * into t from tasks where id=new.task_id for update;
    select * into p from scope_projects where project_id=new.project_id;
    if t.project_id is distinct from new.project_id or t.user_id<>p.user_id or p.accepted_revision is null then
        raise exception 'scope_conflict: La tarea y el acuerdo deben pertenecer al mismo proyecto activo.';
    end if;
    if new.baseline_revision is not null and new.baseline_revision<>p.accepted_revision then
        raise exception 'scope_conflict: La tarea debe referenciar el alcance aceptado.';
    end if;
    if new.change_request_id is not null then
        select * into c from change_requests where id=new.change_request_id;
        if c.project_id<>new.project_id or c.user_id<>t.user_id or c.client_id<>p.client_id then
            raise exception 'scope_conflict: El cambio pertenece a otro proyecto o cliente.';
        end if;
        if c.status not in ('approved','implemented') and
          (t.status<>'todo' or exists(select 1 from time_entries where task_id=t.id)) then
            raise exception 'scope_conflict: El extra pendiente debe ser una tarea nueva sin ejecución ni tiempo.';
        end if;
        if c.status in ('declined','cancelled','implemented') then
            raise exception 'scope_conflict: No se añade trabajo a un cambio rechazado, cancelado o implementado.';
        end if;
        if c.legacy then
            raise exception 'scope_conflict: Un cambio histórico requiere conciliación, no habilita trabajo nuevo.';
        end if;
    end if;
    if t.parent_id is not null and exists(select 1 from scope_task_links l where l.task_id=t.parent_id and
        (l.change_request_id is distinct from new.change_request_id or l.baseline_revision is distinct from new.baseline_revision)) then
        raise exception 'scope_conflict: Una subtarea conserva el origen de su padre; separe el extra.';
    end if;
    if exists(select 1 from tasks child join scope_task_links l on l.task_id=child.id where child.parent_id=new.task_id and
      (l.change_request_id is distinct from new.change_request_id or l.baseline_revision is distinct from new.baseline_revision)) then
        raise exception 'scope_conflict: Las subtareas deben conservar el mismo origen; separe el trabajo adicional.';
    end if;
    return new;
end;
$$;
create trigger scope_link_guard before insert on scope_task_links for each row execute function scope_link_guard();

create function scope_link_children() returns trigger language plpgsql as $$
begin
    insert into scope_task_links(task_id,project_id,baseline_revision,change_request_id,reason,created_by)
    select t.id,new.project_id,new.baseline_revision,new.change_request_id,'Subtarea del trabajo clasificado',new.created_by
    from tasks t where t.parent_id=new.task_id and not exists(select 1 from scope_task_links l where l.task_id=t.id);
    return new;
end;
$$;
create trigger scope_link_children after insert on scope_task_links for each row execute function scope_link_children();

create function scope_task_allowed(p_task uuid) returns boolean language sql stable as $$
    select case when not exists(select 1 from scope_projects s where s.project_id=t.project_id and s.accepted_revision is not null)
        then true else exists(select 1 from scope_task_links l
          left join change_requests c on c.id=l.change_request_id
          left join scope_effects e on e.change_request_id=c.id
          where l.task_id=t.id and (l.baseline_revision is not null or (c.status in ('approved','implemented') and e.change_request_id is not null))) end
    from tasks t where t.id=p_task;
$$;
create function scope_task_guard() returns trigger language plpgsql as $$
declare link scope_task_links%rowtype; controlled boolean;
begin
    if tg_op='UPDATE' then
        if new.project_id is distinct from old.project_id and exists(
            select 1 from scope_projects where project_id=old.project_id and accepted_revision is not null) then
            raise exception 'scope_conflict: Clasificar o mover la tarea no permite salir del proyecto con alcance activo.';
        end if;
        if new.project_id is distinct from old.project_id and exists(
            select 1 from scope_projects where project_id=new.project_id and accepted_revision is not null) and
            (new.status<>'todo' or exists(select 1 from time_entries where task_id=old.id)) then
            raise exception 'scope_conflict: Solo se traslada una propuesta pendiente sin ejecución a un proyecto controlado.';
        end if;
        if exists(select 1 from scope_task_links where task_id=old.id) and
          (new.project_id is distinct from old.project_id or new.user_id<>old.user_id or new.parent_id is distinct from old.parent_id) then
            raise exception 'scope_conflict: No se puede cambiar el origen de trabajo clasificado.';
        end if;
        if new.status<>'todo' and not coalesce(scope_task_allowed(old.id),false) then
            raise exception 'scope_conflict: Trabajo adicional bloqueado hasta aceptar y aplicar el cambio.';
        end if;
    else
        select exists(select 1 from scope_projects where project_id=new.project_id and accepted_revision is not null) into controlled;
        if controlled and new.status<>'todo' then
            raise exception 'scope_conflict: Cree la tarea pendiente y clasifique su alcance antes de ejecutarla.';
        end if;
    end if;
    return new;
end;
$$;
create trigger scope_task_guard before insert or update on tasks for each row execute function scope_task_guard();
create function scope_inherit_task() returns trigger language plpgsql as $$
begin
    if new.parent_id is not null then
        insert into scope_task_links(task_id,project_id,baseline_revision,change_request_id,reason,created_by)
        select new.id,l.project_id,l.baseline_revision,l.change_request_id,'Subtarea del trabajo clasificado',l.created_by
        from scope_task_links l where l.task_id=new.parent_id;
    end if;
    return new;
end;
$$;
create trigger scope_inherit_task after insert on tasks for each row execute function scope_inherit_task();
create function scope_time_guard() returns trigger language plpgsql as $$
begin
    if tg_op='UPDATE' and new.task_id<>old.task_id and exists(
        select 1 from scope_task_links where task_id=old.task_id and change_request_id is not null) then
        raise exception 'scope_conflict: El tiempo de un suplemento conserva su origen; no se traslada para facturarlo otra vez.';
    end if;
    -- El bloqueo de la tarea serializa clasificación, movimientos y registro de tiempo.
    perform 1 from tasks where id=new.task_id for update;
    if not coalesce(scope_task_allowed(new.task_id),false) then
        raise exception 'scope_conflict: No se registra desarrollo antes de aceptar el cambio.';
    end if;
    if new.billable and exists(select 1 from scope_task_links where task_id=new.task_id and change_request_id is not null) then
        raise exception 'scope_conflict: El extra tiene precio fijo; registre su tiempo como no facturable.';
    end if;
    return new;
end;
$$;
create trigger scope_time_guard before insert or update on time_entries for each row execute function scope_time_guard();

-- Sustituye el contador parcial por una versión que cubre toda la oferta.
drop trigger trg_increment_cr_version on change_requests;
create function scope_change_guard() returns trigger language plpgsql as $$
declare s scope_projects%rowtype; m milestones%rowtype; changed boolean; invoice uuid; actor uuid; actor_kind text;
begin
    actor:=nullif(current_setting('horbyta.actor',true),'')::uuid;
    actor_kind:=coalesce(nullif(current_setting('horbyta.actor_kind',true),''),'system');
    if tg_op='DELETE' then
        raise exception 'scope_conflict: Cancele la solicitud; su historial debe conservarse.';
    end if;
    if tg_op='INSERT' then
        new.legacy:=false;
        return new;
    end if;
    if new.project_id<>old.project_id or new.client_id<>old.client_id or new.user_id<>old.user_id or new.origin<>old.origin then
        raise exception 'scope_conflict: No se puede trasladar una negociación a otro acuerdo.';
    end if;
    changed := row(new.title,new.client_request_description,new.technical_assessment,new.impact_hours,new.impact_cost,new.schedule_impact_days,new.target_milestone_id)
       is distinct from row(old.title,old.client_request_description,old.technical_assessment,old.impact_hours,old.impact_cost,old.schedule_impact_days,old.target_milestone_id);
    if old.status in ('approved','declined','implemented','cancelled') then
        if changed or new.task_id is distinct from old.task_id or
           (to_jsonb(new)-'status'-'updated_at') is distinct from (to_jsonb(old)-'status'-'updated_at') then
            raise exception 'scope_conflict: El acuerdo finalizado es inmutable; cree otro cambio.';
        end if;
        if not (new.status=old.status or (old.status='approved' and new.status='implemented')) then
            raise exception 'scope_conflict: Transición final no permitida.';
        end if;
    end if;
    if new.task_id is distinct from old.task_id and old.status not in ('draft','submitted') then
        raise exception 'scope_conflict: Clasifique nuevas tareas con el vínculo de alcance sin alterar la oferta.';
    end if;
    if changed and old.status='assessed' and new.status='assessed' then
        -- Solo publicar explícitamente permite mantener assessed al cambiar términos.
        if current_setting('horbyta.publish_offer',true) is distinct from 'true' then
            new.status:='submitted';
        end if;
    end if;
    if new.status='assessed' and (old.status<>'assessed' or changed or current_setting('horbyta.publish_offer',true)='true') then
        select * into s from scope_projects where project_id=new.project_id for update;
        if s.accepted_revision is null or s.client_id<>new.client_id then
            raise exception 'scope_conflict: Publique y obtenga el aceite del alcance original antes de evaluar extras.';
        end if;
        if nullif(trim(new.technical_assessment),'') is null or new.impact_hours<0 or new.impact_cost<0 or new.schedule_impact_days<0 then
            raise exception 'scope_conflict: Declare evaluación, horas, costo y días no negativos.';
        end if;
        new.scope_revision:=s.accepted_revision;
        new.legacy:=false;
        select currency_type into new.currency_type from project_billing where project_id=new.project_id;
        if new.impact_cost>0 and (new.currency_type is null or not (select billable from projects where id=new.project_id)) then
            raise exception 'scope_conflict: Un suplemento con costo necesita proyecto facturable y moneda.';
        end if;
        new.billing_terms:='Suplemento fijo por este cambio. Las horas son estimadas; su registro de tiempo no se factura otra vez.';
        new.schedule_before:=null; new.schedule_after:=null;
        if new.schedule_impact_days>0 then
            select * into m from milestones where id=new.target_milestone_id for update;
            if not found or m.project_id<>new.project_id or m.due_date is null or m.status in ('approved','released','cancelled') then
                raise exception 'scope_conflict: Elija un hito pendiente del proyecto con fecha para aplicar la extensión.';
            end if;
            new.schedule_before:=m.due_date;
            new.schedule_after:=((m.due_date at time zone 'UTC')+make_interval(days=>new.schedule_impact_days)) at time zone 'UTC';
        else
            new.target_milestone_id:=null;
        end if;
    end if;
    if new.status is distinct from old.status and not (
        (old.status='draft' and new.status in ('submitted','cancelled')) or
        (old.status='submitted' and new.status in ('assessed','cancelled')) or
        (old.status='assessed' and new.status in ('submitted','approved','declined','cancelled')) or
        (old.status='approved' and new.status='implemented')) then
        raise exception 'scope_conflict: Transición de solicitud no permitida.';
    end if;
    -- La escritura condicional en Rust verifica la versión vista. Este contador
    -- también cubre cambios de estado y asociación, serializados por PostgreSQL.
    if (to_jsonb(new)-'updated_at'-'version') is distinct from (to_jsonb(old)-'updated_at'-'version') then
        new.version:=old.version+1;
    end if;
    if new.status='approved' and old.status='assessed' then
        if new.impact_cost>0 and not (select billable from projects where id=new.project_id) then
            raise exception 'scope_conflict: El proyecto ahora no es facturable; solicite una evaluación nueva.';
        end if;
        if new.legacy or new.scope_revision is null or actor_kind<>'client' or not exists(
          select 1 from client_auth where id=actor and client_id=new.client_id and user_id=new.user_id) then
            raise exception 'scope_conflict: Se requiere una oferta vigente y decisión del cliente autenticado.';
        end if;
        if new.schedule_impact_days>0 then
            select * into m from milestones where id=new.target_milestone_id for update;
            if m.due_date is distinct from new.schedule_before or m.status in ('approved','released','cancelled') then
                raise exception 'scope_conflict: El calendario cambió; solicite una evaluación nueva.';
            end if;
            update milestones set due_date=new.schedule_after where id=m.id;
        end if;
        if new.impact_cost>0 then
            insert into invoices(user_id,project_id,amount,currency_type,status,notes)
            values(new.user_id,new.project_id,new.impact_cost,new.currency_type,'draft',
              'Suplemento aprobado: '||new.title||' / cambio '||new.id||' / oferta '||old.version)
            returning id into invoice;
        end if;
        insert into scope_effects(change_request_id,offer_version,invoice_id,amount,currency_type,milestone_id,schedule_before,schedule_after)
        values(new.id,old.version,invoice,new.impact_cost,new.currency_type,new.target_milestone_id,new.schedule_before,new.schedule_after);
        new.client_approved_at:=clock_timestamp();
        new.client_signature_ip:=null;
    end if;
    if new.status='declined' and old.status='assessed' and (actor_kind<>'client' or not exists(
       select 1 from client_auth where id=actor and client_id=new.client_id and user_id=new.user_id)) then
        raise exception 'scope_conflict: Se requiere la decisión del cliente autenticado.';
    end if;
    if new.status='implemented' and old.status='approved' then
        if not exists(select 1 from scope_task_links where change_request_id=new.id) or exists(
          select 1 from scope_task_links l join tasks t on t.id=l.task_id where l.change_request_id=new.id and t.status<>'done') then
            raise exception 'scope_conflict: Vincule y termine todas las tareas del cambio antes de cerrar su implementación.';
        end if;
    end if;
    return new;
end;
$$;
create trigger scope_change_guard before insert or update or delete on change_requests for each row execute function scope_change_guard();
create function scope_change_record() returns trigger language plpgsql as $$
declare actor uuid; actor_kind text; event_id uuid;
begin
    actor:=nullif(current_setting('horbyta.actor',true),'')::uuid;
    actor_kind:=coalesce(nullif(current_setting('horbyta.actor_kind',true),''),'system');
    if tg_op='INSERT' or new.version<>old.version then
        insert into scope_change_revisions(change_request_id,version,snapshot,actor_id,actor_kind)
        values(new.id,new.version,to_jsonb(new)||jsonb_build_object('impact_cost',new.impact_cost::text,'impact_hours',new.impact_hours::text),actor,actor_kind);
        insert into scope_events(change_request_id,version,kind,actor_id,actor_kind,body)
        values(new.id,new.version,new.status,actor,actor_kind,jsonb_build_object('offer_version',case when tg_op='UPDATE' then old.version else null end)) returning id into event_id;
        if actor_kind='client' and new.status in ('approved','declined') then
            insert into scope_delivery(event_id) values(event_id);
        end if;
        perform scope_capture_attachments(new.project_id,new.id);
        if new.task_id is not null and (tg_op='INSERT' or new.task_id is distinct from old.task_id) then
            insert into scope_task_links(task_id,project_id,change_request_id,reason,created_by)
            values(new.task_id,new.project_id,new.id,'Trabajo adicional de la solicitud',actor);
        end if;
        if new.status='assessed' then
            update scope_notices set resolved=true where kind='change' and record_id=new.id;
            insert into scope_notices(user_id,client_id,kind,record_id,revision,title)
            values(new.user_id,new.client_id,'change',new.id,new.version,new.title);
        else
            update scope_notices set resolved=true where kind='change' and record_id=new.id;
        end if;
    end if;
    return new;
end;
$$;
create trigger scope_change_record after insert or update on change_requests for each row execute function scope_change_record();

create function scope_project_guard() returns trigger language plpgsql as $$
begin
    if new.client_id is distinct from old.client_id and exists(select 1 from scope_projects where project_id=old.id) then
        raise exception 'scope_conflict: El cliente forma parte del acuerdo; use otro proyecto para otro cliente.';
    end if;
    return new;
end;
$$;
create trigger scope_project_guard before update on projects for each row execute function scope_project_guard();
create function scope_invoice_guard() returns trigger language plpgsql as $$
begin
    if tg_op='DELETE' and exists(select 1 from scope_effects where invoice_id=old.id) then
        raise exception 'scope_conflict: La factura conserva un suplemento aceptado.';
    end if;
    if tg_op='UPDATE' and exists(select 1 from scope_effects where invoice_id=old.id) and
      row(new.amount,new.currency_type,new.project_id,new.user_id) is distinct from row(old.amount,old.currency_type,old.project_id,old.user_id) then
        raise exception 'scope_conflict: El importe y moneda del suplemento aceptado son inmutables.';
    end if;
    return case when tg_op='DELETE' then old else new end;
end;
$$;
create trigger scope_invoice_guard before update or delete on invoices for each row execute function scope_invoice_guard();

-- Conserva el contrato de la vista existente; el importe es valor aprobado,
-- no ingreso cobrado. Los antecedentes sin moneda acreditada quedan separados.
create or replace view vw_dash_scope_creep_value as
select cr.user_id, coalesce(cr.currency_type,'???')::varchar(3) as currency_type,
 count(cr.id) as total_approved_changes, sum(cr.impact_cost) as total_additional_revenue,
 sum(case when e.change_request_id is not null then cr.schedule_impact_days else 0 end) as total_days_extended
from change_requests cr left join scope_effects e on e.change_request_id=cr.id
where cr.status in ('approved','implemented') group by cr.user_id,coalesce(cr.currency_type,'???')::varchar(3);

-- Los proyectos anteriores conservan compatibilidad; los nuevos requieren acuerdo.
alter table projects add column enforce_scope boolean not null default false;
alter table projects alter column enforce_scope set default true;
update projects set enforce_scope=true where id in(select project_id from scope_projects where accepted_revision is not null);
alter table tasks add column scope_preparation_reason text;
alter table scope_revisions add column bundle_hash text;
alter table scope_revisions add column accepted_ip inet;
alter table scope_revisions add column accepted_ip_source text;
alter table scope_revisions add column acceptance_statement text;
create table scope_project_events (
    id uuid primary key default uuidv7(),
    project_id uuid not null references projects(id) on delete restrict,
    revision integer not null,
    kind text not null check(kind in ('comment','request_changes','reject','prefer')),
    text text not null check(length(trim(text)) between 1 and 4000),
    document_id uuid,
    actor_id uuid not null,
    actor_kind text not null check(actor_kind in ('client','professional')),
    recorded_at timestamptz not null default clock_timestamp(),
    foreign key(project_id,revision) references scope_revisions(project_id,revision) on delete restrict
);
create index scope_project_events_project_idx on scope_project_events(project_id,recorded_at);
create unique index scope_project_decision_once_idx on scope_project_events(project_id,revision)
where kind in ('reject','request_changes');
create trigger immutable_project_events before update or delete on scope_project_events for each row execute function scope_immutable();

-- Serializa conversación, sustitución y decisión sobre el mismo proyecto.
create function scope_project_event_guard() returns trigger language plpgsql as $$
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
create trigger project_event_guard before insert on scope_project_events for each row execute function scope_project_event_guard();

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

create or replace function scope_task_allowed(p_task uuid) returns boolean language sql stable as $$
    select case when t.scope_preparation_reason is not null then true
      when t.project_id is null then true
      when s.accepted_revision is null then not p.enforce_scope
      else exists(select 1 from scope_task_links l left join change_requests c on c.id=l.change_request_id
        left join scope_effects e on e.change_request_id=c.id where l.task_id=t.id and
        (l.baseline_revision is not null or (c.status in ('approved','implemented') and e.change_request_id is not null))) end
    from tasks t left join projects p on p.id=t.project_id left join scope_projects s on s.project_id=t.project_id where t.id=p_task;
$$;
create function scope_preagreement_guard() returns trigger language plpgsql as $$
declare protected boolean; active boolean;
begin
    if tg_op='INSERT' and new.scope_preparation_reason is not null then
        raise exception 'scope_conflict: Cree la tarea pendiente y justifique su preparación desde el control de alcance.';
    end if;
    if tg_op='UPDATE' then
        if old.scope_preparation_reason is not null and (new.scope_preparation_reason is distinct from old.scope_preparation_reason or new.project_id is distinct from old.project_id or new.parent_id is distinct from old.parent_id) then
            raise exception 'scope_conflict: La preparación conserva su origen y justificación.';
        end if;
        if new.scope_preparation_reason is distinct from old.scope_preparation_reason and
           (old.status<>'todo' or new.status<>'todo' or exists(select 1 from time_entries where task_id=old.id)
            or exists(select 1 from scope_task_links where task_id=old.id) or new.project_id is null
            or exists(select 1 from scope_projects where project_id=new.project_id and accepted_revision is not null)
            or length(trim(new.scope_preparation_reason)) not between 1 and 2000) then
            raise exception 'scope_conflict: Solo se justifica preparación pendiente anterior al acuerdo.';
        end if;
        if new.project_id is distinct from old.project_id and
           (exists(select 1 from projects where id=old.project_id and enforce_scope) or
            (exists(select 1 from projects where id=new.project_id and enforce_scope) and
             (new.status<>'todo' or exists(select 1 from time_entries where task_id=old.id)))) then
            raise exception 'scope_conflict: No se traslada ejecución para eludir el acuerdo.';
        end if;
    end if;
    select p.enforce_scope,s.accepted_revision is not null into protected,active
      from projects p left join scope_projects s on s.project_id=p.id where p.id=new.project_id for update of p;
    if protected and not active and new.status<>'todo' and new.scope_preparation_reason is null then
        raise exception 'scope_conflict: Falta aceptar el alcance original; la ejecución está bloqueada.';
    end if;
    return new;
end; $$;
create trigger preagreement_guard before insert or update on tasks for each row execute function scope_preagreement_guard();
create function scope_project_policy_guard() returns trigger language plpgsql as $$
begin
    if old.enforce_scope and not new.enforce_scope then raise exception 'scope_conflict: No se desactiva la protección del alcance.'; end if;
    if new.client_id is distinct from old.client_id and exists(select 1 from scope_projects where project_id=old.id) then
        raise exception 'scope_conflict: El cliente pertenece a la negociación conservada; cree otro proyecto para otro cliente.';
    end if;
    return new;
end; $$;
create trigger scope_project_policy_guard before update on projects for each row execute function scope_project_policy_guard();

create table scope_task_requirements (
    task_id uuid not null references tasks(id) on delete restrict,
    project_id uuid not null,
    revision integer not null,
    document_id uuid not null,
    created_at timestamptz not null default clock_timestamp(),
    primary key(task_id,document_id),
    foreign key(project_id,revision) references scope_revisions(project_id,revision) on delete restrict
);
create trigger immutable_task_requirements before update or delete on scope_task_requirements for each row execute function scope_immutable();
create function scope_task_requirement_guard() returns trigger language plpgsql as $$
begin
    if not exists(select 1 from tasks t join scope_projects s on s.project_id=t.project_id
        join scope_revisions r on r.project_id=s.project_id and r.revision=s.accepted_revision,
        jsonb_array_elements(coalesce(r.document->'documents','[]')) d
        where t.id=new.task_id and t.project_id=new.project_id and new.revision=s.accepted_revision
          and d->>'id'=new.document_id::text and d->>'role'<>'reference') then
        raise exception 'scope_conflict: El requisito debe ser un documento incluido en el alcance aceptado de esta tarea.';
    end if;
    return new;
end; $$;
create trigger task_requirement_guard before insert on scope_task_requirements for each row execute function scope_task_requirement_guard();

create function scope_preparation_time_guard() returns trigger language plpgsql as $$
begin
    if new.billable and exists(select 1 from tasks where id=new.task_id and scope_preparation_reason is not null) then
        raise exception 'scope_conflict: La preparación de propuesta se registra como no facturable; no ejecuta entregables acordados.';
    end if;
    return new;
end; $$;
create trigger preparation_time_guard before insert or update on time_entries for each row execute function scope_preparation_time_guard();

alter table change_requests add column client_signature_ip_source text;
-- El servidor reemplaza cualquier IP enviada por el cliente; corre después del guard.
create function scope_approval_ip() returns trigger language plpgsql as $$
begin
    if new.status='approved' and old.status<>'approved' then
        new.client_signature_ip:=nullif(current_setting('horbyta.accepted_ip',true),'')::inet::text;
        new.client_signature_ip_source:=coalesce(nullif(current_setting('horbyta.ip_source',true),''),'unavailable');
    end if;
    return new;
end; $$;
create trigger zz_scope_approval_ip before update on change_requests for each row execute function scope_approval_ip();
