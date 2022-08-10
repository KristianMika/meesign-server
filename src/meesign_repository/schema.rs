table! {
    use diesel::sql_types::*;
    use crate::meesign_repository::enums::*;

    device (identifier) {
        identifier -> Bytea,
        device_name -> Varchar,
        last_active -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::meesign_repository::enums::*;

    groupparticipant (id) {
        id -> Int4,
        device_id -> Nullable<Bytea>,
        group_id -> Nullable<Bytea>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::meesign_repository::enums::*;

    signinggroup (identifier) {
        identifier -> Bytea,
        group_name -> Varchar,
        threshold -> Int4,
        protocol -> Protocoltype,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::meesign_repository::enums::*;

    task (id) {
        id -> Int4,
        protocol_round -> Int4,
        error_message -> Nullable<Varchar>,
        group_id -> Nullable<Bytea>,
        task_type -> Tasktype,
        task_state -> Taskstate,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::meesign_repository::enums::*;

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

allow_tables_to_appear_in_same_query!(device, groupparticipant, signinggroup, task, taskresult,);
