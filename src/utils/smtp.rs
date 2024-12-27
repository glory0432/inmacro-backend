use crate::{utils::errors::ApiError, AppState};
use lettre::{
    message::{header, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};
use std::sync::Arc;
use tracing::info;

pub fn send_confirmation_code(
    destination: String,
    confirmation_code: String,
    state: Arc<AppState>,
) -> Result<(), ApiError> {
    let html_content = r#"  
    <!DOCTYPE html>  
    <html lang="en">  
    <head>  
        <meta charset="UTF-8">  
        <meta name="viewport" content="width=device-width, initial-scale=1.0">  
        <title>Confirmation Code</title>  
        <style>  
            .wrapper {  
                font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;  
                background-color: #b5b5b5;  
                width: 100%;  
                height: 100%;
                top: 0;
                position: absolute;
                box-sizing: border-box;  
                padding: 50px;
                margin: 0;
            }  
            .container {  
                max-width: 600px;  
                margin: 0 auto;  
                background-color: #ffffff;  
                border-radius: 8px;  
                overflow: hidden;  
                box-shadow: 0 4px 10px rgba(0, 0, 0, 0.1);  
            }  
            .header {  
                background-color: #3d8bbe;  
                color: white;  
                padding: 30px;  
                text-align: center;  
                font-size: 28px;  
            }  
            .content {  
                padding: 20px;  
            }  
            .content p {  
                line-height: 1.6;  
                color: #333;  
            }  
            .code {  
                font-size: 28px;  
                font-weight: bold;  
                color: #3d8bbe;  
                background-color: #f5f5f5;  
                padding: 10px;  
                text-align: center;  
                border-radius: 5px;  
                margin: 20px 0;  
            }  
            .footer {  
                font-size: 12px;  
                color: #777;  
                text-align: center;  
                padding: 10px;  
                background-color: #e0e0e0;  
            }  
            .footer a {  
                color: #3d8bbe;  
                text-decoration: none;  
            }  
        </style>  
    </head>  
    <body>  
        <div class="wrapper">  
            <div class="container">  
                <div class="header">  
                    Thanks for signing up
                </div>  
                <div class="content">  
                    <p>Hi there,</p>  
                    <p>Thank you for signing up! Please use the following confirmation code to complete your registration:</p>  
                    <div class="code">{{CONFIRMATION_CODE}}</div>  
                    <p>If you didn’t request this email, you can safely ignore it.</p>  
                </div>  
                <div class="footer">  
                    <p>Visit our website: <a href="https://www.example.com">www.example.com</a></p>  
                    <p>Contact us: <a href="mailto:support@example.com">support@example.com</a></p>  
                    <p>&copy; 2024 Your Company Name. All rights reserved.</p>  
                </div>  
            </div>  
        </div>  
    </body>  
    </html>  
    "#.replace("{{CONFIRMATION_CODE}}", &confirmation_code);

    let email = Message::builder()
        .from(
            format!("InMacro <{}>", state.env.smtp_sender_email)
                .parse()
                .unwrap(),
        )
        .to(format!("Receiver <{destination}>").parse().unwrap())
        .subject("Confirmation Code to Sign Up")
        .multipart(
            MultiPart::alternative() // This is composed of two parts.
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(String::from("Hello from Lettre! A mailer library for Rust")), // Every message should have a plain text fallback.
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(String::from(html_content)),
                ),
        )?;
    let creds = Credentials::new(
        state.env.smtp_username.clone(),
        state.env.smtp_password.clone(),
    );
    let mailer = SmtpTransport::starttls_relay("smtp-relay.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();
    match mailer.send(&email) {
        Err(e) => {
            info!("{:?}", e);
            return Err(ApiError::EmailSendError(e));
        }
        Ok(_) => Ok(()),
    }
}
