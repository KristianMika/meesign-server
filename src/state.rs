use std::collections::HashMap;

use log::{debug, error, warn};
use uuid::Uuid;

use crate::group::Group;
use crate::interfaces::grpc::format_task;
use crate::persistence::meesign_repo::MeesignRepo;
use crate::persistence::persistance_error::PersistenceError;
use crate::tasks::{Task, TaskResult, TaskStatus};
use crate::utils;
use tokio::sync::mpsc::Sender;
use tonic::codegen::Arc;
use tonic::Status;

pub struct State {
    // devices: HashMap<Vec<u8>, Arc<Device>>,
    groups: HashMap<Vec<u8>, Group>,
    tasks: HashMap<Uuid, Box<dyn Task + Send + Sync>>,
    subscribers: HashMap<Vec<u8>, Sender<Result<crate::proto::Task, Status>>>,
    repo: Arc<dyn MeesignRepo>,
}

impl State {
    pub fn new(repo: Arc<dyn MeesignRepo>) -> Self {
        State {
            groups: HashMap::new(),
            tasks: HashMap::new(),
            subscribers: HashMap::new(),
            repo,
        }
    }

    fn add_task(&mut self, task: Box<dyn Task + Sync + Send>) -> Uuid {
        let uuid = Uuid::new_v4();
        self.tasks.insert(uuid, task);
        uuid
    }

    pub fn update_task(
        &mut self,
        task_id: &Uuid,
        device: &[u8],
        data: &[u8],
        attempt: u32,
    ) -> Result<bool, String> {
        let task = self.tasks.get_mut(task_id).unwrap();
        if attempt != task.get_attempts() {
            warn!(
                "Stale update discarded task_id={} device_id={} attempt={}",
                utils::hextrunc(task_id.as_bytes()),
                utils::hextrunc(device),
                attempt
            );
            return Err("Stale update".to_string());
        }

        let previous_status = task.get_status();
        let update_result = task.update(device, data);
        if previous_status != TaskStatus::Finished && task.get_status() == TaskStatus::Finished {
            // TODO join if statements once #![feature(let_chains)] gets stabilized
            if let TaskResult::GroupEstablished(group) = task.get_result().unwrap() {
                self.groups.insert(group.identifier().to_vec(), group);
            }
        }
        if let Ok(true) = update_result {
            self.send_updates(task_id);
        }
        update_result
    }

    pub fn decide_task(&mut self, task_id: &Uuid, device: &[u8], decision: bool) -> bool {
        let task = self.tasks.get_mut(task_id).unwrap();
        let change = task.decide(device, decision);
        if change.is_some() {
            self.send_updates(task_id);
            if change.unwrap() {
                log::info!(
                    "Task approved task_id={}",
                    utils::hextrunc(task_id.as_bytes())
                );
            } else {
                log::info!(
                    "Task declined task_id={}",
                    utils::hextrunc(task_id.as_bytes())
                );
            }
            return true;
        }
        false
    }

    pub fn acknowledge_task(&mut self, task: &Uuid, device: &[u8]) {
        let task = self.tasks.get_mut(task).unwrap();
        task.acknowledge(device);
    }


    pub fn restart_task(&mut self, task_id: &Uuid) -> bool {
        if self
            .tasks
            .get_mut(task_id)
            .and_then(|task| task.restart().ok())
            .unwrap_or(false)
        {
            self.send_updates(task_id);
            true
        } else {
            false
        }
    }

    pub fn add_subscriber(
        &mut self,
        device_id: Vec<u8>,
        tx: Sender<Result<crate::proto::Task, Status>>,
    ) {
        self.subscribers.insert(device_id, tx);
    }

    pub fn remove_subscriber(&mut self, device_id: &Vec<u8>) {
        self.subscribers.remove(device_id);
        debug!(
            "Removing subscriber device_id={}",
            utils::hextrunc(device_id)
        );
    }

    pub fn get_subscribers(&self) -> &HashMap<Vec<u8>, Sender<Result<crate::proto::Task, Status>>> {
        &self.subscribers
    }

    pub async fn send_updates(&mut self, task_id: &Uuid) -> Result<(), PersistenceError> {
        todo!();
        let task = self.repo.get_task(task_id).await?;
        let mut remove = Vec::new();

        for device_id in task.get_devices().iter().map(|device| device.identifier()) {
            if let Some(tx) = self.subscribers.get(device_id) {
                let result = tx.try_send(Ok(format_task(task_id, task, Some(device_id), None)));

                if result.is_err() {
                    debug!(
                        "Closed channel detected device_id={}…",
                        utils::hextrunc(&device_id[..4])
                    );
                    remove.push(device_id.to_vec());
                }
            }
        }

        for device_id in remove {
            self.remove_subscriber(&device_id);
        }

        Ok(())
    }

    pub fn get_repo(&self) -> &Arc<dyn MeesignRepo> {
        &self.repo
    }
}
