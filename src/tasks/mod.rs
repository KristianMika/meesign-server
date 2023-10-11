pub(crate) mod decrypt;
pub(crate) mod group;
pub(crate) mod sign;
pub(crate) mod sign_pdf;

use std::sync::Arc;

use crate::device::Device;
use crate::group::Group;
use crate::persistence::enums::TaskType;
use crate::persistence::meesign_repo::MeesignRepo;
use crate::persistence::models;
use crate::persistence::persistance_error::PersistenceError;
use async_trait::async_trait;

use self::group::GroupTask;
use self::sign::SignTask;

#[derive(Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Created,
    Running(u16),
    // round
    Finished,
    Failed(String),
}

#[derive(Clone)]
pub enum TaskResult {
    GroupEstablished(Group),
    Signed(Vec<u8>),
    SignedPdf(Vec<u8>),
    Decrypted(Vec<u8>),
}

impl TaskResult {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            TaskResult::GroupEstablished(group) => group.identifier(),
            TaskResult::Signed(data) => data,
            TaskResult::SignedPdf(data) => data,
            TaskResult::Decrypted(data) => data,
        }
    }
}

#[async_trait]
pub trait Task {
    fn get_status(&self) -> TaskStatus;
    fn get_type(&self) -> crate::proto::TaskType;
    fn get_work(&self, device_id: Option<&[u8]>) -> Option<Vec<u8>>;
    fn get_result(&self) -> Option<TaskResult>;
    fn get_decisions(&self) -> (u32, u32);
    /// Update protocol state with `data` from `device_id`
    ///
    /// # Returns
    /// `Ok(true)` if this update caused the next round to start; `Ok(false)` otherwise.
    fn update(&mut self, device_id: &[u8], data: &[u8]) -> Result<bool, String>;

    /// Attempt to restart protocol in task
    ///
    /// # Returns
    /// Ok(true) if task restarted successfully; Ok(false) otherwise.
    fn restart(&mut self) -> Result<bool, String>;

    /// Get timestamp of the most recent task update
    fn last_update(&self) -> u64;

    /// True if the task has been approved
    fn is_approved(&self) -> bool;

    async fn has_device(&self, device_id: &[u8]) -> Result<bool, PersistenceError>;
    async fn get_devices(&self) -> Result<Vec<Device>, PersistenceError>;
    fn waiting_for(&self, device_id: &[u8]) -> bool;

    /// Store `decision` by `device_id`
    ///
    /// # Returns
    /// `Some(true)` if this decision caused the protocol to start;
    /// `Some(false)` if this decision caused the protocol to fail;
    /// `None` otherwise.
    fn decide(&mut self, device_id: &[u8], decision: bool) -> Option<bool>;

    fn acknowledge(&mut self, device_id: &[u8]);
    fn device_acknowledged(&self, device_id: &[u8]) -> bool;
    fn get_request(&self) -> &[u8];

    fn get_attempts(&self) -> u32;
}

fn task_from_model(model: models::Task, repo: Arc<dyn MeesignRepo>) -> Arc<dyn Task> {
    match model.task_type {
        TaskType::Group => Arc::new(GroupTask::from_model(model, repo)).unwrap(),
        TaskType::Sign => Arc::new(SignTask::from_model(model, repo)).unwrap(),
    }
}
