DROP TABLE rsvp.reservation_changes  CASCADE;
DROP TRIGGER reservation_trigger on rsvp.reservations;
DROP FUNCTION rsvp.reservations_trigger;
