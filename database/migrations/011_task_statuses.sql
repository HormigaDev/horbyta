-- Estados de tarea editables por usuario. Cada estado pertenece a una categoría del
-- sistema (todo, in_progress, review, done, cancelled): las reglas de alcance, el
-- tablero y el panel siguen razonando por categoría, así que un estado propio como
-- "Bloqueada" (categoría in_progress) no rompe ninguna regla existente.
create table task_statuses (
    id uuid primary key default uuidv7(),
    user_id uuid not null references users (id) on delete cascade,
    key varchar(30),
    name varchar(50) not null check (length(trim(name)) > 0),
    category varchar(20) not null
        check (category in ('todo', 'in_progress', 'review', 'done', 'cancelled')),
    color varchar(7) not null default '#64748B' check (color ~ '^#[0-9A-Fa-f]{6}$'),
    position integer not null default 0,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);
create unique index uq_task_statuses_name on task_statuses (user_id, lower(name));
create unique index uq_task_statuses_key on task_statuses (user_id, key) where key is not null;
create index idx_task_statuses_user_position on task_statuses (user_id, position);
create trigger trg_task_statuses_updated_at before update on task_statuses
for each row execute function fn_set_updated_at();

-- Cada usuario conserva al menos un estado inicial y uno final, y no se cambia la
-- categoría de un estado en uso (alteraría el significado de tareas existentes).
create function task_status_guard() returns trigger language plpgsql as $$
begin
    if tg_op = 'DELETE' then
        if old.category in ('todo', 'done')
           and exists (select 1 from users where id = old.user_id)
           and not exists (select 1 from task_statuses
                           where user_id = old.user_id and category = old.category and id <> old.id) then
            raise exception 'task_status: Debe existir al menos un estado de la categoría %.', old.category;
        end if;
        return old;
    end if;
    if new.user_id <> old.user_id then
        raise exception 'task_status: Un estado no cambia de propietario.';
    end if;
    if new.category <> old.category and exists (select 1 from tasks where status_id = old.id) then
        raise exception 'task_status: No se cambia la categoría de un estado en uso.';
    end if;
    if new.category <> old.category and old.category in ('todo', 'done')
       and not exists (select 1 from task_statuses
                       where user_id = old.user_id and category = old.category and id <> old.id) then
        raise exception 'task_status: Debe existir al menos un estado de la categoría %.', old.category;
    end if;
    return new;
end;
$$;
create trigger task_status_guard before update or delete on task_statuses
for each row execute function task_status_guard();

-- Estados por defecto para los usuarios existentes.
insert into task_statuses (user_id, key, name, category, color, position)
select u.id, d.key, d.name, d.key, d.color, d.position
from users u
cross join (values
    ('todo', 'Todo', '#64748B', 1),
    ('in_progress', 'In Progress', '#3B82F6', 2),
    ('review', 'Review', '#A855F7', 3),
    ('done', 'Completed', '#10B981', 4),
    ('cancelled', 'Cancelled', '#EF4444', 5)
) as d (key, name, color, position);

-- Asignación sin disparar reglas de negocio ni alterar updated_at: el estado no cambia.
alter table tasks add column status_id uuid references task_statuses (id);
alter table tasks disable trigger user;
update tasks t set status_id = s.id
from task_statuses s where s.user_id = t.user_id and s.key = t.status;
alter table tasks enable trigger user;
alter table tasks alter column status_id set not null;
create index idx_tasks_status_id on tasks (status_id);

alter table tasks drop constraint chk_tasks_status;
alter table tasks add constraint chk_tasks_status
    check (status in ('todo', 'in_progress', 'review', 'done', 'cancelled'));
comment on column tasks.status is 'Categoría del estado (derivada de status_id por task_status_sync).';
comment on column tasks.status_id is 'Estado editable del usuario (task_statuses).';

-- tasks.status es la categoría del estado elegido. Se ejecuta antes que las demás
-- reglas (orden alfabético "a_…") para que estas vean la categoría correcta. Acepta
-- también escrituras que solo indican la categoría y elige el primer estado de ella.
create function task_status_sync() returns trigger language plpgsql as $$
declare chosen task_statuses%rowtype;
begin
    if new.status_id is not null and (tg_op = 'INSERT' or new.status_id is distinct from old.status_id) then
        select * into chosen from task_statuses where id = new.status_id;
        if not found or chosen.user_id <> new.user_id then
            raise exception 'task_status: El estado no pertenece a este usuario.';
        end if;
        new.status := chosen.category;
    elsif tg_op = 'INSERT' or new.status is distinct from old.status then
        select * into chosen from task_statuses
        where user_id = new.user_id and category = new.status
        order by position, created_at limit 1;
        if not found then
            raise exception 'task_status: No hay un estado de la categoría %.', new.status;
        end if;
        new.status_id := chosen.id;
    end if;
    return new;
end;
$$;
create trigger a_task_status_sync before insert or update on tasks
for each row execute function task_status_sync();

-- Los usuarios nuevos reciben los estados por defecto junto al resto de datos iniciales.
create or replace function fn_seed_defaults()
returns trigger
language plpgsql
as $$
begin
    -- Categories
    insert into categories (user_id, name)
    values
        (new.id, 'General'),
        (new.id, 'Work'),
        (new.id, 'Personal'),
        (new.id, 'Admin')
    on conflict (user_id, name) do nothing;

    -- Activities
    insert into activities (user_id, name)
    values
        (new.id, 'Development'),
        (new.id, 'Design'),
        (new.id, 'Meeting'),
        (new.id, 'Research'),
        (new.id, 'Testing'),
        (new.id, 'Deployment'),
        (new.id, 'Maintenance'),
        (new.id, 'Communication'),
        (new.id, 'Planning')
    on conflict (user_id, lower(name)) do nothing;

    -- Tags
    insert into tags (user_id, name, color)
    values
        (new.id, 'urgent', '#ef4444'),
        (new.id, 'bug', '#fb7185'),
        (new.id, 'feature', '#e7bd30'),
        (new.id, 'improvement', '#10b981'),
        (new.id, 'blocked', '#6B7280')
    on conflict (user_id, name) do nothing;

    -- Task statuses (editable; `key` identifies the system defaults)
    insert into task_statuses (user_id, key, name, category, color, position)
    values
        (new.id, 'todo', 'Todo', 'todo', '#64748B', 1),
        (new.id, 'in_progress', 'In Progress', 'in_progress', '#3B82F6', 2),
        (new.id, 'review', 'Review', 'review', '#A855F7', 3),
        (new.id, 'done', 'Completed', 'done', '#10B981', 4),
        (new.id, 'cancelled', 'Cancelled', 'cancelled', '#EF4444', 5)
    on conflict do nothing;

    return new;
end;
$$;

-- Una tarea cancelada no cuenta como vencida.
create or replace view vw_dash_task_bottlenecks as
select
    t.user_id,
    p.name as project_name,
    p.id as project_id,
    count(t.id) as total_tasks,
    sum(case when t.status = 'todo' then 1 else 0 end) as tasks_todo,
    sum(case when t.status = 'in_progress' then 1 else 0 end) as tasks_in_progress,
    sum(case when t.status = 'review' then 1 else 0 end) as tasks_in_review,
    sum(case when t.due_date < now() and t.status not in ('done', 'cancelled') then 1 else 0 end) as overdue_tasks
from tasks t
inner join projects p on t.project_id = p.id
where p.status = 'open'
group by t.user_id, p.id, p.name;
