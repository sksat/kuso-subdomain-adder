use std::sync::*;

use lazy_static::lazy_static;

use actix_web::{middleware, web, App, HttpResponse, HttpServer, Result};

mod api;
mod config;
mod dns;
mod subdomain;

#[derive(Debug)]
struct Output {
    url: String,
    url_visual: String,
}

pub struct Data {
    api_client: dns::ProviderClient,
    subdomain: Option<subdomain::Subdomain>,
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use clap::Arg;

    let verstr = format!(
        "{} ({} {})",
        env!("CARGO_PKG_VERSION"),
        env!("VERGEN_GIT_SHA_SHORT"),
        env!("VERGEN_BUILD_DATE")
    );
    let ver: &str = &verstr;
    let matches = clap::App::new(env!("CARGO_PKG_NAME"))
        .version(ver)
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .default_value("./config.toml")
                .help("set config file")
                .takes_value(true),
        )
        .arg(
            Arg::new("debug")
                .long("debug")
                .short('d')
                .help("print debug information verbosely"),
        )
        .subcommand(clap::App::new("srv").about("start server"))
        .subcommand(
            clap::App::new("add")
                .about("add kuso subdomain")
                .arg(Arg::new("subdomain").required(true).help("subdomain"))
                .arg(Arg::new("target").required(true).help("target URL")),
        )
        .subcommand(
            clap::App::new("delete")
                .about("delete kuso subdomain")
                .arg(Arg::new("subdomain").required(true).help("subdomain")),
        )
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
    let data = config::cfg2data(cfg_file).unwrap();

    if let Some(_m) = matches.subcommand_matches("srv") {
        log::info!("kuso version {}", ver);

        let data = Arc::new(Mutex::new(data));
        HttpServer::new(move || {
            App::new()
                .wrap(middleware::Logger::default())
                .app_data(web::Data::new(data.clone()))
                .configure(app_config)
        })
        .bind("0.0.0.0:8101")?
        .run()
        .await?
    } else if let Some(m) = matches.subcommand_matches("add") {
        log::info!("add subdomain manually");

        let subdomain = m.value_of("subdomain").unwrap();
        let target_url = m.value_of("target").unwrap();
        let result_sd = subdomain::add(&data.api_client, subdomain, target_url).await;

        log::info!("result URL: http://{}.teleka.su", result_sd);
    } else if let Some(m) = matches.subcommand_matches("delete") {
        log::info!("delete subdomain manually");

        let subdomain = m.value_of("subdomain").unwrap();
        subdomain::delete(&data.api_client, subdomain).await;
    }

    Ok(())
}

fn app_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/result").route(web::get().to(page_result)))
            .service(api::add_subdomain),
    );
}

async fn index() -> Result<HttpResponse> {
    //let data = &data.lock().unwrap();
    //let context = &mut data.context;
    let mut context = tera::Context::new();
    let verstr = format!(
        "{} ({} {})",
        env!("CARGO_PKG_VERSION"),
        env!("VERGEN_GIT_SHA_SHORT"),
        env!("VERGEN_BUILD_DATE")
    );
    context.insert("version", &verstr);

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
    let verstr = format!(
        "{} ({} {})",
        env!("CARGO_PKG_VERSION"),
        env!("VERGEN_GIT_SHA_SHORT"),
        env!("VERGEN_BUILD_DATE")
    );
    context.insert("version", &verstr);

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
