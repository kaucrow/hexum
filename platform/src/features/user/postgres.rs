use std::str::FromStr;

use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;
use anyhow::Result;

use crate::postgres::*;
use super::*;

#[derive(Clone)]
pub struct PostgresAdapter {
    pub pool: PgPool,
}

impl PostgresAdapter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn do_add_new_user(&self, user: User) -> Result<(), LocalError> {
        let user_by_username = sqlx::query_as::<_, UserDbRow>(sql(&QUERIES.user.get_by_username))
            .bind(user.username.as_str())
            .fetch_optional(&self.pool)
            .await?;

        if user_by_username.is_some() {
            return Err(LocalError::Logic(RepositoryError::UsernameInUse));
        }

        let user_by_email = sqlx::query_as::<_, UserDbRow>(sql(&QUERIES.user.get_by_email))
            .bind(user.email.as_str())
            .fetch_optional(&self.pool)
            .await?;

        if user_by_email.is_some() {
            return Err(LocalError::Logic(RepositoryError::EmailInUse));
        }

        let roles_strings: Vec<String> = user.roles
            .iter()
            .map(|r| r.to_string())
            .collect();

        sqlx::query(sql(&QUERIES.user.insert))
            .bind(user.id)
            .bind(user.username.as_str())
            .bind(user.email.as_str())
            .bind(roles_strings)
            .bind(user.is_active)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn do_get_authenticator(
        &self,
        user_id: &Uuid,
        auth_provider: AuthProvider,
    ) -> Result<Option<UserAuthenticator>, LocalError> {
        let user_authenticator = sqlx::query_as::<_, UserAuthenticatorDbRow>(
            sql(&QUERIES.user_authenticator.get_by_user_id_and_provider)
        )
        .bind(user_id)
        .bind(auth_provider.to_string())
        .fetch_optional(&self.pool)
        .await?
        .map(|row| UserAuthenticator {
            id: row.id,
            user_id: row.user_id,
            provider: auth_provider,
            provider_id: row.provider_id,
            passwd: row.passwd,
            is_verified: row.is_verified,
        });

        Ok(user_authenticator)
    }

    async fn do_add_authenticator(&self, user_authenticator: UserAuthenticator) -> Result<(), LocalError> {
        sqlx::query(sql(&QUERIES.user_authenticator.insert))
            .bind(user_authenticator.id)
            .bind(user_authenticator.user_id)
            .bind(user_authenticator.provider.to_string())
            .bind(user_authenticator.provider_id)
            .bind(user_authenticator.passwd)
            .bind(user_authenticator.is_verified)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn do_verify_local_auth_by_user_id(&self, id: &Uuid) -> Result<(), LocalError> {
        sqlx::query(sql(&QUERIES.user_authenticator.verify_local_by_user_id))
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn do_delete_user_by_id(&self, id: &Uuid) -> Result<(), LocalError> {
        sqlx::query(sql(&QUERIES.user.delete_by_id))
            .bind(id)
            .execute(&self.pool)
            .await?;

       Ok(())
    }
}

#[async_trait]
impl Repository for PostgresAdapter {
    async fn get_user_by_id(&self, id: &Uuid) -> Option<User> {
        let record = sqlx::query_as::<_, UserDbRow>(sql(&QUERIES.user.get_by_id))
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .ok()??;

        let parsed_roles: Vec<Role> = record.roles
            .into_iter()
            .filter_map(|r| Role::from_str(&r).ok())
            .collect();

        let username_vo = Username::new(record.username).ok()?;
        let email_vo = EmailAddress::new(record.email).ok()?;

        Some(User {
            id: record.id,
            username: username_vo,
            email: email_vo,
            roles: parsed_roles,
            is_active: record.is_active,
        })
    }

    async fn get_user_by_username(&self, username: &str) -> Option<User> {
        let user = sqlx::query_as::<_, UserDbRow>(sql(&QUERIES.user.get_by_username))
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .ok()??;

        let parsed_roles: Vec<Role> = user.roles
            .into_iter()
            .filter_map(|r| Role::from_str(&r).ok())
            .collect();

        let username_vo = Username::new(user.username).ok()?;
        let email_vo = EmailAddress::new(user.email).ok()?;

        Some(User {
            id: user.id,
            username: username_vo,
            email: email_vo,
            roles: parsed_roles,
            is_active: user.is_active,
        })
    }

    async fn get_user_by_email(&self, email: &EmailAddress) -> Option<User> {
        let user_db_row = sqlx::query_as::<_, UserDbRow>(sql(&QUERIES.user.get_by_email))
            .bind(email.as_str())
            .fetch_optional(&self.pool)
            .await
            .ok()??;

        let parsed_roles: Vec<Role> = user_db_row.roles
            .into_iter()
            .filter_map(|r| Role::from_str(&r).ok())
            .collect();

        let username_vo = Username::new(user_db_row.username).ok()?;
        let email_vo = EmailAddress::new(user_db_row.email).ok()?;

        Some(User {
            id: user_db_row.id,
            username: username_vo,
            email: email_vo,
            roles: parsed_roles,
            is_active: user_db_row.is_active,
        })
    }

    async fn add_new_user(&self, user: User) -> Result<(), RepositoryError> {
        Ok(self.do_add_new_user(user).await?)
    }

    async fn delete_user_by_id(&self, id: &Uuid) -> Result<(), RepositoryError> {
        Ok(self.do_delete_user_by_id(id).await?)
    }

    async fn get_authenticator(
        &self,
        user_id: &Uuid,
        auth_provider: AuthProvider,
    ) -> Result<Option<UserAuthenticator>, RepositoryError> {
        Ok(self.do_get_authenticator(user_id, auth_provider).await?)
    }

    async fn verify_local_auth_by_user_id(&self, id: &Uuid) -> Result<(), RepositoryError> {
        Ok(self.do_verify_local_auth_by_user_id(id).await?)
    }

    async fn add_authenticator(
        &self,
        user_authenticator: UserAuthenticator
    ) -> Result<(), RepositoryError> {
        Ok(self.do_add_authenticator(user_authenticator).await?)
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error("{0}")]
    Logic(RepositoryError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

impl From<LocalError> for RepositoryError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::Logic(e) => e,
            LocalError::Sqlx(e) => RepositoryError::Internal(e.to_string()),
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct UserDbRow {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
    pub is_active: bool,
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
pub struct UserAuthenticatorDbRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_id: Option<String>,
    pub passwd: Option<String>,
    pub is_verified: Option<bool>,
}