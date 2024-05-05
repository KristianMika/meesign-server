use crate::communicator::Communicator;
use crate::proto::ProtocolType;
use crate::protocols::Protocol;
use meesign_crypto::proto::{Message, ProtocolGroupInit, ProtocolInit};
use tokio::sync::RwLockWriteGuard;

pub struct FROSTGroup {
    parties: u32,
    threshold: u32,
    round: u16,
}

impl FROSTGroup {
    pub fn new(parties: u32, threshold: u32) -> Self {
        Self {
            parties,
            threshold,
            round: 0,
        }
    }
}

impl Protocol for FROSTGroup {
    fn initialize(&mut self, mut communicator: RwLockWriteGuard<'_, Communicator>, _: &[u8]) {
        communicator.set_active_devices(None);
        let parties = self.parties;
        let threshold = self.threshold;
        communicator.send_all(|idx| {
            (ProtocolGroupInit {
                protocol_type: ProtocolType::Frost as i32,
                index: idx + 1,
                parties,
                threshold,
            })
            .encode_to_vec()
        });

        self.round = 1;
    }

    fn advance(&mut self, mut communicator: RwLockWriteGuard<'_, Communicator>) {
        assert!((0..self.last_round()).contains(&self.round));

        communicator.relay();
        self.round += 1;
    }

    fn finalize(&mut self, communicator: RwLockWriteGuard<'_, Communicator>) -> Option<Vec<u8>> {
        assert_eq!(self.last_round(), self.round);
        self.round += 1;
        communicator.get_final_message()
    }

    fn round(&self) -> u16 {
        self.round
    }

    fn last_round(&self) -> u16 {
        3
    }

    fn get_type(&self) -> ProtocolType {
        ProtocolType::Frost
    }
}

pub struct FROSTSign {
    round: u16,
}

impl FROSTSign {
    pub fn new() -> Self {
        Self { round: 0 }
    }
}

impl Protocol for FROSTSign {
    fn initialize(&mut self, mut communicator: RwLockWriteGuard<'_, Communicator>, data: &[u8]) {
        communicator.set_active_devices(None);
        let participant_indices: Vec<_> = communicator
            .get_protocol_indices()
            .into_iter()
            .map(|idx| idx + 1)
            .collect();
        communicator.send_all(|idx| {
            (ProtocolInit {
                protocol_type: ProtocolType::Frost as i32,
                indices: participant_indices.clone(),
                index: idx + 1,
                data: Vec::from(data),
            })
            .encode_to_vec()
        });

        self.round = 1;
    }

    fn advance(&mut self, mut communicator: RwLockWriteGuard<'_, Communicator>) {
        assert!((0..self.last_round()).contains(&self.round));

        communicator.relay();
        self.round += 1;
    }

    fn finalize(&mut self, communicator: RwLockWriteGuard<'_, Communicator>) -> Option<Vec<u8>> {
        assert_eq!(self.last_round(), self.round);
        self.round += 1;
        communicator.get_final_message()
    }

    fn round(&self) -> u16 {
        self.round
    }

    fn last_round(&self) -> u16 {
        3
    }

    fn get_type(&self) -> ProtocolType {
        ProtocolType::Frost
    }
}
