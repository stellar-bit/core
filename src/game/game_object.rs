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
    pub fn body_mut(&mut self) -> &mut GameObjectBody {
        match self {
            GameObject::Asteroid(asteroid) => &mut asteroid.body,
            GameObject::StarBase(star_base) => &mut star_base.body,
            GameObject::Spacecraft(spacecraft) => &mut spacecraft.body,
            GameObject::Projectile(projectile) => &mut projectile.body,
        }
    }
    pub fn body(&self) -> &GameObjectBody {
        match self {
            GameObject::Asteroid(asteroid) => &asteroid.body,
            GameObject::StarBase(star_base) => &star_base.body,
            GameObject::Spacecraft(spacecraft) => &spacecraft.body,
            GameObject::Projectile(projectile) => &projectile.body,
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
        let pos_offset = self.body().position - other.body().position;

        let other_bounds = other
            .body().bounds
            .iter()
            .map(|&ver| {
                let rotation = other.body().rotation - self.body().rotation;
                ver.rotate_rad(rotation) + pos_offset
            })
            .collect();

        collision_detection::sat_collision_detect(&self.body().bounds, &other_bounds)
    }
    pub fn update(&mut self, time: f32) -> Vec<GameObjectEffect> {
        let result = match self {
            GameObject::Asteroid(asteroid) => vec![],
            GameObject::StarBase(star_base) => star_base.update(time),
            GameObject::Spacecraft(spacecraft) => spacecraft.update(time),
            GameObject::Projectile(projectile) => projectile.update(time),
        };
        self.body_mut().update(time);
        result
    }
    /// Less accurate update but can be used during collisions
    pub fn update_fixed(&mut self, time: f32) -> Vec<GameObjectEffect> {
        let result = match self {
            GameObject::Asteroid(asteroid) => vec![],
            GameObject::StarBase(star_base) => star_base.update(time),
            GameObject::Spacecraft(spacecraft) => spacecraft.update(time),
            GameObject::Projectile(projectile) => projectile.update(time)
        };
        self.body_mut().update_fixed(time);
        result
    }
}

pub enum GameObjectEffect {
    LaunchProjectile(Projectile),
    SpawnSpacecraft(Spacecraft),
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct GameObjectBody {
    pub position: Vec2,
    pub velocity: Vec2,
    pub rotation: f32,
    pub angular_velocity: f32,
    pub cur_time: f32,
    pub acceleration: Vec2,
    pub angular_acceleration: f32,
    pub bounds: Vec<Vec2>,
    pub updated: usize
}

impl GameObjectBody {
    pub fn new(position: Vec2, velocity: Vec2, rotation: f32, cur_time: f32, bounds: Vec<Vec2>) -> Self {
        Self {
            position,
            velocity,
            rotation,
            angular_velocity: 0.,
            acceleration: Vec2::ZERO,
            angular_acceleration: 0.,
            cur_time,
            bounds,
            updated: 0
        }
    }
    pub fn from_position(position: Vec2) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }
    pub fn update(&mut self, time: f32) {
        assert!(time > self.cur_time);

        let dt = time - self.cur_time;

        self.position += self.velocity * dt + 0.5 * self.acceleration * dt * dt;
        self.velocity += self.acceleration * dt;

        self.rotation = (self.rotation
            + self.angular_velocity * dt
            + 0.5 * self.angular_acceleration * dt * dt)
            % (PI * 2.);
        self.angular_velocity += self.angular_acceleration * dt;

        self.angular_acceleration = 0.;
        self.acceleration = Vec2::ZERO;
        self.cur_time = time;
        self.updated += 1;
    }
    pub fn update_fixed(&mut self, time: f32) {
        assert!(time > self.cur_time);

        let dt = time - self.cur_time;

        self.position += self.velocity * dt;
        self.velocity += self.acceleration * dt;

        self.angular_velocity += self.angular_acceleration * dt;

        self.angular_acceleration = 0.;
        self.acceleration = Vec2::ZERO;
        self.cur_time = time;
        self.updated += 1;
    }
    pub fn relative_to_world(&self, relative_pos: Vec2) -> Vec2 {
        relative_pos.rotate_rad(self.rotation) + self.position
    }
    pub fn point_position(&self, index: usize) -> Vec2 {
        self.relative_to_world(self.bounds[index])
    }
}

pub type GameObjectId = u16;
