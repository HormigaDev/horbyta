-- Despierta la entrega solo al confirmar trabajo nuevo o adelantar un reintento.
-- El canal distingue esquemas; no publica datos del cliente ni del mensaje.
create function scope_delivery_signal() returns trigger language plpgsql as $$
begin
    if new.delivered_at is null then
        if tg_op='INSERT' then
            perform pg_notify('horbyta_scope_delivery_'||md5(tg_table_schema),'');
        elsif new.next_attempt_at < old.next_attempt_at then
            perform pg_notify('horbyta_scope_delivery_'||md5(tg_table_schema),'');
        end if;
    end if;
    return null;
end;
$$;
create trigger scope_delivery_signal after insert or update of next_attempt_at on scope_delivery
for each row execute function scope_delivery_signal();
