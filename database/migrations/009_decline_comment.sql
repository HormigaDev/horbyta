-- Motivo "otro" con explicación del cliente, y corrección del valor
-- 'will_solve_internaly': el backend siempre envió 'will_solve_internally'.
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
        'will_solve_internally',
        'thought_it_was_included',
        'other'
    )
);
alter table change_requests add column decline_comment text
    check (decline_comment is null or length(decline_comment) <= 2000);
alter table change_requests add constraint chk_change_requests_other_reason check (
    decline_reason is distinct from 'other' or nullif(trim(decline_comment), '') is not null
);
