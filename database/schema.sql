begin;

-- ---------------------------------
-- TABLAS
-- ---------------------------------

create table users (
    id uuid primary key default uuidv7(),
    name varchar(100) not null unique,
    email varchar(255) not null,
    preferences jsonb,
    password_hash text not null,
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

create table clients (
    id uuid primary key default uuidv7(),
    user_id uuid not null references users (id),
    name varchar(255) not null,
    status varchar (50) not null default 'active',
    email varchar(255),
    phone varchar(20),
    address_line varchar(512),
    company varchar(255),
    notes text,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),

    constraint chk_clients_status check (status in ('active', 'inactive', 'blocked'))
);

create table client_auth (
    -- identificadores
    id uuid primary key default uuidv7(),
    client_id uuid not null references clients (id) on delete cascade,
    user_id uuid not null references users (id) on delete cascade,
    
    -- si tiene login por credenciales activadas
    email varchar(255),
    password_hash text,

    -- si tiene login por token mágico
    magic_link_hash text,
    magic_link_exp timestamptz,

    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),

    constraint uq_client_auth_client unique (client_id),
    constraint chk_magic_link_not_credentials check (
        (
            email is not null and password_hash is not null
            and magic_link_hash is null and magic_link_exp is null
        ) or (
            email is null and password_hash is null
            and magic_link_hash is not null and magic_link_exp is not null
        )
    )
);

create table projects (
    id uuid primary key default uuidv7(),
    user_id uuid not null references users (id) on delete cascade,
    client_id uuid references clients (id) on delete set null,
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

create table conversation_history (
    id uuid primary key default uuidv7(),
    user_id uuid not null references users (id) on delete cascade,
    session_id uuid not null,
    title varchar(255) not null,
    messages jsonb not null default '[]'::jsonb,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    constraint uq_conversation_user_session unique (user_id, session_id)
);

create table notifications (
    id uuid primary key default uuidv7(),
    user_id uuid not null references users (id) on delete cascade,
    preference_key varchar(80) not null,
    title varchar(255) not null,
    body text,
    payload jsonb,
    severity varchar(20) not null default 'info',
    source varchar(40) not null default 'system',
    is_read boolean not null default false,
    read_at timestamptz,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),

    constraint chk_notifications_preference_key check (
        preference_key in (
            'clients_projects.proposal_decision',
            'clients_projects.deliverable_approved',
            'clients_projects.comment_or_review',
            'clients_projects.added_to_new_project',
            'deliveries_deadlines.deadline_24h_48h',
            'deliveries_deadlines.deliverable_overdue',
            'deliveries_deadlines.milestone_completed',
            'contracts.contract_sent',
            'contracts.contract_signed',
            'contracts.contract_expiring',
            'billing.invoice_sent',
            'billing.invoice_overdue_unpaid',
            'billing.payment_received',
            'billing.auto_payment_reminder',
            'time_tracking.long_running_timer',
            'time_tracking.weekly_hours_summary'
        )
    ),
    constraint chk_notifications_severity check (severity in ('info', 'positive', 'warning', 'negative'))
);

-- ---------------------------------
-- ÍNDICES
-- ---------------------------------

create unique index uq_users_email on users (lower(email));
create index idx_categories_user_id on categories(user_id);
create index idx_projects_user_id on projects(user_id);
create index idx_projects_category_id on projects(category_id);
create index idx_projects_client_id on projects(client_id);
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
create index idx_conversation_history_user on conversation_history(user_id);
create index idx_notifications_user_read_created on notifications(user_id, is_read, created_at desc);
create index idx_notifications_user_preference on notifications(user_id, preference_key);
create index idx_notifications_created_at on notifications(created_at desc);
create index idx_clients_user_id on clients(user_id);
create index idx_clients_status on clients(status);
create index idx_clients_name on clients(name);
create unique index idx_uq_client_auth_email_per_user on client_auth (user_id, lower(email)) where email is not null;

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
        'tasks',
        'conversation_history',
        'notifications',
        'clients',
        'client_auth'
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
comment on table conversation_history is 'Tabla para guardar los mensajes de chat con la IA';
comment on table notifications is 'Notificaciones asíncronas persistentes para el usuario, respetando sus preferencias.';
comment on table clients is 'Clientes del usuario.';

comment on column tasks.parent_id is 'Referencia recursiva para soportar subtareas de n-niveles.';
comment on column tasks.status is 'Estado del flujo de trabajo. Restringido a 4 valores predefinidos por CHECK.';
comment on column tasks.priority is 'Nivel de prioridad (1-4). 1 es más bajo, 4 es crítico.';
comment on column tasks.complexity is 'Estimación de esfuerzo/complejidad (1-4).';

commit;
