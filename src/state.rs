use std::collections::HashMap;
use std::convert::TryInto;

use log::{debug, error, warn};
use uuid::Uuid;

use crate::group::Group;
use crate::interfaces::grpc::format_task;
use crate::persistence::enums::TaskState;
use crate::persistence::meesign_repo::MeesignRepo;
use crate::persistence::persistance_error::PersistenceError;
use crate::tasks::{Task, TaskResult, TaskStatus};
use crate::utils;
use tokio::sync::mpsc::Sender;
use tonic::codegen::Arc;
use tonic::Status;

pub struct State {
    // groups: HashMap<Vec<u8>, Group>,
    tasks: HashMap<Uuid, Box<dyn Task + Send + Sync>>,
    subscribers: HashMap<Vec<u8>, Sender<Result<crate::proto::Task, Status>>>,
    repo: Arc<dyn MeesignRepo>,
}

impl State {
    pub fn new(repo: Arc<dyn MeesignRepo>) -> Self {
        State {
            // groups: HashMap::new(),
            tasks: HashMap::new(),
            subscribers: HashMap::new(),
            repo,
        }
    }

    pub async fn update_task(
        &mut self,
        task_id: &Uuid,
        device_identifier: &[u8],
        data: &[u8],
        attempt: u32,
    ) -> Result<bool, PersistenceError> {
        let Some(task) = self.repo.get_task(task_id).await? else {
            return Err(PersistenceError::InvalidArgumentError(format!("Task {task_id} doesn't exist")));
        };
        let current_attempt: u32 = task.attempt_count.try_into()?;
        if attempt != current_attempt {
            warn!(
                "Stale update discarded task_id={} device_id={} attempt={}",
                utils::hextrunc(task_id),
                utils::hextrunc(device_identifier),
                attempt
            );
            return Err(PersistenceError::GeneralError("Stale update".to_string()));
        }

        let previous_status = task.task_state;
        // let update_result = task.update(device_identifier, data);
        // if previous_status != TaskState::Finished && task.task_state == TaskState::Finished {
        //     if let TaskResult::GroupEstablished(group) = task.get_result().unwrap() {
        //         self.repo
        //             .add_group(
        //                 group.identifier(),
        //                 group.name(),
        //                 group.devices(),
        //                 group.threshold(),
        //                 group.protocol(),
        //                 group.certificate(),
        //             )
        //             .await?;
        //     }
        // }
        // if let Ok(true) = update_result {
        //     self.send_updates(task_id);
        // }
        // Ok(update_result)
        todo!()
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
        let task = self.repo.get_task(task_id).await?;
        let mut remove = Vec::new();

        for device in self.repo.get_task_devices(task_id).await? {
            if let Some(tx) = self.subscribers.get(&device.identifier) {
                let result = tx.try_send(Ok(format_task(
                    task_id,
                    task,
                    Some(&device.identifier),
                    None,
                )));

                if result.is_err() {
                    debug!(
                        "Closed channel detected device_id={}",
                        utils::hextrunc(&device.identifier[..4])
                    );
                    remove.push(device.identifier);
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
