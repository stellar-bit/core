use rand::prelude::Distribution;

use crate::prelude::*;

#[derive(Serialize, Clone, Deserialize, Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Copy)]
pub enum Material {
    Gold,
    Copper,
    Steel,
    Silver,
    Titanium,
    Carbon,
    Silicon,
    Uranium,
}

impl Material {
    pub fn density(&self) -> f32 {
        match self {
            Material::Gold => 19.3,
            Material::Copper => 8.96,
            Material::Steel => 7.8,
            Material::Silver => 10.5,
            Material::Titanium => 4.5,
            Material::Carbon => 2.26,
            Material::Silicon => 2.33,
            Material::Uranium => 19.1,
        }
    }
    pub fn health_per_area(&self) -> f32 {
        match self {
            Material::Gold => 10.,
            Material::Copper => 7.8,
            Material::Steel => 5.,
            Material::Silver => 8.,
            Material::Titanium => 10.,
            Material::Carbon => 5.,
            Material::Silicon => 5.,
            Material::Uranium => 10.,
        }
    }
}

impl Distribution<Material> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Material {
        match rng.gen_range(0..8) {
            0 => Material::Gold,
            1 => Material::Copper,
            2 => Material::Steel,
            3 => Material::Silver,
            4 => Material::Titanium,
            5 => Material::Carbon,
            6 => Material::Silicon,
            7 => Material::Uranium,
            _ => unreachable!(),
        }
    }
}
