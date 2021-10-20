use serde::{Deserialize, Serialize};

use crate::dns;
use crate::dns::ProviderClientTrait;

#[derive(Serialize, Deserialize, Clone)]
pub struct Subdomain {
    pub subdomain: String,
    pub url: String,
}

fn str2punycode_str(s: &str) -> String {
    if s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        log::info!("subdomain: {}", s);
        s.to_string()
    } else {
        let pcode = punycode::encode(s).unwrap();
        log::info!("subdomain: {} -> {}", &s, &pcode);
        "xn--".to_string() + &pcode
    }
}

pub async fn add(
    api_client: &crate::dns::ProviderClient,
    subdomain: &str,
    target_url: &str,
) -> String {
    let subdomain = str2punycode_str(subdomain);

    let content = "redirect.kuso.domains";
    log::info!("add CNAME: {}", content);
    let record = dns::cname(&subdomain, content);
    api_client.create_record(record.into()).await;

    let content = target_url;
    log::info!("add TXT: {}", content);
    let txt_name = "_kuso-domains-to.".to_string() + &subdomain;
    let record = dns::txt(&txt_name, content);
    api_client.create_record(record.into()).await;

    subdomain
}

pub async fn delete(api_client: &crate::dns::ProviderClient, subdomain: &str) {
    let rname = str2punycode_str(subdomain);
    let rname = rname + ".teleka.su";
    let txt_name = "_kuso-domains-to.".to_string() + &rname;

    log::info!("delete CNAME record: {}", rname);
    api_client.delete_record(&rname).await;

    log::info!("delete TXT record: {}", txt_name);
    api_client.delete_record(&txt_name).await;
}
