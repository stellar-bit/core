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

impl ComponentWrapper for Block {
    fn body(&self) -> &ComponentBody {
        &self.body
    }
    fn update(&mut self, dt: f32) -> Vec<ComponentEffect> {
        vec![]
    }
    fn mass(&self) -> f32 {
        self.mass
    }
    fn health(&self) -> f32 {
        self.health
    }
    fn handle_cmd(&mut self, cmd: ComponentCmd) {
        if let ComponentCmd::SelfDestruct = cmd {
            self.health = 0.;
        }
    }
    fn apply_damage(&mut self, damage: f32) {
        self.health -= damage;
    }
}
