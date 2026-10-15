-- Revierte la conversación unificada. Elimina mensajes, lecturas y orígenes de cambios.
drop view project_conversation_timeline;
drop table conversation_change_sources, project_conversation_reads,
    project_message_attachments, project_messages;
