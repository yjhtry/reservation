CREATE OR REPLACE FUNCTION rsvp.query(
    uid varchar(64),
    rid varchar(64),
    during TSTZRANGE,
    status rsvp.reservation_status DEFAULT 'pending',
    is_desc boolean DEFAULT false,
    page integer DEFAULT 1,
    page_size integer DEFAULT 10
) RETURNS TABLE (LIKE rsvp.reservations)
AS $$
DECLARE
    _sql TEXT;
BEGIN
    -- format the sql query based on the parameters
    _sql := format(
        'SELECT * FROM rsvp.reservations WHERE %L @> timespan AND status = %L::rsvp.reservation_status AND %s
         ORDER BY lower(timespan) %s LIMIT %L::integer OFFSET %L::integer',
         during,
         status,
        CASE
            WHEN rid IS NOT NULL AND uid IS NOT NULL THEN
                'user_id = ' || quote_literal(uid) || 'AND resource_id =' || quote_literal(rid)
            WHEN uid IS NOT NULL THEN
                'user_id = ' || quote_literal(uid)
            WHEN rid IS NOT NULL THEN
                'resource_id = ' || quote_literal(rid)
            ELSE
                'TRUE'
        END,
        CASE
            WHEN is_desc THEN 'DESC'
            ELSE 'ASC'
        END,
        page_size,
        (page - 1) * page_size
    );

    -- log the query
    RAISE NOTICE 'Executing query: %', _sql;
    -- execute the query
    RETURN QUERY EXECUTE _sql;
END;
$$ LANGUAGE plpgsql;
