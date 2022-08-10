extern crate dotenv;

use crate::meesign_repository::models::Group;
use self::models::{Device, NewDevice};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PoolError};
use std::env;
use std::sync::Arc;

pub mod models;
pub mod schema;
pub mod enums;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

fn init_pool() -> Result<PgPool, PoolError> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder().build(manager)
}

pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub trait MeesignRepo {
    //async fn add_todo(&self, description: String) -> anyhow::Result<i64>;
    //async fn complete_todo(&self, id: i64) -> anyhow::Result<bool>;
    //async fn list_todos(&self) -> anyhow::Result<()>;
    fn create_device<'a>(&self, identifier: &'a Vec<u8>, device_name: &'a str) -> Device;
    fn get_device(&self, identifier: &Vec<u8>) -> Option<Device>;

    fn get_group(&self, group_identifier: &Vec<u8>) -> Option<Group>;
}

pub struct PostgresMeesignRepo {
    pg_pool: Arc<PgPool>,
}

impl PostgresMeesignRepo {
    pub fn new(pg_pool: PgPool) -> Self {
        Self {
            pg_pool: Arc::new(pg_pool),
        }
    }

    pub fn init() -> Result<Self, PoolError> {
        let pool = init_pool()?;
        Ok(PostgresMeesignRepo::new(pool))
    }
    /// TODO: fn get_connection {}
    pub fn create_device<'a>(&self, identifier: &'a Vec<u8>, device_name: &'a str) -> Device {
        use schema::device;

        let new_device = NewDevice {
            identifier,
            device_name,
        };

        diesel::insert_into(device::table)
            .values(&new_device)
            .get_result(&self.pg_pool.get().unwrap())
            .expect("Error saving new device")
    }

    pub fn get_device(&self, device_identifier: &Vec<u8>) -> Option<Device> {
        use schema::device::dsl::*;

        let result = device
            .find(device_identifier)
            .load::<Device>(&self.pg_pool.get().unwrap())
            .expect("Error loading posts");

        println!("{}", result[0].device_name);

        println!("{:?}", result[0].identifier);

        Some(result[0].clone())
    }

    pub fn get_group(&self, group_identifier: &Vec<u8>) -> Option<Group> {
        use schema::signinggroup::dsl::*;


        let result = signinggroup
            .find(group_identifier)
            .load::<Group>(&self.pg_pool.get().unwrap())
            .expect("Error loading posts");

        println!("{:?}", result[0].identifier);

        Some(result[0].clone())
    }
}
