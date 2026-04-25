begin;

-- ---------------------------------
-- TABLES
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

create table seq_by_user (
    user_id uuid not null references users (id) on delete cascade,
    entity varchar(50) not null,
    last_value bigint not null default 0,
    primary key (user_id, entity)
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
    magic_link_hash bytea,
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
    record_number bigint not null,
    user_id uuid not null references users (id) on delete cascade,
    client_id uuid references clients (id) on delete set null,
    category_id uuid references categories (id) on delete set null,
    name varchar(255) not null,
    description text,
    status varchar(20) not null default 'open',
    billable boolean not null default true,

    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),

    constraint chk_projects_status check (status in ('open', 'closed', 'cancelled'))
);

create table project_billing (
    id uuid primary key default uuidv7(),
    user_id uuid not null references users (id) on delete cascade,
    project_id uuid not null references projects (id) on delete cascade,
    currency_type varchar(3) not null,
    billing_type varchar(30) not null default 'fixed',
    hourly_rate numeric(15,4),
    fixed_price numeric(15,4),
    retainer_amount numeric(15,4),
    payment_day smallint,

    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),

    constraint chk_payment_day_is_in_range check (payment_day >= 0 and payment_day <= 28),
    constraint chk_project_billing_billing_type check (billing_type in ('fixed', 'hourly', 'retainer')),
    constraint chk_project_billing_billing check (
        (
            billing_type = 'fixed'
            and fixed_price is not null
            and hourly_rate is null
            and retainer_amount is null
            and payment_day is null
        ) or (
            billing_type = 'hourly'
            and hourly_rate is not null
            and fixed_price is null
            and retainer_amount is null
            and payment_day is null
        ) or (
            billing_type = 'retainer'
            and payment_day is not null
            and retainer_amount is not null
            and fixed_price is null
            and hourly_rate is null
        )
    ),
    constraint uq_project_billing_project unique (user_id, project_id)
);

create table milestones (
    id uuid primary key default uuidv7(),
    project_id uuid not null references projects(id) on delete cascade,
    name varchar(255) not null,
    description text,
    status varchar(20) not null default 'open',
    due_date timestamptz,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),

    constraint chk_milestones_status check (status in ('open', 'review', 'approved', 'released', 'cancelled'))
);

create table invoices (
    id uuid primary key default uuidv7(),
    record_number bigint not null,
    user_id uuid not null references users (id) on delete cascade,
    project_id uuid not null references projects (id) on delete cascade,
    
    amount numeric(15,4) not null,
    currency_type varchar(3) not null,

    status varchar(30) not null default 'draft',
    due_date date,
    issued_at timestamptz,

    provider varchar(30) not null default 'manual',
    provider_external_id varchar(255),
    payment_link varchar(1024),

    notes text,

    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),

    constraint chk_invoices_status check (status in ('draft', 'open', 'paid', 'void', 'uncollectible')),
    constraint chk_invoices_provider check (
        (provider = 'manual') or
        (provider <> 'manual' and provider_external_id is not null)
    )
);

create table payments (
    id uuid primary key default uuidv7(),
    record_number bigint not null,
    user_id uuid not null references users (id) on delete cascade,
    project_id uuid not null references projects (id) on delete cascade,
    invoice_id uuid references invoices (id) on delete set null,
    parent_payment_id uuid references payments (id) on delete restrict,

    amount numeric(15,4) not null,
    currency_type varchar(3) not null,
    transaction_type varchar(20) not null default 'charge',
    
    payment_date date not null default current_date,

    provider varchar(30) not null default 'manual',
    provider_external_id varchar(255),
    payment_method varchar(50),
    reference_code varchar(255),

    notes text,

    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),

    constraint chk_payments_amount check (amount > 0),
    constraint chk_payments_provider check (
        (provider = 'manual') or 
        (provider <> 'manual' and provider_external_id is not null)
    ),
    constraint chk_payments_type check (transaction_type in ('charge', 'refund')),
    constraint chk_payments_refund_logic check (
        (transaction_type = 'charge' and parent_payment_id is null)
        or
        (transaction_type = 'refund' and parent_payment_id is not null)
    )
);

create table tags (
    id uuid primary key default uuidv7(),
    user_id uuid not null references users (id) on delete cascade,
    name varchar(50) not null,
    color varchar(7) not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    unique(user_id, name)
);

create table tasks (
    id uuid primary key default uuidv7(),
    record_number bigint not null,
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

create table activities (
    id uuid primary key default uuidv7(),
    user_id uuid not null references users (id) on delete cascade,
    name varchar(100) not null,

    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create table time_entries (
    id uuid primary key default uuidv7(),
    user_id uuid not null references users (id) on delete cascade,
    task_id uuid not null references tasks (id) on delete cascade,
    activity uuid not null references activities (id) on delete restrict,
    invoice_id uuid references invoices (id) on delete set null,
    start_time timestamptz,
    end_time timestamptz,
    duration_minutes integer not null default 0,
    work_date date not null default current_date,
    billable boolean not null default true,
    applied_rate numeric(15,4),
    notes text,

    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),

    constraint chk_time_entries_logic check (
        (start_time is not null and end_time is null and duration_minutes = 0)
        or
        (start_time is not null and end_time is not null and end_time > start_time and duration_minutes > 0)
        or
        (start_time is null and end_time is null and duration_minutes > 0)
    )
);

create table task_tags (
    task_id uuid not null references tasks (id) on delete cascade,
    tag_id uuid not null references tags (id) on delete cascade,
    primary key(task_id, tag_id)
);

create table attachments (
    id uuid primary key default uuidv7(),
    user_id uuid not null references users (id) on delete cascade,
    project_id uuid references projects (id) on delete cascade,
    task_id uuid references tasks (id) on delete cascade,

    original_name varchar(255) not null,
    storage_key varchar(512) not null unique,
    mime_type varchar(100) not null default 'application/octet-stream',
    size_bytes bigint not null default 0,

    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),

    constraint chk_attachments_linked check (
        project_id is not null or task_id is not null
    )
);

create table change_requests (
    id uuid primary key default uuidv7(),
    record_number bigint not null,
    user_id uuid not null references users (id) on delete cascade,
    project_id uuid not null references projects (id) on delete cascade,
    client_id uuid not null references clients (id) on delete cascade,
    task_id uuid references tasks (id) on delete set null,

    origin varchar(30) not null default 'freelancer_dashboard',

    title varchar(255) not null,
    client_request_description text not null,
    
    technical_assessment text,
    impact_hours numeric(6,2) not null default 0.00,
    impact_cost numeric(15,4) not null default 0.00,
    schedule_impact_days integer not null default 0,

    status varchar(30) not null default 'draft',
    decline_reason varchar(100),

    client_approved_at timestamptz,
    client_signature_ip varchar(45),

    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),

    version integer not null default 1,

    constraint chk_change_requests_origin check (origin in ('client_portal', 'freelancer_dashboard')),
    constraint chk_change_requests_status check (
        status in (
            'draft',
            'submitted',
            'assessed',
            'approved',
            'declined',
            'implemented',
            'cancelled'
        )
    ),
    constraint chk_change_requests_decline_reason check (
        decline_reason is null or decline_reason in (
            'budget_exceeded',
            'roi_not_justified',

            'schedule_too_long',
            'jeopardizes_main_deadline',

            'postponed_to_next_phase',
            'rethinking_requirements',
            'not_necessary_anymore',
            
            'will_solve_internaly',
            'thought_it_was_included'
        )
    )
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
-- INDEXES
-- ---------------------------------

create index idx_attachments_user_id on attachments(user_id);
create index idx_attachments_project_id on attachments(project_id);
create index idx_attachments_task_id on attachments(task_id);
create index idx_attachments_user_created_at on attachments(user_id, created_at desc);
create unique index uq_users_email on users (lower(email));
create index idx_categories_user_id on categories(user_id);
create index idx_categories_user_created_at on categories(user_id, created_at desc);
create index idx_categories_user_updated_at on categories(user_id, updated_at desc);
create unique index idx_uq_projects_number_per_user on projects(user_id, record_number);
create index idx_projects_user_id on projects(user_id);
create index idx_projects_category_id on projects(category_id);
create index idx_projects_client_id on projects(client_id);
create index idx_projects_status on projects(status);
create index idx_projects_user_name on projects(user_id, name);
create index idx_projects_user_created_at on projects(user_id, created_at desc);
create index idx_projects_user_updated_at on projects(user_id, updated_at desc);
create index idx_milestones_project_id on milestones(project_id);
create index idx_milestones_project_name on milestones(project_id, name);
create index idx_milestones_status on milestones(status);
create index idx_milestones_project_due_date on milestones(project_id, due_date);
create index idx_milestones_project_created_at on milestones(project_id, created_at desc);
create index idx_milestones_project_updated_at on milestones(project_id, updated_at desc);
create index idx_invoices_user on invoices(user_id);
create index idx_invoices_project on invoices(project_id);
create index idx_invoices_record_number on invoices(record_number);
create unique index idx_uq_invoices_external_id on invoices(provider, provider_external_id) where provider_external_id is not null;
create index idx_payments_user on payments(user_id);
create index idx_payments_project on payments(project_id);
create index idx_payments_invoice on payments(invoice_id);
create index idx_payments_record_number on payments(record_number);
create index idx_payments_parent on payments(parent_payment_id);
create unique index idx_uq_payments_external_id on payments(provider, provider_external_id) where provider_external_id is not null;
create index idx_tags_user_id on tags(user_id);
create index idx_tags_user_created_at on tags(user_id, created_at desc);
create index idx_tags_user_updated_at on tags(user_id, updated_at desc);
create index idx_tasks_user_id on tasks(user_id);
create unique index idx_uq_tasks_number_per_user on tasks (user_id, record_number);
create index idx_tasks_project_id on tasks(project_id);
create index idx_tasks_parent_id on tasks(parent_id);
create index idx_tasks_milestone_id on tasks(milestone_id);
create index idx_tasks_status on tasks(status);
create index idx_tasks_priority on tasks(priority);
create index idx_tasks_complexity on tasks(complexity);
create index idx_tasks_user_title on tasks(user_id, title);
create index idx_tasks_user_block_parent on tasks(user_id, block_parent);
create index idx_tasks_user_due_date on tasks(user_id, due_date);
create index idx_tasks_user_created_at on tasks(user_id, created_at desc);
create index idx_tasks_user_updated_at on tasks(user_id, updated_at desc);
create index idx_task_tags_tag_id on task_tags(tag_id);
create unique index idx_activities_name_per_user on activities(user_id, lower(name));
create index idx_activities_user_created_at on activities(user_id, created_at desc);
create index idx_activities_user_updated_at on activities(user_id, updated_at desc);
create index idx_time_entries_task on time_entries(task_id);
create index idx_time_entries_user on time_entries(user_id);
create index idx_time_entries_invoice on time_entries(invoice_id);
create index idx_time_entries_created_at on time_entries(user_id, created_at desc);
create index idx_conversation_history_user on conversation_history(user_id);
create index idx_conversation_history_user_title on conversation_history(user_id, title);
create index idx_conversation_history_user_created_at on conversation_history(user_id, created_at desc);
create index idx_conversation_history_user_updated_at on conversation_history(user_id, updated_at desc);
create index idx_notifications_user_read_created on notifications(user_id, is_read, created_at desc);
create index idx_notifications_user_preference on notifications(user_id, preference_key);
create index idx_notifications_created_at on notifications(created_at desc);
create index idx_clients_user_id on clients(user_id);
create index idx_clients_status on clients(status);
create index idx_clients_name on clients(name);
create index idx_clients_user_created_at on clients(user_id, created_at desc);
create index idx_clients_user_updated_at on clients(user_id, updated_at desc);
create unique index idx_uq_client_auth_email_per_user on client_auth (user_id, lower(email)) where email is not null;
create unique index idx_uq_change_requests_number_per_user on change_requests (user_id, record_number);
create index idx_change_requests_user_id on change_requests(user_id);
create index idx_change_requests_project_id on change_requests(project_id);
create index idx_change_requests_client_id on change_requests(client_id);
create index idx_change_requests_status on change_requests(status);
create index idx_change_requests_created_at on change_requests(user_id, created_at desc);

-- ---------------------------------
-- FUNCTIONS AND TRIGGERS (AUTOMATIC UPDATED_AT)
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
        'client_auth',
        'attachments',
        'change_requests',
        'project_billing',
        'activities',
        'time_entries',
        'invoices',
        'payments'
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

create or replace function next_seq_for_user(p_user_id uuid, p_entity varchar)
returns bigint
language sql
as $$
    insert into seq_by_user (user_id, entity, last_value)
    values (p_user_id, p_entity, 1)
    on conflict (user_id, entity) do update
        set last_value = seq_by_user.last_value + 1
    returning last_value;
$$;

create or replace function trg_set_seq_number()
returns trigger
language plpgsql
as $$
begin
    new.record_number := next_seq_for_user(new.user_id, TG_TABLE_NAME::varchar);
    return new;
end;
$$;

do $$
declare t text;
begin
    foreach t in array array[
        'tasks',
        'projects',
        'change_requests',
        'invoices',
        'payments'
    ]
    loop
        execute format(
            'create trigger trg_%I_seq_number
            before insert on %I
            for each row
            execute function trg_set_seq_number()',
            t,
            t
        );
    end loop;
end;
$$;

---------------------------------------------

create or replace function fn_increment_cr_version()
returns trigger language plpgsql as $$
begin
    if new.client_request_description <> old.client_request_description or new.title <> old.title then
        new.version := old.version + 1;
        if old.status not in ('draft', 'submitted') then
            raise exception 'No se puede editar la descripción o título de una solicitud de cambio si este ya fue evaluado.';
        end if;
    end if;
    return new;
end;
$$;

create trigger trg_increment_cr_version
before update on change_requests
for each row
execute function fn_increment_cr_version();

---------------------------------------------

create or replace function fn_check_daily_hours_limit()
returns trigger language plpgsql as $$
declare
    total_minutes_registered integer;
    user_max_minutes integer;
begin
    select (
        least(
            greatest(coalesce((preferences->>'max_daily_hours')::integer, 8), 1),
            14
        ) * 60
    )
    into user_max_minutes
    from users
    where id = new.user_id;

    select coalesce(sum(duration_minutes), 0)
    into total_minutes_registered
    from time_entries
    where user_id = new.user_id
        and work_date = new.work_date
        and (TG_OP = 'INSERT' or id <> new.id);

    if (total_minutes_registered + new.duration_minutes) > user_max_minutes then
        raise exception 'daily_limit_exceeded: personal limit exceeded. yout maximum is % hours. you are trying to add % hours to a day that already has % hours',
            (user_max_minutes / 60),
            round(new.duration_minutes / 60.0, 1),
            round(total_minutes_registered / 60.0, 1)
        using hint = 'Check if you left a timer running, or adjust your maximum hours settings.';
    end if;

    return new;
end;
$$;

create trigger trg_check_daily_hours_limit
before insert or update on time_entries
for each row
execute function fn_check_daily_hours_limit();

---------------------------------------------

create or replace function fn_prevent_payment_tampering()
returns trigger language plpgsql as $$
begin
    if (
        new.amount <> old.amount
        or new.currency_type <> old.currency_type
        or new.payment_date <> old.payment_date
        or new.invoice_id is distinct from old.invoice_id
        or new.provider <> old.provider
        or new.provider_external_id is distinct from old.provider_external_id
    ) then
        raise exception 'payment_tampering_detected: Payments records are immutable. You cannot change amounts, dates, currencies, vendors, or associated invoices. If there is an error, you must record a refund or adjustment.';
    end if;

    return new;
end;
$$;

create trigger trg_prevent_payment_tampering
before update on payments
for each row
execute function fn_prevent_payment_tampering();

---------------------------------------------

create or replace function fn_release_voided_time_entries()
returns trigger language plpgsql as $$
begin
    if new.status = 'void' and old.status <> 'void' then
        if exists (select 1 from payments where invoice_id = new.id) then
            raise exception 'invoice_has_payments: You cannot void an invoice that already has payments recorded. You must refund the payments first.';
        end if;

        update time_entries
        set invoice_id = null
        where invoice_id = new.id;
    end if;

    return new;
end;
$$;

create trigger trg_release_voided_time_entries
after update on invoices
for each row
execute function fn_release_voided_time_entries();

-- ---------------------------------
-- VIEWS
-- ---------------------------------

create view vw_dash_monthly_revenue as
select
    user_id,
    currency_type,
    date_trunc('month', payment_date)::date as month_date,
    sum(
        case when transaction_type = 'charge' then amount else -amount end
    ) as total_revenue
from payments
group by user_id, currency_type, date_trunc('month', payment_date)
order by month_date desc;

create view vw_dash_accounts_receivable as
with invoice_balances as (
    select
        i.id as invoice_id,
        i.user_id,
        i.currency_type,
        i.due_date,
        prj.client_id,
        i.amount - coalesce((
            select sum(case when p.transaction_type = 'charge' then p.amount else -p.amount end)
            from payments p
            where p.invoice_id = i.id
        ), 0) as invoice_outstanding
    from invoices i
    inner join projects prj on i.project_id = prj.id
    where i.status = 'open'
)
select
    ib.user_id,
    c.name as client_name,
    ib.currency_type,
    sum(ib.invoice_outstanding) as outstanding_balance,
    min(ib.due_date) as oldest_due_date
from invoice_balances ib
inner join clients c on ib.client_id = c.id
group by ib.user_id, c.name, ib.currency_type
having sum(ib.invoice_outstanding) > 0;


create view vw_dash_time_by_activity as
select
    te.user_id,
    a.name as activity_name,
    te.billable,
    date_trunc('week', te.work_date)::date as week_date,
    round(sum(te.duration_minutes) / 60.0, 2) as total_hours
from time_entries te
inner join activities a on te.activity = a.id
group by te.user_id, a.name, te.billable, date_trunc('week', te.work_date);

create view vw_dash_billable_ratio_daily as
select
    user_id,
    work_date,
    round(sum(case when billable then duration_minutes else 0 end) / 60.0, 2) as billable_hours,
    round(sum(case when not billable then duration_minutes else 0 end) / 60.0, 2) as no_billable_hours,
    round(sum(duration_minutes) / 60.0, 2) as total_hours
from time_entries
group by user_id, work_date
order by work_date desc;

create view vw_dash_scope_creep_value as
select
    cr.user_id,
    pb.currency_type,
    count(cr.id) as total_approved_changes,
    sum(cr.impact_cost) as total_additional_revenue,
    sum(cr.schedule_impact_days) as total_days_extended
from change_requests cr
inner join projects p on cr.project_id = p.id
inner join project_billing pb on p.id = pb.project_id
where cr.status in ('approved', 'implemented')
group by cr.user_id, pb.currency_type;

create view vw_dash_task_bottlenecks as
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

create view vw_client_dash_project_status as
select
    p.user_id,
    p.client_id,
    p.status,
    count(*) as total_projects
from projects p
where p.client_id is not null
group by p.user_id, p.client_id, p.status;

create view vw_client_dash_milestone_health as
select
    p.user_id,
    p.client_id,
    p.id as project_id,
    p.name as project_name,
    count(m.id) as total_milestones,
    sum(case when m.status in ('approved', 'released') then 1 else 0 end) as completed_milestones,
    sum(
        case
            when m.status not in ('approved', 'released', 'cancelled')
                and m.due_date is not null
                and m.due_date < now()
            then 1
            else 0
        end
    ) as overdue_milestones,
    sum(
        case
            when m.status not in ('approved', 'released', 'cancelled')
                and m.due_date is not null
                and m.due_date >= now()
                and m.due_date < now() + interval '7 days'
            then 1
            else 0
        end
    ) as upcoming_milestones,
    min(m.due_date::date) filter (
        where m.status not in ('approved', 'released', 'cancelled')
            and m.due_date is not null
    ) as next_due_date
from projects p
left join milestones m on m.project_id = p.id
where p.client_id is not null
group by p.user_id, p.client_id, p.id, p.name;

create view vw_client_dash_change_request_status as
select
    cr.user_id,
    cr.client_id,
    cr.status,
    count(*) as total_requests
from change_requests cr
group by cr.user_id, cr.client_id, cr.status;

create view vw_client_dash_approved_change_value as
select
    cr.user_id,
    cr.client_id,
    pb.currency_type,
    count(cr.id) as total_approved_changes,
    sum(cr.impact_cost) as total_approved_value
from change_requests cr
inner join projects p on p.id = cr.project_id
inner join project_billing pb on pb.project_id = p.id and pb.user_id = p.user_id
where cr.status in ('approved', 'implemented')
group by cr.user_id, cr.client_id, pb.currency_type;

create view vw_client_dash_project_portfolio as
with next_milestone as (
    select
        m.project_id,
        m.name as next_milestone_name,
        m.due_date::date as next_milestone_due_date,
        row_number() over (
            partition by m.project_id
            order by m.due_date asc nulls last, m.created_at asc
        ) as rn
    from milestones m
    where m.status not in ('approved', 'released', 'cancelled')
)
select
    p.user_id,
    p.client_id,
    p.id as project_id,
    p.name as project_name,
    p.status as project_status,
    p.billable,
    pb.currency_type,
    pb.billing_type,
    pb.hourly_rate,
    pb.fixed_price,
    pb.retainer_amount,
    pb.payment_day,
    coalesce(mh.total_milestones, 0) as total_milestones,
    coalesce(mh.completed_milestones, 0) as completed_milestones,
    coalesce(mh.overdue_milestones, 0) as overdue_milestones,
    coalesce(mh.upcoming_milestones, 0) as upcoming_milestones,
    nm.next_milestone_name,
    nm.next_milestone_due_date
from projects p
left join project_billing pb on pb.project_id = p.id and pb.user_id = p.user_id
left join vw_client_dash_milestone_health mh on mh.project_id = p.id
left join next_milestone nm on nm.project_id = p.id and nm.rn = 1
where p.client_id is not null;

-- ---------------------------------
-- COMMENTS
-- ---------------------------------

comment on table users is 'System tenants. Root entity for data isolation (Row Level Security in the future if necessary).';
comment on table categories is 'High-level logical groupings defined by the user (e.g. Personal, Work).';
comment on table projects is 'Main containers for tasks. A project can belong to a category.';
comment on table milestones is 'Temporary checkpoints within a project.';
comment on table tags is 'Cross-cutting tags for free classification. Unique per user.';
comment on table tasks is 'Core of the PM. Supports infinite hierarchy (parent_id) and can exist without a project (inbox).';
comment on table task_tags is 'Pivot table for many-to-many relationship between tasks and tags.';
comment on table conversation_history is 'Table to store chat messages with the AI';
comment on table notifications is 'Persistent asynchronous notifications for the user, respecting their preferences.';
comment on table clients is 'User clients.';
comment on table client_auth is 'Table containing client authentication data for the portal';
comment on table seq_by_user is 'Sequences of tables with incremental numeric fields';
comment on table attachments is 'Attachments uploaded by users. Can be linked to tasks or directly to projects. Stores metadata and the path (storage_key) for the ObjectStore.';
comment on table change_requests is 'Transactional Anti-Scope-Creep engine. Forces the client to formally accept cost overruns and delays through a state action, eliminating negotiation by chat.';
comment on table activities is 'User-defined activities for time tracking (e.g. Design, Coding, Meeting). Used to categorize time entries.';
comment on table time_entries is 'Time tracking entries. Each entry is linked to a task and an activity. Enforces daily hour limits per user.';

comment on column tasks.parent_id is 'Recursive reference to support n-level subtasks.';
comment on column tasks.status is 'Workflow state. Restricted to 4 predefined values by CHECK.';
comment on column tasks.priority is 'Priority level (1-4). 1 is lowest, 4 is critical.';
comment on column tasks.complexity is 'Effort/complexity estimation (1-4).';
comment on column change_requests.client_request_description is 'The immutable description of what the client requested.';
comment on column change_requests.technical_assessment is 'The freelancer''s formal response explaining what the change implies to justify the cost.';
comment on column change_requests.schedule_impact_days is 'Days to be added to the delivery date (milestones) if the client approves this change.';
comment on column change_requests.client_approved_at is 'Timestamp when the client clicked the button to accept cost/delay.';
comment on column change_requests.version is 'Change record version. Increments when the client edits the title or description';
comment on column time_entries.applied_rate is 'Snapshot del hourly_rate. Solo aplica para proyectos hourly.';
comment on column time_entries.invoice_id is 'Liga los tiempos trabajados a una factura consolidada.';

commit;
