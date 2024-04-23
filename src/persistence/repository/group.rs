use std::convert::TryInto;

use diesel::{pg::Pg, result::Error::NotFound, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, RunQueryDsl};

use crate::persistence::{
    enums::{KeyType, ProtocolType},
    error::PersistenceError,
    models::{Group, NewGroup, NewGroupParticipant},
    repository::device::device_ids_to_identifiers,
};

pub async fn get_groups<Conn>(connection: &mut Conn) -> Result<Vec<Group>, PersistenceError>
where
    Conn: AsyncConnection<Backend = Pg>,
{
    use crate::persistence::schema::signinggroup;
    Ok(signinggroup::table.load(connection).await?)
}

pub async fn add_group<'a, Conn>(
    connection: &mut Conn,
    identifier: &[u8],
    name: &str,
    devices: &[&[u8]],
    threshold: u32,
    protocol: ProtocolType,
    key_type: KeyType,
    certificate: Option<&[u8]>,
) -> Result<Group, PersistenceError>
where
    Conn: AsyncConnection<Backend = Pg>,
{
    use crate::persistence::schema::groupparticipant;
    use crate::persistence::schema::signinggroup;

    let threshold: i32 = threshold.try_into()?;
    if !(1..=devices.len() as i32).contains(&threshold) {
        return Err(PersistenceError::InvalidArgumentError(format!(
            "Invalid threshold {threshold}"
        )));
    }

    if identifier.is_empty() {
        return Err(PersistenceError::InvalidArgumentError(format!(
            "Empty identifier"
        )));
    }

    let new_group = NewGroup {
        identifier,
        threshold: threshold as i32,
        protocol,
        group_name: name,
        round: 0, // TODO: check why
        group_certificate: certificate,
        key_type,
    };

    let devices: Vec<Vec<u8>> = devices
        .iter()
        .map(|identifier| identifier.to_vec())
        .collect();
    let ids = device_ids_to_identifiers(connection, devices).await?;
    let group = diesel::insert_into(signinggroup::table)
        .values(new_group)
        .returning(Group::as_returning())
        .get_result(connection)
        .await?;

    let group_id = group.id;
    let group_participants: Vec<NewGroupParticipant> = ids
        .into_iter()
        .map(|device_id| NewGroupParticipant {
            device_id,
            group_id,
        })
        .collect();

    diesel::insert_into(groupparticipant::table)
        .values(group_participants)
        .execute(connection)
        .await?;
    Ok(group)
}

pub async fn get_group<Conn>(
    connection: &mut Conn,
    group_identifier: &[u8],
) -> Result<Option<Group>, PersistenceError>
where
    Conn: AsyncConnection<Backend = Pg>,
{
    use crate::persistence::schema::signinggroup::dsl::*;

    let group: Option<Group> = match signinggroup
        .filter(identifier.eq(group_identifier))
        .first(connection)
        // .optional() TODO
        .await
    {
        Ok(val) => Some(val),
        Err(NotFound) => None,
        Err(err) => return Err(PersistenceError::ExecutionError(err)),
    };

    Ok(group)
}

pub async fn get_device_groups<Conn>(
    connection: &mut Conn,
    device_identifier: &[u8],
) -> Result<Vec<Group>, PersistenceError>
where
    Conn: AsyncConnection<Backend = Pg>,
{
    use crate::persistence::schema::device::dsl::{device, identifier};
    use crate::persistence::schema::groupparticipant::dsl::groupparticipant;
    use crate::persistence::schema::signinggroup::dsl::signinggroup;

    // TODO: consider not using artificial IDs
    let groups: Vec<Group> = groupparticipant
        .inner_join(signinggroup)
        .inner_join(device)
        .filter(identifier.eq(device_identifier))
        .select(Group::as_select())
        .load(connection)
        .await?;

    Ok(groups)
}

#[cfg(test)]
mod test {
    use diesel_async::AsyncPgConnection;

    use crate::persistence::{
        repository::device::add_device,
        tests::persistency_unit_test_context::PersistencyUnitTestContext,
    };

    use super::*;

    const GROUP_IDENTIFIER: [u8; 32] = [1; 32];
    const GROUP_NAME: &str = "Test Group";
    const GROUP_CERTIFICATE: [u8; 150] = [42; 150];
    const DEVICE_1_ID: [u8; 32] = [1; 32];
    const DEVICE_2_ID: [u8; 32] = [2; 32];

    #[tokio::test]
    async fn given_valid_arguments_create_group() -> Result<(), PersistenceError> {
        let ctx = PersistencyUnitTestContext::new();
        let mut connection: AsyncPgConnection = ctx.get_test_connection().await?;

        let devices = &vec![&DEVICE_1_ID[..], &DEVICE_2_ID[..]];
        let threshold = 2;
        add_device(&mut connection, &DEVICE_1_ID, "Device 1", &vec![42; 5]).await?;
        add_device(&mut connection, &DEVICE_2_ID, "Device 2", &vec![42; 5]).await?;
        let created_group = add_group(
            &mut connection,
            &GROUP_IDENTIFIER,
            GROUP_NAME,
            devices,
            threshold,
            ProtocolType::ElGamal,
            KeyType::Decrypt,
            Some(&GROUP_CERTIFICATE),
        )
        .await?;
        let retrieved_group = get_group(&mut connection, &GROUP_IDENTIFIER).await?;
        assert!(
            retrieved_group.is_some(),
            "Created group couldn't be retrieved"
        );
        let retrieved_group = retrieved_group.unwrap();
        assert_eq!(created_group, retrieved_group);
        let target_group = Group {
            id: retrieved_group.id,
            identifier: Vec::from(GROUP_IDENTIFIER),
            group_name: GROUP_NAME.into(),
            threshold: threshold as i32,
            protocol: ProtocolType::ElGamal,
            round: 0,
            key_type: KeyType::Decrypt,
            group_certificate: Some(GROUP_CERTIFICATE.into()),
        };

        assert_eq!(retrieved_group, target_group);
        Ok(())
    }
}
