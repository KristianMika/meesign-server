table! {
    device (identifier) {
        identifier -> Bytea,
        device_name -> Varchar,
    }
}

table! {
    groupparticipant (id) {
        id -> Int4,
        device_id -> Nullable<Bytea>,
        group_id -> Nullable<Bytea>,
    }
}

table! {
    signinggroup (identifier) {
        identifier -> Bytea,
        group_name -> Varchar,
        threshold -> Int4,
        protocol -> Protocoltype,
    }
}

table! {
    task (id) {
        id -> Int4,
        protocol_round -> Int4,
        error_message -> Nullable<Varchar>,
        group_id -> Nullable<Bytea>,
        task_type -> Nullable<Tasktype>,
        task_state -> Nullable<Taskstate>,
    }
}

table! {
    taskresult (id) {
        id -> Int4,
        signed_data -> Nullable<Bytea>,
        result_type -> Nullable<Taskresulttype>,
        signing_group -> Nullable<Bytea>,
    }
}

joinable!(groupparticipant -> device (device_id));
joinable!(groupparticipant -> signinggroup (group_id));
joinable!(task -> signinggroup (group_id));
joinable!(taskresult -> signinggroup (signing_group));

allow_tables_to_appear_in_same_query!(
    device,
    groupparticipant,
    signinggroup,
    task,
    taskresult,
);
