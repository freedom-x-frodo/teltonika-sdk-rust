// teltonika-rest/examples/gps.rs
//! Reads /gps/global and /gps/position/status from a live RUTX50.
//!
//! RUTX_URL=https://192.168.123.1 RUTX_USER=admin RUTX_PASS=... cargo run --example gps

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

    println!("{:#?}\n", client.gps().global().await?);
    println!("{:#?}", client.gps().position_status().await?);

    Ok(())
}