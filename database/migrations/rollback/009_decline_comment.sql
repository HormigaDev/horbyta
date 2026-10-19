-- Revierte el motivo "otro". Se rechaza si ya hay rechazos que el esquema anterior no admite.
do $$ begin
    if exists(select 1 from change_requests where decline_reason in ('other', 'will_solve_internally')
              or decline_comment is not null) then
        raise exception 'Hay rechazos con motivo "otro", explicación o "will_solve_internally"; revertir la migración 9 los invalidaría.';
    end if;
end; $$;
alter table change_requests drop constraint chk_change_requests_other_reason;
alter table change_requests drop column decline_comment;
alter table change_requests drop constraint chk_change_requests_decline_reason;
alter table change_requests add constraint chk_change_requests_decline_reason check (
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
    );
