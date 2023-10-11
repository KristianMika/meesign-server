use crate::communicator::Communicator;
use crate::device::Device;
use crate::group::Group;
use crate::persistence::enums;
use crate::persistence::meesign_repo::MeesignRepo;
use crate::persistence::models::{FromModelParts, Task as TaskModel};
use crate::persistence::persistance_error::PersistenceError;
use crate::proto::{KeyType, ProtocolType, TaskType};
use crate::protocols::elgamal::ElgamalGroup;
use crate::protocols::frost::FROSTGroup;
use crate::protocols::gg18::GG18Group;
use crate::protocols::Protocol;
use crate::tasks::{Task, TaskResult, TaskStatus};
use crate::{get_timestamp, utils};
use async_trait::async_trait;
use log::{info, warn};
use meesign_crypto::proto::{Message as _, ProtocolMessage};
use prost::Message as _;
use std::io::Read;
use std::process::{Command, Stdio};
use tonic::codegen::Arc;

pub struct GroupTask {
    name: String,
    threshold: u32,
    key_type: KeyType,
    devices: Vec<Arc<Device>>,
    communicator: Communicator,
    result: Option<Result<Group, String>>,
    protocol: Box<dyn Protocol + Send + Sync>,
    request: Vec<u8>,
    last_update: u64,
    attempts: u32,
}

impl GroupTask {
    pub fn try_new(
        repo: Arc<dyn MeesignRepo>,
        name: &str,
        devices: &[Arc<Device>],
        threshold: u32,
        protocol_type: ProtocolType,
        key_type: KeyType,
    ) -> Result<Self, String> {
        let devices_len = devices.len() as u32;
        let protocol: Box<dyn Protocol + Send + Sync> = match (protocol_type, key_type) {
            (ProtocolType::Gg18, KeyType::SignPdf) => {
                Box::new(GG18Group::new(devices_len, threshold))
            }
            (ProtocolType::Gg18, KeyType::SignChallenge) => {
                Box::new(GG18Group::new(devices_len, threshold))
            }
            (ProtocolType::Frost, KeyType::SignChallenge) => {
                Box::new(FROSTGroup::new(devices_len, threshold))
            }
            (ProtocolType::Elgamal, KeyType::Decrypt) => {
                Box::new(ElgamalGroup::new(devices_len, threshold))
            }
            _ => {
                warn!(
                    "Protocol {:?} does not support {:?} key type",
                    protocol_type, key_type
                );
                return Err("Unsupported protocol type and key type combination".into());
            }
        };

        if devices_len < 1 {
            warn!("Invalid number of devices {}", devices_len);
            return Err("Invalid input".into());
        }
        if !protocol.get_type().check_threshold(threshold, devices_len) {
            warn!("Invalid group threshold {}-of-{}", threshold, devices_len);
            return Err("Invalid input".into());
        }

        let mut devices = devices.to_vec();
        devices.sort_by_key(|x| x.identifier().to_vec());

        let communicator = Communicator::new(&devices, devices.len() as u32, protocol.get_type());

        let request = (crate::proto::GroupRequest {
            device_ids: devices.iter().map(|x| x.identifier().to_vec()).collect(),
            name: String::from(name),
            threshold,
            protocol: protocol.get_type() as i32,
            key_type: key_type as i32,
        })
        .encode_to_vec();

        Ok(GroupTask {
            name: name.into(),
            threshold,
            devices,
            key_type,
            communicator,
            result: None,
            protocol,
            request,
            last_update: get_timestamp(),
            attempts: 0,
        })
    }

    pub fn from_model(task: TaskModel, repo: Arc<dyn MeesignRepo>) -> Option<Self> {
        // todo: key type and protocol type brobably shouldn't be nullable
        let threshold = task.threshold as u32;
        let party_count = 0; // todo
        let protocol: Box<dyn Protocol + Send + Sync> =
            match (task.protocol_type.unwrap(), task.key_type.unwrap()) {
                (enums::ProtocolType::Gg18, enums::KeyType::SignPDF) => {
                    Box::new(GG18Group::new(party_count, threshold))
                }
                (enums::ProtocolType::Gg18, enums::KeyType::SignChallenge) => {
                    Box::new(GG18Group::new(party_count, threshold))
                }
                (enums::ProtocolType::Frost, enums::KeyType::SignChallenge) => {
                    Box::new(FROSTGroup::new(party_count, threshold))
                }
                (enums::ProtocolType::ElGamal, enums::KeyType::Decrypt) => {
                    Box::new(ElgamalGroup::new(party_count, threshold))
                }
                _ => {
                    warn!(
                        "Protocol {:?} does not support {:?} key type",
                        task.protocol_type,
                        task.key_type.unwrap()
                    );
                    return Err("Unsupported protocol type and key type combination".into());
                }
            };

        Some(Self {
            protocol,
            request: task.request.unwrap(),
            last_update: task.last_update.timestamp() as u64,
            attempts: task.attempt_count as u32,
            name: todo!(),
            threshold,
            key_type: task.key_type.unwrap(),
            devices: todo!(),
            communicator: todo!(),
            result: todo!(),
        })
    }

    fn start_task(&mut self) {
        self.protocol.initialize(&mut self.communicator, &[]);
    }

    fn advance_task(&mut self) {
        self.protocol.advance(&mut self.communicator);
    }

    fn finalize_task(&mut self) {
        let identifier = self.protocol.finalize(&mut self.communicator);
        if identifier.is_none() {
            self.result = Some(Err("Task failed (group key not output)".to_string()));
            return;
        }
        let identifier = identifier.unwrap();
        // TODO
        let certificate = if self.protocol.get_type() == ProtocolType::Gg18 {
            Some(issue_certificate(&self.name, &identifier))
        } else {
            None
        };

        info!(
            "Group established group_id={} devices={:?}",
            utils::hextrunc(&identifier),
            self.devices
                .iter()
                .map(|device| utils::hextrunc(device.identifier()))
                .collect::<Vec<_>>()
        );

        self.result = Some(Ok(Group::new(
            identifier,
            self.name.clone(),
            self.devices.iter().map(Arc::clone).collect(),
            self.threshold,
            self.protocol.get_type(),
            self.key_type,
            certificate,
        )));

        self.communicator.clear_input();
    }

    fn next_round(&mut self) {
        if self.protocol.round() == 0 {
            self.start_task();
        } else if self.protocol.round() < self.protocol.last_round() {
            self.advance_task()
        } else {
            self.finalize_task()
        }
    }
}

#[async_trait]
impl Task for GroupTask {
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
        TaskType::Group
    }

    fn get_work(&self, device_id: Option<&[u8]>) -> Option<Vec<u8>> {
        if device_id.is_none() || !self.waiting_for(device_id.unwrap()) {
            return None;
        }

        self.communicator.get_message(device_id.unwrap())
    }

    fn get_result(&self) -> Option<TaskResult> {
        if let Some(Ok(group)) = &self.result {
            Some(TaskResult::GroupEstablished(group.clone()))
        } else {
            None
        }
    }

    fn get_decisions(&self) -> (u32, u32) {
        (
            self.communicator.accept_count(),
            self.communicator.reject_count(),
        )
    }

    fn update(&mut self, device_id: &[u8], data: &[u8]) -> Result<bool, String> {
        if self.communicator.accept_count() != self.devices.len() as u32 {
            return Err("Not enough agreements to proceed with the protocol.".to_string());
        }

        if !self.waiting_for(device_id) {
            return Err("Wasn't waiting for a message from this ID.".to_string());
        }

        let data =
            ProtocolMessage::decode(data).map_err(|_| String::from("Expected ProtocolMessage."))?;
        self.communicator.receive_messages(device_id, data.message);
        self.last_update = get_timestamp();

        if self.communicator.round_received() && self.protocol.round() <= self.protocol.last_round()
        {
            self.next_round();
            return Ok(true);
        }

        Ok(false)
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
        self.communicator.accept_count() == self.devices.len() as u32
    }

    async fn has_device(&self, device_id: &[u8]) -> Result<bool, PersistenceError> {
        // return self
        //     .devices
        //     .iter()
        //     .map(|device| device.identifier())
        //     .any(|x| x == device_id);
        todo!()
    }

    async fn get_devices(&self) -> Result<Vec<Device>, PersistenceError> {
        // self.devices.clone()
        todo!()
    }

    fn waiting_for(&self, device: &[u8]) -> bool {
        if self.protocol.round() == 0 {
            return !self.communicator.device_decided(device);
        } else if self.protocol.round() >= self.protocol.last_round() {
            return !self.communicator.device_acknowledged(device);
        }

        self.communicator.waiting_for(device)
    }

    fn decide(&mut self, device_id: &[u8], decision: bool) -> Option<bool> {
        self.communicator.decide(device_id, decision);
        self.last_update = get_timestamp();
        if self.result.is_none() && self.protocol.round() == 0 {
            if self.communicator.reject_count() > 0 {
                self.result = Some(Err("Task declined".to_string()));
                return Some(false);
            } else if self.communicator.accept_count() == self.devices.len() as u32 {
                self.next_round();
                return Some(true);
            }
        }
        None
    }

    fn acknowledge(&mut self, device_id: &[u8]) {
        self.communicator.acknowledge(device_id);
    }

    fn device_acknowledged(&self, device_id: &[u8]) -> bool {
        self.communicator.device_acknowledged(device_id)
    }

    fn get_request(&self) -> &[u8] {
        &self.request
    }

    fn get_attempts(&self) -> u32 {
        self.attempts
    }
}

fn issue_certificate(name: &str, public_key: &[u8]) -> Vec<u8> {
    assert_eq!(public_key.len(), 65);
    let mut process = Command::new("java")
        .arg("-jar")
        .arg("MeeSignHelper.jar")
        .arg("cert")
        .arg(name)
        .arg(hex::encode(public_key))
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut result = Vec::new();
    process
        .stdout
        .as_mut()
        .unwrap()
        .read_to_end(&mut result)
        .unwrap();
    result
}
