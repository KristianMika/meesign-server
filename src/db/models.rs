use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};

use crate::db::enums::ProtocolType;
use crate::db::postgres::schema::*;

use super::enums::KeyType;

#[derive(Insertable)]
#[diesel(table_name = device)]
pub struct NewDevice<'a> {
    pub identifier: &'a Vec<u8>,
    pub device_name: &'a str,
    pub certificate: &'a Vec<u8>,
}

#[derive(Queryable)]
#[diesel(table_name = device)]
pub struct Device {
    pub id: i32,
    pub identifier: Vec<u8>,
    pub device_name: String,
    pub certificate: Vec<u8>,
    pub last_active: NaiveDateTime,
}

impl From<&Device> for crate::proto::Device {
    fn from(device: &Device) -> Self {
        crate::proto::Device {
            identifier: device.identifier.to_vec(),
            name: device.device_name.to_string(),
            certificate: device.certificate.to_vec(),
            last_active: device.last_active.timestamp_millis() as u64,
        }
    }
}

#[derive(Queryable, Clone, Eq, PartialEq)]
pub struct Group {
    pub identifier: Vec<u8>,
    pub group_name: String,
    pub threshold: i32,
    pub protocol: ProtocolType,
    pub round: i32,
    pub key_type: KeyType,
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
