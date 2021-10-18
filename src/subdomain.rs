use cloudflare::endpoints::dns;
use cloudflare::framework::async_api;

use cloudflare::framework::{async_api::ApiClient, response::ApiFailure};

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
    create_record(api_client, zone_identifier, record).await;

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
    create_record(api_client, zone_identifier, record).await;

    subdomain
}

pub async fn create_record(
    api_client: &async_api::Client,
    zone_identifier: &str,
    params: dns::CreateDnsRecordParams<'_>,
) {
    let zone_identifier = zone_identifier;
    let cdr = dns::CreateDnsRecord {
        zone_identifier,
        params,
    };
    let response = api_client.request(&cdr).await;
    match response {
        Ok(success) => log::info!("success: {:?}", success),
        Err(e) => match e {
            ApiFailure::Error(status, err) => {
                log::error!("HTTP {}: {:?}", status, err);
            }
            ApiFailure::Invalid(req_err) => log::error!("Error: {}", req_err),
        },
    }
}

pub async fn list_records(
    api_client: &async_api::Client,
    zone_identifier: &str,
    params: dns::ListDnsRecordsParams,
) {
    let ldr = dns::ListDnsRecords {
        zone_identifier,
        params,
    };

    let response = api_client.request(&ldr).await;
    match response {
        Ok(success) => {
            //log::info!("success: {:?}", success);
            let record: Vec<dns::DnsRecord> = success.result;
            for r in record {
                log::info!("{:?}", r);
            }
        }
        Err(e) => match e {
            ApiFailure::Error(status, err) => {
                log::error!("HTTP {}: {:?}", status, err);
            }
            ApiFailure::Invalid(req_err) => log::error!("Error: {}", req_err),
        },
    }
}
