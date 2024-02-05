CREATE OR REPLACE FUNCTION rsvp.query(
    uid varchar(64),
    rid varchar(64),
    during TSTZRANGE,
    page integer DEFAULT 1,
    is_desc boolean DEFAULT false,
    page_size integer DEFAULT 10
) RETURNS TABLE (LIKE rsvp.reservations)
AS $$
DECLARE
    _sql TEXT;
BEGIN
    -- format the sql query based on the parameters
    _sql := format(
        'SELECT * FROM rsvp.reservations WHERE %L @> timespan AND %s
         ORDER BY lower(timespan) %s LIMIT %s OFFSET %s',
         during,
        CASE
            WHEN rid IS NOT NULL AND uid IS NOT NULL THEN
                'user_id = ' || quote_literal(rid) || 'AND resource_id =' || quote_literal(uid)
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
    -- execute the query
    RETURN QUERY EXECUTE _sql;
END;
$$ LANGUAGE plpgsql;
