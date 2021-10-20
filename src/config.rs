use std::fs;
use std::io::Read;

use serde::Deserialize;

use cloudflare::framework::{async_api, auth::Credentials, Environment, HttpApiClientConfig};

use crate::dns;
use crate::Data;

#[derive(Deserialize)]
struct Config {
    token: String,
    zone_identifier: String,
}

pub fn cfg2data(cfg_file: &str) -> Result<crate::Data, ()> {
    let mut cfg_file = fs::File::open(cfg_file).unwrap();
    let mut config = String::new();
    cfg_file.read_to_string(&mut config).unwrap();
    let config: Config = toml::from_str(&config).unwrap();

    let credentials = Credentials::UserAuthToken {
        token: config.token,
    };

    let cf_client = async_api::Client::new(
        credentials,
        HttpApiClientConfig::default(),
        Environment::Production,
    )
    .unwrap();
    let cf_client = dns::CloudflareClient {
        client: cf_client,
        zone_identifier: config.zone_identifier,
    };
    let api_client = dns::ProviderClient::Cloudflare(cf_client);

    Ok(Data {
        api_client,
        subdomain: None,
        output: None,
    })
}
