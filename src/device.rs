use chrono::{NaiveDateTime, Utc};
#[derive(Queryable, Clone, Eq, Debug)]
pub struct Device {
    identifier: Vec<u8>,
    name: String,
    last_active: NaiveDateTime,
    // protocol: ProtocolType
}

impl Device {

    pub fn identifier(&self) -> &[u8] {
        &self.identifier
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn last_active(&self) -> u64 {
        self.last_active.timestamp() as u64
    }

    pub fn activated(&mut self) -> NaiveDateTime {
        self.last_active = Utc::now().naive_utc();
        self.last_active
    }
}

impl PartialEq for Device {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}


impl From<&Device> for crate::proto::Device {
    fn from(device: &Device) -> Self {
        crate::proto::Device {
            identifier: device.identifier().to_vec(),
            name: device.name().to_string(),
            last_active: device.last_active()
        }
    }
}
