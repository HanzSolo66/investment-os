use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Asset {
    pub id: i64,
    pub name: String,
    pub unit_value: f64,
    pub quantity: f64,
}

impl Asset {
    pub fn total_value(&self) -> f64 {
        self.unit_value * self.quantity
    }
}

#[derive(Serialize)]
pub struct PortfolioSummary {
    pub total_assets: usize,
    pub portfolio_value: f64,
}

impl PortfolioSummary {
    pub fn from_assets(assets: &[Asset]) -> Self {
        let portfolio_value = assets.iter().map(Asset::total_value).sum();

        Self {
            total_assets: assets.len(),
            portfolio_value,
        }
    }
}

pub struct UserRecord {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}
