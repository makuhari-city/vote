use crate::VoteInfo;
use actix_web::client::Client;
use bs58::encode;
use futures::FutureExt;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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

    pub fn id(&self) -> String {
        self.id.to_string()
    }

    pub fn vote_info(&self) -> VoteInfo {
        serde_json::from_value(self.params.to_owned()).expect("params should be a VoteInfo")
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
    pub fn new(id: &str) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: id.to_string(),
            result: None,
            error: None,
        }
    }

    pub fn is_success(&self) -> bool {
        self.result.is_some()
    }

    pub fn result(&mut self, r: &Value) {
        self.result = Some(r.to_owned());
    }

    pub fn error(&mut self, error: &str) {
        let value: Value = json!(error);
        self.error = Some(value);
    }
}

pub async fn calculate(module_name: &str, address: &str, info: &VoteInfo) -> Option<Value> {
    let mut rpc = JsonRPCRequest::new();
    let hash = &info.hash().await;
    rpc.method = "calculate".to_string();
    rpc.params = json!(info);
    rpc.id = encode(hash).into_string();

    let endpoint = format!("{}/{}/rpc/", &address, &module_name);

    log::info!("{}", &endpoint);

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
