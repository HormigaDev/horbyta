-- Revierte los adjuntos de la conversación de solicitudes. Elimina esos vínculos.
drop table scope_change_event_attachments;
drop function scope_change_attachment_guard();
drop index scope_change_messages_window_idx;
