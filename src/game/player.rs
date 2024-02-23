use crate::prelude::*;

pub type PlayerId = u64;

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct Player {
    pub materials: BTreeMap<Material, f32>,
}

impl Player {
    pub fn new() -> Self {
        let materials = BTreeMap::new();

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
