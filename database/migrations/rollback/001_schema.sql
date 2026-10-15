-- Revierte el esquema base (schema.sql + uuidv7.sql). Elimina TODOS los datos.
drop view vw_client_dash_project_portfolio, vw_client_dash_approved_change_value,
    vw_client_dash_change_request_status, vw_client_dash_milestone_health,
    vw_client_dash_project_status, vw_dash_task_bottlenecks, vw_dash_scope_creep_value,
    vw_dash_billable_ratio_daily, vw_dash_time_by_activity, vw_dash_accounts_receivable,
    vw_dash_monthly_revenue;
drop table notifications, user_ui_state, conversation_history, change_requests, attachments,
    task_tags, time_entries, activities, tasks, tags, payments, invoices, milestones,
    project_billing, projects, client_auth, clients, categories, seq_by_user, users;
drop function fn_seed_defaults(), fn_release_voided_time_entries(), fn_prevent_payment_tampering(),
    fn_check_daily_hours_limit(), fn_increment_cr_version(), trg_set_seq_number(),
    next_seq_for_user(uuid, varchar), fn_set_updated_at();
-- Solo la función de compatibilidad creada en este esquema; nunca la nativa de PostgreSQL 18.
DO $compat$
BEGIN
    IF to_regprocedure(format('%I.uuidv7()', current_schema())) IS NOT NULL THEN
        EXECUTE format('DROP FUNCTION %I.uuidv7()', current_schema());
    END IF;
END;
$compat$;
