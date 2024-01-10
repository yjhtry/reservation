CREATE OR REPLACE FUNCTION rsvp.query(user_id varchar(64), resource_id varchar(64), during tstzrange) RETURNS TABLE (LIKE rsvp.reservation)
AS $$
BEGIN
  -- if user_id is null, then return all reservations within the during
  IF user_id IS NULL THEN
      RETURN QUERY
      SELECT * FROM rsvp.reservation WHERE rsvp.resource_id = resource_id AND during @> rsvp.reservation.timespan;
  -- if resource_id is null, then return all reservations within the during
  ELSIF resource_id IS NULL THEN
      RETURN QUERY
      SELECT * FROM rsvp.reservation WHERE rsvp.user_id = user_id AND during @> rsvp.reservation.timespan;
  -- if both are null, then return all reservations within the during
  ELSIF user_id IS NULL AND resource_id IS NULL THEN
      RETURN QUERY
      SELECT * FROM rsvp.reservation WHERE during @> rsvp.reservation.timespan;
  -- if both set, then return all reservations within the during for the resource and user
  ELSE
      RETURN QUERY
      SELECT * FROM rsvp.reservation
        WHERE rsvp.reservation.user_id = user_id
            AND rsvp.reservation.resource_id = resource_id
            AND during @> rsvp.reservation.timespan;
  END IF;
END;
$$ LANGUAGE plpgsql;
