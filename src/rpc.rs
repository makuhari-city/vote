use actix_web::client::Client;
use futures::FutureExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use vote::VoteInfo;

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRPCRequest {
    jsonrpc: String,
    id: String,
    method: String,
    params: Value,
}

impl JsonRPCRequest {
    pub fn new() -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: "".to_string(),
            method: "".to_string(),
            params: json!({}),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRPCResponse {
    jsonrpc: String,
    id: String,
    result: Option<Value>,
    error: Option<Value>,
}

impl JsonRPCResponse {
    pub fn is_success(&self) -> bool {
        self.result.is_some()
    }
}

pub async fn calculate(module_name: &str, address: &str, info: &VoteInfo) -> Option<Value> {
    let mut rpc = JsonRPCRequest::new();
    rpc.method = "calculate".to_string();
    rpc.params = json!(info);

    let endpoint = format!("{}/{}/rpc/", &address, &module_name);

    let client = Client::new();
    let data = client
        .post(&endpoint)
        .header("ContentType", "application/json")
        .send_json(&rpc)
        .then(|r| async move { r.unwrap().json().await })
        .await;

    match data {
        Ok(r) => {
            let json: Result<JsonRPCResponse, serde_json::Error> = serde_json::from_value(r);
            match json {
                Ok(res) => match res.is_success() {
                    true => return res.result,
                    _ => return None,
                },
                Err(_) => return None,
            };
        }
        Err(_err) => None,
    }
}
