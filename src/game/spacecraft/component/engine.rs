use super::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Engine {
    mass: f32,
    health: f32,
    pub body: ComponentBody,
    pub fuel: f32,
    pub active: bool,
    pub thrust: f32,
    rotation: f32,
    fuel_density: f32,
    pub power: f32,
    pub ignition_point: Vec2,
}

impl Engine {
    pub fn new(
        body: ComponentBody,
        mass: f32,
        health: f32,
        thrust: f32,
        fuel: f32,
        fuel_density: f32,
        ignition_point: Vec2,
    ) -> Self {
        Self {
            mass,
            health,
            body,
            fuel,
            active: false,
            thrust,
            rotation: 0.,
            fuel_density,
            power: 1.,
            ignition_point,
        }
    }
}

impl ComponentWrapper for Engine {
    fn body(&self) -> &ComponentBody {
        &self.body
    }

    fn update(&mut self, time: f32) -> Vec<ComponentEffect> {
        let mut result = vec![];
        let dt = time-self.body.cur_time;
        if self.active && self.fuel > 0. {
            let thrust = self.fuel.min(self.thrust * dt * self.power);
            self.fuel -= thrust;

            let force_vector = Vec2::from_angle(self.rotation + self.body.orientation.to_radians())
                .rotate(Vec2::new(1., 0.));
            result.push(ComponentEffect::ApplyForce(force_vector * thrust));
        }
        result
    }
    fn mass(&self) -> f32 {
        self.fuel * self.fuel_density
    }
    fn health(&self) -> f32 {
        self.health
    }
    fn handle_cmd(&mut self, cmd: ComponentCmd) {
        match cmd {
            ComponentCmd::SetPower(power) => {
                self.power = power.min(1.).max(0.);
            }
            ComponentCmd::SetActive(activness) => {
                self.active = activness;
            }
            ComponentCmd::SetRotation(rotation) => {
                let rotation = normalize_radians(rotation);
                self.rotation = rotation.min(1.2).max(-1.2);
            }
            ComponentCmd::SelfDestruct => {
                self.health = 0.;
            }
        }
    }
    fn apply_damage(&mut self, damage: f32) {
        self.health -= damage;
    }
}
