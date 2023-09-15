use uuid::Uuid;

use super::{
    enums::{KeyType, ProtocolType},
    models::{Device, Group, Task},
    persistance_error::PersistenceError,
};

#[tonic::async_trait]
pub trait MeesignRepo: Send + Sync {
    /* Devices */
    async fn add_device(
        &self,
        identifier: &[u8],
        name: &str,
        certificate: &[u8],
    ) -> Result<Device, PersistenceError>;
    async fn get_devices(&self) -> Result<Vec<Device>, PersistenceError>;
    async fn activate_device(&self, identifier: &[u8]) -> Result<Option<Device>, PersistenceError>;

    // async fn get_device(&self, identifier: &Vec<u8>) -> Option<Device>;

    // /* Groups */
    async fn add_group<'a>(
        &self,
        identifier: &[u8],
        name: &str,
        devices: &[&[u8]],
        threshold: u32,
        protocol: ProtocolType,
        certificate: Option<&[u8]>,
    ) -> Result<Group, PersistenceError>;
    // async fn get_group(&self, group_identifier: &Vec<u8>) -> Option<Group>;
    async fn get_device_groups(&self, identifier: &[u8]) -> Result<Vec<Group>, PersistenceError>;

    async fn get_groups(&self) -> Result<Vec<Group>, PersistenceError>;

    async fn create_group_task<'a>(
        &self,
        name: &str,
        devices: &[Vec<u8>],
        threshold: u32,
        protocol: ProtocolType,
        key_type: KeyType,
    ) -> Result<Task, PersistenceError>;

    async fn create_sign_task<'a>(
        &self,
        group_identifier: &Vec<u8>,
        name: &str,
        data: &Vec<u8>,
    ) -> Result<Task, PersistenceError>;

    async fn create_decrypt_task(
        &self,
        group_identifier: &Vec<u8>,
        name: &str,
        data: &Vec<u8>,
    ) -> Result<Task, PersistenceError>;

    async fn get_tasks(&self) -> Result<Vec<Task>, PersistenceError>;
    async fn get_task(&self, task_id: &Uuid) -> Result<Option<Task>, PersistenceError>;

    async fn get_tasks_for_restart(&self) -> Result<Vec<Task>, PersistenceError>;
}
