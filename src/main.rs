mod discord_canary;
mod email_canary;
mod info;

use std::{
    fs,
    path::PathBuf,
    thread::{self, JoinHandle},
    time,
};

use clap::{
    Parser,
    builder::{RangedU64ValueParser, TypedValueParser},
};
use lettre::{message::Mailbox, transport::smtp::SmtpTransport};
use log::{debug, info};
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
/// Start an email Internet Canary, a process that periodically emails a
/// specific address to report the (outbound) Internet connectivity status of
/// the host.
///
/// This can be useful to help diagnose issues on servers you do not have
/// immediate physical access to.
struct RunEmailCanary {
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
        default_value = "60",
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

fn run_email_canary(
    RunEmailCanary {
        credentials_path,
        email_address,
        interval: DurationAsMinutes(interval),
        smtp_server,
    }: RunEmailCanary,
) -> JoinHandle<()> {
    let sender = {
        let Credentials { username, password } = {
            let contents = std::fs::read_to_string(&credentials_path).unwrap_or_else(|e| {
                panic!("Could not read credentials from {credentials_path:?}: {e:?}")
            });
            serde_json::from_str(&contents).unwrap()
        };

        // Create TLS transport on port 465
        SmtpTransport::relay(&smtp_server)
            .unwrap()
            .credentials((username, password).into())
            .build()
    };

    thread::spawn(move || {
        email_canary::run_forever(&sender, &email_address, interval);
    })
}

#[derive(Parser)]
/// Start a Discord Internet Canary, a process that maintains a Discord bot in
/// some channel, whose status is updated to report the (outbound) Internet
/// connectivity status of the host.
///
/// This can be useful to help diagnose issues on servers you do not have
/// immediate physical access to.
struct RunDiscordCanary {
    #[arg(
        short,
        long,
        help = "should contain username and password for SMTP server",
        id = "TOKEN-FILE",
        env = "INTERNET_CANARY_DISCORD_TOKEN_FILE"
    )]
    token_path: PathBuf,
}

fn run_discord_canary(RunDiscordCanary { token_path }: RunDiscordCanary) -> JoinHandle<()> {
    let token = fs::read_to_string(&token_path)
        .unwrap_or_else(|e| panic!("Could not read token from {token_path:?}: {e}"));
    thread::spawn(move || discord_canary::run_forever_sync(discord_canary::Config { token }))
}

fn main() {
    env_logger::init();
    let mut handles = Vec::new();
    match RunDiscordCanary::try_parse() {
        Ok(discord_config) => {
            info!("Running Discord canary");
            let discord_handle = run_discord_canary(discord_config);
            handles.push(discord_handle);
        }
        Err(e) => {
            info!("Discord canary not configured; skipping");
            debug!("Could not configure Discord canary: {e}");
        }
    }
    match RunEmailCanary::try_parse() {
        Ok(email_config) => {
            info!("Running email canary");
            let email_handle = run_email_canary(email_config);
            handles.push(email_handle);
        }
        Err(e) => {
            info!("Email canary not configured; skipping");
            debug!("Could not configure email canary: {e}");
        }
    }
    for h in handles {
        h.join().unwrap();
    }
}
