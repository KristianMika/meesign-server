use crate::proto::{KeyType, ProtocolType};

#[derive(Clone)]
pub struct Group {
    identifier: Vec<u8>,
    name: String,
    threshold: u32,
    protocol: ProtocolType,
    key_type: KeyType,
    certificate: Option<Vec<u8>>,
}

impl Group {
    pub fn new(
        identifier: Vec<u8>,
        name: String,
        threshold: u32,
        protocol: ProtocolType,
        key_type: KeyType,
        certificate: Option<Vec<u8>>,
    ) -> Self {
        assert!(!identifier.is_empty());
        assert!(threshold >= 1);
        Group {
            identifier,
            name,
            threshold,
            protocol,
            key_type,
            certificate,
        }
    }

    pub fn identifier(&self) -> &[u8] {
        &self.identifier
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn threshold(&self) -> u32 {
        self.threshold
    }

    pub fn reject_threshold(&self) -> u32 {
        // self.devices.len() as u32 - self.threshold + 1 // rejects >= threshold_reject => fail
        3
    }

    pub fn protocol(&self) -> ProtocolType {
        self.protocol
    }

    pub fn key_type(&self) -> KeyType {
        self.key_type
    }

    pub fn certificate(&self) -> Option<&Vec<u8>> {
        self.certificate.as_ref()
    }
}

// impl From<&Group> for crate::proto::Group {
//     fn from(group: &Group) -> Self {
//         crate::proto::Group {
//             identifier: group.identifier().to_vec(),
//             name: group.name().to_owned(),
//             threshold: group.threshold(),
//             device_ids: group
//                 .devices()
//                 .iter()
//                 .map(|x| x.identifier())
//                 .map(Vec::from)
//                 .collect(),
//             protocol: group.protocol().into(),
//             key_type: group.key_type().into(),
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn empty_identifier() {
        Group::new(
            vec![],
            String::from("Sample Group"),
            2,
            ProtocolType::Gg18,
            KeyType::SignPdf,
            None,
        );
    }

    // #[test]
    // fn protobuf_group() {
    //     let group = Group::new(
    //         vec![0x00],
    //         String::from("Sample Group"),
    //         2,
    //         ProtocolType::Gg18,
    //         KeyType::SignPdf,
    //         None,
    //     );
    //     let protobuf = crate::proto::Group::from(&group);
    //     assert_eq!(protobuf.identifier, group.identifier());
    //     assert_eq!(protobuf.name, group.name());
    //     assert_eq!(protobuf.threshold, group.threshold());
    //     assert_eq!(protobuf.protocol, group.protocol() as i32);
    //     assert_eq!(protobuf.key_type, group.key_type() as i32);
    // }

    #[test]
    fn sample_group() {
        let identifier = vec![0x01, 0x02, 0x03, 0x04];
        let name = String::from("Sample Group");
        let threshold = 3;
        let protocol_type = ProtocolType::Gg18;
        let key_type = KeyType::SignPdf;
        let group = Group::new(
            identifier.clone(),
            name.clone(),
            threshold,
            protocol_type,
            key_type,
            None,
        );
        assert_eq!(group.identifier(), &identifier);
        assert_eq!(group.name(), &name);
        assert_eq!(group.threshold(), threshold);
        assert_eq!(group.reject_threshold(), 3);
        assert_eq!(group.protocol(), protocol_type.into());
        assert_eq!(group.key_type(), key_type.into());
        assert_eq!(group.certificate(), None);
    }
}
