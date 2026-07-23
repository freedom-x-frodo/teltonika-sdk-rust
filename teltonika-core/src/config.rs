pub struct ConnConfig{
    pub username: String, 
    pub password: String,
    pub ip: String
}
impl ConnConfig {
    pub fn new(username: String, password: String, ip: String) -> Self {
        Self {
            username,
            password,
            ip
        }
    }
}