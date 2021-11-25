use actix_web::web;
use actix_files::Files;


mod handler;

pub(super) fn route(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/upload").route(web::get().to(handler::upload)).route(web::post().to(handler::save_file)))
        .service(Files::new("/ex", "/root/ex").show_files_listing());
    
}