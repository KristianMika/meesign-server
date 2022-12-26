CREATE TABLE Device (
    id serial PRIMARY KEY,
    identifier bytea UNIQUE,
    device_name varchar NOT NULL,
    certificate bytea NOT NULL,
    last_active timestamp NOT NULL DEFAULT NOW()
);