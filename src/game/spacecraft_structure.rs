use crate::prelude::*;

use super::spacecraft::Orient;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SpacecraftStructure {
    pub component_placeholders: Vec<ComponentPlaceholder>,
    pub tags: Vec<String>,
}

impl SpacecraftStructure {
    pub fn build_time(&self) -> f32 {
        self.component_placeholders
            .iter()
            .map(|c| c.component_type.build_time())
            .sum()
    }
    /// Checks if the structure is valid. A structure is valid if:
    /// - It has exactly one central component
    /// - All components are connected
    /// - No components overlap
    /// - All top components are connected to a bottom component
    pub fn valid(&self) -> bool {
        let mut central = false;
        let mut top_occupies = BTreeSet::new();
        let mut bot_occupies = BTreeSet::new();

        for component_placeholder in &self.component_placeholders {
            if let ComponentType::Central = component_placeholder.component_type {
                if central {
                    return false;
                }
                central = true;
            }

            if let Some(top) = component_placeholder.component_type.top() {
                for occupied_pos in &top.occupies {
                    let offset = occupied_pos.orient(component_placeholder.orientation);
                    if !top_occupies.insert((
                        component_placeholder.position.x + offset.x,
                        component_placeholder.position.y + offset.y,
                    )) {
                        return false;
                    }
                }
            } else {
                for x in 0..component_placeholder.component_type.scale().x {
                    for y in 0..component_placeholder.component_type.scale().y {
                        let offset = uvec2(x, y)
                            .as_ivec2()
                            .orient(component_placeholder.orientation);
                        if !bot_occupies.insert((
                            component_placeholder.position.x + offset.x,
                            component_placeholder.position.y + offset.y,
                        )) {
                            return false;
                        }
                    }
                }
            }
        }

        for top_occupy in top_occupies {
            if !bot_occupies.contains(&top_occupy) {
                return false;
            }
        }

        return true;
    }
    pub fn materials(&self) -> BTreeMap<Material, f32> {
        let mut materials = BTreeMap::new();
        for component_placeholder in &self.component_placeholders {
            for (material, amount) in component_placeholder.component_type.materials() {
                *materials.entry(material).or_insert(0.) += amount;
            }
        }
        materials
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ComponentPlaceholder {
    pub component_type: ComponentType,
    pub position: IVec2,
    pub orientation: Orientation,
}

impl ComponentPlaceholder {
    pub fn new(component_type: ComponentType, position: IVec2, orientation: Orientation) -> Self {
        Self {
            component_type,
            position,
            orientation,
        }
    }
}
