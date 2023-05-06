use crate::prelude::*;

pub type PlayerToken = u64;

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct Player {
    pub materials: BTreeMap<Material, f32>,
}

impl Player {
    pub fn new() -> Self {
        let materials = vec![
            (Material::Steel, 200.),
            (Material::Titanium, 200.),
            (Material::Carbon, 100.),
            (Material::Silicon, 200.),
            (Material::Uranium, 100.),
            (Material::Gold, 100.),
            (Material::Silver, 230.),
            (Material::Copper, 200.),
        ]
        .into_iter()
        .collect();

        Self { materials }
    }
    pub fn take_materials(&mut self, materials: &BTreeMap<Material, f32>) -> bool {
        if self.has_materials(materials) {
            for (material, amount) in materials {
                self.materials
                    .entry(*material)
                    .and_modify(|value| *value -= amount);
            }
            true
        } else {
            false
        }
    }
    pub fn has_materials(&self, materials: &BTreeMap<Material, f32>) -> bool {
        materials
            .iter()
            .all(|(material, amount)| self.materials.get(material).unwrap_or(&0.) >= amount)
    }
    pub fn give_materials(&mut self, materials: Vec<(Material, f32)>) {
        for (material, amount) in materials {
            *self.materials.entry(material).or_default() += amount;
        }
    }
}
