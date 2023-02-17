CREATE TABLE Task (
    id SERIAL PRIMARY KEY,
    protocol_round integer NOT NULL CHECK (protocol_round > 0),
    error_message varchar,
    group_id bytea REFERENCES SigningGroup(identifier),
    task_type TaskType NOT NULL,
    task_state TaskState NOT NULL
);