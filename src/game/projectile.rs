use crate::prelude::*;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Projectile {
    pub transform: Transform,
    pub owner: PlayerToken,
    pub health: f32,
    pub mass: f32,
    pub size: Vec2,
    pub lifetime: f32,
    pub origin: ProjectileType,
    pub destructive_power: f32,
}

impl Projectile {
    pub fn update(&mut self, dt: f32) {
        self.lifetime -= dt;
    }
}

impl Projectile {
    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
    pub fn bounds(&self) -> Vec<Vec2> {
        vec![
            vec2(self.size.x, self.size.y) / 2.,
            vec2(-self.size.x, self.size.y) / 2.,
            vec2(-self.size.x, -self.size.y) / 2.,
            vec2(self.size.x, -self.size.y) / 2.,
        ]
    }
    pub fn mass(&self) -> f32 {
        self.mass
    }

    pub fn destroyed(&self) -> bool {
        self.health <= 0.
    }

    pub fn owner(&self) -> Option<PlayerToken> {
        Some(self.owner)
    }
    pub fn apply_damage(&mut self, damage: f32, _position: Vec2) -> Vec<(Material, f32)> {
        self.health -= damage;
        vec![(Material::Steel, damage / 10.)]
    }
    pub fn health(&self) -> f32 {
        if self.lifetime < 0. {
            0.
        } else {
            self.health
        }
    }
    pub fn collides_point(&self, position: Vec2) -> bool {
        self.transform.position.distance(position) < 10.
    }
    pub fn destructive_power(&self) -> f32 {
        self.destructive_power
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Copy)]
pub enum ProjectileType {
    Bullet,
    Missile,
    Laser,
}

impl ProjectileType {
    pub fn construct(
        &self,
        position: Vec2,
        velocity: Vec2,
        rotation: f32,
        owner: PlayerToken,
    ) -> Projectile {
        let (health, mass, scale, lifetime, destructive_power) = match &self {
            ProjectileType::Bullet => (50., 10., vec2(0.5, 0.2), 5., 2.),
            ProjectileType::Missile => (500., 100., vec2(0.3, 0.3), 20., 3.),
            ProjectileType::Laser => (80., 10., vec2(0.5, 0.1), 3., 1.),
        };

        Projectile {
            transform: Transform::new(position, velocity, rotation),
            owner,
            health,
            mass,
            lifetime,
            size: scale,
            origin: *self,
            destructive_power,
        }
    }
}
