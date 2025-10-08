pub fn list() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({"uri":"ectus://metrics/generation","name":"Code Generation Metrics","description":"Recent generation metrics","mimeType":"application/json"}),
        serde_json::json!({"uri":"ectus://metrics/qa","name":"QA Metrics","description":"QA statistics","mimeType":"application/json"}),
        serde_json::json!({"uri":"ectus://templates/library","name":"Template Library","description":"Pre-built code templates","mimeType":"application/json"})
    ]
}
