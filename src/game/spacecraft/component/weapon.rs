use super::*;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Weapon {
    pub rotation: f32,
    fire_rate: f32,
    fire_rate_timer: f32,
    pub projectile_speed: f32,
    projectile_type: ProjectileType,
    pub body: ComponentBody,
    pub health: f32,
    pub mass: f32,
    launch_point: Vec2,
    pub active: bool,
    max_angle: f32,
}

impl Weapon {
    pub fn new(
        body: ComponentBody,
        mass: f32,
        health: f32,
        fire_rate: f32,
        projectile_type: ProjectileType,
        projectile_speed: f32,
        launch_point: Vec2,
        max_angle: f32,
    ) -> Self {
        Self {
            rotation: 0.,
            fire_rate,
            fire_rate_timer: 0.,
            projectile_speed,
            projectile_type,
            body,
            health,
            mass,
            launch_point,
            active: false,
            max_angle,
        }
    }
}

impl Weapon {
    pub fn body(&self) -> &ComponentBody {
        &self.body
    }
    pub fn update(&mut self, time: f32) -> Vec<ComponentEffect> {
        let dt = time - self.body.cur_time;
        
        let mut effects = vec![];

        self.fire_rate_timer += dt;
        if self.active {
            if self.fire_rate_timer > self.fire_rate {
                self.fire_rate_timer = 0.;
                let projectile_rotation = self.body.orientation.to_radians() + self.rotation;
                effects.push(ComponentEffect::CreateProjectile(
                    self.projectile_type,
                    self.body.position.as_vec2()
                        + self.launch_point.rotate_rad(projectile_rotation),
                    Vec2::from_angle(projectile_rotation) * self.projectile_speed,
                    projectile_rotation,
                ));
            }
        }

        effects
    }
    pub fn handle_cmd(&mut self, cmd: ComponentCmd) {
        match cmd {
            ComponentCmd::SetRotation(rotation) => {
                let rotation = normalize_radians(rotation);
                self.rotation = rotation.min(self.max_angle).max(-self.max_angle);
            }
            ComponentCmd::SelfDestruct => {
                self.health = 0.;
            }
            ComponentCmd::SetActive(active) => {
                self.active = active;
            }
            _ => {}
        }
    }
    pub fn apply_damage(&mut self, damage: f32) {
        self.health -= damage;
    }
}
