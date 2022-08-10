use std::collections::HashMap;

use crate::meesign_repository::PostgresMeesignRepo;
use crate::task::{Task, TaskStatus, TaskResult};
use crate::group::Group;
use crate::device::Device;
use uuid::Uuid;
use crate::protocols::ProtocolType;
use crate::protocols::gg18::{GG18Group, GG18Sign};
use log::{warn, error};

pub struct State {
    groups: HashMap<Vec<u8>, Group>,
    tasks: HashMap<Uuid, Box<dyn Task + Send + Sync>>,
    pub meesign_repo: PostgresMeesignRepo
}

impl State {
    pub fn new() -> Self {
        State {
            groups: HashMap::new(),
            tasks: HashMap::new(),
            meesign_repo: PostgresMeesignRepo::init().expect("Coudln't initialize Posgtres MeeSign repo.")
        }
    }

    pub fn add_group_task(&mut self, name: &str, devices: &[Vec<u8>], threshold: u32, protocol: ProtocolType) -> Option<Uuid> {
        if name.chars().count() > 64 || name.chars().any(|x| x.is_ascii_punctuation() || x.is_control()) {
            warn!("Invalid Group name {}", name);
            return None
        }
        let mut device_list = Vec::new();
        for device in devices {
            if !self.devices.contains_key(device.as_slice()) {
                warn!("Unknown Device ID {}", hex::encode(device));
                return None
            }
            device_list.push(self.devices.get(device.as_slice()).unwrap().clone());
        }

        let task: Box<dyn Task + Send + Sync + 'static> = match protocol {
            ProtocolType::GG18 => Box::new(GG18Group::new(name, &device_list, threshold))
        };

        Some(self.add_task(task))
    }

    pub fn add_sign_task(&mut self, group: &[u8], name: &str, data: &[u8]) -> Option<Uuid> {
        if data.len() > 8 * 1024 * 1024 || name.len() > 256 || name.chars().any(|x| x.is_control()) {
            warn!("Invalid PDF name {} ({} B)", name, data.len());
            return None
        }

        self.groups.get(group).cloned().map(|group| {
            let task: Box<dyn Task + Send + Sync + 'static> = match group.protocol() {
                ProtocolType::GG18 => Box::new(GG18Sign::new(group, name.to_string(), data.to_vec())),
            };
            self.add_task(task)
        })
    }

    fn add_task(&mut self, task: Box<dyn Task + Send + Sync>) -> Uuid {
        let uuid = Uuid::new_v4();
        self.tasks.insert(uuid, task);
        uuid
    }

    pub fn get_device_tasks(&self, device: &[u8]) -> Vec<(Uuid, &Box<dyn Task + Send + Sync>)> {
        let mut tasks = Vec::new();
        for (uuid, task) in self.tasks.iter() {
            if task.waiting_for(device) {
                tasks.push((uuid.clone(), task));
            }
        }
        tasks
    }

    pub fn get_device_groups(&self, device: &Vec<u8>) -> Vec<Group> {
        let mut groups = Vec::new();
        for group in self.groups.values() {
            if group.contains(device) {
                groups.push(group.clone());
            }
        }
        groups
    }

    pub fn get_groups(&self) -> &HashMap<Vec<u8>, Group> {
        &self.groups
    }

    pub fn get_tasks(&self) -> &HashMap<Uuid, Box<dyn Task + Send + Sync>> {
        &self.tasks
    }

    pub fn get_task(&self, task: &Uuid) -> Option<&Box<dyn Task + Send + Sync>> {
        self.tasks.get(task)
    }

    pub fn get_work(&self, task: &Uuid, device: &[u8]) -> Option<Vec<u8>> {
        self.tasks.get(task).unwrap().get_work(Some(device))
    }

    pub fn update_task(&mut self, task: &Uuid, device: &[u8], data: &[u8]) -> Result<(), String> {
        let task = self.tasks.get_mut(task).unwrap();
        let previous_status = task.get_status();
        task.update(device, data)?;
        if previous_status != TaskStatus::Finished && task.get_status() == TaskStatus::Finished {
            // TODO join if statements once #![feature(let_chains)] gets stabilized
            if let TaskResult::GroupEstablished(group) = task.get_result().unwrap() {
                self.groups.insert(group.identifier().to_vec(), group);
            }
        }
        Ok(())
    }

    pub fn get_devices(&self) -> anyhow::Result<Vec<Device>> {
        self.meesign_repo.get_devices()
    }

    pub fn device_activated(&mut self, device_id: &[u8]) {
        if let Some(device) = self.devices.get_mut(device_id) {
            device.activated();
        } else {
            error!("Unknown Device ID {}", hex::encode(device_id));
        }
    }
}
