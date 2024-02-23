use crate::prelude::*;
use glam::Vec2;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Asteroid {
    pub body: GameObjectBody,
    pub radius: f32,
    health: f32,
    pub material: Material,
}

impl Asteroid {
    pub fn new(pos: Vec2, vel: Vec2, time: f32, radius: f32, material: Material) -> Self {
        let segments = rand::random::<usize>() % 30 + 10;

        let mut bounds = vec![];

        for i in 0..segments {
            let angle = i as f32 / segments as f32 * 2. * PI;

            let dist_mp = rand::random::<f32>() / 5. + 0.9;

            bounds.push(vec2(angle.cos(), angle.sin()) * radius * dist_mp);
        }

        let body = GameObjectBody::new(pos, vel, rand::random::<f32>() * PI * 2., time, bounds);

        Self {
            body,
            radius,
            health: material.health_per_area() * radius * radius * std::f32::consts::PI * 40.,
            material,
        }
    }
    pub fn mass(&self) -> f32 {
        self.radius * self.radius * std::f32::consts::PI * self.material.density()
    }
}

impl Asteroid {
    pub fn destroyed(&self) -> bool {
        self.health <= 0.
    }
    pub fn owner(&self) -> Option<PlayerId> {
        None
    }
    pub fn collides_point(&self, position: Vec2) -> bool {
        self.body.position.distance(position) < self.radius
    }
    pub fn apply_damage(&mut self, damage: f32, _position: Vec2) -> Vec<(Material, f32)> {
        self.health -= damage;
        vec![(self.material, damage)]
    }
    pub fn health(&self) -> f32 {
        self.health
    }
}
