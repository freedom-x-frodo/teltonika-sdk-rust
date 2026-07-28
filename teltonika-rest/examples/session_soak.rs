//! Long-run soak: polls past the 5-minute session expiry to exercise re-auth.
//!
//! RUTX_URL=https://192.168.123.1 RUTX_USER=admin RUTX_PASS=... \
//!   cargo run --example session_soak

use std::env;
use std::time::{Duration, Instant};

use teltonika_core::config::ConnConfig;
use teltonika_rest::auth::AuthType;
use teltonika_rest::client::RestClient;

const POLL_INTERVAL: Duration = Duration::from_secs(15);
const RUN_FOR: Duration = Duration::from_secs(7 * 60);

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = env::var("RUTX_URL")?;
    let user = env::var("RUTX_USER")?;
    let pass = env::var("RUTX_PASS")?;

    let config = ConnConfig::new(user, pass, endpoint);
    let client = RestClient::connect(config, AuthType::Session).await?;

    let started = Instant::now();
    let mut polls = 0u32;
    let mut failures = 0u32;

    println!("polling every {POLL_INTERVAL:?} for {RUN_FOR:?} (token TTL ~5min)");

    while started.elapsed() < RUN_FOR {
        polls += 1;
        let elapsed = started.elapsed().as_secs();

        match client.modems().status().await {
            Ok(modems) => {
                let first = modems.first().and_then(|m| m.rsrp);
                println!("[{elapsed:>3}s] poll {polls:>3} ok  modems={} rsrp={first:?}", modems.len());
            }
            Err(e) => {
                failures += 1;
                println!("[{elapsed:>3}s] poll {polls:>3} ERR {e}");
            }
        }

        tokio::time::sleep(POLL_INTERVAL).await;
    }

    println!("\n{polls} polls, {failures} failures over {:?}", started.elapsed());
    if failures == 0 {
        println!("re-auth held across the expiry boundary");
    }
    Ok(())
}