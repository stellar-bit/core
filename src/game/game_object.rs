use super::*;

#[derive(Clone, Serialize, Deserialize, Debug, IntoStaticStr)]
// #[serde(tag = "type")]
pub enum GameObject {
    Asteroid(Asteroid),
    StarBase(StarBase),
    Spacecraft(Spacecraft),
    Projectile(Projectile),
}

impl GameObject {
    pub fn transform_mut(&mut self) -> &mut Transform {
        match self {
            GameObject::Asteroid(asteroid) => &mut asteroid.transform,
            GameObject::StarBase(star_base) => &mut star_base.transform,
            GameObject::Spacecraft(spacecraft) => &mut spacecraft.transform,
            GameObject::Projectile(projectile) => &mut projectile.transform,
        }
    }
    pub fn transform(&self) -> &Transform {
        match self {
            GameObject::Asteroid(asteroid) => &asteroid.transform,
            GameObject::StarBase(star_base) => &star_base.transform,
            GameObject::Spacecraft(spacecraft) => &spacecraft.transform,
            GameObject::Projectile(projectile) => &projectile.transform,
        }
    }
    pub fn mass(&self) -> f32 {
        match self {
            GameObject::Asteroid(asteroid) => asteroid.mass(),
            GameObject::StarBase(star_base) => star_base.mass(),
            GameObject::Spacecraft(spacecraft) => spacecraft.mass,
            GameObject::Projectile(projectile) => projectile.mass,
        }
    }
    pub fn destroyed(&self) -> bool {
        match self {
            GameObject::Asteroid(asteroid) => asteroid.destroyed(),
            GameObject::StarBase(star_base) => star_base.destroyed(),
            GameObject::Spacecraft(spacecraft) => spacecraft.destroyed(),
            GameObject::Projectile(projectile) => projectile.destroyed(),
        }
    }
    pub fn owner(&self) -> Option<PlayerToken> {
        match self {
            GameObject::Asteroid(_) => None,
            GameObject::StarBase(star_base) => star_base.owner(),
            GameObject::Spacecraft(spacecraft) => spacecraft.owner(),
            GameObject::Projectile(projectile) => projectile.owner(),
        }
    }
    pub fn collides_point(&self, position: Vec2) -> bool {
        match self {
            GameObject::Asteroid(asteroid) => asteroid.collides_point(position),
            GameObject::StarBase(star_base) => star_base.collides_point(position),
            GameObject::Spacecraft(spacecraft) => spacecraft.collides_point(position),
            GameObject::Projectile(projectile) => projectile.collides_point(position),
        }
    }
    pub fn health(&self) -> f32 {
        match self {
            GameObject::Asteroid(asteroid) => asteroid.health(),
            GameObject::StarBase(star_base) => star_base.health(),
            GameObject::Spacecraft(spacecraft) => spacecraft.health(),
            GameObject::Projectile(projectile) => projectile.health(),
        }
    }
    pub fn bounds(&self) -> Vec<Vec2> {
        match self {
            GameObject::Asteroid(asteroid) => asteroid.bounds(),
            GameObject::StarBase(star_base) => star_base.bounds(),
            GameObject::Spacecraft(spacecraft) => spacecraft.bounds(),
            GameObject::Projectile(projectile) => projectile.bounds(),
        }
    }
    pub fn destructive_power(&self) -> f32 {
        match self {
            GameObject::Projectile(projectile) => projectile.destructive_power(),
            _ => 1.,
        }
    }
    pub fn apply_damage(&mut self, damage: f32, position: Vec2) -> Vec<(Material, f32)> {
        match self {
            GameObject::Asteroid(asteroid) => asteroid.apply_damage(damage, position),
            GameObject::StarBase(star_base) => star_base.apply_damage(damage, position),
            GameObject::Spacecraft(spacecraft) => spacecraft.apply_damage(damage, position),
            GameObject::Projectile(projectile) => projectile.apply_damage(damage, position),
        }
    }
    pub fn collides(&self, other: &GameObject) -> bool {
        let pos_offset = self.transform().position - other.transform().position;

        let other_bounds = other
            .bounds()
            .iter()
            .map(|&ver| {
                let rotation = other.transform().rotation - self.transform().rotation;
                ver.rotate_rad(rotation) + pos_offset
            })
            .collect();

        collision_detection::sat_collision_detect(&self.bounds(), &other_bounds)
    }
}

pub type GameObjectId = u16;
