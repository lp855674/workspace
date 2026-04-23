alter table workspace_sessions add column schema_version integer not null default 2;

alter table panel_states add column panel_instance_key text not null default '';

alter table panel_states add column focused integer not null default 0 check (focused in (0, 1));

alter table panel_states add column panel_state_json text check (
    panel_state_json is null or json_valid(panel_state_json)
);
