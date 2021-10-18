

use cloudflare::endpoints::dns;
use cloudflare::framework::{
    async_api,
};

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

    let content = "redirect.kuso.domains".to_string();
    log::info!("add CNAME: {}", content);
    let record = dns::CreateDnsRecordParams {
        name: &subdomain,
        content: dns::DnsContent::CNAME { content },
        priority: None,
        proxied: None,
        ttl: None,
    };
    crate::create_record(api_client, zone_identifier, record).await;

    let content = target_url.to_string();
    log::info!("add TXT: {}", content);
    let txt_name = "_kuso-domains-to.".to_string() + &subdomain;
    let record = dns::CreateDnsRecordParams {
        name: &txt_name,
        content: dns::DnsContent::TXT { content },
        priority: None,
        proxied: None,
        ttl: None,
    };
    crate::create_record(api_client, zone_identifier, record).await;

    subdomain
}
