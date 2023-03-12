CREATE OR REPLACE FUNCTION rsvp.query(
  uid text,
  rid text,
  during TSTZRANGE,
  status rsvp.reservation_status,
  page integer DEFAULT 1,
  page_size integer DEFAULT 10,
  is_desc bool DEFAULT FALSE
  ) RETURNS TABLE (LIKE rsvp.reservations) AS $$
DECLARE
  _sql text;
BEGIN
  _sql := format(
    'SELECT * FROM rsvp.reservations WHERE %L @> timespan AND status = %L AND %s ORDER BY lower(timespan) %s LIMIT %L::integer OFFSET %L::integer',
    during,
    status,
    CASE
      WHEN uid IS NULL AND rid IS NULL THEN 'TRUE'
      WHEN uid IS NULL THEN 'resource_id=' || quote_literal(rid)
      WHEN rid IS NULL THEN 'user_id=' || quote_literal(uid)
      ELSE 'user_id=' || quote_literal(uid) || ' AND resource_id=' || quote_literal(rid)
    END,
    CASE
      WHEN is_desc THEN 'DESC'
      ELSE 'ASC'
    END,
    page_size,
    (page - 1) * page_size
  );
  -- log the sql
  RAISE NOTICE '%', _sql;

  RETURN QUERY EXECUTE _sql;
END;
$$ LANGUAGE plpgsql;
