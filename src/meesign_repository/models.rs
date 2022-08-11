use super::schema::*;
use crate::meesign_repository::enums::ProtocolType;

#[derive(Insertable)]
#[table_name = "device"]
pub struct NewDevice<'a> {
    pub identifier: &'a Vec<u8>,
    pub device_name: &'a str,
}

#[derive(Queryable, Clone, Eq, PartialEq)]
pub struct Group {
    pub identifier: Vec<u8>,
    pub name: String,
    pub threshold: i32,
    pub protocol: ProtocolType,
    pub round: i32,
    pub group_certificate: Option<Vec<u8>>
}
