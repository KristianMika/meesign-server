CREATE TABLE Device (
    identifier bytea PRIMARY KEY,
    device_name varchar NOT NULL,
    last_active timestamp NOT NULL DEFAULT NOW()
);