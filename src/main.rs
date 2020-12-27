use std::fs;
use std::io::Read;
use std::sync::*;

use actix_web::http::StatusCode;
use actix_web::{get, middleware, web, App, HttpResponse, HttpServer, Responder, Result};

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
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
    Ok(HttpResponse::Ok()
        .content_type("text/html; chaset=utf-8")
        .body(include_str!("../static/index.html")))
}

async fn handle_subdomain(
    data: web::Data<Arc<Mutex<Data>>>,
    params: web::Form<Subdomain>,
) -> Result<HttpResponse> {
    let subdomain = if params.subdomain.chars().all(|c| c.is_ascii_alphanumeric()) {
        println!("{}", params.subdomain);
        params.subdomain.clone()
    } else {
        let pcode = punycode::encode(&params.subdomain).unwrap();
        "xn--".to_string() + &pcode
    };

    // add CNAME record
    let record = dns::CreateDnsRecordParams {
        name: &subdomain,
        content: dns::DnsContent::CNAME {
            content: "redirect.kuso.domains".to_string(),
        },
        priority: None,
        proxied: None,
        ttl: None,
    };
    create_records(&data.lock().unwrap(), record).await;

    // add TXT record
    let txt_name = "_kuso-domains-to.".to_string() + &subdomain;
    let record = dns::CreateDnsRecordParams {
        name: &txt_name,
        content: dns::DnsContent::TXT {
            content: params.url.clone(),
        },
        priority: None,
        proxied: None,
        ttl: None,
    };
    create_records(&data.lock().unwrap(), record).await;

    // final URL
    let url = "http://".to_string() + &subdomain + ".teleka.su";
    let html = format!("<h2>URL: <a href=\"{}\">{}</a></h2>", url, url);

    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

async fn create_records(data: &Data, params: dns::CreateDnsRecordParams<'_>) {
    let zone_identifier = &data.zone_identifier;
    let cdr = dns::CreateDnsRecord {
        zone_identifier,
        params,
    };
    let response = data.api_client.request(&cdr).await;
    match response {
        Ok(success) => println!("success: {:?}", success),
        Err(e) => match e {
            ApiFailure::Error(status, errors) => {
                println!("HTTP {}", status);
            }
            ApiFailure::Invalid(req_err) => println!("Error: {}", req_err),
        },
    }
    //print_response(response);
}

fn print_response<T: ApiResult>(response: ApiResponse<T>) {
    match response {
        Ok(success) => println!("Success: {:#?}", success),
        Err(e) => match e {
            ApiFailure::Error(status, errors) => {
                println!("HTTP {}:", status);
                for err in errors.errors {
                    println!("Error {}: {}", err.code, err.message);
                    for (k, v) in err.other {
                        println!("{}: {}", k, v);
                    }
                }
                for (k, v) in errors.other {
                    println!("{}: {}", k, v);
                }
            }
            ApiFailure::Invalid(reqwest_err) => println!("Error: {}", reqwest_err),
        },
    }
}
