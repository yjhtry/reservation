DROP TRIGGER update_updated_at on rsvp.reservation;
DROP FUNCTION rsvp.update_updated_at_column;

DROP TABLE rsvp.reservation CASCADE;
DROP TYPE rsvp.reservation_status;
DROP TYPE rsvp.reservation_update_type;
