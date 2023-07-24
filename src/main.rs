use std::{path::PathBuf, time};

use clap::{
    builder::{RangedU64ValueParser, TypedValueParser},
    Parser,
};
use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::SmtpTransport,
    Message, Transport,
};
use serde::{Deserialize, Serialize};

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
/// Start an Internet Canary, a process that periodically emails a specific
/// address to report the (outbound) Internet connectivity status of the host.
///
/// This can be useful to help diagnose issues on servers you do not have
/// immediate physical access to.
struct RunCanary {
    #[arg(
        default_value = "credentials.json",
        short,
        long,
        help = "should contain username and password for SMTP server",
        id = "JSON-FILE",
        env = "INTERNET_CANARY_CREDENTIALS_FILE"
    )]
    credentials_path: PathBuf,
    #[arg(
        short,
        long,
        help = "canary emails are sent from and to this mailbox",
        env = "INTERNET_CANARY_EMAIL"
    )]
    email_address: Mailbox,
    #[arg(
        default_value = "5",
        short,
        long,
        help = "interval to wait between consecutive updates",
        id = "MINUTES"
    )]
    interval: DurationAsMinutes,
    #[arg(
        short,
        long,
        help = "hostname of SMTP server",
        env = "INTERNET_CANARY_SMTP_SERVER"
    )]
    smtp_server: String,
}

fn main() {
    let RunCanary {
        credentials_path,
        email_address,
        interval: DurationAsMinutes(interval),
        smtp_server,
    } = RunCanary::parse();

    let start_time = chrono::Local::now();

    let Credentials { username, password } = {
        let contents = std::fs::read_to_string(&credentials_path).unwrap_or_else(|e| {
            panic!("Could not read credentials from {credentials_path:?}: {e:?}")
        });
        serde_json::from_str(&contents).unwrap()
    };

    // Create TLS transport on port 465
    let sender = SmtpTransport::relay(&smtp_server)
        .unwrap()
        .credentials((username, password).into())
        .build();

    let hostname = hostname::get().unwrap().into_string().unwrap();
    let subject = format!("Internet connection canary message from {hostname}");

    loop {
        let current_time = chrono::Local::now();
        let (uptime_days, uptime_hours) = {
            let uptime = uptime_lib::get().unwrap();
            let secs = uptime.as_secs();
            let hours = secs / 3600;
            let days = hours / 24;
            (days, hours % 24)
        };

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

        std::thread::sleep(interval);
    }
}
