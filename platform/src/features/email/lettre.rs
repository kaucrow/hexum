use ::lettre::{
    Message,
    AsyncTransport,
    AsyncSmtpTransport,
    Tokio1Executor,
    message::{SinglePart, MultiPart},
    transport::smtp::authentication::Credentials,
};
use async_trait::async_trait;
use thiserror::Error;
use askama::Template;

use crate::{
    Config,
    prelude::*,
    features::user,
};
use super::*;

#[derive(Clone)]
pub struct LettreAdapter {
    pub mailer: AsyncSmtpTransport<Tokio1Executor>,
    pub frontend_url: String,
    pub from_addr: String,
}

impl LettreAdapter {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        let smtp_config = config.email.sender.smtp_config()?;

        let creds = Credentials::new(
            smtp_config.user.clone(),
            smtp_config.passwd.clone()
        );

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_config.host)
            .context("Failed to create SMTP relay.")?
            .credentials(creds)
            .port(smtp_config.port)
            .build();

        Ok(Self {
            mailer,
            frontend_url: config.frontend.url.clone(),
            from_addr: config.email.from.clone(),
        })
    }
}

#[derive(Template)]
#[template(path = "verification_email.html")]
struct VerificationEmailTemplate<'a> {
    code: &'a str,
    action_name: &'static str,
    description: &'static str,
}

impl LettreAdapter {
    async fn do_send_verification_email(&self, to: &user::EmailAddress, code: &str, context: VerificationContext) -> Result<(), LocalError> {
        let template = VerificationEmailTemplate {
            code,
            action_name: context.action_name(),
            description: context.description(),
        };
        let html_body = template.render()?;

        let subject = context.subject();

        let email = Message::builder()
            .from(self.from_addr.parse()?)
            .to(to.as_str().parse()?)
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::plain(format!("{} - Your verification code is: {}", subject, code))
                    )
                    .singlepart(
                        SinglePart::html(html_body)
                    )
            )?;

        self.mailer.send(email).await?;

        Ok(())
    }
}

#[async_trait]
impl Port for LettreAdapter {
    async fn send_verification_email(&self, to: &user::EmailAddress, code: &str, context: VerificationContext) -> Result<(), PortError> {
        Ok(self.do_send_verification_email(to, code, context).await?)
    }
}

#[derive(Error, Debug)]
pub enum LocalError {
    #[error(transparent)]
    LettreError(#[from] ::lettre::error::Error),
    #[error(transparent)]
    LettreSmtp(#[from] ::lettre::transport::smtp::Error),
    #[error(transparent)]
    LettreAddress(#[from] ::lettre::address::AddressError),
    #[error(transparent)]
    Askama(#[from] askama::Error),
}

impl From<LocalError> for PortError {
    fn from(e: LocalError) -> Self {
        PortError::Internal(e.to_string())
    }
}