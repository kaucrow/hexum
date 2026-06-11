use resend_rs::{
    Resend,
    types::CreateEmailBaseOptions
};
use async_trait::async_trait;
use thiserror::Error;
use askama::Template;

use crate::{
    Config,
    features::user,
};
use super::*;

#[derive(Template)]
#[template(path = "verification_email.html")]
struct VerificationEmailTemplate<'a> {
    code: &'a str,
    action_name: &'static str,
    description: &'static str,
}

#[derive(Clone)]
pub struct ResendAdapter {
    pub client: Resend,
    pub frontend_url: String,
    pub from_addr: String,
}

impl ResendAdapter {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        let resend_config = config.email.sender.resend_config()?;

        let client = Resend::new(&resend_config.api_key);

        Ok(Self {
            client,
            frontend_url: config.frontend.url.clone(),
            from_addr: config.email.from.clone(),
        })
    }
}

impl ResendAdapter {
    async fn do_send_verification_email(&self, to: &user::EmailAddress, code: &str, context: VerificationContext) -> Result<(), LocalError> {
        let template = VerificationEmailTemplate {
            code,
            action_name: context.action_name(),
            description: context.description(),
        };
        let html_body = template.render()?;

        let from = &self.from_addr;
        let to = [to.as_str()];
        let subject = context.subject();

        let email = CreateEmailBaseOptions::new(from, to, subject)
            .with_html(&html_body);

        self.client.emails.send(email).await?;

        Ok(())
    }
}

#[async_trait]
impl Port for ResendAdapter {
    async fn send_verification_email(&self, to: &user::EmailAddress, code: &str, context: VerificationContext) -> Result<(), PortError> {
        Ok(self.do_send_verification_email(to, code, context).await?)
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error(transparent)]
    Resend(#[from] resend_rs::Error),
    #[error(transparent)]
    Askama(#[from] askama::Error),
}

impl From<LocalError> for PortError {
    fn from(e: LocalError) -> Self {
        PortError::Internal(e.to_string())
    }
}