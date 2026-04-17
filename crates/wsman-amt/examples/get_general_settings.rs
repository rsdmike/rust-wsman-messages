// crates/wsman-amt/examples/get_general_settings.rs
//! Run against a real AMT endpoint:
//!     AMT_ENDPOINT=https://10.0.0.5:16993/wsman \
//!     AMT_USER=admin AMT_PASSWORD='P@ssw0rd' \
//!     cargo run -p wsman-amt --example get_general_settings

use wsman_amt::general::Settings;
use wsman_core::client::{Client, Credentials};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = std::env::var("AMT_ENDPOINT").expect("set AMT_ENDPOINT");
    let user = std::env::var("AMT_USER").expect("set AMT_USER");
    let pass = std::env::var("AMT_PASSWORD").expect("set AMT_PASSWORD");

    let client = Client::builder()
        .endpoint(endpoint)
        .credentials(Credentials::digest(user, pass))
        .accept_invalid_certs(true) // AMT ships self-signed certs
        .build()?;

    let settings = Settings::new(client);
    let resp = settings.get().await?;

    println!("host_name      : {}", resp.host_name);
    println!("domain_name    : {}", resp.domain_name);
    println!("digest_realm   : {}", resp.digest_realm);
    println!("network_enabled: {:?}", resp.amt_network_enabled);
    Ok(())
}
