pub fn get(name: &str) -> serde_json::Value {
    match name {
        "generate_code_template" => serde_json::json!({"messages":[{"role":"user","content":{"type":"text","text":"Generate production code for ..."}}]}),
        "run_qa_template" => serde_json::json!({"messages":[{"role":"user","content":{"type":"text","text":"Run QA on the following code ..."}}]}),
        _ => serde_json::json!({"messages":[]} ),
    }
}
