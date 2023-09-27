use crate::communicator::Communicator;
use crate::device::Device;
use crate::group::Group;
use crate::persistence::meesign_repo::MeesignRepo;
use crate::persistence::models::Task as TaskModel;
use crate::persistence::persistance_error::PersistenceError;
use crate::proto::{ProtocolType, SignRequest, TaskType};
use crate::protocols::frost::FROSTSign;
use crate::protocols::gg18::GG18Sign;
use crate::protocols::Protocol;
use crate::tasks::{Task, TaskResult, TaskStatus};
use crate::{get_timestamp, utils};
use async_trait::async_trait;
use log::{info, warn};
use meesign_crypto::proto::{Message as _, ProtocolMessage};
use prost::Message as _;
use tonic::codegen::Arc;
use uuid::Uuid;

pub struct SignTask {
    // group: Group,
    id: Uuid,
    result: Option<Result<Vec<u8>, String>>,
    pub(super) data: Vec<u8>,
    preprocessed: Option<Vec<u8>>,
    pub(super) protocol: Box<dyn Protocol + Send + Sync>,
    request: Vec<u8>,
    pub(super) last_update: u64,
    pub(super) attempts: u32,
    repo: Arc<dyn MeesignRepo>,
}

impl SignTask {
    pub fn try_new(group: Group, name: String, data: Vec<u8>) -> Result<Self, String> {
        let mut devices: Vec<Arc<Device>> = group.devices().to_vec();
        devices.sort_by_key(|x| x.identifier().to_vec());
        let protocol_type = group.protocol();

        let communicator = Communicator::new(&devices, group.threshold(), protocol_type);
        // TODO: store communicator
        let request = (SignRequest {
            group_id: group.identifier().to_vec(),
            name,
            data: data.clone(),
        })
        .encode_to_vec();

        Ok(SignTask {
            result: None,
            data,
            preprocessed: None,
            protocol: match protocol_type {
                ProtocolType::Gg18 => Box::new(GG18Sign::new()),
                ProtocolType::Frost => Box::new(FROSTSign::new()),
                _ => {
                    warn!("Protocol type {:?} does not support signing", protocol_type);
                    return Err("Unsupported protocol type for signing".into());
                }
            },
            request,
            last_update: get_timestamp(),
            attempts: 0,
        })
    }

    pub fn from_model(task: TaskModel, repo: Arc<dyn MeesignRepo>) -> Option<Self> {
        let protocol: Box<dyn Protocol + Send + Sync> = match task.protocol_type.unwrap() {
            crate::persistence::enums::ProtocolType::Gg18 => Box::new(GG18Sign::new()),
            crate::persistence::enums::ProtocolType::Frost => Box::new(FROSTSign::new()),
            _ => {
                warn!(
                    "Protocol type {:?} does not support signing",
                    task.protocol_type.unwrap()
                );
                return None;
            }
        };
        let mut result = None;
        if task.error_message.is_some() {
            result = Some(Err(task.error_message.unwrap()));
        }
        if task.result_data.is_some() {
            result = Some(Ok(task.result_data.unwrap()));
        }
        Some(Self {
            repo,
            result,
            id: task.id,
            data: task.task_data.unwrap(),
            preprocessed: task.preprocessed,
            protocol,
            request: task.request.unwrap(),
            last_update: task.last_update.timestamp() as u64,
            attempts: task.attempt_count as u32,
        })
    }

    pub fn get_group(&self) -> &Group {
        todo!()
    }

    /// Use this method to change data to be used for signing
    pub(super) fn set_preprocessed(&mut self, preprocessed: Vec<u8>) {
        self.preprocessed = Some(preprocessed);
    }

    pub(super) fn start_task(&mut self) {
        let mut communicator = self.get_communicator().unwrap();
        assert!(communicator.accept_count() >= self.group.threshold());
        self.protocol.initialize(
            &mut communicator,
            self.preprocessed.as_ref().unwrap_or(&self.data),
        );
    }

    pub(super) fn advance_task(&mut self) {
        let communicator = self.get_communicator().unwrap();
        self.protocol.advance(communicator)
    }

    pub(super) fn finalize_task(&mut self) {
        let communicator = self.get_communicator().unwrap();

        let signature = self.protocol.finalize(communicator);
        if signature.is_none() {
            self.result = Some(Err("Task failed (signature not output)".to_string()));
            return;
        }
        let signature = signature.unwrap();

        info!(
            "Signature created by group_id={}",
            utils::hextrunc(self.group.identifier())
        );

        self.result = Some(Ok(signature));
        self.get_communicator().unwrap().clear_input();
    }

    pub(super) fn next_round(&mut self) {
        if self.protocol.round() == 0 {
            self.start_task();
        } else if self.protocol.round() < self.protocol.last_round() {
            self.advance_task()
        } else {
            self.finalize_task()
        }
    }

    pub(super) fn update_internal(
        &mut self,
        device_id: &[u8],
        data: &[u8],
    ) -> Result<bool, String> {
        let communicator = self.get_communicator().unwrap();
        if communicator.accept_count() < self.group.threshold() {
            return Err("Not enough agreements to proceed with the protocol.".to_string());
        }

        if !self.waiting_for(device_id) {
            return Err("Wasn't waiting for a message from this ID.".to_string());
        }

        let data =
            ProtocolMessage::decode(data).map_err(|_| String::from("Expected ProtocolMessage."))?;
        communicator.receive_messages(device_id, data.message);
        self.last_update = get_timestamp();

        if communicator.round_received() && self.protocol.round() <= self.protocol.last_round() {
            return Ok(true);
        }
        Ok(false)
    }

    pub(super) fn decide_internal(&mut self, device_id: &[u8], decision: bool) -> Option<bool> {
        let communicator = self.get_communicator().unwrap();
        communicator.decide(device_id, decision);
        self.last_update = get_timestamp();
        if self.result.is_none() && self.protocol.round() == 0 {
            if communicator.reject_count() >= self.group.reject_threshold() {
                self.result = Some(Err("Task declined".to_string()));
                return Some(false);
            } else if communicator.accept_count() >= self.group.threshold() {
                return Some(true);
            }
        }
        None
    }

    fn get_communicator(&self) -> Option<&mut Communicator> {
        todo!()
    }
}

#[async_trait]
impl Task for SignTask {
    fn get_status(&self) -> TaskStatus {
        match &self.result {
            Some(Err(e)) => TaskStatus::Failed(e.clone()),
            Some(Ok(_)) => TaskStatus::Finished,
            None => {
                if self.protocol.round() == 0 {
                    TaskStatus::Created
                } else {
                    TaskStatus::Running(self.protocol.round())
                }
            }
        }
    }

    fn get_type(&self) -> TaskType {
        TaskType::SignChallenge
    }

    fn get_work(&self, device_id: Option<&[u8]>) -> Option<Vec<u8>> {
        if device_id.is_none() || !self.waiting_for(device_id.unwrap()) {
            return None;
        }
        let communicator = self.get_communicator().unwrap();

        communicator.get_message(device_id.unwrap())
    }

    fn get_result(&self) -> Option<TaskResult> {
        if let Some(Ok(signature)) = &self.result {
            Some(TaskResult::Signed(signature.clone()))
        } else {
            None
        }
    }

    fn get_decisions(&self) -> (u32, u32) {
        let communicator = self.get_communicator().unwrap();
        (communicator.accept_count(), communicator.reject_count())
    }

    fn update(&mut self, device_id: &[u8], data: &[u8]) -> Result<bool, String> {
        let result = self.update_internal(device_id, data);
        if let Ok(true) = result {
            self.next_round();
        };
        result
    }

    fn restart(&mut self) -> Result<bool, String> {
        self.last_update = get_timestamp();
        if self.result.is_some() {
            return Ok(false);
        }

        if self.is_approved() {
            self.attempts += 1;
            self.start_task();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn last_update(&self) -> u64 {
        self.last_update
    }

    fn is_approved(&self) -> bool {
        let communicator = self.get_communicator().unwrap();

        communicator.accept_count() >= self.group.threshold()
    }

    fn has_device(&self, device_id: &[u8]) -> bool {
        self.group.contains(device_id)
    }

    async fn get_devices(&self) -> Result<Vec<Device>, PersistenceError> {
        self.repo.get_task_devices(&self.id).await.unwrap()
    }

    fn waiting_for(&self, device: &[u8]) -> bool {
        let communicator = self.get_communicator().unwrap();

        if self.protocol.round() == 0 {
            return !communicator.device_decided(device);
        } else if self.protocol.round() >= self.protocol.last_round() {
            return !communicator.device_acknowledged(device);
        }

        communicator.waiting_for(device)
    }

    fn decide(&mut self, device_id: &[u8], decision: bool) -> Option<bool> {
        let result = self.decide_internal(device_id, decision);
        if let Some(true) = result {
            self.next_round();
        };
        result
    }

    fn acknowledge(&mut self, device_id: &[u8]) {
        let communicator = self.get_communicator().unwrap();
        communicator.acknowledge(device_id);
    }

    fn device_acknowledged(&self, device_id: &[u8]) -> bool {
        let communicator = self.get_communicator().unwrap();
        communicator.device_acknowledged(device_id)
    }

    fn get_request(&self) -> &[u8] {
        &self.request
    }

    fn get_attempts(&self) -> u32 {
        self.attempts
    }
}
