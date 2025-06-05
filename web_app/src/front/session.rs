use crate::models;

/// Cookie session data stored (encrypt) on user side
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct WebAppSession {
    pub user: models::user_app::User,
    pub add_pet_balance: u32,
}

impl WebAppSession {
    pub fn has_pet_balance(&self) -> bool {
        self.add_pet_balance > 0
    }
}
