-- reservation change queue
CREATE TABLE rsvp.reservation_change (
  id SERIAL NOT NULL,
  reservation_id varchar(64) NOT NULL,
  op rsvp.reservation_update_type NOT NULL
);

-- trigger for add/update/delete reservation
CREATE OR REPLACE FUNCTION rsvp.reservation_trigger() RETURNS TRIGGER AS $$
BEGIN
  IF (TG_OP = 'INSERT') THEN
    INSERT INTO rsvp.reservation_change (reservation_id, op) VALUES (NEW.id, 'create');
  ELSIF (TG_OP = 'UPDATE') THEN
    IF (OLD.status <> NEW.status) THEN
      INSERT INTO rsvp.reservation_change (reservation_id, op) VALUES (NEW.id, 'update');
    END IF;
  ELSIF (TG_OP = 'DELETE') THEN
    INSERT INTO rsvp.reservation_change (reservation_id, op) VALUES (OLD.id, 'delete');
  END IF;
  -- notify a channel called reservation_update
  NOTIFY reservation_update;
  RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER reservation_trigger AFTER INSERT OR UPDATE OR DELETE ON rsvp.reservation
FOR EACH ROW EXECUTE PROCEDURE rsvp.reservation_trigger();
