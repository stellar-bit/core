use rand::{prelude::Distribution, distributions::Standard, Rng};

use crate::prelude::*;

#[derive(Serialize, Clone, Deserialize, Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Copy)]
pub enum Material {
    Iron,
    Nickel,
    Silicates, // Grouping rocky material
    Copper,
    Carbon,
}

impl Material {
    pub fn density(&self) -> f32 {
        match self {
            Material::Iron => 7.87,
            Material::Nickel => 8.91,
            Material::Silicates => 2.6, // Average density, can vary
            Material::Copper => 8.96,
            Material::Carbon => 2.26,
        }
    }
    pub fn health_per_area(&self) -> f32 {
        match self {
            Material::Iron => 8.,
            Material::Nickel => 7.5,
            Material::Silicates => 4.,
            Material::Copper => 8.,
            Material::Carbon => 7.,
        }
    }
}

impl Distribution<Material> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Material {
        match rng.gen_range(0..5) {
            0 => Material::Iron,
            1 => Material::Nickel,
            2 => Material::Silicates,
            3 => Material::Copper,
            _ => Material::Carbon,
        }
    }
}