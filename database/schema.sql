begin;

-- ---------------------------------
-- TABLAS
-- ---------------------------------

create table users (
    id uuid primary key default uuidv7(),
    name varchar(100) not null unique,
    email varchar(255) not null unique,
    preferences jsonb,
    password_hash varchar(255) not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create table categories (
    id uuid primary key default uuidv7(),
    user_id uuid not null references users (id) on delete cascade,
    name varchar(100) not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    unique(user_id, name)
);

create table projects (
    id uuid primary key default uuidv7(),
    user_id uuid not null references users (id) on delete cascade,
    category_id uuid references categories (id) on delete set null,
    name varchar(255) not null,
    description text,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create table milestones (
    id uuid primary key default uuidv7(),
    project_id uuid not null references projects(id) on delete cascade,
    name varchar(255) not null,
    due_date timestamptz,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create table tags (
    id uuid primary key default uuidv7(),
    user_id uuid not null references users (id) on delete cascade,
    name varchar(50) not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    unique(user_id, name)
);

create table tasks (
    id uuid primary key default uuidv7(),
    user_id uuid not null references users (id) on delete cascade,
    project_id uuid references projects (id) on delete cascade,
    parent_id uuid references tasks (id) on delete cascade,
    milestone_id uuid references milestones (id) on delete set null,
    title varchar(255) not null,
    description text,
    status varchar(50) not null default 'todo',
    priority smallint not null default 1,
    complexity smallint not null default 1,
    block_parent boolean not null default true,
    due_date timestamptz,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),

    constraint chk_tasks_status check (status in ('todo', 'in_progress', 'review', 'done')),
    constraint chk_tasks_priority check (priority in (1,2,3,4)),
    constraint chk_tasks_complexity check (complexity in (1,2,3,4)),
    constraint chk_tasks_parent_id check (parent_id <> id)
);

create table task_tags (
    task_id uuid not null references tasks (id) on delete cascade,
    tag_id uuid not null references tags (id) on delete cascade,
    primary key(task_id, tag_id)
);

-- ---------------------------------
-- ÍNDICES
-- ---------------------------------

create index idx_categories_user_id on categories(user_id);
create index idx_projects_user_id on projects(user_id);
create index idx_projects_category_id on projects(category_id);
create index idx_milestones_project_id on milestones(project_id);
create index idx_tags_user_id on tags(user_id);
create index idx_tasks_user_id on tasks(user_id);
create index idx_tasks_project_id on tasks(project_id);
create index idx_tasks_parent_id on tasks(parent_id);
create index idx_tasks_milestone_id on tasks(milestone_id);
create index idx_tasks_status on tasks(status);
create index idx_tasks_priority on tasks(priority);
create index idx_tasks_complexity on tasks(complexity);
create index idx_task_tags_tag_id on task_tags(tag_id);

-- ---------------------------------
-- FUNCIONES Y TRIGGERS (UPDATED_AT AUTOMÁTICO)
-- ---------------------------------

create or replace function fn_set_updated_at()
returns trigger language plpgsql as $$
begin
    new.updated_at = now();
    return new;
end;
$$;

do $$
declare t text;
begin
    foreach t in array array[
        'users',
        'categories',
        'projects',
        'milestones',
        'tags',
        'tasks'
    ]
    loop
        execute format(
            'create trigger trg_%I_updated_at
            before update on %I
            for each row
            execute function fn_set_updated_at()',
            t,
            t
        );
    end loop;
end;
$$;

-- ---------------------------------
-- COMENTARIOS
-- ---------------------------------

comment on table users is 'Inquilinos del sistema. Entidad raíz para el aislamiento de datos (Row Level Security en el futuro si es necesario).';
comment on table categories is 'Agrupaciones lógicas de alto nivel definidas por el usuario (ej. Personal, Trabajo).';
comment on table projects is 'Contenedores principales de tareas. Un proyecto puede pertenecer a una categoría.';
comment on table milestones is 'Puntos de control temporales dentro de un proyecto.';
comment on table tags is 'Etiquetas transversales para clasificación libre. Únicas por usuario.';
comment on table tasks is 'Núcleo del PM. Soporta jerarquía infinita (parent_id) y puede existir sin proyecto (inbox).';
comment on table task_tags is 'Tabla pivote para relación muchos a muchos entre tareas y etiquetas.';

comment on column tasks.parent_id is 'Referencia recursiva para soportar subtareas de n-niveles.';
comment on column tasks.status is 'Estado del flujo de trabajo. Restringido a 4 valores predefinidos por CHECK.';
comment on column tasks.priority is 'Nivel de prioridad (1-4). 1 es más bajo, 4 es crítico.';
comment on column tasks.complexity is 'Estimación de esfuerzo/complejidad (1-4).';

commit;
