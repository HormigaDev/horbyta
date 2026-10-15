-- Revierte la notificación de entregas; el envío vuelve a depender del sondeo.
drop trigger scope_delivery_signal on scope_delivery;
drop function scope_delivery_signal();
