-- Revierte el control de alcance. Elimina sus tablas y datos; las tablas base permanecen.
drop trigger zz_scope_approval_ip on change_requests;
drop trigger scope_change_record on change_requests;
drop trigger scope_change_guard on change_requests;
drop trigger scope_project_policy_guard on projects;
drop trigger scope_project_guard on projects;
drop trigger scope_invoice_guard on invoices;
drop trigger preagreement_guard on tasks;
drop trigger scope_inherit_task on tasks;
drop trigger scope_task_guard on tasks;
drop trigger preparation_time_guard on time_entries;
drop trigger scope_time_guard on time_entries;
drop trigger scope_attachment_guard on attachments;

-- Vista original de schema.sql (mismas columnas, sin scope_effects).
create or replace view vw_dash_scope_creep_value as
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

alter table change_requests
    drop column client_signature_ip_source,
    drop column legacy,
    drop column schedule_after,
    drop column schedule_before,
    drop column target_milestone_id,
    drop column billing_terms,
    drop column currency_type,
    drop column scope_revision;
alter table tasks drop column scope_preparation_reason;
alter table projects drop column enforce_scope;

drop table scope_task_requirements, scope_project_events, scope_preserved_attachments,
    scope_notices, scope_effects, scope_delivery, scope_events, scope_change_revisions,
    scope_task_links, scope_revisions, scope_projects;

drop function scope_approval_ip(), scope_preparation_time_guard(), scope_task_requirement_guard(),
    scope_project_policy_guard(), scope_preagreement_guard(), scope_project_event_guard(),
    scope_invoice_guard(), scope_project_guard(), scope_change_record(), scope_change_guard(),
    scope_time_guard(), scope_inherit_task(), scope_task_guard(), scope_task_allowed(uuid),
    scope_link_children(), scope_link_guard(), scope_baseline_guard(), scope_attachment_guard(),
    scope_capture_attachments(uuid, uuid, text), scope_immutable();

-- La migración sustituyó este disparador por scope_change_guard.
create trigger trg_increment_cr_version
before update on change_requests
for each row
execute function fn_increment_cr_version();
