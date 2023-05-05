use crate::prelude::*;
use glam::Vec2;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Asteroid {
    pub body: GameObjectBody,
    pub radius: f32,
    health: f32,
    pub material: Material,
    pub updated: usize,
}

impl Asteroid {
    pub fn new(transform: GameObjectBody, radius: f32, material: Material) -> Self {
        Self {
            body: transform,
            radius,
            health: material.health_per_area() * radius * radius * std::f32::consts::PI,
            material,
            updated: 0,
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
    pub fn transform_mut(&mut self) -> &mut GameObjectBody {
        &mut self.body
    }
    pub fn transform(&self) -> &GameObjectBody {
        &self.body
    }
    pub fn destroyed(&self) -> bool {
        self.health <= 0.
    }
    pub fn owner(&self) -> Option<PlayerToken> {
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
