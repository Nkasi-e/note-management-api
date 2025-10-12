use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use tracing::{info, error};

use crate::config::settings::EmailConfig;

#[derive(Clone)]
pub struct EmailService {
    config: EmailConfig,
}

impl EmailService {
    pub fn new(config: EmailConfig) -> Self {
        Self { config }
    }

    pub async fn send_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let from_email: Mailbox = self.config.from_email.parse()?;
        let to_email: Mailbox = to.parse()?;

        let email = Message::builder()
            .from(from_email)
            .to(to_email)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body.to_string())?;

        let creds = Credentials::new(
            self.config.username.clone(),
            self.config.password.clone(),
        );

        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::relay(&self.config.smtp_host)?
                .port(self.config.smtp_port)
                .credentials(creds)
                .build();

        match mailer.send(email).await {
            Ok(_) => {
                info!("Email sent successfully to: {}", to);
                Ok(())
            }
            Err(e) => {
                error!("Failed to send email to {}: {}", to, e);
                Err(Box::new(e))
            }
        }
    }

    /// Generate welcome email content
    pub fn generate_welcome_email_content(&self, user_name: &str) -> (String, String) {
        let subject = "Welcome to Note Task API!".to_string();
        let body = format!(
            r#"
            <html>
                <body>
                    <h2>Welcome, {}!</h2>
                    <p>Thank you for joining our note-taking application.</p>
                    <p>You can now start creating and managing your tasks.</p>
                    <br>
                    <p>Best regards,<br>The Note Task Team</p>
                </body>
            </html>
            "#,
            user_name
        );
        (subject, body)
    }

    /// Send a simple welcome email
    pub async fn send_welcome_email(&self, to: &str, user_name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (subject, body) = self.generate_welcome_email_content(user_name);
        self.send_email(to, &subject, &body).await
    }

    /// Send a task reminder email
    pub async fn send_task_reminder(&self, to: &str, task_title: &str, due_date: Option<&str>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let subject = "Task Reminder";
        let due_info = if let Some(date) = due_date {
            format!("<p><strong>Due Date:</strong> {}</p>", date)
        } else {
            String::new()
        };

        let body = format!(
            r#"
            <html>
                <body>
                    <h2>Task Reminder</h2>
                    <p>This is a reminder for your task:</p>
                    <h3>{}</h3>
                    {}
                    <p>Don't forget to complete it!</p>
                    <br>
                    <p>Best regards,<br>The Note Task Team</p>
                </body>
            </html>
            "#,
            task_title,
            due_info
        );

        self.send_email(to, subject, &body).await
    }
}
