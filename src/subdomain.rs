use cloudflare::framework::async_api;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Subdomain {
    pub subdomain: String,
    pub url: String,
}

pub async fn add(
    api_client: &async_api::Client,
    zone_identifier: &str,
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

    let content = "redirect.kuso.domains";
    log::info!("add CNAME: {}", content);
    let record = crate::dns::cname(&subdomain, content);
    crate::dns::create_record(api_client, zone_identifier, record.into()).await;

    let content = target_url;
    log::info!("add TXT: {}", content);
    let txt_name = "_kuso-domains-to.".to_string() + &subdomain;
    let record = crate::dns::txt(&txt_name, content);
    crate::dns::create_record(api_client, zone_identifier, record.into()).await;

    subdomain
}
