use crate::prelude::*;
use glam::Vec2;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Asteroid {
    pub transform: Transform,
    pub radius: f32,
    health: f32,
    pub material: Material,
}

impl Asteroid {
    pub fn new(transform: Transform, radius: f32, material: Material) -> Self {
        Self {
            transform,
            radius,
            health: material.health_per_area() * radius * radius * std::f32::consts::PI,
            material,
        }
    }
    pub fn mass(&self) -> f32 {
        self.radius * self.radius * std::f32::consts::PI * self.material.density()
    }
    pub fn bounds(&self) -> Vec<Vec2> {
        vec![
            vec2(self.radius, self.radius),
            vec2(-self.radius, self.radius),
            vec2(-self.radius, -self.radius),
            vec2(self.radius, -self.radius),
        ]
    }
}

impl Asteroid {
    pub fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn destroyed(&self) -> bool {
        self.health <= 0.
    }
    pub fn owner(&self) -> Option<PlayerToken> {
        None
    }
    pub fn collides_point(&self, position: Vec2) -> bool {
        self.transform.position.distance(position) < self.radius
    }
    pub fn apply_damage(&mut self, damage: f32, _position: Vec2) -> Vec<(Material, f32)> {
        self.health -= damage;
        vec![(self.material, damage)]
    }
    pub fn health(&self) -> f32 {
        self.health
    }
}
