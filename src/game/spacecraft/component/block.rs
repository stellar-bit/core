use super::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block {
    pub body: ComponentBody,
    mass: f32,
    health: f32,
}

impl Block {
    pub fn new(body: ComponentBody, mass: f32, health: f32) -> Self {
        Self { body, mass, health }
    }
}

impl Block {
    pub fn body(&self) -> &ComponentBody {
        &self.body
    }
    pub fn update(&mut self, _time: f32) -> Vec<ComponentEffect> {
        vec![]
    }
    pub fn mass(&self) -> f32 {
        self.mass
    }
    pub fn health(&self) -> f32 {
        self.health
    }
    pub fn handle_cmd(&mut self, cmd: ComponentCmd) {
        if let ComponentCmd::SelfDestruct = cmd {
            self.health = 0.;
        }
    }
    pub fn apply_damage(&mut self, damage: f32) {
        self.health -= damage;
    }
}
