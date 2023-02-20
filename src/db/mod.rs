use std::{
    error::Error,
    fmt::{self, Display},
};

use error_stack::Result;

use self::{
    enums::ProtocolType,
    models::{Device, Group},
};

pub mod enums;
pub mod models;
pub mod postgres;

#[tonic::async_trait]
pub trait MeesignRepo {
    /* Devices */
    async fn add_device(
        &self,
        identifier: &[u8],
        name: &str,
        certificate: &[u8],
    ) -> Result<(), DbAccessError>;
    async fn get_devices(&self) -> Result<Vec<Device>, DbAccessError>;
    async fn activate_device(&self, identifier: &Vec<u8>) -> Result<(), DbAccessError>;

    // async fn get_device(&self, identifier: &Vec<u8>) -> Option<Device>;

    // /* Groups */
    // async fn add_group<'a>(
    //     &self,
    //     name: &str,
    //     devices: &[Vec<u8>],
    //     threshold: u32,
    //     protocol: ProtocolType,
    // ) -> Result<Group, ()>;
    // async fn get_group(&self, group_identifier: &Vec<u8>) -> Option<Group>;
    async fn get_groups(&self) -> Result<Vec<Group>, DbAccessError>;
}

#[derive(Debug)]
pub struct DbAccessError;

impl Display for DbAccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Database access error: Coudln't interract with the db")
    }
}

impl Error for DbAccessError {}

#[derive(Debug)]
enum DbError {
    InvalidInput(String),
    DbError,
    Other,
}

impl Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DbError: Coudln't not interract with the db.")
    }
}

impl Error for DbError {}
