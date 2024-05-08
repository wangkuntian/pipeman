use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Images {
    pub images: Vec<Image>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
    pub id: String,
    pub name: String,
    pub status: String,
}
