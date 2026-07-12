DROP SCHEMA IF EXISTS lab CASCADE;
CREATE SCHEMA lab;

CREATE TABLE lab.event_log (
    stream text NOT NULL,
    sequence integer NOT NULL,
    event_id text NOT NULL UNIQUE,
    payload jsonb NOT NULL,
    PRIMARY KEY (stream, sequence)
);
CREATE TABLE lab.projection (
    stream text PRIMARY KEY,
    state text NOT NULL,
    revision integer NOT NULL
);

CREATE FUNCTION lab.append_event(p_event text, p_stream text, p_payload jsonb)
RETURNS boolean LANGUAGE plpgsql AS $$
DECLARE
    next_sequence integer;
BEGIN
    SELECT coalesce(max(sequence), 0) + 1 INTO next_sequence
    FROM lab.event_log WHERE stream = p_stream;
    INSERT INTO lab.event_log (stream, sequence, event_id, payload)
    VALUES (p_stream, next_sequence, p_event, p_payload)
    ON CONFLICT (event_id) DO NOTHING;
    IF NOT FOUND THEN
        RETURN false;
    END IF;
    RETURN true;
END;
$$;

CREATE FUNCTION lab.rebuild_projection() RETURNS void LANGUAGE sql AS $$
    TRUNCATE lab.projection;
    INSERT INTO lab.projection (stream, state, revision)
    SELECT stream,
           (array_agg(payload ->> 'state' ORDER BY sequence DESC))[1],
           max(sequence)
    FROM lab.event_log GROUP BY stream;
$$;

CREATE TABLE lab.leases (
    instance_id text PRIMARY KEY,
    owner text NOT NULL,
    generation integer NOT NULL,
    expires_at timestamptz NOT NULL
);
CREATE TABLE lab.fenced_instance (
    instance_id text PRIMARY KEY,
    generation integer NOT NULL,
    state text NOT NULL
);
INSERT INTO lab.fenced_instance VALUES ('instance-fence', 0, 'empty');

CREATE FUNCTION lab.acquire_lease(p_owner text) RETURNS integer LANGUAGE plpgsql AS $$
DECLARE
    acquired integer;
BEGIN
    INSERT INTO lab.leases (instance_id, owner, generation, expires_at)
    VALUES ('instance-fence', p_owner, 1, clock_timestamp() + interval '10 seconds')
    ON CONFLICT (instance_id) DO UPDATE
    SET owner = excluded.owner,
        generation = lab.leases.generation + 1,
        expires_at = clock_timestamp() + interval '10 seconds'
    WHERE lab.leases.expires_at < clock_timestamp()
    RETURNING generation INTO acquired;
    RETURN acquired;
END;
$$;

CREATE FUNCTION lab.fenced_write(p_owner text, p_generation integer, p_state text)
RETURNS boolean LANGUAGE plpgsql AS $$
DECLARE
    accepted boolean;
BEGIN
    UPDATE lab.fenced_instance AS instance
    SET generation = p_generation, state = p_state
    FROM lab.leases AS lease
    WHERE instance.instance_id = lease.instance_id
      AND lease.owner = p_owner
      AND lease.generation = p_generation
      AND lease.expires_at > clock_timestamp()
    RETURNING true INTO accepted;
    RETURN coalesce(accepted, false);
END;
$$;

CREATE TABLE lab.outbox (
    event_id text PRIMARY KEY,
    channel text NOT NULL,
    payload jsonb NOT NULL,
    simple_claimed boolean NOT NULL DEFAULT false,
    broker_published boolean NOT NULL DEFAULT false
);
