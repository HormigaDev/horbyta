-- Restaura las funciones de la migración 2: el rechazo vuelve a ser definitivo.
create or replace function scope_change_guard() returns trigger language plpgsql as $$
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

create or replace function scope_change_record() returns trigger language plpgsql as $$
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
