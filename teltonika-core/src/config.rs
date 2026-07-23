pub struct ConnConfig{
    pub username: String, 
    pub password: String,
    pub ip: String,
    pub accept_invalid_certs: bool
}
impl ConnConfig {
    pub fn new(username: String, password: String, ip: String) -> Self {
        Self {
            username,
            password,
            ip,
            accept_invalid_certs: true
        }
    }
}