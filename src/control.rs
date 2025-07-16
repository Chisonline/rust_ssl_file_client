use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ControlBlock {
    pub jwt: String,
    exp: i64
}