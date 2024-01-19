DROP TRIGGER update_updated_at on rsvp.reservations;
DROP FUNCTION rsvp.update_updated_at_column;

DROP TABLE rsvp.reservations CASCADE;
DROP TYPE rsvp.reservations_status;
DROP TYPE rsvp.reservations_update_type;
