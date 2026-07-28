//! Vertical slice against a live RUTX50: connect -> auth -> call.
//!
//! RUTX_HOST=192.168.123.1 RUTX_USER=admin RUTX_PASS=... cargo run --example probe

use std::env;

use teltonika_core::config::ConnConfig;
use teltonika_rest::auth::AuthType;
use teltonika_rest::client::RestClient;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = env::var("RUTX_URL")?;
    let user = env::var("RUTX_USER")?;
    let pass = env::var("RUTX_PASS")?;

    println!("connecting to {endpoint} ...");
    let config = ConnConfig::new(user, pass, endpoint);
    let client = RestClient::connect(config, AuthType::Session).await?;
    println!("authenticated");

    let usage = client.system().usage_status().await?;
    println!("{usage:#?}");

    // The two questions the live device is here to answer.
    if let Some(load) = &usage.load {
        println!(
            "\nload units check -- min1={:?} loadavg={:?}",
            load.min1, usage.loadavg
        );
        println!("  min1 < 100  => plain loadavg, spec's \"%\" wording is wrong");
        println!("  min1 > 1000 => raw kernel fixed-point, needs /65536");
    }
    if let Some(mem) = &usage.memory {
        println!(
            "ram {:?}/{:?} MB ({:?}%)",
            mem.ram_used, mem.ram_total, mem.ram_percentage
        );
    }

    Ok(())
}
