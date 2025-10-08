use anyhow::Context;
use reqwest::Client;
use serde_json::Value;

#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    base: String,
    api_key: Option<String>,
}

impl ApiClient {
    pub fn new(base: String, api_key: Option<String>) -> Self {
        let client = Client::builder()
            .user_agent("ectusr2/0.1")
            .build()
            .expect("reqwest client");
        Self { client, base, api_key }
    }

    pub async fn post_json(&self, path: &str, body: &Value) -> anyhow::Result<Value> {
        let url = format!("{}{}", self.base, path);
        let mut req = self.client.post(url).json(body);
        if let Some(k) = &self.api_key {
            req = req.bearer_auth(k);
        }
        let res = req.send().await.context("send request")?;
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        if !status.is_success() {
            anyhow::bail!("status {}: {}", status, text);
        }
        let val: Value = serde_json::from_str(&text).unwrap_or(Value::String(text));
        Ok(val)
    }
}
