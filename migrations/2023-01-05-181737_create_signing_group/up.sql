CREATE TABLE SigningGroup (
    identifier bytea PRIMARY KEY,
    group_name varchar NOT NULL,
    threshold integer NOT NULL CHECK (threshold > 0),
    protocol ProtocolType NOT NULL,
    round integer NOT NULL CHECK(round >= 0),
    key_type KeyType NOT NULL,
    group_certificate bytea
);