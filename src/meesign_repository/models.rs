
use crate::meesign_repository::enums::ProtocolType;
use super::schema::*;


#[derive(Queryable, Clone, PartialEq, Eq, Debug)]
pub struct Device {
    pub identifier: Vec<u8>,
    pub device_name: String,
}

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
}

