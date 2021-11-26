use actix_files::Files;
use actix_web::web;

mod handler;

pub(super) fn route(cfg: &mut web::ServiceConfig) {
    cfg.service( web::resource("/upload").route(web::post().to(handler::upload)))
       .service(Files::new("/ex", "/root/ex").show_files_listing());
}
