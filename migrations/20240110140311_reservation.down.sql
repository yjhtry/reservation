DROP TRIGGER update_updated_at on rsvp.reservations;
DROP FUNCTION rsvp.update_updated_at_column;

DROP TABLE rsvp.reservations CASCADE;
DROP TYPE rsvp.reservation_status;
DROP TYPE rsvp.reservation_update_type;
