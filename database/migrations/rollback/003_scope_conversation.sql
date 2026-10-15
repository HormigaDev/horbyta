-- Revierte los adjuntos de la conversación de alcance. Elimina esos vínculos.
drop table scope_project_event_attachments;
drop function scope_event_attachment_guard();
drop index scope_project_events_window_idx;
