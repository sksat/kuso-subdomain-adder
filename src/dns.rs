//use domain::base::record::Record;
//use domain::master::entry::MasterRecord;
use domain::rdata::MasterRecordData;

use async_trait::async_trait;

use cloudflare::endpoints::dns;
use cloudflare::endpoints::dns::CreateDnsRecordParams;
use cloudflare::framework::async_api;
use cloudflare::framework::{async_api::ApiClient, response::ApiFailure};

type RecordData<'a> = MasterRecordData<bytes::Bytes, &'a str>;
type RecordImpl<'a> = domain::base::record::Record<&'a str, RecordData<'a>>;

pub struct Record<'a>(RecordImpl<'a>);

#[async_trait]
pub trait ProviderClientTrait {
    async fn create_record(&self, record: Record<'_>);
    async fn delete_record(&self, rname: &str);
}

pub enum ProviderClient {
    Cloudflare(CloudflareClient),
}

#[async_trait]
impl ProviderClientTrait for ProviderClient {
    async fn create_record(&self, record: Record<'_>) {
        match &self {
            ProviderClient::Cloudflare(cf) => cf.create_record(record).await,
        }
    }

    async fn delete_record(&self, rname: &str) {
        match &self {
            ProviderClient::Cloudflare(cf) => cf.delete_record(rname).await,
        }
    }
}

pub struct CloudflareClient {
    pub client: cloudflare::framework::async_api::Client,
    pub zone_identifier: String,
}

#[async_trait]
impl ProviderClientTrait for CloudflareClient {
    async fn create_record(&self, record: Record<'_>) {
        let zone_identifier = &self.zone_identifier;
        let params = record.into();

        let cdr = dns::CreateDnsRecord {
            zone_identifier,
            params,
        };
        let response = &self.client.request(&cdr).await;
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

    async fn delete_record(&self, name: &str) {
        let zone_identifier = &self.zone_identifier;

        let name = Some(name.to_string());
        let lparam = dns::ListDnsRecordsParams {
            record_type: None,
            name,
            page: None,
            per_page: None,
            order: None,
            direction: None,
            search_match: None,
        };
        let ldr = dns::ListDnsRecords {
            zone_identifier,
            params: lparam,
        };
        let res = &self.client.request(&ldr).await;
        let res = &res.as_ref().unwrap();
        let records = &res.result;
        if records.is_empty() {
            // TODO: return Err
            return;
        }
        assert_eq!(records.len(), 1);
        let r = records.into_iter().nth(0).unwrap();

        let ddr = dns::DeleteDnsRecord {
            zone_identifier,
            identifier: &r.id,
        };
        log::info!("{:?}", ddr);
        let res = &self.client.request(&ddr).await;

        match res {
            Ok(success) => log::info!("success: {:?}", success),
            Err(e) => match e {
                ApiFailure::Error(status, err) => {
                    log::error!("HTTP {}: {:?}", status, err);
                }
                ApiFailure::Invalid(req_err) => log::error!("Error: {}", req_err),
            },
        }
    }
}

pub fn cname<'a>(rname: &'a str, cname: &'a str) -> Record<'a> {
    let class = domain::base::iana::class::Class::In; // internet
    let cname = domain::rdata::rfc1035::Cname::new(cname);
    let rd: RecordData = cname.into();
    let record = domain::base::record::Record::new(rname, class, 0, rd);
    record.into()
}

pub fn txt<'a>(rname: &'a str, txt: &'a str) -> Record<'a> {
    let class = domain::base::iana::class::Class::In; // internet
    let txt = domain::rdata::Txt::from_slice(txt.as_bytes()).unwrap();
    let rd: RecordData = txt.into();
    let record = domain::base::record::Record::new(rname, class, 0, rd);
    record.into()
}

#[deprecated(since = "0.4.0", note = "list subcommand removed")]
#[allow(dead_code)]
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

impl<'a> From<RecordImpl<'a>> for Record<'a> {
    fn from(r: RecordImpl<'a>) -> Record<'a> {
        Record(r)
    }
}

impl<'a> From<Record<'a>> for cloudflare::endpoints::dns::DnsContent {
    fn from(r: Record) -> cloudflare::endpoints::dns::DnsContent {
        use cloudflare::endpoints::dns::DnsContent;

        let r = r.0;
        let data = r.data();
        match data {
            RecordData::Cname(cn) => DnsContent::CNAME {
                content: cn.cname().to_string(),
            },
            RecordData::Txt(txt) => DnsContent::TXT {
                content: std::str::from_utf8(txt.text::<bytes::Bytes>().unwrap().as_ref())
                    .unwrap()
                    .to_string(),
            },
            _ => todo!(),
        }
    }
}

impl<'a> From<Record<'a>> for CreateDnsRecordParams<'a> {
    fn from(r: Record<'a>) -> CreateDnsRecordParams<'a> {
        let ttl = r.0.ttl();
        let ttl = if ttl == 0 { None } else { Some(ttl) };

        let name = <&str>::clone(r.0.owner());
        let content = r.into();

        CreateDnsRecordParams {
            ttl,
            priority: None,
            proxied: None,
            name,
            content,
        }
    }
}

#[test]
fn convert_record() {
    use cloudflare::endpoints::dns::DnsContent;
    let class = domain::base::iana::class::Class::In; // internet

    let cname = domain::rdata::Cname::new("cname");
    let rd: RecordData = cname.into();
    let record = domain::base::record::Record::new("rcname", class, 0, rd);
    let record: Record = record.into();
    let params: CreateDnsRecordParams = record.into();

    println!("{:?}", params);
    assert_eq!(params.name, "rcname");
    assert!(
        matches!(params.content, DnsContent::CNAME { content } if content == "cname".to_string())
    );

    let txt = "txt".to_string();
    let txt = domain::rdata::Txt::from_slice(txt.as_bytes()).unwrap();
    let rd: RecordData = txt.into();
    let record = domain::base::record::Record::new("rtname", class, 0, rd);
    let record: Record = record.into();
    let params: CreateDnsRecordParams = record.into();

    println!("{:?}", params);
    assert_eq!(params.name, "rtname");
    assert!(matches!(params.content, DnsContent::TXT{ content } if content == "txt".to_string()));
}
