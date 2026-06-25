create table if not exists player_mail_messages (
    id uuid primary key,
    recipient_uuid uuid not null references player_identities(player_uuid),
    sender_uuid uuid references player_identities(player_uuid),
    sender_name text not null,
    body text not null,
    read_at timestamptz,
    created_at timestamptz not null default now()
);

create index if not exists player_mail_messages_recipient_idx
    on player_mail_messages (recipient_uuid, created_at desc);
