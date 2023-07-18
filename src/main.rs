use std::path::PathBuf;

use clap::{
    builder::{RangedU64ValueParser, TypedValueParser},
    Parser,
};
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

#[derive(Clone)]
struct DurationAsMinutes(time::Duration);

impl DurationAsMinutes {
    fn new(minutes: u64) -> Self {
        Self(time::Duration::from_secs(60 * minutes))
    }
}

impl clap::builder::ValueParserFactory for DurationAsMinutes {
    type Parser = clap::builder::ValueParser;

    fn value_parser() -> Self::Parser {
        RangedU64ValueParser::new().range(1..).map(Self::new).into()
    }
}

#[derive(Parser)]
struct RunCanary {
    #[arg(
        default_value = "credentials.json",
        short,
        long,
        help = "should contain username and password for SMTP server",
        id = "JSON-FILE"
    )]
    credentials_path: PathBuf,
    #[arg(short, long, help = "canary emails are sent from and to this mailbox")]
    email_address: Mailbox,
    #[arg(
        default_value = "5",
        short,
        long,
        help = "interval to wait between consecutive updates",
        id = "MINUTES"
    )]
    interval: DurationAsMinutes,
    #[arg(short, long, help = "hostname of SMTP server")]
    smtp_server: String,
}

#[tokio::main]
async fn main() {
    let RunCanary {
        credentials_path,
        email_address,
        interval,
        smtp_server,
    } = RunCanary::parse();

    let start_time = chrono::Local::now();

    let Credentials { username, password } = {
        let contents = std::fs::read_to_string(&credentials_path).unwrap();
        serde_json::from_str(&contents).unwrap()
    };

    // Create TLS transport on port 465
    let sender = AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_server)
        .unwrap()
        .credentials((username, password).into())
        .build();

    let hostname = hostname::get().unwrap().into_string().unwrap();
    let subject = format!("Internet connection canary message from {}", hostname);

    let mut interval = time::interval(interval.0);
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

        eprintln!("Sending email with body...\n{}\n\n", body);

        let email = Message::builder()
            .from(email_address.clone())
            .reply_to(email_address.clone())
            .to(email_address.clone())
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
