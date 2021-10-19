use serde::{Deserialize, Serialize};

use crate::dns::ProviderClientTrait;

#[derive(Serialize, Deserialize, Clone)]
pub struct Subdomain {
    pub subdomain: String,
    pub url: String,
}

pub async fn add(
    api_client: &crate::dns::ProviderClient,
    subdomain: &str,
    target_url: &str,
) -> String {
    let subdomain = if subdomain.chars().all(|c| c.is_ascii_alphanumeric()) {
        log::info!("subdomain: {}", subdomain);
        subdomain.to_string()
    } else {
        let pcode = punycode::encode(subdomain).unwrap();
        log::info!("subdomain: {} -> {}", &subdomain, &pcode);
        "xn--".to_string() + &pcode
    };

    // TODO: remove
    let cf_client = match api_client {
        crate::dns::ProviderClient::Cloudflare(cf) => cf,
        _ => unreachable!(),
    };

    let content = "redirect.kuso.domains";
    log::info!("add CNAME: {}", content);
    let record = crate::dns::cname(&subdomain, content);
    cf_client.create_record(record.into()).await;

    let content = target_url;
    log::info!("add TXT: {}", content);
    let txt_name = "_kuso-domains-to.".to_string() + &subdomain;
    let record = crate::dns::txt(&txt_name, content);
    cf_client.create_record(record.into()).await;

    subdomain
}
