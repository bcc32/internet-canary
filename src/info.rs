use chrono::{DateTime, Local};

pub fn current(hostname: &str, start_time: DateTime<Local>) -> String {
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

    format!(
        r"<h2>Internet is UP for host {hostname}</h2>

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
",
    )
}
