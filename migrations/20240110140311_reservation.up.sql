CREATE TYPE rsvp.reservation_status AS ENUM (
  'unknown',
  'pending',
  'confirmed',
  'blocked'
);

CREATE TYPE rsvp.reservation_update_type AS ENUM (
  'unknown',
  'create',
  'update',
  'delete'
);

CREATE TABLE rsvp.reservations (
  id UUID NOT NULL DEFAULT gen_random_uuid(),
  resource_id varchar(64 ) NOT NULL,
  user_id varchar(64) NOT NULL,
  status rsvp.reservation_status NOT NULL DEFAULT 'pending',
  timespan tstzrange NOT NULL,
  note text,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ,

  CONSTRAINT reservation_pkey PRIMARY KEY (id),
  CONSTRAINT reservations_conflict EXCLUDE USING gist (resource_id WITH =, timespan WITH &&)
);

CREATE INDEX reservation_resource_id_idx ON rsvp.reservations (resource_id);
CREATE INDEX reservation_user_id_idx ON rsvp.reservations (user_id);

CREATE OR REPLACE FUNCTION rsvp.update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE 'plpgsql';

CREATE TRIGGER update_updated_at BEFORE UPDATE ON rsvp.reservations
FOR EACH ROW EXECUTE PROCEDURE rsvp.update_updated_at_column();
