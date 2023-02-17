CREATE TABLE TaskResult (
    id serial PRIMARY KEY,
    signed_data bytea,
    result_type TaskResultType,
    signing_group bytea REFERENCES SigningGroup(identifier)
);