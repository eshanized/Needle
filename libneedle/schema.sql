-- Author : Eshan Roy <eshanized@proton.me>
-- SPDX-License-Identifier: MIT

-- ============================================================
-- Needle database schema for Supabase (PostgreSQL)
-- Run this in the Supabase SQL Editor to bootstrap your tables.
-- ============================================================

-- enable the uuid extension if not already enabled
create extension if not exists "uuid-ossp";

-- users table
-- stores account info; auth is handled by supabase auth,
-- this table holds needle-specific profile data
create table if not exists users (
    id uuid primary key default uuid_generate_v4(),
    email text unique not null,
    username text unique not null,
    password_hash text not null,
    tier text not null default 'free',
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create index idx_users_email on users (email);
create index idx_users_username on users (username);

-- tunnels table
-- each row represents one tunnel mapping (subdomain -> local port)
create table if not exists tunnels (
    id uuid primary key default uuid_generate_v4(),
    user_id uuid not null references users(id) on delete cascade,
    subdomain text unique not null,
    custom_domain text,
    target_port integer not null,
    protocol text not null default 'http',
    is_active boolean not null default false,
    is_persistent boolean not null default false,
    created_at timestamptz not null default now(),
    last_active timestamptz not null default now()
);

create index idx_tunnels_user_id on tunnels (user_id);
create index idx_tunnels_subdomain on tunnels (subdomain);
create index idx_tunnels_active on tunnels (is_active) where is_active = true;

-- tunnel_requests table
-- logs individual http requests flowing through a tunnel
-- for the traffic inspector and analytics features
create table if not exists tunnel_requests (
    id uuid primary key default uuid_generate_v4(),
    tunnel_id uuid not null references tunnels(id) on delete cascade,
    method text not null,
    path text not null,
    status_code integer not null default 0,
    latency_ms integer not null default 0,
    request_size integer not null default 0,
    response_size integer not null default 0,
    request_headers jsonb,
    response_headers jsonb,
    request_body text,
    response_body text,
    client_ip text,
    timestamp timestamptz not null default now()
);

create index idx_tunnel_requests_tunnel_id on tunnel_requests (tunnel_id);
create index idx_tunnel_requests_timestamp on tunnel_requests (timestamp desc);

-- api_keys table
-- long-lived tokens for authenticating tunnel connections
-- without needing email/password every time
create table if not exists api_keys (
    id uuid primary key default uuid_generate_v4(),
    user_id uuid not null references users(id) on delete cascade,
    name text not null,
    key_hash text unique not null,
    key_prefix text not null,
    scopes text[] not null default '{"tunnels:read", "tunnels:write"}',
    last_used timestamptz,
    expires_at timestamptz,
    created_at timestamptz not null default now()
);

create index idx_api_keys_user_id on api_keys (user_id);
create index idx_api_keys_hash on api_keys (key_hash);

-- analytics_daily table
-- pre-aggregated daily stats to avoid scanning tunnel_requests
-- for dashboard charts
create table if not exists analytics_daily (
    id uuid primary key default uuid_generate_v4(),
    tunnel_id uuid not null references tunnels(id) on delete cascade,
    date date not null,
    total_requests integer not null default 0,
    total_bytes_in bigint not null default 0,
    total_bytes_out bigint not null default 0,
    avg_latency_ms integer not null default 0,
    error_count integer not null default 0,
    unique_ips integer not null default 0,
    created_at timestamptz not null default now(),
    unique (tunnel_id, date)
);

create index idx_analytics_daily_tunnel on analytics_daily (tunnel_id, date desc);

-- revoked_tokens table
-- tracks JWT tokens that have been explicitly revoked
-- tokens are identified by jti (JWT ID) claim
create table if not exists revoked_tokens (
    jti text primary key,
    user_id uuid not null references users(id) on delete cascade,
    revoked_at timestamptz not null default now(),
    expires_at timestamptz not null
);

create index idx_revoked_tokens_expires on revoked_tokens (expires_at);
create index idx_revoked_tokens_user on revoked_tokens (user_id);

-- row level security policies
-- these ensure users can only see their own data through the api
alter table users enable row level security;
alter table tunnels enable row level security;
alter table tunnel_requests enable row level security;
alter table api_keys enable row level security;
alter table analytics_daily enable row level security;
alter table revoked_tokens enable row level security;

-- users can read/update their own row
create policy "users_self_access" on users
    for all using (id = auth.uid());

-- users can manage their own tunnels
create policy "tunnels_owner_access" on tunnels
    for all using (user_id = auth.uid());

-- users can see requests for their own tunnels
create policy "requests_owner_access" on tunnel_requests
    for all using (
        tunnel_id in (select id from tunnels where user_id = auth.uid())
    );

-- users can manage their own api keys
create policy "api_keys_owner_access" on api_keys
    for all using (user_id = auth.uid());

-- users can see analytics for their own tunnels
create policy "analytics_owner_access" on analytics_daily
    for all using (
        tunnel_id in (select id from tunnels where user_id = auth.uid())
    );

-- users can manage their own revoked tokens
create policy "revoked_tokens_owner_access" on revoked_tokens
    for all using (user_id = auth.uid());

-- auto-update the updated_at column for users
create or replace function update_updated_at()
returns trigger as $$
begin
    new.updated_at = now();
    return new;
end;
$$ language plpgsql;

create trigger users_updated_at
    before update on users
    for each row execute function update_updated_at();
