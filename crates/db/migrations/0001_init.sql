create table workspace_sessions (
    session_id text not null primary key,
    created_at text not null
);

create table panel_states (
    session_id text not null,
    panel_id text not null,
    dock text not null check (dock in ('Left', 'Center', 'Bottom', 'Right')),
    visible integer not null check (visible in (0, 1)),
    primary key (session_id, panel_id),
    foreign key (session_id) references workspace_sessions(session_id)
);

create table command_history (
    command_id text not null primary key,
    created_at text not null
);

create table notifications (
    notification_id text not null primary key,
    level text not null check (level in ('info', 'warning', 'error')),
    message text not null,
    created_at text not null
);

create table module_state (
    module_id text not null,
    state_key text not null,
    state_json text not null check (json_valid(state_json)),
    updated_at text not null,
    primary key (module_id, state_key)
);
