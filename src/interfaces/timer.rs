use crate::state::State;
use crate::tasks::TaskStatus;
use crate::{get_timestamp, utils};

use hex::ToHex;
use log::{error, info, debug};
use tokio::sync::MutexGuard;
use tokio::{sync::Mutex, time};
use tonic::codegen::Arc;

pub async fn run_timer(state: Arc<Mutex<State>>) -> Result<(), String> {
    let mut interval = time::interval(time::Duration::from_secs(1));
    loop {
        interval.tick().await;
        let mut state = state.lock().await;
        check_tasks(&mut state);
        check_subscribers(&mut state).await;
    }
}

async fn check_tasks(state: &mut MutexGuard<'_, State>) {
    let tasks_for_restart = match state.get_repo().get_tasks_for_restart().await {
        Ok(val) => val,
        Err(err) => {
            error!("Couldn't get tasks for restart: {err}");
            return;
        }
    };
    for task in tasks_for_restart {
        state.restart_task(&task.id);
    }
}

async fn check_subscribers(state: &mut MutexGuard<'_, State>) {
    let mut remove = Vec::new();
    for (device_id, tx) in state.get_subscribers() {
        if tx.is_closed() {
            debug!(
                "Closed channel detected device_id={:?}",
                utils::hextrunc(device_id)
            );
            remove.push(device_id.clone());
        } else {
            if let Err(err) = state.get_repo().activate_device(&device_id).await {
                error!(
                    "Couldn't activate device {}: {}",
                    device_id.encode_hex::<String>(),
                    err
                )
            }
        }
    }
    for device_id in remove {
        state.remove_subscriber(&device_id);
    }
}
