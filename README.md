# Reservation

学习Rust构建预订服务系统


## Database schema
Use Postgresql as the database, below is the schema:
```sql
CREATE SCHEMA rsvp;
CREATE TYPE rsvp.reservation_status AS ENUM('unknown', 'pending', 'confirmed', 'blocked');
CREATE TYPE rsvp.reservation_update_type AS ENUM('unknown', 'create', 'update', 'delete');

CREATE TABLE rsvp.reservations (
  id uuid NOT NULL DEFAULT gen_random_uuid(),
  user_id VARCHAR(64) NOT NULL,
  status rsvp.reservation_status NOT NULL DEFAULT 'pending',
  resource_id VARCHAR(64) NOT NULL,
  timespan TSTZRANGE NOT NULL,
  note TEXT,

  CONSTRAINT reservations_pkey PRIMARY KEY (id),
  -- 当资源id相同且预订时间块有重叠，则判断为冲突，拒绝插入数据库
  CONSTRAINT reservations_conflict EXCLUDE USING gist (resource_id WITH =, timespan WITH &&)
);

CREATE INDEX reservations_resource_id_idx ON rsvp.reservations (resource_id);
CREATE INDEX reservation_user_id_idx ON rsvp.reservations (user_id);

CREATE OR REPLACE FUNCTION rsvp.query(uid text, rid text, during TSTZRANGE) RETURNS TABLE rsvp.reservations AS $$ $$ LANGUAGE plpgsql;

CREATE TABLE rsvp.reservation_changes(
  id SERIAL NOT NULL,
  reservation_id uuid NOT NULL,
  op rsvp.reservation_update_type NOT NULL
);

-- 当对预订进行增删改操作时，同步记录相关信息便于之后用于监听等操作
CREATE OR REPLACE FUNCTION rsvp.reservations_trigger() RETURNS TRIGGER AS $$
BEGIN
  IF TG_OP = 'INSERT' THEN
    INSERT INTO rsvp.reservation_changes (reservation_id, op) VALUES (NEW.id, 'create');
  ELSIF TG_OP = 'UPDATE' THEN
    IF OLD.status<>NEW.status THEN
      INSERT INTO rsvp.reservation_changes (reservation_id, op) VALUES (NEW.id, 'update');
    END IF;
  ELSIF TG_OP = 'DELETE' THEN
    INSERT INTO rsvp.reservation_changes (reservation_id, op) VALUES (OLD.id, 'delete');
  END IF;
  NOTIFY reservation_update;
  RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER reservations_trigger AFTER INSERT OR UPDATE OR DELETE ON rsvp.reservations FOR EACH ROW EXECUTE PROCEDURE rsvp.reservations_trigger();

```
