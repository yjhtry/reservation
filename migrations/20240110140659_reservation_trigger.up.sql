-- reservation change queue
CREATE TABLE rsvp.reservation_changes (
  id SERIAL NOT NULL,
  reservation_id BIGSERIAL NOT NULL,
  old JSONB,
  new JSONB,
  op rsvp.reservation_update_type NOT NULL,

  CONSTRAINT reservation_changes_pk PRIMARY KEY (id)
);

CREATE INDEX reservation_changes_reservation_id_op_idx ON rsvp.reservation_changes (reservation_id, op);

-- server read cursor
CREATE TABLE rsvp.server_read_cursor (
  server_id VARCHAR(64) NOT NULL,
  last_change_id BIGSERIAL NOT NULL,
  CONSTRAINT server_read_cursor_pk PRIMARY KEY (server_id)
);

-- trigger for add/update/delete reservation
CREATE OR REPLACE FUNCTION rsvp.reservations_trigger() RETURNS TRIGGER AS $$
BEGIN
  IF (TG_OP = 'INSERT') THEN
    INSERT INTO rsvp.reservation_changes (reservation_id, old, new, op) VALUES (NEW.id, NULL, row_to_json(NEW), 'create');
  ELSIF (TG_OP = 'UPDATE') THEN
    IF (OLD.status <> NEW.status) THEN
      INSERT INTO rsvp.reservation_changes (reservation_id, old, new, op) VALUES (NEW.id, row_to_json(OLD), row_to_json(NEW), 'update');
    END IF;
  ELSIF (TG_OP = 'DELETE') THEN
    INSERT INTO rsvp.reservation_changes (reservation_id, old, new, op) VALUES (OLD.id, row_to_json(OLD), NULL, 'delete');
  END IF;
  -- notify a channel called reservation_update
  NOTIFY reservation_update;
  RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER reservation_trigger AFTER INSERT OR UPDATE OR DELETE ON rsvp.reservations
FOR EACH ROW EXECUTE PROCEDURE rsvp.reservations_trigger();
