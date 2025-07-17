use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ControlBlock {
    pub jwt: String,
    pub exp: i64
}