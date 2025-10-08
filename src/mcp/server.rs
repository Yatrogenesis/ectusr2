use crate::{api::client::ApiClient, config::Config};
use crate::mcp::types::*;
use serde_json::{json, Value};
use tokio::sync::mpsc;
use std::io::{self, BufRead, Write};

pub async fn run(cfg: Config) -> anyhow::Result<()> {
    let client = ApiClient::new(cfg.api_url.clone(), cfg.api_key.clone());

    // Channel for lines read from stdin (blocking thread)
    let (tx, mut rx) = mpsc::channel::<String>(100);
    std::thread::spawn(move || {
        let stdin = io::stdin();
        let lock = stdin.lock();
        for line in lock.lines() {
            if let Ok(l) = line { let _ = tx.blocking_send(l); } else { break; }
        }
    });

    while let Some(line) = rx.recv().await {
        if line.trim().is_empty() { continue; }
        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = json!({
                    "jsonrpc":"2.0","id":null,
                    "error": {"code": -32700, "message": format!("parse error: {}", e)}
                });
                println!("{}", resp.to_string());
                continue;
            }
        };
        let response = handle_request(&client, &cfg, req).await;
        println!("{}", response.to_string());
        io::stdout().flush().ok();
    }
    Ok(())
}

async fn handle_request(client: &ApiClient, cfg: &Config, req: JsonRpcRequest) -> Value {
    match req.method.as_str() {
        "initialize" => json!({
            "jsonrpc":"2.0", "id": req.id,
            "result": {
                "protocolVersion":"2024-11-05",
                "capabilities": {"tools": {}, "resources": {}, "prompts": {}},
                "serverInfo": {"name":"ectusr2","version": env!("CARGO_PKG_VERSION")}
            }
        }),
        "tools/list" => json!({
            "jsonrpc":"2.0", "id": req.id,
            "result": {"tools": crate::tools::list()}
        }),
        "tools/call" => {
            let (name, args) = match req.params.and_then(|v| parse_call_params(v)) {
                Some(t) => t,
                None => return error(req.id, -32602, "invalid params", None),
            };
            match crate::tools::call(client, cfg, &name, args).await {
                Ok(v) => json!({"jsonrpc":"2.0","id":req.id,"result": {"content":[{"type":"text","text": v}]}}),
                Err(e) => error(req.id, -32000, &e.to_string(), None),
            }
        }
        ,
        "resources/list" => json!({
            "jsonrpc":"2.0", "id": req.id,
            "result": {"resources": crate::resources::registry::list()}
        }),
        "prompts/list" => json!({
            "jsonrpc":"2.0", "id": req.id,
            "result": {"prompts": crate::prompts::list::list()}
        }),
        "prompts/get" => {
            let name = req.params.and_then(|v| v.get("name").and_then(|n| n.as_str().map(|s| s.to_string())));
            match name {
                Some(n) => json!({"jsonrpc":"2.0","id":req.id,"result": crate::prompts::get::get(&n)}),
                None => error(req.id, -32602, "missing name", None)
            }
        }
        ,
        _ => error(req.id, -32601, "method not found", None),
    }
}

fn parse_call_params(v: Value) -> Option<(String, Value)> {
    let name = v.get("name")?.as_str()?.to_string();
    let args = v.get("arguments").cloned().unwrap_or_else(|| Value::Object(Default::default()));
    Some((name, args))
}

fn error(id: Value, code: i32, msg: &str, data: Option<Value>) -> Value {
    serde_json::json!({"jsonrpc":"2.0","id": id, "error": {"code": code, "message": msg, "data": data}})
}
