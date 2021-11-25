use actix_web::web;
use actix_files::Files;


mod handler;
pub mod model;

pub(super) fn route(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/graphql").route(web::post().to(handler::graphql)))
        .service(web::resource("/playground").route(web::get().to(handler::playground)))
        .service(Files::new("/ex", "/root/ex").show_files_listing())
        .service(web::resource("/").route(web::get().to(handler::index)));
}