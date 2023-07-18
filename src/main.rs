use tokio::time;

#[tokio::main]
async fn main() {
    // TODO: Every 5 minutes, send email to admin@bcc32.com, with a mail rule
    // that automatically trashes it.  Include:
    //
    // - hostname
    // - process start timestamp
    // - current timestamp
    // - host uptime
    // - ip address

    let mut interval = time::interval(time::Duration::from_secs(60));

    loop {
        interval.tick().await;
        println!("Hello, world!");
    }
}
