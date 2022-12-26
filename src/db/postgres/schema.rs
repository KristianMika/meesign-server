// @generated automatically by Diesel CLI.

diesel::table! {
    use diesel::sql_types::*;
    use crate::db::enums::*;

    device (identifier) {
        id -> Int4,
        identifier -> Bytea,
        device_name -> Varchar,
        certificate -> Nullable<Bytea>,
        last_active -> Timestamp,
    }
}
