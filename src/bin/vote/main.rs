use actix_cors::Cors;
use actix_web::{client::Client, get, middleware, post, web, App, HttpServer, Responder};
use futures::future::join_all;
use log::info;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;
use vote::{TopicInfo, VoteMethodResult};

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
            .service(hello)
            .service(api)
            .service(add_module)
            .service(get_modules)
            .service(dummy_info)
    })
    .bind("0.0.0.0:8100")?
    .run()
    .await
}

#[post("rpc/")]
async fn api(modules: web::Data<ModuleMap>, topic: web::Json<TopicInfo>) -> impl Responder {
    let topic = topic.into_inner();
    let modules = modules.lock().unwrap();

    let calculations = modules.iter().map(|m| {
        let (_, address) = m;
        let endpoint = format!("{}/rpc/", address);
        info!("endpoint: {}", &endpoint);
        calculate(endpoint, &topic)
    });

    let result: HashMap<String, Value> = join_all(calculations)
        .await
        .iter()
        .filter(|r| r.is_some())
        .map(|r| r.as_ref().unwrap().to_tuple())
        .collect();
    web::Json(result)
}

async fn calculate(address: String, topic: &TopicInfo) -> Option<VoteMethodResult> {
    let client = Client::new();

    let mut response = client
        .post(address)
        .header("ContentType", "application/json")
        .send_json(&topic)
        .await
        .unwrap();

    match response.json().await {
        Ok(r) => Some(r),
        Err(_) => None,
    }
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
    web::Json(TopicInfo::dummy())
}
