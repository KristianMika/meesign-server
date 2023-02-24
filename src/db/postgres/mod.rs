use super::models::{Device, Group};
use super::{DbAccessError, MeesignRepo};
use crate::db::models::NewDevice;
use chrono::Utc;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PoolError, PooledConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use error_stack::{IntoReport, Result};
use error_stack::{Report, ResultExt};
use std::sync::Arc;

pub mod schema;
#[cfg(test)]
mod tests;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub struct PostgresMeesignRepo {
    pg_pool: Arc<PgPool>,
}

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

impl PostgresMeesignRepo {
    pub fn from_url(database_url: &str) -> Result<Self, DbAccessError> {
        let repo = Self {
            pg_pool: Arc::new(
                PostgresMeesignRepo::init_pool(database_url)
                    .change_context(DbAccessError)
                    .attach_printable_lazy(|| "Coudln't initalize pg pool")?,
            ),
        };
        repo.apply_migrations()?;
        Ok(repo)
    }

    pub fn apply_migrations(&self) -> Result<(), DbAccessError> {
        let mut conn = self.get_connection()?;
        // TODO: return an error instead of panicking
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Couldn't apply migrations");
        Ok(())
    }
    fn init_pool(database_url: &str) -> Result<PgPool, PoolError> {
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
        identifier: &[u8],
        name: &str,
        certificate: &[u8],
    ) -> Result<(), DbAccessError> {
        const MAX_NAME_LEN: usize = 64;

        if name.chars().count() > MAX_NAME_LEN
            || name
                .chars()
                .any(|x| x.is_ascii_punctuation() || x.is_control())
        {
            return Err(Report::new(DbAccessError))
                .attach_printable_lazy(|| format!("Invalid device name: {name}"));
        }

        let new_device = NewDevice {
            identifier: &identifier.to_vec(),
            device_name: name,
            certificate: &certificate.to_vec(),
        };
        use crate::db::postgres::schema::device;

        diesel::insert_into(device::table)
            .values(new_device)
            .execute(&mut self.get_connection()?)
            .into_report()
            .change_context(DbAccessError)?;
        Ok(())
    }

    async fn activate_device(&self, target_identifier: &Vec<u8>) -> Result<(), DbAccessError> {
        use crate::db::postgres::schema::device::dsl::*;
        let rows_affected = diesel::update(device)
            .filter(identifier.eq(target_identifier))
            .set(last_active.eq(Utc::now().naive_utc()))
            .execute(&mut self.get_connection()?)
            .into_report()
            .change_context(DbAccessError)?;

        let expected_affect_rows_count = 1;
        if rows_affected != expected_affect_rows_count {
            return Err(Report::new(DbAccessError)).attach_printable_lazy(|| format!(
                "Invalid number of affected rows: Expected {expected_affect_rows_count}, but got {rows_affected}."
            ));
        }
        Ok(())
    }

    async fn get_devices(&self) -> Result<Vec<Device>, DbAccessError> {
        use crate::db::postgres::schema::device;
        Ok(device::table
            .load(&mut self.get_connection()?)
            .into_report()
            .change_context(DbAccessError)?)
    }

    async fn get_groups(&self) -> Result<Vec<Group>, DbAccessError> {
        use crate::db::postgres::schema::signinggroup;
        Ok(signinggroup::table
            .load(&mut self.get_connection()?)
            .into_report()
            .change_context(DbAccessError)?)
    }
}
