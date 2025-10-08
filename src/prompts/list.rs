pub fn list() -> serde_json::Value {
    serde_json::json!({
        "prompts": [
            {"name":"generate_code_template","description":"Template for generate_code"},
            {"name":"run_qa_template","description":"Template for run_qa"}
        ]
    })
}
