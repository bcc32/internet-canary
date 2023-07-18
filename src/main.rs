use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use serde::{Deserialize, Serialize};
use tokio::time;

#[derive(Deserialize, Serialize)]
struct Credentials {
    username: String,
    password: String,
}

#[tokio::main]
async fn main() {
    let start_time = chrono::Local::now();

    let Credentials { username, password } = {
        let contents = std::fs::read_to_string("credentials.json").unwrap();
        serde_json::from_str(&contents).unwrap()
    };

    // Create TLS transport on port 465
    let sender = AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.fastmail.com")
        .unwrap()
        .credentials((username, password).into())
        .build();

    let addr: Mailbox = "admin <admin@bcc32.com>".parse().unwrap();
    let hostname = hostname::get().unwrap().into_string().unwrap();
    let subject = format!("Internet connection canary message from {}", hostname);

    let mut interval = time::interval(time::Duration::from_secs(5 * 60));
    loop {
        interval.tick().await;

        let current_time = chrono::Local::now();
        let (uptime_days, uptime_hours) = {
            let uptime = uptime_lib::get().unwrap();
            let secs = uptime.as_secs();
            let hours = secs / 3600;
            let days = hours / 24;
            (days, hours % 24)
        };

        let ip_address = match reqwest::get("https://api.ipify.org").await {
            Err(_) => "Error obtaining IP address".to_string(),
            Ok(response) => response.text().await.unwrap(),
        };

        let body = format!(
            r#"Internet is UP for host {hostname}

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

        eprintln!("Sending email with body...\n{}\n\n", body);

        let email = Message::builder()
            .from(addr.clone())
            .reply_to(addr.clone())
            .to(addr.clone())
            .subject(&subject)
            .header(ContentType::TEXT_HTML)
            .body(body)
            .unwrap();

        // Send the email via remote relay
        let result = sender.send(email).await.unwrap();
        if !result.is_positive() {
            eprintln!("Error sending email:");
            for line in result.message() {
                eprintln!("{line}");
            }
        }
    }
}
