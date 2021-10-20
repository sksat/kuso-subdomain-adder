use std::sync::*;

use actix_web::{post, web, HttpResponse, Responder};

use crate::subdomain;

#[post("/api/add_subdomain")]
async fn add_subdomain(
    data: web::Data<Arc<Mutex<crate::Data>>>,
    params: web::Form<crate::subdomain::Subdomain>,
) -> impl Responder {
    log::info!("[API] add_subdomain");

    let params = params.into_inner();

    let data = data.lock();
    if let Err(_e) = data {
        return HttpResponse::Conflict().body("kowareta");
    }

    let data = &mut data.unwrap();
    data.subdomain = Some(params.clone());

    let subdomain = subdomain::add(&data.api_client, &params.subdomain, &params.url).await;

    // final URL
    let protocol = "http://".to_string();
    let domain = ".teleka.su";
    let url = protocol.clone() + &subdomain + domain;
    let url_visual = protocol + &params.subdomain + domain;
    log::info!("URL: {}", url);

    //let data = &mut data.lock().unwrap();
    let out = crate::Output { url, url_visual };
    data.output = Some(out);

    HttpResponse::Found()
        .append_header(("Location", "/result"))
        .finish()
}
