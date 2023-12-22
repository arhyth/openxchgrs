use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Rate {
    pub timestamp: i64,
    pub base: String,
    #[serde(rename = "rates")]
    pub pairs: HashMap<String, f64>,
}
