use diesel::{Insertable, Queryable};

use crate::db::enums::ProtocolType;
use crate::db::postgres::schema::*;

#[derive(Insertable)]
#[table_name = "device"]
pub struct NewDevice<'a> {
    pub identifier: &'a Vec<u8>,
    pub device_name: &'a str,
    pub certificate: &'a Vec<u8>,
}

#[derive(Queryable, Clone, Eq, PartialEq)]
pub struct Group {
    pub identifier: Vec<u8>,
    pub group_name: String,
    pub threshold: i32,
    pub protocol: ProtocolType,
    pub round: i32,
    pub group_certificate: Option<Vec<u8>>,
}

// #[derive(Insertable)]
// #[table_name = "signinggroup"]
// pub struct NewSigningGroup<'a> {
//     pub identifier: &'a Vec<u8>,
//     pub group_name: &'a str,
//     pub threshold: i32,
//     pub protocol: ProtocolType,
//     pub round: i32,
//     pub group_certificate: Option<Vec<u8>>,
// }
