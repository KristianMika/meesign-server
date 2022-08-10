use crate::device::Device;
use crate::meesign_repository::models::Group;
use self::models::{NewDevice};
use anyhow::bail;
use async_trait::async_trait;
use chrono::Utc;
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


#[async_trait]
pub trait MeesignRepo {
    async fn add_device<'a>(&self, identifier: &'a Vec<u8>, device_name: &'a str) -> anyhow::Result<Device>;
    async fn activate_device<'a>(&self, identifier: &'a Vec<u8>) -> anyhow::Result<()>;
    async fn get_device(&self, identifier: &Vec<u8>) -> Option<Device>;
    async fn get_devices(&self) -> anyhow::Result<Vec<Device>>;

    async fn get_group(&self, group_identifier: &Vec<u8>) -> Option<Group>;
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

    pub async fn add_device<'a>(&self, identifier: &'a Vec<u8>, name: &'a str) -> anyhow::Result<Device> {
        const MAX_NAME_LEN:usize = 64;
        if name.chars().count() > MAX_NAME_LEN || name.chars().any(|x| x.is_ascii_punctuation() || x.is_control()) {
            bail!("Invalid Device name {}", &name[..MAX_NAME_LEN]);
        }
        use schema::device;

        let new_device = NewDevice {
            identifier,
            device_name:name,
        };

        let device:Device = diesel::insert_into(device::table)
            .values(&new_device)
            .get_result(&self.pg_pool.get().unwrap())?;
        Ok(device)
    }

    pub async fn activate_device<'a>(&self, target_identifier: &'a Vec<u8>) -> anyhow::Result<()> {
        use schema::device::dsl::*;

        // TODO
        // let _ = diesel::update(device.filter(identifier.eq(target_identifier)))
        //         .set(last_active.eq(Utc::now().naive_utc()))
        //         .get_result(&self.pg_pool.get().unwrap())?;
        Ok(())
    }
    pub async fn get_devices(&self) -> anyhow::Result<Vec<Device>> {
        use schema::device::dsl::*;

        let devices = device
            .load::<Device>(&self.pg_pool.get().unwrap())?;

        Ok(devices)
    }

    pub async fn get_device(&self, device_identifier: &Vec<u8>) -> Option<Device> {
        use schema::device::dsl::*;

        let result = device
            .find(device_identifier)
            .load::<Device>(&self.pg_pool.get().unwrap())
            .expect("Error loading posts");

        Some(result[0].clone())
    }

    pub async fn get_group(&self, group_identifier: &Vec<u8>) -> Option<Group> {
        use schema::signinggroup::dsl::*;


        let result = signinggroup
            .find(group_identifier)
            .load::<Group>(&self.pg_pool.get().unwrap())
            .expect("Error loading posts");

        println!("{:?}", result[0].identifier);

        Some(result[0].clone())
    }
}
