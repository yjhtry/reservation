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
    -- if page is less than 1, set it to 1
    IF page < 1 THEN
        page := 1;
    END IF;

    -- if page_size is less than 1 or more than 10000, set it to 1
    IF page_size < 1 OR page_size > 10000 THEN
        page_size := 1;
    END IF;

    -- format the sql query based on the parameters
    _sql := format(
        'SELECT * FROM rsvp.reservations WHERE %L @> timespan AND status = %L::rsvp.reservation_status AND %s
         ORDER BY lower(timespan) %s LIMIT %L::integer OFFSET %L::integer',
         during,
         status,
        CASE
            WHEN rid IS NOT NULL AND uid IS NOT NULL THEN
                'user_id = ' || quote_literal(uid) || ' AND resource_id =' || quote_literal(rid)
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


CREATE OR REPLACE FUNCTION rsvp.filter(
    uid varchar(64),
    rid varchar(64),
    status rsvp.reservation_status DEFAULT 'pending',
    cursor bigint DEFAULT NULL,
    is_desc boolean DEFAULT false,
    page_size integer DEFAULT 10
) RETURNS TABLE (LIKE rsvp.reservations)
AS $$
DECLARE
    _sql TEXT;
BEGIN
    -- if cursor is less is null when is_desc is true, set it to int64 max or 0
    IF cursor IS NULL THEN
        IF is_desc THEN
            cursor := 9223372036854775807;
        ELSE
            cursor := 0;
        END IF;
    END IF;

    -- if page_size is less than 1 or more than 10000, set it to 1
    IF page_size < 1 OR page_size > 10000 THEN
        page_size := 1;
    END IF;

    -- format the sql query based on the parameters
    _sql := format(
        'SELECT * FROM rsvp.reservations WHERE %s AND status = %L::rsvp.reservation_status AND %s
         ORDER BY id %s LIMIT %L::integer',
         CASE
            WHEN is_desc THEN 'id < ' || cursor
            ELSE 'id > ' || cursor
        END,
         status,
        CASE
            WHEN rid IS NOT NULL AND uid IS NOT NULL THEN
                'user_id = ' || quote_literal(uid) || ' AND resource_id =' || quote_literal(rid)
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
        page_size
    );

    -- log the query
    RAISE NOTICE 'Executing query: %', _sql;
    -- execute the query
    RETURN QUERY EXECUTE _sql;
END;
$$ LANGUAGE plpgsql;
