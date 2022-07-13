use actix_cors::Cors;
use actix_web::dev::ServiceFactory;
use actix_web::{middleware, web, App, HttpServer, Responder};
use actix_web_httpauth::extractors::bearer::Config;
use actix_web_httpauth::middleware::HttpAuthentication;
use mongodb::options::ClientOptions;
use redis::Client;

use crate::game_api::define_services;

pub mod auth;
pub mod game_api;

async fn version() -> impl Responder {
    env!("CARGO_PKG_VERSION")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let port: u16 = std::env::var("BARYON_PORT")
        .expect("port number")
        .parse()
        .expect("number");
    let redis_uri = std::env::var("BARYON_REDIS_HOST").expect("Redis host URI");

    let redis_client = Client::open(format!("redis://{}", redis_uri)).unwrap();

    let mongodb_uri = std::env::var("BARYON_MONGODB_HOST").expect("MongoDB host URI");
    let mongodb_user = std::env::var("BARYON_MONGODB_USER").expect("MongoDB username"));
    let mongodb_pass = std::env::var("BARYON_MONGODB_PASSWORD").expect("MongoDB password");
    let uri = format!(
        "mongodb://{}:{}@{}/?replicaSet=rs0&appName=baryonic",
        urlencoding::encode(&mongodb_user), urlencoding::encode(&mongodb_pass), mongodb_uri
    );
    let options = ClientOptions::parse(&uri).await.unwrap();

    let mongodb_client =
        mongodb::Client::with_options(options).expect("Failed to connect to MongoDB");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_header()
            .allow_any_method()
            .allow_any_origin();

        let auth_middleware = HttpAuthentication::bearer(auth::jwt::validator);

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .route("/version", web::get().to(version))
            .app_data(
                Config::default()
                    .realm("Authenticated endpoint")
                    .scope("game"),
            )
            .app_data(web::Data::new(redis_client.clone()))
            .app_data(web::Data::new(mongodb_client.clone()))
            .service(
                web::scope("")
                    .wrap(auth_middleware)
                    .configure(define_services),
            )
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
