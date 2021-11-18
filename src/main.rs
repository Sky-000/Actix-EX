#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

mod cli_args;
mod database;
mod errors;
mod graphql;
mod jwt;
mod schema;
mod user;

use std::fs::File;
use std::io::BufReader;

use actix_cors::Cors;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use rustls::internal::pemfile::{certs, rsa_private_keys};
use rustls::{NoClientAuth, ServerConfig};
use time::Duration;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Gets enviroment variables from `.env.example`
    dotenv::dotenv().ok();

    // Initiates error logger
    env_logger::init();

    // Sets options to enviroment variables
    let opt = {
        use structopt::StructOpt;
        cli_args::Opt::from_args()
    };

    // Database
    let pool = database::pool::establish_connection(opt.clone());
    let schema = std::sync::Arc::new(crate::graphql::model::create_schema());

    // Authorisation
    let domain = opt.domain.clone();
    let cookie_secret_key = opt.auth_secret_key.clone();
    let secure_cookie = opt.secure_cookie;
    let auth_duration = Duration::hours(i64::from(opt.auth_duration_in_hour));

    // Server port
    let port = opt.port;

    // load ssl keys
    let mut config = ServerConfig::new(NoClientAuth::new());
    let cert_file = &mut BufReader::new(File::open("keys/actix_ex.crt").unwrap());
    let key_file = &mut BufReader::new(File::open("keys/actix_ex.key").unwrap());
    let cert_chain = certs(cert_file).unwrap();
    let mut keys = rsa_private_keys(key_file).unwrap();
    if keys.is_empty() {
        eprintln!("Could not locate RSA private keys.");
        std::process::exit(1);
    }
    config.set_single_cert(cert_chain, keys.remove(0)).unwrap();


    // Server
    let server = HttpServer::new(move || {
        // prevents double Arc
        let schema: web::Data<graphql::model::Schema> = schema.clone().into();
        let policy = CookieIdentityPolicy::new(cookie_secret_key.as_bytes())
            .name("auth")
            .path("/")
            .domain(&domain)
            .max_age_time(auth_duration)
            .secure(secure_cookie);

        App::new()
            // Database
            .data(pool.clone())
            .app_data(schema)
            // Options
            .data(opt.clone())
            // Error logging
            .wrap(Logger::default())
            // Authorisation
            .wrap(IdentityService::new(policy))
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .supports_credentials()
                    .max_age(3600),
            )
            // Sets routes via secondary files
            .configure(user::route)
            .configure(graphql::route)
    })
    // Running at `format!("{}:{}",port,"0.0.0.0")`
    .bind_rustls(("0.0.0.0", port), config)
    .unwrap()
    // Starts server
    .run();
    eprintln!("Start Actix-EX Successful");
    eprintln!("Listening on 0.0.0.0:{}", port);

    // Awaiting server to exit
    server.await
}
