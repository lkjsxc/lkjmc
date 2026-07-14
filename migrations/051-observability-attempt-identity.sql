alter table observability_operations
    drop constraint observability_operations_request_id_key;

alter table observability_operations
    alter column request_id drop not null;

alter table observability_events
    drop constraint observability_events_attributes_check,
    add constraint observability_events_attributes_check check (
        jsonb_typeof(attributes) = 'object'
        and attributes - array['command','serverId','route','runtime','fault','queue','reason','migration','retention','bundle','transport','source','redacted'] = '{}'::jsonb
        and pg_column_size(attributes) <= 4096
    );
