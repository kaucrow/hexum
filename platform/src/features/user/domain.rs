use strum::{Display, EnumString};
use thiserror::Error;

use crate::prelude::*;

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Display, EnumString)]
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

    #[error("Password must contain at least 12 characters, 1 number, and 1 symbol.")]
    InvalidPassword,

    #[error("This user is already deactivated.")]
    UserAlreadyDeactivated,

    #[error("User lacks the required role.")]
    InsufficientPermissions,

}

#[derive(Clone)]
pub struct UserAuthenticator {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: AuthProvider,
    pub provider_id: Option<String>,            // OAuth provider ID or None for Local provider
    pub hashed_passwd: Option<HashedPassword>,  // Hashed Local provider password or None for OAuth provider
    pub is_verified: Option<bool>,              // Email verified flag or None for OAuth provider
}

impl UserAuthenticator {
    /// Creates a new authenticator linked to a user.
    /// Handles the logic of ensuring local has a password and OAuth has a provider_id.
    pub fn new(
        user_id: Uuid,
        provider: AuthProvider,
        provider_id: Option<String>,
        hashed_passwd: Option<HashedPassword>,
        is_verified: Option<bool>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            provider,
            provider_id,
            hashed_passwd,
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

#[derive(Serialize, Deserialize, Debug, Clone, Display, EnumString)]
#[serde(rename_all = "lowercase")]
pub enum AuthProvider {
    Local,
    Google,
    GitHub,
}

pub type HashedPassword = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Password(String);

impl Password {
    /// Creates a password for a Local User Authenticator.
    /// Refuses creation if it doesn't contain at least 12 ASCII characters, 1 number, and 1 symbol.
    pub fn new(password: String) -> Result<Self, UserError> {
        if password.chars().count() < 12 {
            return Err(UserError::InvalidPassword);
        }

        // Every single character must be either
        // an English alphanumeric character or a standard ASCII symbol.
        let is_pure_ascii = password.chars().all(|c| {
            c.is_ascii_alphanumeric() || c.is_ascii_punctuation()
        });

        if !is_pure_ascii {
            return Err(UserError::InvalidPassword);
        }

        // Complexity checks
        let has_number = password.chars().any(|c| c.is_ascii_digit());
        let has_symbol = password.chars().any(|c| c.is_ascii_punctuation());

        if !has_number || !has_symbol {
            return Err(UserError::InvalidPassword);
        }

        Ok(Self(password))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Password {
    type Error = UserError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod username_tests {
        use super::*;

        #[test]
        fn test_valid_username() {
            let un = Username::new("john".into()).unwrap();
            assert_eq!(un.as_str(), "john");
        }

        #[test]
        fn test_username_lowercased() {
            let un = Username::new("JohnDoe".into()).unwrap();
            assert_eq!(un.as_str(), "johndoe");
        }

        #[test]
        fn test_username_rejects_empty() {
            let err = Username::new("".into()).unwrap_err();
            assert!(matches!(err, UserError::InvalidUsername));
        }

        #[test]
        fn test_username_rejects_symbols() {
            let err = Username::new("john_doe".into()).unwrap_err();
            assert!(matches!(err, UserError::InvalidUsername));
        }

        #[test]
        fn test_username_rejects_spaces() {
            let err = Username::new("john doe".into()).unwrap_err();
            assert!(matches!(err, UserError::InvalidUsername));
        }

        #[test]
        fn test_username_alphanumeric_accepted() {
            let un = Username::new("abc123".into()).unwrap();
            assert_eq!(un.as_str(), "abc123");
        }
    }

    mod email_tests {
        use super::*;

        #[test]
        fn test_valid_email() {
            let email = EmailAddress::new("test@example.com".into()).unwrap();
            assert_eq!(email.as_str(), "test@example.com");
        }

        #[test]
        fn test_email_rejects_missing_at() {
            let err = EmailAddress::new("notanemail".into()).unwrap_err();
            assert!(matches!(err, UserError::InvalidEmail));
        }

        #[test]
        fn test_email_with_plus_accepted() {
            let email = EmailAddress::new("test+tag@example.com".into()).unwrap();
            assert_eq!(email.as_str(), "test+tag@example.com");
        }
    }

    mod user_tests {
        use super::*;

        #[test]
        fn test_user_new_success() {
            let user = User::new("alice", "alice@example.com").unwrap();
            assert_eq!(user.username.as_str(), "alice");
            assert_eq!(user.email.as_str(), "alice@example.com");
            assert!(user.is_active);
            assert_eq!(user.roles, vec![Role::BasicUser]);
        }

        #[test]
        fn test_user_has_any_role_match() {
            let user = User::new("bob", "bob@test.com").unwrap();
            assert!(user.has_any_role(&[Role::BasicUser]));
            assert!(!user.has_any_role(&[Role::Admin]));
        }

        #[test]
        fn test_user_has_any_role_multiple() {
            let user = User::new("bob", "bob@test.com").unwrap();
            assert!(user.has_any_role(&[Role::Admin, Role::Manager, Role::BasicUser]));
        }

        #[test]
        fn test_user_deactivate() {
            let mut user = User::new("charlie", "charlie@test.com").unwrap();
            assert!(user.is_active);
            user.deactivate().unwrap();
            assert!(!user.is_active);
        }

        #[test]
        fn test_user_deactivate_when_inactive_fails() {
            let mut user = User::new("dave", "dave@test.com").unwrap();
            assert!(user.is_active);

            user.deactivate().unwrap();
            assert!(!user.is_active);

            let err = user.deactivate().unwrap_err();
            assert!(!user.is_active);
            assert!(matches!(err, UserError::UserAlreadyDeactivated));
        }

        #[test]
        fn test_user_grant_admin() {
            let mut user = User::new("eve", "eve@test.com").unwrap();
            assert!(!user.has_any_role(&[Role::Admin]));

            user.grant_admin();
            assert!(user.has_any_role(&[Role::Admin]));
        }

        #[test]
        fn test_user_grant_admin_idempotent() {
            let mut user = User::new("frank", "frank@test.com").unwrap();
            user.grant_admin();
            user.grant_admin(); // second call should not panic
            assert!(user.has_any_role(&[Role::Admin]));
        }

        #[test]
        fn test_user_invalid_username_on_new() {
            let err = User::new("bad user!", "test@test.com").unwrap_err();
            assert!(matches!(err, UserError::InvalidUsername));
        }

        #[test]
        fn test_user_invalid_email_on_new() {
            let err = User::new("validuser", "notanemail").unwrap_err();
            assert!(matches!(err, UserError::InvalidEmail));
        }
    }

    mod authenticator_tests {
        use super::*;

        #[test]
        fn test_local_authenticator_creation() {
            let user_id = Uuid::new_v4();
            let auth = UserAuthenticator::new_local(user_id, "hashed_password".into());

            assert_eq!(auth.user_id, user_id);
            assert!(matches!(auth.provider, AuthProvider::Local));
            assert_eq!(auth.hashed_passwd, Some("hashed_password".into()));
            assert_eq!(auth.is_verified, Some(false));
            assert!(auth.provider_id.is_none());
        }

        #[test]
        fn test_oauth_authenticator_creation() {
            let user_id = Uuid::new_v4();
            let auth = UserAuthenticator::new_oauth(user_id, AuthProvider::Google, "google_123".into());

            assert_eq!(auth.user_id, user_id);
            assert!(matches!(auth.provider, AuthProvider::Google));
            assert_eq!(auth.provider_id, Some("google_123".into()));
            assert!(auth.hashed_passwd.is_none());
            assert!(auth.is_verified.is_none());
        }

        #[test]
        fn test_github_oauth_authenticator() {
            let user_id = Uuid::new_v4();
            let auth = UserAuthenticator::new_oauth(user_id, AuthProvider::GitHub, "gh_456".into());

            assert_eq!(auth.user_id, user_id);
            assert!(matches!(auth.provider, AuthProvider::GitHub));
            assert_eq!(auth.provider_id, Some("gh_456".into()));
        }
    }

    mod password_tests {
        use super::*;

        #[test]
        fn test_valid_password() {
            let pwd = Password::new("MyP@ssword123".into()).unwrap();
            assert_eq!(pwd.as_str(), "MyP@ssword123");
        }

        #[test]
        fn test_password_too_short() {
            let err = Password::new("Sh0rt!".into()).unwrap_err();
            assert!(matches!(err, UserError::InvalidPassword));
        }

        #[test]
        fn test_password_missing_number() {
            let err = Password::new("Password!!!".into()).unwrap_err();
            assert!(matches!(err, UserError::InvalidPassword));
        }

        #[test]
        fn test_password_missing_symbol() {
            let err = Password::new("Password12345".into()).unwrap_err();
            assert!(matches!(err, UserError::InvalidPassword));
        }

        #[test]
        fn test_password_with_space_rejected() {
            let err = Password::new("Pass word123!".into()).unwrap_err();
            assert!(matches!(err, UserError::InvalidPassword));
        }

        #[test]
        fn test_password_with_tab_rejected() {
            let err = Password::new("Pass\tword123!".into()).unwrap_err();
            assert!(matches!(err, UserError::InvalidPassword));
        }

        #[test]
        fn test_password_with_newline_rejected() {
            let err = Password::new("Pass\nword123!".into()).unwrap_err();
            assert!(matches!(err, UserError::InvalidPassword));
        }

        #[test]
        fn test_password_with_emoji_rejected() {
            let err = Password::new("Pass😀word123!".into()).unwrap_err();
            assert!(matches!(err, UserError::InvalidPassword));
        }

        #[test]
        fn test_password_with_unicode_rejected() {
            let err = Password::new("Pässword123!".into()).unwrap_err();
            assert!(matches!(err, UserError::InvalidPassword));
        }

        #[test]
        fn test_password_all_alphanumeric_rejected() {
            let err = Password::new("password12345".into()).unwrap_err();
            assert!(matches!(err, UserError::InvalidPassword));
        }
    }
}