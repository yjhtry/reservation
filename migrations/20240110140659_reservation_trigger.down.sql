DROP TABLE rsvp.reservation_changes  CASCADE;
DROP TABLE rsvp.server_read_cursor  CASCADE;
DROP TRIGGER reservation_trigger on rsvp.reservations;
DROP FUNCTION rsvp.reservations_trigger;
