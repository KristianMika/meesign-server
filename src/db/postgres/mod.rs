use crate::db::models::NewDevice;
use crate::device::{self, Device};
use chrono::{NaiveDateTime, Utc};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PoolError, PooledConnection};
use error_stack::{IntoReport, Result};
use error_stack::{Report, ResultExt};
use std::env;
use std::sync::Arc;
use tonic::IntoRequest;

use super::{DbAccessError, MeesignRepo};
pub mod schema;

pub struct PostgresMeesignRepo {
    pg_pool: Arc<PgPool>,
}

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

fn to_vec(bytes: &[u8; 16]) -> Vec<u8> {
    let mut out = Vec::new();

    bytes.iter().for_each(|b| {
        out.push(b.clone());
    });

    out
}

impl PostgresMeesignRepo {
    pub fn new() -> Result<Self, PoolError> {
        Ok(Self {
            pg_pool: Arc::new(PostgresMeesignRepo::init_pool()?),
        })
    }

    fn init_pool() -> Result<PgPool, PoolError> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        Pool::builder().build(manager).into_report()
    }

    fn get_connection(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<PgConnection>>, DbAccessError> {
        self.pg_pool
            .get()
            .into_report()
            .change_context(DbAccessError)
    }
}

#[tonic::async_trait]
impl MeesignRepo for PostgresMeesignRepo {
    async fn add_device(
        &self,
        identifier: &Vec<u8>,
        name: &str,
        certificate: &Vec<u8>,
    ) -> Result<(), DbAccessError> {
        const MAX_NAME_LEN: usize = 64;

        if name.chars().count() > MAX_NAME_LEN
            || name
                .chars()
                .any(|x| x.is_ascii_punctuation() || x.is_control())
        {
            return Err(Report::new(DbAccessError))
                .attach_printable(format!("Invalid device name: {name}"));
        }

        let new_device = NewDevice {
            identifier,
            device_name: name,
            certificate,
        };
        use crate::db::postgres::schema::device;

        diesel::insert_into(device::table)
            .values(new_device)
            .execute(&self.get_connection()?)
            .into_report()
            .change_context(DbAccessError)?;
        Ok(())
    }

    async fn activate_device(&self, target_identifier: &Vec<u8>) -> Result<(), DbAccessError> {
        use crate::db::postgres::schema::device::dsl::*;
        let rows_affected = diesel::update(device)
            .filter(identifier.eq(target_identifier))
            .set(last_active.eq(Utc::now().naive_utc()))
            .execute(&self.get_connection()?)
            .into_report()
            .change_context(DbAccessError)?;

        let expected_affect_rows_count = 1;
        if rows_affected != expected_affect_rows_count {
            return Err(Report::new(DbAccessError)).attach_printable(format!(
                "Invalid number of affected rows: Expected {expected_affect_rows_count}, but got {rows_affected}."
            ));
        }
        Ok(())
    }
}
