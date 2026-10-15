-- Compatibilidad de la instalación base: PostgreSQL 18 usa su función nativa.
DO $compat$
BEGIN
    IF to_regprocedure('uuidv7()') IS NULL AND to_regprocedure('pg_catalog.uuidv7(interval)') IS NULL THEN
        EXECUTE $definition$
            CREATE FUNCTION uuidv7() RETURNS uuid LANGUAGE sql VOLATILE AS $uuid$
                WITH r AS (SELECT replace(gen_random_uuid()::text, '-', '') AS v)
                SELECT (lpad(to_hex((extract(epoch FROM clock_timestamp())*1000)::bigint),12,'0')
                    || '7' || substr(v,14,3) || substr(v,17,16))::uuid FROM r
            $uuid$
        $definition$;
    END IF;
END;
$compat$;
