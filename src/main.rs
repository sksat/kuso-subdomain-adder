use std::fs;
use std::io::Read;
use std::sync::*;

use lazy_static::lazy_static;

use actix_web::{middleware, web, App, HttpResponse, HttpServer, Result};

use serde::{Deserialize, Serialize};

use cloudflare::endpoints::dns;
use cloudflare::framework::{
    async_api,
    async_api::ApiClient,
    auth::Credentials,
    response::{ApiFailure, ApiResponse, ApiResult},
    Environment, HttpApiClientConfig,
};

#[derive(Deserialize)]
struct Config {
    token: String,
    zone_identifier: String,
}

struct Data {
    api_client: async_api::Client,
    zone_identifier: String,
    context: tera::Context,
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "kuso_subdomain_adder=info");
    env_logger::init();

    log::info!("kuso start");
    //println!("{}", punycode::encode("バーチャル六畳半").unwrap());

    let mut cfg_file = fs::File::open("./config.toml")?;
    let mut config = String::new();
    cfg_file.read_to_string(&mut config)?;
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

    let data = Data {
        api_client,
        zone_identifier: config.zone_identifier,
        context: tera::Context::new(),
    };
    let data = Arc::new(Mutex::new(data));

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(data.clone())
            .configure(app_config)
    })
    .bind("0.0.0.0:8101")?
    .run()
    .await
}

#[derive(Serialize, Deserialize)]
pub struct Subdomain {
    subdomain: String,
    url: String,
}

fn app_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/subdomain").route(web::post().to(handle_subdomain))),
    );
}

async fn index(data: web::Data<Arc<Mutex<Data>>>) -> Result<HttpResponse> {
    let data = &mut data.lock().unwrap();
    let context = &mut data.context;
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

async fn handle_subdomain(
    data: web::Data<Arc<Mutex<Data>>>,
    params: web::Form<Subdomain>,
) -> Result<HttpResponse> {
    let subdomain = if params.subdomain.chars().all(|c| c.is_ascii_alphanumeric()) {
        log::info!("subdomain: {}", params.subdomain);
        params.subdomain.clone()
    } else {
        let pcode = punycode::encode(&params.subdomain).unwrap();
        log::info!("subdomain: {} -> {}", &params.subdomain, &pcode);
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
    create_records(&data.lock().unwrap(), record).await;

    let content = params.url.clone();
    log::info!("add TXT: {}", content);
    let txt_name = "_kuso-domains-to.".to_string() + &subdomain;
    let record = dns::CreateDnsRecordParams {
        name: &txt_name,
        content: dns::DnsContent::TXT { content },
        priority: None,
        proxied: None,
        ttl: None,
    };
    create_records(&data.lock().unwrap(), record).await;

    // final URL
    let protocol = "http://".to_string();
    let domain = ".teleka.su";
    let url = protocol.clone() + &subdomain + domain;
    let url_visual = protocol + &params.subdomain + domain;
    log::info!("URL: {}", url);

    //let html = format!("<h2>URL: <a href=\"{}\">{}</a></h2>", url, url);
    //Ok(HttpResponse::Ok().content_type("text/html").body(html))

    let data = &mut data.lock().unwrap();
    let context = &mut data.context;
    context.insert("version", env!("CARGO_PKG_VERSION"));
    context.insert("url", &url);
    context.insert("url_visual", &url_visual);
    context.insert("target_url", &params.url);
    context.insert("share_text", &params.subdomain);

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

async fn create_records(data: &Data, params: dns::CreateDnsRecordParams<'_>) {
    let zone_identifier = &data.zone_identifier;
    let cdr = dns::CreateDnsRecord {
        zone_identifier,
        params,
    };
    let response = data.api_client.request(&cdr).await;
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

fn print_response<T: ApiResult>(response: ApiResponse<T>) {
    match response {
        Ok(success) => log::info!("Success: {:#?}", success),
        Err(e) => match e {
            ApiFailure::Error(status, errors) => {
                log::info!("HTTP {}:", status);
                for err in errors.errors {
                    log::error!("Error {}: {}", err.code, err.message);
                    for (k, v) in err.other {
                        log::error!("{}: {}", k, v);
                    }
                }
                for (k, v) in errors.other {
                    log::error!("{}: {}", k, v);
                }
            }
            ApiFailure::Invalid(reqwest_err) => log::error!("Error: {}", reqwest_err),
        },
    }
}
