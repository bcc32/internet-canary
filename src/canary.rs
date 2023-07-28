use chrono::{DateTime, Local};
use lettre::{
    message::{header::ContentType, Mailbox},
    Message, SmtpTransport, Transport,
};

fn send_email(
    sender: &SmtpTransport,
    email_address: &Mailbox,
    hostname: &str,
    start_time: DateTime<Local>,
) {
    let current_time = chrono::Local::now();
    let (uptime_days, uptime_hours) = {
        let uptime = uptime_lib::get().unwrap();
        let secs = uptime.as_secs();
        let hours = secs / 3600;
        let days = hours / 24;
        (days, hours % 24)
    };

    let subject = format!("Internet connection canary message from {hostname}");

    let ip_address = ureq::get("https://api.ipify.org")
        .call()
        .map_err(|_| ())
        .and_then(|r| r.into_string().map_err(|_| ()))
        .unwrap_or_else(|()| "Error obtaining IP address".to_string());

    let body = format!(
        r#"<h2>Internet is UP for host {hostname}</h2>

<table>
<tr>
<td>Current time</td>
<td>{current_time}</td>
</tr>
<tr>
<td>Canary start time</td>
<td>{start_time}</td>
</tr>
<tr>
<td>Host uptime</td>
<td>{uptime_days}d {uptime_hours}h</td>
</tr>
<tr>
<td>IP address</td>
<td>{ip_address}</td>
</tr>
</table>
"#,
    );

    eprintln!("Sending email with body...\n{body}\n\n");

    let email = Message::builder()
        .from(email_address.clone())
        .reply_to(email_address.clone())
        .to(email_address.clone())
        .subject(&subject)
        .header(ContentType::TEXT_HTML)
        .body(body)
        .unwrap();

    // Send the email via remote relay
    match sender.send(&email) {
        Ok(response) => {
            if !response.is_positive() {
                eprintln!("Error from SMTP server:");
                for line in response.message() {
                    eprintln!("{line}");
                }
            }
        }
        Err(e) => {
            eprintln!("Error sending email: {e:?}");
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
