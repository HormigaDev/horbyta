-- Revierte los estados editables: tasks vuelve a los cuatro valores fijos.
-- El esquema anterior no tiene "cancelada"; si hay tareas así, se rechaza.
do $$ begin
    if exists (select 1 from tasks where status = 'cancelled') then
        raise exception 'Hay tareas en estado cancelado; el esquema anterior no puede representarlas.';
    end if;
end; $$;
create or replace view vw_dash_task_bottlenecks as
select
    t.user_id,
    p.name as project_name,
    p.id as project_id,
    count(t.id) as total_tasks,
    sum(case when t.status = 'todo' then 1 else 0 end) as tasks_todo,
    sum(case when t.status = 'in_progress' then 1 else 0 end) as tasks_in_progress,
    sum(case when t.status = 'review' then 1 else 0 end) as tasks_in_review,
    sum(case when t.due_date < now() and t.status != 'done' then 1 else 0 end) as overdue_tasks
from tasks t
inner join projects p on t.project_id = p.id
where p.status = 'open'
group by t.user_id, p.id, p.name;

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

    return new;
end;
$$;

drop trigger a_task_status_sync on tasks;
drop function task_status_sync();
drop index idx_tasks_status_id;
alter table tasks drop column status_id;
alter table tasks drop constraint chk_tasks_status;
alter table tasks add constraint chk_tasks_status check (status in ('todo', 'in_progress', 'review', 'done'));
comment on column tasks.status is 'Workflow state. Restricted to 4 predefined values by CHECK.';
drop table task_statuses;
drop function task_status_guard();
