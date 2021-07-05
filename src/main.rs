mod rpc;

use actix_cors::Cors;
use actix_web::{get, middleware, post, web, App, HttpServer, Responder};
use futures::future::join_all;
use rpc::calculate;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;
use vote::{TopicData, VoteData};

type ModuleMap = Mutex<HashMap<String, String>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=trace,actix_redis=trace,vote=debug");
    env_logger::init();

    let modules: web::Data<ModuleMap> = web::Data::new(Mutex::new(HashMap::new()));

    HttpServer::new(move || {
        // TODO: change this
        let cors = Cors::permissive();

        App::new()
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .app_data(modules.clone())
            .service(
                web::scope("/rpc/")
                    .service(hello)
                    .service(api)
                    .service(add_module)
                    .service(get_modules)
                    .service(dummy_info),
            )
    })
    .bind("0.0.0.0:8081")?
    .run()
    .await
}

#[post("")]
async fn api(modules: web::Data<ModuleMap>, topic: web::Json<TopicData>) -> impl Responder {
    let topic = topic.into_inner();

    let info: VoteData = topic.into();

    let modules = modules.lock().unwrap();

    let calculations = modules.iter().map(|m| {
        let (name, uri) = m;
        calculate(&name, &uri, &info)
    });

    let module_responses = join_all(calculations).await;

    let result: HashMap<String, Value> = modules
        .keys()
        .zip(module_responses.iter())
        .filter(|(_, r)| r.is_some())
        .map(|(k, r)| (k.to_string(), r.to_owned().unwrap()))
        .collect();

    web::Json(result)
}

#[post("module/")]
async fn add_module(
    modules: web::Data<ModuleMap>,
    module: web::Json<(String, String)>,
) -> impl Responder {
    let mut modules = modules.lock().unwrap();
    let (name, uri): (String, String) = module.into_inner();
    modules.insert(name, uri);
    web::Json(json!({"status":"ok"}))
}

#[get("modules/")]
async fn get_modules(modules: web::Data<ModuleMap>) -> impl Responder {
    let modules = modules.lock().unwrap();
    web::Json(json!(*modules))
}

#[get("hello/")]
async fn hello() -> impl Responder {
    "world!"
}

#[get("dummy/")]
async fn dummy_info() -> impl Responder {
    web::Json(TopicData::dummy())
}
