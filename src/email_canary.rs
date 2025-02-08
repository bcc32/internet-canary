use chrono::{DateTime, Local};
use lettre::{
    message::{header::ContentType, Mailbox},
    Message, SmtpTransport, Transport,
};
use log::{debug, error, info};

fn send_email(
    sender: &SmtpTransport,
    email_address: &Mailbox,
    hostname: &str,
    start_time: DateTime<Local>,
) {
    let subject = format!("Internet connection canary message from {hostname}");

    let body = super::info::current(hostname, start_time);

    info!("Sending email to {email_address}");
    debug!("Email body:\n{body}");

    let email = Message::builder()
        .from(email_address.clone())
        .reply_to(email_address.clone())
        .to(email_address.clone())
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(body)
        .unwrap();

    // Send the email via remote relay
    match sender.send(&email) {
        Ok(response) => {
            if !response.is_positive() {
                let message = response.message().collect::<Vec<_>>().join("\n");
                error!("Error from SMTP server:\n{message}");
            }
        }
        Err(e) => {
            error!("Error sending email: {e:?}");
        }
    }
}

pub fn run_forever(sender: &SmtpTransport, email_address: &Mailbox, interval: std::time::Duration) {
    let start_time = chrono::Local::now();
    let hostname = hostname::get().unwrap().into_string().unwrap();

    loop {
        send_email(sender, email_address, &hostname, start_time);
        std::thread::sleep(interval);
    }
}
