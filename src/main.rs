use std::fs;
use std::io::Read;
use std::sync::*;

use lazy_static::lazy_static;

use actix_web::{middleware, post, web, App, HttpResponse, HttpServer, Responder, Result};

use serde::{Deserialize, Serialize};

use cloudflare::endpoints::dns;
use cloudflare::framework::{
    async_api, async_api::ApiClient, auth::Credentials, response::ApiFailure, Environment,
    HttpApiClientConfig,
};

#[derive(Deserialize)]
struct Config {
    token: String,
    zone_identifier: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Subdomain {
    subdomain: String,
    url: String,
}

#[derive(Debug)]
struct Output {
    url: String,
    url_visual: String,
}

struct Data {
    api_client: async_api::Client,
    zone_identifier: String,
    subdomain: Option<Subdomain>,
    output: Option<Output>,
}

lazy_static! {
    pub static ref TEMPLATES: tera::Tera = {
        let mut tera = match tera::Tera::new("template/*") {
            Ok(t) => t,
            Err(e) => {
                log::error!("template parse error: {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec!["html"]);
        tera
    };
}

fn cfg2data(cfg_file: &str) -> Result<Data, ()> {
    let mut cfg_file = fs::File::open(cfg_file).unwrap();
    let mut config = String::new();
    cfg_file.read_to_string(&mut config).unwrap();
    let config: Config = toml::from_str(&config).unwrap();

    let credentials = Credentials::UserAuthToken {
        token: config.token,
    };

    let api_client = async_api::Client::new(
        credentials,
        HttpApiClientConfig::default(),
        Environment::Production,
    )
    .unwrap();

    Ok(Data {
        api_client,
        zone_identifier: config.zone_identifier,
        subdomain: None,
        output: None,
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use clap::Arg;

    let matches = clap::App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .default_value("./config.toml")
                .help("set config file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("debug")
                .long("debug")
                .short("d")
                .help("print debug information verbosely"),
        )
        .subcommand(clap::SubCommand::with_name("srv").about("start server"))
        .subcommand(
            clap::SubCommand::with_name("add")
                .about("add kuso subdomain")
                .arg(Arg::with_name("subdomain").required(true).help("subdomain"))
                .arg(Arg::with_name("target").required(true).help("target URL")),
        )
        .subcommand(clap::SubCommand::with_name("list"))
        .get_matches();

    let debug_level = if matches.is_present("debug") {
        "debug"
    } else {
        "info"
    };
    std::env::set_var("RUST_LOG", format!("kuso_subdomain_adder={}", debug_level));
    env_logger::init();
    log::debug!("debug mode");

    //println!("{}", punycode::encode("バーチャル六畳半").unwrap());

    let cfg_file = matches.value_of("config").unwrap();
    let data = cfg2data(cfg_file).unwrap();

    if let Some(_m) = matches.subcommand_matches("srv") {
        log::info!("kuso start(version {})", env!("CARGO_PKG_VERSION"));

        let data = Arc::new(Mutex::new(data));
        HttpServer::new(move || {
            App::new()
                .wrap(middleware::Logger::default())
                .data(data.clone())
                .configure(app_config)
        })
        .bind("0.0.0.0:8101")?
        .run()
        .await?
    } else if let Some(m) = matches.subcommand_matches("add") {
        log::info!("add subdomain manually");

        let subdomain = m.value_of("subdomain").unwrap();
        let target_url = m.value_of("target").unwrap();
        let result_sd = add_subdomain(
            &data.api_client,
            &data.zone_identifier,
            &subdomain,
            &target_url,
        )
        .await;

        log::info!("result URL: http://{}.teleka.su", result_sd);
    } else if let Some(_m) = matches.subcommand_matches("list") {
        log::info!("list");

        let params = dns::ListDnsRecordsParams {
            record_type: None,
            name: None,
            page: None,
            per_page: None,
            order: None,
            direction: None,
            search_match: None,
        };
        let _ = list_records(&data.api_client, &data.zone_identifier, params).await;
    }

    Ok(())
}

fn app_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/result").route(web::get().to(page_result)))
            .service(api_add_subdomain),
    );
}

async fn index() -> Result<HttpResponse> {
    //let data = &data.lock().unwrap();
    //let context = &mut data.context;
    let mut context = tera::Context::new();
    context.insert("version", env!("CARGO_PKG_VERSION"));

    let html = match TEMPLATES.render("index.html", &context) {
        Ok(s) => s,
        Err(e) => {
            log::error!("render error: {}", e);
            return Ok(HttpResponse::Ok()
                .content_type("text/html")
                .body("<script>alert('render error')</script>"));
        }
    };
    Ok(HttpResponse::Ok()
        .content_type("text/html; chaset=utf-8")
        .body(html))
}

async fn page_result(data: web::Data<Arc<Mutex<Data>>>) -> Result<HttpResponse> {
    let data = &data.lock().unwrap();
    //let context = &mut data.context;
    let mut context = tera::Context::new();
    context.insert("version", env!("CARGO_PKG_VERSION"));

    if let Some(output) = &data.output {
        log::info!("output {:?}", output);
        context.insert("url", &output.url);
        context.insert("url_visual", &output.url_visual);
        if let Some(subdomain) = &data.subdomain {
            context.insert("target_url", &subdomain.url);
            context.insert("share_text", &subdomain.subdomain);
        } else {
            log::error!("hoge");
        }
    }

    let html = match TEMPLATES.render("result.html", &context) {
        Ok(s) => s,
        Err(e) => {
            log::error!("render error: {}", e);
            return Ok(HttpResponse::Ok()
                .content_type("text/html")
                .body("<script>alert('render error')</script>"));
        }
    };
    Ok(HttpResponse::Ok()
        .content_type("text/html; chaset=utf-8")
        .body(html))
}

#[post("/api/add_subdomain")]
async fn api_add_subdomain(
    data: web::Data<Arc<Mutex<Data>>>,
    params: web::Form<Subdomain>,
) -> impl Responder {
    log::info!("[API] add_subdomain");

    let params = params.into_inner();

    let data = data.lock();
    if let Err(_e) = data {
        return HttpResponse::Conflict().body("kowareta");
    }

    let data = &mut data.unwrap();
    data.subdomain = Some(params.clone());

    let subdomain = add_subdomain(
        &data.api_client,
        &data.zone_identifier,
        &params.subdomain,
        &params.url,
    )
    .await;

    // final URL
    let protocol = "http://".to_string();
    let domain = ".teleka.su";
    let url = protocol.clone() + &subdomain + domain;
    let url_visual = protocol + &params.subdomain + domain;
    log::info!("URL: {}", url);

    //let data = &mut data.lock().unwrap();
    let out = Output { url, url_visual };
    data.output = Some(out);

    HttpResponse::Found()
        .append_header(("Location", "/result"))
        .finish()
}

async fn add_subdomain(
    api_client: &async_api::Client,
    zone_identifier: &str,
    subdomain: &str,
    target_url: &str,
) -> String {
    let subdomain = if subdomain.chars().all(|c| c.is_ascii_alphanumeric()) {
        log::info!("subdomain: {}", subdomain);
        subdomain.to_string()
    } else {
        let pcode = punycode::encode(&subdomain).unwrap();
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
    create_record(&api_client, &zone_identifier, record).await;

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
    create_record(&api_client, &zone_identifier, record).await;

    subdomain
}

async fn create_record(
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

async fn list_records(
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
