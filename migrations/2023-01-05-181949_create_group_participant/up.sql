CREATE TABLE GroupParticipant (
    id serial PRIMARY KEY,
    device_id bytea REFERENCES Device(identifier),
    group_id bytea REFERENCES SigningGroup(identifier)
);