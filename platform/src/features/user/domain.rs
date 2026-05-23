use strum::{Display, EnumString};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: Username,
    pub email: EmailAddress,
    pub roles: Vec<Role>,
    pub is_active: bool,
}

impl User {
    /// Creates a new User.
    pub fn new(username: &str, email: &str) -> Result<Self, UserError> {
        Ok(Self {
            id: uuid::Uuid::new_v4(),
            username: Username::new(username.to_string())?,
            email: EmailAddress::new(email.to_string())?,
            roles: vec![Role::BasicUser],
            is_active: true,
        })
    }

    // Check if the user has any of the roles provided
    pub fn has_any_role(&self, allowed_roles: &[Role]) -> bool {
        self.roles.iter().any(|user_role| allowed_roles.contains(user_role))
    }

    // Deactivate a user
    pub fn deactivate(&mut self) -> Result<(), UserError> {
        if !self.is_active {
            return Err(UserError::UserAlreadyDeactivated);
        }

        self.is_active = false;
        Ok(())
    }

    // Give admin permissions to a user
    pub fn grant_admin(&mut self) {
        if !self.roles.contains(&Role::Admin) {
            self.roles.push(Role::Admin);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Display, EnumString)]
pub enum Role {
    Admin,
    Manager,
    BasicUser,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Username(String);

impl Username {
    /// Creates a Username.
    /// Only accepts letters and numbers (alphanumeric).
    /// Rejects spaces and symbols.
    /// Automatically converts the input to lowercase.
    pub fn new(username: String) -> Result<Self, UserError> {
        if username.is_empty() {
            return Err(UserError::InvalidUsername);
        }

        for c in username.chars() {
            if !c.is_alphanumeric() {
                return Err(UserError::InvalidUsername);
            }
        }

        Ok(Self(username.to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailAddress(String);

impl EmailAddress {
    /// Creates an EmailAddress.
    /// If it doesn't have an '@', it refuses to be created.
    pub fn new(email: String) -> Result<Self, UserError> {
        if !email.contains('@') {
            return Err(UserError::InvalidEmail);
        }
        Ok(Self(email))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Error, Debug)]
pub enum UserError {
    #[error("The username provided is invalid.")]
    InvalidUsername,

    #[error("The email address provided is invalid.")]
    InvalidEmail,

    #[error("Password must be at least 8 characters.")]
    PasswordTooShort,

    #[error("This user is already deactivated.")]
    UserAlreadyDeactivated,

    #[error("User lacks the required role.")]
    InsufficientPermissions,

}

pub struct UserAuthenticator {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: AuthProvider,
    pub provider_id: Option<String>,    // OAuth provider ID or None for 'Local' provider
    pub passwd: Option<String>,         // Hashed password or None for OAuth provider
    pub is_verified: Option<bool>,      // Email verified flag or None for OAuth provider
}

impl UserAuthenticator {
    /// Creates a new authenticator linked to a user.
    /// Handles the logic of ensuring local has a password and OAuth has a provider_id.
    pub fn new(
        user_id: Uuid,
        provider: AuthProvider,
        provider_id: Option<String>,
        passwd: Option<String>,
        is_verified: Option<bool>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            provider,
            provider_id,
            passwd,
            is_verified,
        }
    }

    /// Helper to create a local password authenticator
    pub fn new_local(user_id: Uuid, passwd_hash: String) -> Self {
        Self::new(user_id, AuthProvider::Local, None, Some(passwd_hash), Some(false))
    }

    /// Helper to create an OAuth authenticator
    pub fn new_oauth(user_id: Uuid, provider: AuthProvider, external_id: String) -> Self {
        Self::new(user_id, provider, Some(external_id), None, None)
    }
}

#[derive(Debug, Clone, Display, EnumString)]
pub enum AuthProvider {
    Local,
    Google,
    GitHub,
}