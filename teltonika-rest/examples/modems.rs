//! Reads /modems/status from a live RUTX50.
//!
//! RUTX_HOST=192.168.123.1 RUTX_USER=admin RUTX_PASS=... cargo run --example modems

use std::env;

use teltonika_core::config::ConnConfig;
use teltonika_rest::auth::AuthType;
use teltonika_rest::client::RestClient;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = env::var("RUTX_URL")?;
    let user = env::var("RUTX_USER")?;
    let pass = env::var("RUTX_PASS")?;

    let config = ConnConfig::new(user, pass, endpoint);
    let client = RestClient::connect(config, AuthType::Session).await?;

    let modems = client.modems().status().await?;
    println!("{} modem(s)\n", modems.len());
    for modem in &modems {
        println!("{modem:#?}\n");
    }

    Ok(())
}