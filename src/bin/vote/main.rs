use actix_cors::Cors;
use actix_web::{middleware, post, web, App, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::task::spawn_blocking;
use vote::{fractional::FractionalVoting, liquid_democracy::LiquidDemocracy};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=trace,actix_redis=trace,vote=debug");
    env_logger::init();

    HttpServer::new(|| {
        // TODO: change this
        let cors = Cors::permissive();

        App::new()
            .app_data(web::JsonConfig::default().limit(1024 * 1024 * 10)) // 10mb.... really?
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .service(api)
    })
    .bind("0.0.0.0:8081")?
    .run()
    .await
}

#[derive(Deserialize, Serialize)]
struct JsonRPCRequest {
    jsonrpc: String,
    id: Value,
    params: Value,
    method: String,
}

#[derive(Deserialize, Serialize)]
struct JsonRPCResponseSuccess {
    jsonrpc: String,
    id: Value,
    result: Value,
}

#[derive(Deserialize, Serialize)]
struct JsonRPCResponseError {
    jsonrpc: String,
    id: Value,
    error: String,
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
enum JsonRPCResponse {
    Success(JsonRPCResponseSuccess),
    Error(JsonRPCResponseError),
}

#[derive(Deserialize, Serialize)]
struct RPCParamsFormat {
    normalize: Option<bool>,
    quadratic: Option<bool>,
    voters: HashMap<String, HashMap<String, f64>>,
}

#[post("rpc/")]
async fn api<'a, 'de>(data: web::Json<JsonRPCRequest>) -> impl Responder {
    let rpc = data.into_inner();

    let result = match rpc.method.as_ref() {
        "liquid" => {
            let params: RPCParamsFormat =
                serde_json::from_value(rpc.params).expect("FIX this easy error");

            // This is computationally heavy so it goes to a dedicated thread
            let result = spawn_blocking(move || {
                let voters_ref = params
                    .voters
                    .iter()
                    .map(|(v, vts)| {
                        (
                            v.as_ref(),
                            vts.iter()
                                .map(|(to, v)| (to.as_ref(), *v))
                                .collect::<HashMap<&str, f64>>(),
                        )
                    })
                    .collect();
                let liq = LiquidDemocracy::new(voters_ref);
                let (result, influence) = liq.calculate();
                let r: HashMap<String, f64> =
                    result.iter().map(|(v, f)| (v.to_string(), *f)).collect();
                let inf: HashMap<String, f64> =
                    influence.iter().map(|(u, f)| (u.to_string(), *f)).collect();
                (r, inf)
            })
            .await
            .unwrap();

            json!(result)
        }
        "frac" => {
            let params: RPCParamsFormat =
                serde_json::from_value(rpc.params).expect("fix this easy issue");

            let voters_ref = params
                .voters
                .iter()
                .map(|(_, vts)| vts.iter().map(|(to, v)| (to.as_ref(), *v)).collect())
                .collect();
            let mut frac = FractionalVoting::new(voters_ref);
            if let Some(b) = params.quadratic {
                frac.quadratic(b);
            }
            if let Some(b) = params.normalize {
                frac.normalize(b);
            }
            json!(frac.calculate())
        }
        _ => {
            let error = "method not found";
            let response = JsonRPCResponseError {
                jsonrpc: "2.0".to_string(),
                id: rpc.id,
                error: error.to_string(),
            };
            return web::Json(JsonRPCResponse::Error(response));
        }
    };

    let response = JsonRPCResponseSuccess {
        jsonrpc: "2.0".to_string(),
        id: rpc.id,
        result,
    };

    web::Json(JsonRPCResponse::Success(response))
}
