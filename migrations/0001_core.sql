CREATE EXTENSION IF NOT EXISTS btree_gist;

CREATE TABLE principals (
    id uuid PRIMARY KEY,
    kind text NOT NULL CHECK (kind IN ('account','team','system')),
    name text NOT NULL CHECK (length(name) BETWEEN 1 AND 64),
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE trust_ranks (
    id smallint PRIMARY KEY,
    name text NOT NULL,
    server_count integer NOT NULL CHECK (server_count >= 0),
    concurrent_servers integer NOT NULL CHECK (concurrent_servers >= 0),
    memory_mib integer NOT NULL CHECK (memory_mib >= 0),
    cpu_millis integer NOT NULL CHECK (cpu_millis >= 0),
    storage_mib bigint NOT NULL CHECK (storage_mib >= 0)
);
INSERT INTO trust_ranks VALUES (0,'プレイヤー',0,0,0,0,0);
CREATE TABLE accounts (
    id uuid PRIMARY KEY REFERENCES principals,
    trust_rank smallint NOT NULL DEFAULT 0 REFERENCES trust_ranks,
    administrator boolean NOT NULL DEFAULT false,
    banned_until timestamptz,
    dm_policy text NOT NULL DEFAULT 'friends' CHECK (dm_policy IN ('friends','everyone','none')),
    activity_policy text NOT NULL DEFAULT 'friends' CHECK (activity_policy IN ('friends','everyone','none')),
    merged_into uuid REFERENCES accounts,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE profiles (
    id uuid PRIMARY KEY,
    account_id uuid NOT NULL REFERENCES accounts,
    native_uuid uuid UNIQUE,
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active','archived','moving')),
    archive_reason text,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX one_active_profile ON profiles(account_id) WHERE status IN ('active','moving');
CREATE TABLE identities (
    issuer text NOT NULL,
    subject text NOT NULL,
    account_id uuid NOT NULL REFERENCES accounts,
    display_name text NOT NULL,
    verified_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY(issuer,subject)
);
CREATE TABLE sessions (
    token_hash text PRIMARY KEY,
    account_id uuid NOT NULL REFERENCES accounts,
    csrf text NOT NULL,
    expires_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX ON sessions(expires_at);
CREATE TABLE oidc_flows (
    state_hash text PRIMARY KEY,
    browser_hash text NOT NULL,
    nonce text NOT NULL,
    verifier text NOT NULL,
    expires_at timestamptz NOT NULL
);
CREATE TABLE link_requests (
    id uuid PRIMARY KEY,
    code_hash text NOT NULL UNIQUE,
    initiator uuid NOT NULL REFERENCES accounts,
    candidate uuid REFERENCES accounts,
    selected_profile uuid REFERENCES profiles,
    state text NOT NULL DEFAULT 'pending' CHECK(state IN ('pending','confirmed','migrating','complete','cancelled')),
    expires_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE idempotency (
    actor uuid NOT NULL REFERENCES accounts,
    key uuid NOT NULL,
    request_hash text NOT NULL,
    response jsonb NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY(actor,key)
);

CREATE TABLE blocks (
    actor uuid NOT NULL REFERENCES accounts,
    target uuid NOT NULL REFERENCES accounts,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY(actor,target), CHECK(actor <> target)
);
CREATE TABLE friendships (
    first_id uuid NOT NULL REFERENCES accounts,
    second_id uuid NOT NULL REFERENCES accounts,
    requester uuid NOT NULL REFERENCES accounts,
    state text NOT NULL CHECK(state IN ('pending','accepted')),
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY(first_id,second_id), CHECK(first_id < second_id),
    CHECK(requester IN (first_id,second_id))
);
CREATE TABLE rooms (
    id uuid PRIMARY KEY,
    kind text NOT NULL CHECK(kind IN ('dm','group','team','party','server','network')),
    name text NOT NULL CHECK(length(name) BETWEEN 1 AND 80),
    owner uuid REFERENCES accounts,
    archived_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE room_members (
    room_id uuid NOT NULL REFERENCES rooms,
    account_id uuid NOT NULL REFERENCES accounts,
    role text NOT NULL DEFAULT 'member' CHECK(role IN ('owner','moderator','member')),
    joined_at timestamptz NOT NULL DEFAULT now(),
    read_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY(room_id,account_id)
);
CREATE TABLE direct_rooms (
    first_id uuid NOT NULL REFERENCES accounts,
    second_id uuid NOT NULL REFERENCES accounts,
    room_id uuid NOT NULL UNIQUE REFERENCES rooms,
    PRIMARY KEY(first_id,second_id), CHECK(first_id < second_id)
);
CREATE TABLE messages (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    room_id uuid NOT NULL REFERENCES rooms,
    author uuid NOT NULL REFERENCES accounts,
    body text NOT NULL CHECK(length(body) <= 4000),
    deleted_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX messages_room_page ON messages(room_id,id DESC);
CREATE INDEX messages_search ON messages USING gin(to_tsvector('simple',body)) WHERE deleted_at IS NULL;
CREATE TABLE teams (
    id uuid PRIMARY KEY REFERENCES principals,
    leader uuid NOT NULL REFERENCES accounts,
    room_id uuid NOT NULL UNIQUE REFERENCES rooms,
    disbanded_at timestamptz
);
CREATE TABLE team_members (
    team_id uuid NOT NULL REFERENCES teams,
    account_id uuid NOT NULL UNIQUE REFERENCES accounts,
    can_build boolean NOT NULL DEFAULT true,
    can_sell boolean NOT NULL DEFAULT false,
    can_spend boolean NOT NULL DEFAULT false,
    can_manage_members boolean NOT NULL DEFAULT false,
    can_administer boolean NOT NULL DEFAULT false,
    PRIMARY KEY(team_id,account_id)
);
CREATE TABLE parties (
    id uuid PRIMARY KEY,
    leader uuid NOT NULL REFERENCES accounts,
    room_id uuid NOT NULL UNIQUE REFERENCES rooms,
    closed_at timestamptz
);
CREATE TABLE party_members (
    party_id uuid NOT NULL REFERENCES parties,
    account_id uuid NOT NULL UNIQUE REFERENCES accounts,
    ready boolean NOT NULL DEFAULT false,
    PRIMARY KEY(party_id,account_id)
);
CREATE TABLE communities (
    id uuid PRIMARY KEY,
    name text NOT NULL CHECK(length(name) BETWEEN 1 AND 64),
    owner uuid NOT NULL REFERENCES accounts
);
CREATE TABLE community_members (
    community_id uuid NOT NULL REFERENCES communities,
    account_id uuid NOT NULL REFERENCES accounts,
    administrator boolean NOT NULL DEFAULT false,
    PRIMARY KEY(community_id,account_id)
);
CREATE TABLE invitations (
    id uuid PRIMARY KEY,
    sender uuid NOT NULL REFERENCES accounts,
    recipient uuid NOT NULL REFERENCES accounts,
    kind text NOT NULL CHECK(kind IN ('room','team','party','community','server','teleport')),
    resource_id uuid NOT NULL,
    state text NOT NULL DEFAULT 'pending' CHECK(state IN ('pending','accepted','declined','cancelled')),
    expires_at timestamptz NOT NULL DEFAULT now() + interval '7 days',
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK(sender<>recipient)
);
CREATE UNIQUE INDEX pending_invite ON invitations(recipient,kind,resource_id) WHERE state='pending';
CREATE TABLE notifications (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    account_id uuid NOT NULL REFERENCES accounts,
    kind text NOT NULL,
    body jsonb NOT NULL,
    read_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX ON notifications(account_id,id DESC);
CREATE TABLE reports (
    id uuid PRIMARY KEY,
    reporter uuid NOT NULL REFERENCES accounts,
    target uuid REFERENCES accounts,
    reason text NOT NULL CHECK(length(reason) BETWEEN 1 AND 4000),
    evidence jsonb NOT NULL,
    status text NOT NULL DEFAULT 'open' CHECK(status IN ('open','investigating','resolved','dismissed')),
    resolution text,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE audit (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    actor uuid REFERENCES accounts,
    service text,
    action text NOT NULL,
    resource text NOT NULL,
    detail jsonb NOT NULL DEFAULT '{}',
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK(actor IS NOT NULL OR service IS NOT NULL)
);

CREATE TABLE servers (
    id uuid PRIMARY KEY,
    owner uuid REFERENCES accounts,
    community_id uuid REFERENCES communities,
    name text NOT NULL CHECK(length(name) BETWEEN 1 AND 64),
    kind text NOT NULL CHECK(kind IN ('lobby','official','custom','legacy')),
    visibility text NOT NULL DEFAULT 'private' CHECK(visibility IN ('public','invite','private')),
    desired text NOT NULL DEFAULT 'stopped' CHECK(desired IN ('running','stopped')),
    observed text NOT NULL DEFAULT 'unprovisioned' CHECK(observed IN ('unprovisioned','provisioning','stopped','starting','running','stopping','unknown','error')),
    version text NOT NULL,
    software text NOT NULL,
    capabilities jsonb NOT NULL DEFAULT '{}',
    memory_mib integer NOT NULL CHECK(memory_mib >= 512),
    cpu_millis integer NOT NULL CHECK(cpu_millis >= 100),
    storage_mib bigint NOT NULL CHECK(storage_mib >= 1024),
    players integer NOT NULL DEFAULT 0 CHECK(players >= 0),
    address text,
    last_observed_at timestamptz,
    empty_since timestamptz,
    maintenance boolean NOT NULL DEFAULT false,
    error text,
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK(kind <> 'custom' OR owner IS NOT NULL)
);
CREATE UNIQUE INDEX one_lobby ON servers(kind) WHERE kind='lobby';
CREATE UNIQUE INDEX one_official ON servers(kind) WHERE kind='official';
CREATE TABLE server_members (
    server_id uuid NOT NULL REFERENCES servers,
    account_id uuid NOT NULL REFERENCES accounts,
    role text NOT NULL CHECK(role IN ('guest','operator','administrator')),
    PRIMARY KEY(server_id,account_id)
);
CREATE TABLE artifacts (
    id uuid PRIMARY KEY,
    server_id uuid REFERENCES servers,
    owner uuid NOT NULL REFERENCES accounts,
    sha256 text NOT NULL CHECK(length(sha256)=64),
    bytes bigint NOT NULL CHECK(bytes > 0),
    name text NOT NULL,
    kind text NOT NULL CHECK(kind IN ('jar','mod','plugin','world','file')),
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE service_credentials (
    id uuid PRIMARY KEY,
    name text NOT NULL,
    token_hash text NOT NULL UNIQUE,
    role text NOT NULL CHECK(role IN ('host','proxy','official','lobby')),
    server_id uuid REFERENCES servers,
    revoked_at timestamptz,
    last_seen_at timestamptz
);
CREATE TABLE jobs (
    id uuid PRIMARY KEY,
    actor uuid REFERENCES accounts,
    server_id uuid REFERENCES servers,
    worker text NOT NULL CHECK(worker IN ('host','proxy','official','lobby','core')),
    kind text NOT NULL,
    payload jsonb NOT NULL,
    state text NOT NULL DEFAULT 'queued' CHECK(state IN ('queued','leased','waiting','succeeded','failed','cancelled')),
    progress jsonb NOT NULL DEFAULT '{}',
    result jsonb,
    error text,
    attempts integer NOT NULL DEFAULT 0,
    lease_token uuid,
    lease_owner uuid REFERENCES service_credentials,
    lease_until timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX jobs_pending ON jobs(worker,server_id,created_at) WHERE state IN ('queued','leased','waiting');
CREATE UNIQUE INDEX server_transition ON jobs(server_id) WHERE kind IN ('server.create','server.start','server.stop','server.restore') AND state IN ('queued','leased','waiting');
CREATE TABLE game_sessions (
    account_id uuid PRIMARY KEY REFERENCES accounts,
    profile_id uuid NOT NULL REFERENCES profiles,
    native_uuid uuid NOT NULL,
    session_id uuid NOT NULL UNIQUE,
    server_id uuid REFERENCES servers,
    lease_until timestamptz NOT NULL,
    combat_until timestamptz,
    last_position jsonb,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE worlds (
    id uuid PRIMARY KEY,
    server_id uuid NOT NULL REFERENCES servers,
    name text NOT NULL,
    kind text NOT NULL CHECK(kind IN ('living','nether','end','private_end','holding','lobby')),
    native_uuid uuid,
    enabled boolean NOT NULL DEFAULT true,
    UNIQUE(server_id,name)
);
CREATE UNIQUE INDEX one_living ON worlds(kind) WHERE kind='living' AND enabled;
CREATE TABLE land_allowances (
    owner uuid PRIMARY KEY REFERENCES principals,
    chunks integer NOT NULL CHECK(chunks>=0)
);
CREATE TABLE claims (
    id uuid PRIMARY KEY,
    owner uuid NOT NULL REFERENCES principals,
    world_id uuid NOT NULL REFERENCES worlds,
    name text NOT NULL CHECK(length(name) BETWEEN 1 AND 64),
    min_x integer NOT NULL,
    min_z integer NOT NULL,
    max_x integer NOT NULL,
    max_z integer NOT NULL,
    chunks integer GENERATED ALWAYS AS ((max_x-min_x+1)*(max_z-min_z+1)) STORED,
    state text NOT NULL DEFAULT 'pending' CHECK(state IN ('pending','active','transferring','releasing','released')),
    job_id uuid REFERENCES jobs,
    CHECK(max_x>=min_x AND max_z>=min_z),
    CHECK(max_x-min_x<1024 AND max_z-min_z<1024),
    EXCLUDE USING gist (world_id WITH =, int4range(min_x,max_x,'[]') WITH &&, int4range(min_z,max_z,'[]') WITH &&) WHERE(state<>'released')
);
CREATE TABLE spawn_points (
    id uuid PRIMARY KEY,
    profile_id uuid NOT NULL REFERENCES profiles,
    world_id uuid NOT NULL REFERENCES worlds,
    x integer NOT NULL,
    z integer NOT NULL,
    y integer,
    state text NOT NULL CHECK(state IN ('reserved','ready','used','rejected')),
    reason text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX ON spawn_points(world_id,state);
CREATE UNIQUE INDEX pending_spawn_per_profile ON spawn_points(profile_id) WHERE state IN ('reserved','ready');
CREATE TABLE homes (
    id uuid PRIMARY KEY,
    profile_id uuid NOT NULL REFERENCES profiles,
    name text NOT NULL CHECK(length(name) BETWEEN 1 AND 32),
    location jsonb NOT NULL,
    UNIQUE(profile_id,name)
);
CREATE TABLE achievements (
    key text PRIMARY KEY,
    title text NOT NULL,
    description text NOT NULL,
    event text NOT NULL,
    target bigint NOT NULL CHECK(target>0),
    land_chunks integer NOT NULL DEFAULT 0 CHECK(land_chunks>=0),
    coins bigint NOT NULL DEFAULT 0 CHECK(coins>=0),
    team boolean NOT NULL DEFAULT false
);
CREATE TABLE achievement_progress (
    owner uuid NOT NULL REFERENCES principals,
    achievement text NOT NULL REFERENCES achievements,
    progress bigint NOT NULL DEFAULT 0 CHECK(progress>=0),
    earned_at timestamptz,
    PRIMARY KEY(owner,achievement)
);
CREATE TABLE game_events (
    id uuid PRIMARY KEY,
    credential uuid NOT NULL REFERENCES service_credentials,
    account_id uuid NOT NULL REFERENCES accounts,
    kind text NOT NULL,
    payload jsonb NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE wallets (
    owner uuid PRIMARY KEY REFERENCES principals,
    balance bigint NOT NULL DEFAULT 0 CHECK(balance>=0),
    reserved bigint NOT NULL DEFAULT 0 CHECK(reserved>=0 AND reserved<=balance)
);
CREATE TABLE ledger (
    id uuid PRIMARY KEY,
    reference text NOT NULL UNIQUE,
    kind text NOT NULL,
    actor uuid REFERENCES accounts,
    detail jsonb NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE ledger_entries (
    transaction_id uuid NOT NULL REFERENCES ledger,
    owner uuid NOT NULL REFERENCES principals,
    amount bigint NOT NULL CHECK(amount<>0),
    balance_after bigint NOT NULL CHECK(balance_after>=0),
    PRIMARY KEY(transaction_id,owner)
);
CREATE TABLE assets (
    id uuid PRIMARY KEY,
    owner uuid NOT NULL REFERENCES principals,
    kind text NOT NULL CHECK(kind IN ('items','building','land')),
    title text NOT NULL CHECK(length(title) BETWEEN 1 AND 100),
    state text NOT NULL CHECK(state IN ('capturing','escrowed','listed','placing','placed','delivered','quarantined')),
    manifest jsonb NOT NULL DEFAULT '{}',
    manifest_sha256 text,
    claim_id uuid REFERENCES claims,
    job_id uuid REFERENCES jobs,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE listings (
    id uuid PRIMARY KEY,
    asset_id uuid NOT NULL REFERENCES assets,
    seller uuid NOT NULL REFERENCES principals,
    price bigint NOT NULL CHECK(price BETWEEN 1 AND 1000000000000),
    state text NOT NULL DEFAULT 'active' CHECK(state IN ('active','sold','cancelled')),
    buyer uuid REFERENCES principals,
    created_at timestamptz NOT NULL DEFAULT now(),
    sold_at timestamptz
);
CREATE UNIQUE INDEX one_listing_per_asset ON listings(asset_id) WHERE state='active';
CREATE TABLE trades (
    id uuid PRIMARY KEY,
    listing_id uuid NOT NULL UNIQUE REFERENCES listings,
    buyer uuid NOT NULL REFERENCES principals,
    seller uuid NOT NULL REFERENCES principals,
    price bigint NOT NULL,
    fee bigint NOT NULL,
    ledger_id uuid NOT NULL REFERENCES ledger,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE asset_consents (
    asset_id uuid NOT NULL REFERENCES assets,
    owner uuid NOT NULL REFERENCES accounts,
    manifest_sha256 text NOT NULL,
    granted_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY(asset_id,owner)
);
CREATE TABLE npc_prices (
    material text PRIMARY KEY,
    price bigint NOT NULL CHECK(price>0),
    enabled boolean NOT NULL DEFAULT true
);
CREATE TABLE npc_daily (
    profile_id uuid NOT NULL REFERENCES profiles,
    day date NOT NULL,
    coins bigint NOT NULL DEFAULT 0 CHECK(coins BETWEEN 0 AND 2000),
    PRIMARY KEY(profile_id,day)
);
CREATE TABLE adventures (
    id uuid PRIMARY KEY,
    owner uuid NOT NULL REFERENCES accounts,
    party_id uuid REFERENCES parties,
    world_id uuid REFERENCES worlds,
    state text NOT NULL CHECK(state IN ('preparing','activating','active','closing','closed','refunding','refunded')),
    coin_cost bigint NOT NULL DEFAULT 1000 CHECK(coin_cost=1000),
    material_asset uuid REFERENCES assets,
    opens_at timestamptz,
    expires_at timestamptz,
    job_id uuid REFERENCES jobs,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX active_adventure_owner ON adventures(owner) WHERE state NOT IN ('closed','refunded');
CREATE UNIQUE INDEX active_adventure_party ON adventures(party_id) WHERE state NOT IN ('closed','refunded');
CREATE TABLE backups (
    id uuid PRIMARY KEY,
    server_id uuid REFERENCES servers,
    kind text NOT NULL CHECK(kind IN ('official','server')),
    state text NOT NULL CHECK(state IN ('queued','freezing','saving','verifying','ready','failed','restoring')),
    manifest jsonb,
    error text,
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE settings (key text PRIMARY KEY, value jsonb NOT NULL);
INSERT INTO principals(id,kind,name) VALUES ('00000000-0000-0000-0000-000000000001','system','取引手数料');
INSERT INTO wallets(owner) VALUES ('00000000-0000-0000-0000-000000000001');
INSERT INTO achievements VALUES
('first_claim','暮らしのはじまり','最初の土地を保護する','claim.created',1,0,100,false),
('builder_256','小さな家から','自分でブロックを256個置く','block.placed',256,2,200,false),
('builder_2048','まちづくり','自分でブロックを2,048個置く','block.placed',2048,4,500,false),
('explorer_10000','遠くの景色','徒歩で10,000ブロック進む','walk.distance',10000,2,300,false),
('farmer_512','畑と暮らす','作物を512個収穫する','crop.harvest',512,2,300,false),
('team_builder_4096','みんなの拠点','チームでブロックを4,096個置く','block.placed',4096,16,1000,true),
('team_adventure_3','冒険仲間','チームで一時 End の目標を3回達成する','adventure.completed',3,8,1000,true);
INSERT INTO npc_prices VALUES
('COBBLESTONE',1,true),('DEEPSLATE',1,true),('IRON_INGOT',8,true),('GOLD_INGOT',12,true),
('DIAMOND',64,true),('WHEAT',2,true),('CARROT',1,true),('POTATO',1,true),('PUMPKIN',3,true),('MELON_SLICE',1,true);
INSERT INTO settings VALUES
('official_mutations_paused','false'),('spawn_separation','10000'),('homes','3'),('combat_seconds','30'),
('idle_seconds','600'),('market_fee_bps','500'),('npc_daily_cap','2000'),('adventure_seconds','10800');
