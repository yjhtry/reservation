CREATE OR REPLACE FUNCTION rsvp.query(user_id varchar(64), resource_id varchar(64), during tstzrange) RETURNS TABLE (LIKE rsvp.reservations)
AS $$
BEGIN
  -- if user_id is null, then return all reservations within the during
  IF user_id IS NULL THEN
      RETURN QUERY
      SELECT * FROM rsvp.reservations WHERE rsvp.resource_id = resource_id AND during @> rsvp.reservations.timespan;
  -- if resource_id is null, then return all reservations within the during
  ELSIF resource_id IS NULL THEN
      RETURN QUERY
      SELECT * FROM rsvp.reservations WHERE rsvp.user_id = user_id AND during @> rsvp.reservations.timespan;
  -- if both are null, then return all reservations within the during
  ELSIF user_id IS NULL AND resource_id IS NULL THEN
      RETURN QUERY
      SELECT * FROM rsvp.reservations WHERE during @> rsvp.reservations.timespan;
  -- if both set, then return all reservations within the during for the resource and user
  ELSE
      RETURN QUERY
      SELECT * FROM rsvp.reservations
        WHERE rsvp.reservations.user_id = user_id
            AND rsvp.reservations.resource_id = resource_id
            AND during @> rsvp.reservations.timespan;
  END IF;
END;
$$ LANGUAGE plpgsql;
