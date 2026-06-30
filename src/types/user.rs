#[derive(Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub api_key: String,
}

#[derive(Debug, Clone)]
pub struct User {
    pub tachi_id: u64,
    pub card_id: String,
    pub profile: Profile,
}
