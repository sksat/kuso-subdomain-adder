//use domain::base::record::Record;
//use domain::master::entry::MasterRecord;
use domain::rdata::MasterRecordData;

use cloudflare::endpoints::dns::CreateDnsRecordParams;

type RecordData<'a> = MasterRecordData<&'a str, &'a str>;
type RecordImpl<'a> = domain::base::record::Record<&'a str, RecordData<'a>>;

struct Record<'a>(RecordImpl<'a>);

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
            MasterRecordData::Cname::<&str, &str>(cn) => DnsContent::CNAME {
                content: cn.cname().to_string(),
            },
            _ => unreachable!(),
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
    let class = domain::base::iana::class::Class::In; // internet
    let cname = domain::rdata::rfc1035::Cname::new("cname");
    let rd: MasterRecordData<&str, &str> = cname.into();
    let record = domain::base::record::Record::new("rname", class, 0, rd);
    let record: Record = record.into();
    let params: CreateDnsRecordParams = record.into();
    println!("{:?}", params);
}
