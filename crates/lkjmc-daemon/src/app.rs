#[derive(Debug, Clone)]
pub struct AppState {
    pub database_url: Option<String>,
}

impl AppState {
    pub fn new(database_url: Option<String>) -> Self {
        Self { database_url }
    }
}
