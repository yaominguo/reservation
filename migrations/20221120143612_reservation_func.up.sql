CREATE OR REPLACE FUNCTION rsvp.query(uid text, rid text, during TSTZRANGE) RETURNS TABLE (LIKE rsvp.reservations)
AS $$
BEGIN
  IF uid IS NULL AND rid IS NULL THEN
    RETURN QUERY SELECT * FRom rsvp.reservations WHERE during @> timespan;
  ELSIF uid IS NULL THEN
    RETURN QUERY SELECT * FROM rsvp.reservations WHERE resource_id = rid AND during @> timespan;
  ELSIF ris IS NULL THEN
    RETURN QUERY SELECT * FROM rsvp.reservations WHERE user_id = uid AND during @> timespan;
  ELSE
    RETURN QUERY SELECT * FROM rsvp.reservations WHERE resource_id = rid AND user_id = uid AND during @> timespan;
  END IF;
END;
$$ LANGUAGE plpgsql;
