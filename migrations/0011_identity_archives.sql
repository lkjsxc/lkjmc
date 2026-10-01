-- Native archive hashes and owner remapping remain part of the same official backup.
CREATE TABLE identity_archives (
    link_id uuid PRIMARY KEY REFERENCES link_requests,
    selected_profile uuid NOT NULL REFERENCES profiles,
    archived_profile uuid NOT NULL REFERENCES profiles,
    archive_owner uuid NOT NULL UNIQUE REFERENCES principals,
    native_plan jsonb NOT NULL,
    manifest jsonb NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
