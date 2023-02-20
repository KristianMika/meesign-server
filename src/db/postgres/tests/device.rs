use super::test_context::TestDbContext;

use crate::db::postgres::PostgresMeesignRepo;
use crate::db::{DbAccessError, MeesignRepo};

/// Tests insertion of a single device
#[tokio::test]
async fn test_insert_device() -> error_stack::Result<(), DbAccessError> {
    let _ctx = TestDbContext::new()?;

    let repo = PostgresMeesignRepo::new(&_ctx.ephemeral_db_url()).unwrap();
    let identifier = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let name = "Test User 123";
    let certificate = vec![10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
    repo.add_device(&identifier, name, &certificate).await?;

    let devices = repo.get_devices().await?;
    assert_eq!(devices.len(), 1);
    let fetched_device = devices.first().unwrap();
    assert_eq!(fetched_device.identifier, identifier);
    assert_eq!(fetched_device.device_name, name);
    assert_eq!(fetched_device.certificate, certificate);
    Ok(())
}

/// Tests the DB won't allow multiple devices with the same identifier == public key
#[tokio::test]
async fn test_identifier_unique_constraint() -> error_stack::Result<(), DbAccessError> {
    let _ctx = TestDbContext::new()?;
    let repo = PostgresMeesignRepo::new(&_ctx.ephemeral_db_url()).unwrap();
    let identifier = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let first_device_name = "user1";
    repo.add_device(&identifier, first_device_name, &vec![1, 2, 3])
        .await?;
    let Err(_) = repo.add_device(&identifier, "user2", &vec![3, 2, 1]).await else {
        panic!("DB shoudln't have allowed to insert 2 devices with the same identifier");
    };
    let devices = repo.get_devices().await?;
    assert_eq!(devices.len(), 1);
    assert_eq!(devices.first().unwrap().device_name, first_device_name);

    Ok(())
}
