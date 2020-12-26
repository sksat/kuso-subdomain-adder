extern crate punycode;

use actix_web::http::StatusCode;
use actix_web::{get, middleware, web, App, HttpResponse, HttpServer, Responder, Result};

use serde::{Deserialize, Serialize};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //println!("{}", punycode::encode("バーチャル六畳半").unwrap());

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .configure(app_config)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

#[derive(Serialize, Deserialize)]
pub struct Subdomain {
    subdomain: String,
}

fn app_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/subdomain").route(web::post().to(handle_subdomain))),
    );
}

async fn index() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; chaset=utf-8")
        .body(include_str!("../static/index.html")))
}

async fn handle_subdomain(params: web::Form<Subdomain>) -> Result<HttpResponse> {
    let subdomain = if params.subdomain.chars().all(|c| c.is_ascii_alphanumeric()) {
        println!("{}", params.subdomain);
        params.subdomain.clone()
    } else {
        punycode::encode(&params.subdomain).unwrap()
    };
    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body(subdomain))
}
