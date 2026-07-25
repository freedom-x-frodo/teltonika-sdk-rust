pub fn base64_encode(input: &str) -> String {
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, input)
}