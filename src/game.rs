mod asteroid;
pub mod collision_detection;
mod game_object;
pub mod material;
pub mod player;
mod projectile;
pub mod spacecraft;
pub mod spacecraft_structure;
mod star_base;

use crate::prelude::*;
pub use asteroid::Asteroid;
pub use game_object::*;
pub use material::Material;
pub use player::{Player, PlayerToken};
pub use spacecraft::Spacecraft;
pub use spacecraft::{
    Component, ComponentCmd, ComponentId, ComponentType, ComponentWrapper, Orientation,
    SpacecraftEffect,
};
pub use star_base::{StarBase, StarBaseEffect};
pub use {projectile::Projectile, projectile::ProjectileType};

use std::{time::Duration};

use strum::IntoStaticStr;

pub use spacecraft_structure::{ComponentPlaceholder, SpacecraftStructure};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GameSync {
    pub last_update: Duration,
    pub delta_time: f32,
    pub frame: usize,
}

impl Default for GameSync {
    fn default() -> Self {
        Self {
            last_update: now(),
            delta_time: 0.,
            frame: 0,
        }
    }
}

impl GameSync {
    pub fn update(&mut self) {
        self.delta_time =
            (now() - self.last_update.min(now() - Duration::from_millis(1))).as_secs_f32();
        self.last_update = now();
        self.frame += 1;
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ExecutedGameCmd {
    pub user: User,
    pub cmd: GameCmd,
    pub time: Duration,
}

#[derive(Debug, Clone)]
pub enum GameEvent {
    ProjectileLaunched(Projectile),
    SpacecraftDeployed(Spacecraft),
    GameObjectDestroyed(GameObject, GameObject),
}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct Game {
    pub sync: GameSync,
    pub players: HashMap<PlayerToken, Player>, // public keys as public keys
    #[serde(skip)]
    pub cmds_history: Vec<ExecutedGameCmd>,
    pub game_objects: HashMap<GameObjectId, GameObject>,
    #[serde(skip)]
    pub events: Vec<GameEvent>,
}

impl Game {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self) {
        self.sync.update();
        self.events = vec![];

        self.update_collisions();
        self.update_game_objects();
    }

    fn update_game_objects(&mut self) {
        self.game_objects
            .drain_filter(|_, game_object| game_object.health() <= 0.0);

        let mut spacecraft_effects = Vec::new();
        let mut star_base_effects = Vec::new();
        for game_object in self.game_objects.values_mut() {
            match game_object {
                GameObject::Spacecraft(spacecraft) => {
                    spacecraft_effects.extend(spacecraft.update(self.sync.delta_time));
                }
                GameObject::StarBase(star_base) => {
                    star_base_effects.extend(star_base.update(self.sync.delta_time));
                }
                GameObject::Projectile(projectile) => {
                    projectile.update(self.sync.delta_time);
                }
                _ => (),
            }
            game_object.transform_mut().update(self.sync.delta_time);
        }
        for effect in spacecraft_effects {
            self.handle_spacecraft_effect(effect);
        }
        for effect in star_base_effects {
            self.handle_star_base_effect(effect);
        }

        // apply gravity between asteroids

        let mut asteroids = self
            .game_objects
            .values_mut()
            .filter_map(|game_object| match game_object {
                GameObject::Asteroid(asteroid) => Some(asteroid),
                _ => None,
            })
            .collect::<Vec<_>>();

        for i in 0..asteroids.len() {
            for j in i + 1..asteroids.len() {
                let distance = asteroids[i]
                    .transform
                    .position
                    .distance(asteroids[j].transform().position);
                let force = 100. / distance.powi(2).max(5.);
                let direction = (asteroids[j].transform.position
                    - asteroids[i].transform().position)
                    .normalize();
                asteroids[i].transform_mut().acceleration += direction * force;
                asteroids[j].transform_mut().acceleration -= direction * force;
            }
        }
    }

    pub fn execute_cmd(&mut self, user: User, cmd: GameCmd) -> Result<(), GameCmdExecutionError> {
        self.cmds_history.push(ExecutedGameCmd {
            user,
            cmd: cmd.clone(),
            time: now(),
        });
        match cmd {
            GameCmd::SpawnAsteroid(pos, vel) => {
                let new_asteroid = Asteroid::new(
                    Transform::new(pos, vel, 0.),
                    rand::random::<f32>() * 5. + 2.,
                    rand::random(),
                );
                self.game_objects
                    .insert_with_unique_key(GameObject::Asteroid(new_asteroid));
            }
            GameCmd::BuildSpacecraft(game_object_id, spacecraft_structure, hangar_index) => {
                let Some(GameObject::StarBase(star_base)) = self.game_objects.get_mut(&game_object_id) else {
                    return Err(GameCmdExecutionError::InvalidGameObjectId);
                };
                match user {
                    User::Player(player_id) => {
                        if star_base.owner != player_id {
                            return Err(GameCmdExecutionError::NotAuthorized);
                        }
                    }
                    User::Spectator => {
                        return Err(GameCmdExecutionError::NotAuthorized);
                    }
                    User::Server => (),
                }

                let player_id = star_base.owner;

                let materials_required = &spacecraft_structure.materials();
                if spacecraft_structure.valid()
                    && star_base.can_build_spacecraft(&spacecraft_structure, hangar_index)
                    && self
                        .players
                        .get_mut(&player_id)
                        .unwrap()
                        .take_materials(materials_required)
                {
                    star_base.build_spacecraft(&spacecraft_structure, hangar_index)
                }
                else {
                    return Err(GameCmdExecutionError::Other("Couldn't build spacecraft".to_string()));
                }
            }
            GameCmd::DeploySpacecraft(game_object_id, hangar_index) => {
                let Some(GameObject::StarBase(star_base)) = self.game_objects.get_mut(&game_object_id) else {
                    return Err(GameCmdExecutionError::InvalidGameObjectId);
                };

                match user {
                    User::Player(player_id) => {
                        if star_base.owner != player_id {
                            return Err(GameCmdExecutionError::NotAuthorized);
                        }
                    }
                    User::Spectator => {
                        return Err(GameCmdExecutionError::NotAuthorized);
                    }
                    User::Server => (),
                }

                star_base.deploy_spacecraft(hangar_index);
            }
            GameCmd::ExecuteComponentCmd(game_object_id, component_id, component_cmd) => {
                let Some(GameObject::Spacecraft(spacecraft)) = self.game_objects.get_mut(&game_object_id) else {
                    return Err(GameCmdExecutionError::InvalidGameObjectId);
                };
                match user {
                    User::Player(player_id) => {
                        if spacecraft.owner != player_id {
                            return Err(GameCmdExecutionError::NotAuthorized);
                        }
                    }
                    User::Spectator => {
                        return Err(GameCmdExecutionError::NotAuthorized);
                    }
                    User::Server => (),
                }

                spacecraft.execute_component_cmd(component_id, component_cmd);
            }
            GameCmd::AddPlayer(player_id) => {
                match user {
                    User::Player(_) => return Err(GameCmdExecutionError::NotAuthorized),
                    _ => (),
                }

                self.players.insert(player_id, Player::new());
                self.game_objects
                    .insert_with_unique_key(GameObject::StarBase(StarBase::new(
                        Transform::from_position(
                            Vec2::random_direction() * 1000. * rand::random::<f32>().sqrt(),
                        ),
                        player_id,
                    )));
            }
        }
        Ok(())
    }

    fn handle_star_base_effect(&mut self, effect: StarBaseEffect) {
        match effect {
            StarBaseEffect::SpawnSpacecraft(spacecraft) => {
                self.events
                    .push(GameEvent::SpacecraftDeployed(spacecraft.clone()));
                self.game_objects
                    .insert_with_unique_key(GameObject::Spacecraft(spacecraft));
            }
        }
    }

    fn handle_spacecraft_effect(&mut self, effect: SpacecraftEffect) {
        match effect {
            SpacecraftEffect::LaunchProjectile(projectile) => {
                self.events
                    .push(GameEvent::ProjectileLaunched(projectile.clone()));
                self.game_objects
                    .insert_with_unique_key(GameObject::Projectile(projectile));
            }
        }
    }

    fn update_collisions(&mut self) {
        let mut destroyed_game_objects: Vec<(GameObjectId, GameObjectId)> = vec![];

        let game_object_ids = self.game_objects.keys().copied().collect::<Vec<_>>();

        for (i, id1) in game_object_ids.iter().enumerate() {
            for id2 in game_object_ids.iter().skip(i + 1) {
                if (self.game_objects[id1].destroyed() || self.game_objects[id2].destroyed())
                    || (self.game_objects[id1].owner().is_some()
                        && self.game_objects[id1].owner() == self.game_objects[id2].owner())
                {
                    continue;
                }
                if self.game_objects[id1].collides(&self.game_objects[id2]) {
                    // why is this inaccurate? many reasons, first of all the damage isn't applied as an multiple of area overlapping, each object will absorb energy in a different way, the collision center is not actually the point of collision
                    let energy = (self.game_objects[id1].transform().velocity
                        - self.game_objects[id2].transform().velocity)
                        .length()
                        .powi(2)
                        * self.game_objects[id1].destructive_power()
                        * self.game_objects[id2].destructive_power()
                        * self.sync.delta_time;

                    let damage = energy
                        .min(self.game_objects[id1].health())
                        .min(self.game_objects[id2].health());
                    // let position = (game_objects[i].1.transform().position+game_objects[j].1.transform().position) / 2.;

                    let position1 = self.game_objects[id1].transform().position;
                    let position2 = self.game_objects[id2].transform().position;
                    let materials_reward1 = self
                        .game_objects
                        .get_mut(id1)
                        .unwrap()
                        .apply_damage(damage, position2);
                    let materials_reward2 = self
                        .game_objects
                        .get_mut(id2)
                        .unwrap()
                        .apply_damage(damage, position1);

                    if let Some(owner) = self.game_objects[id1].owner() {
                        for (material, amount) in materials_reward2 {
                            *self
                                .players
                                .get_mut(&owner)
                                .unwrap()
                                .materials
                                .entry(material)
                                .or_insert(0.) += amount;
                        }
                    }

                    if let Some(owner) = self.game_objects[id2].owner() {
                        for (material, amount) in materials_reward1 {
                            *self
                                .players
                                .get_mut(&owner)
                                .unwrap()
                                .materials
                                .entry(material)
                                .or_insert(0.) += amount;
                        }
                    }

                    if self.game_objects[id1].health() <= 0. {
                        destroyed_game_objects.push((*id1, *id2));
                    }

                    if self.game_objects[id2].health() <= 0. {
                        destroyed_game_objects.push((*id2, *id1));
                    }
                }
            }
        }

        for (destroyed, destroyer) in destroyed_game_objects {
            self.events.push(GameEvent::GameObjectDestroyed(
                self.game_objects.get(&destroyed).unwrap().clone(),
                self.game_objects.get(&destroyer).unwrap().clone(),
            ));
        }
    }

    pub fn spacecrafts(&self) -> Vec<&Spacecraft> {
        self.game_objects
            .values()
            .filter_map(|game_object| {
                if let GameObject::Spacecraft(spacecraft) = game_object {
                    Some(spacecraft)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn star_bases(&self) -> Vec<&StarBase> {
        self.game_objects
            .values()
            .filter_map(|game_object| {
                if let GameObject::StarBase(star_base) = game_object {
                    Some(star_base)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn asteroids(&self) -> Vec<&Asteroid> {
        self.game_objects
            .values()
            .filter_map(|game_object| {
                if let GameObject::Asteroid(asteroid) = game_object {
                    Some(asteroid)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn projectiles(&self) -> Vec<&Projectile> {
        self.game_objects
            .values()
            .filter_map(|game_object| {
                if let GameObject::Projectile(projectile) = game_object {
                    Some(projectile)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, Copy, PartialEq)]
pub struct Transform {
    pub position: Vec2,
    pub velocity: Vec2,
    pub rotation: f32,
    pub angular_velocity: f32,
    #[serde(skip)]
    pub acceleration: Vec2,
    pub angular_acceleration: f32,
}

// the velocity applied each frame is the average velocity
// at the end of each frame velocity is updated and acceleration set to zero
impl Transform {
    pub fn new(position: Vec2, velocity: Vec2, rotation: f32) -> Self {
        Self {
            position,
            velocity,
            rotation,
            angular_velocity: 0.,
            acceleration: Vec2::ZERO,
            angular_acceleration: 0.,
        }
    }
    pub fn from_position(position: Vec2) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }
    pub fn update(&mut self, dt: f32) {
        self.position += self.velocity * dt + 0.5 * self.acceleration * dt * dt;
        self.velocity += self.acceleration * dt;

        self.rotation = (self.rotation
            + self.angular_velocity * dt
            + 0.5 * self.angular_acceleration * dt * dt)
            % (PI * 2.); // doesn't need to be as accurate (no average)
        self.angular_velocity += self.angular_acceleration * dt;

        self.angular_acceleration = 0.;
        self.acceleration = Vec2::ZERO;
    }
    pub fn relative_to_world(&self, relative_pos: Vec2) -> Vec2 {
        relative_pos.rotate_rad(self.rotation) + self.position
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
// #[serde(tag = "cmd", content = "args")]
pub enum GameCmd {
    SpawnAsteroid(Vec2, Vec2),
    BuildSpacecraft(GameObjectId, SpacecraftStructure, usize),
    ExecuteComponentCmd(GameObjectId, ComponentId, ComponentCmd),
    DeploySpacecraft(GameObjectId, usize),
    AddPlayer(PlayerToken),
}

pub fn run_game(game: Arc<RwLock<Game>>, tick_rate: u32) {
    let target_duration = Duration::from_secs_f32(1. / tick_rate as f32);
    loop {
        let start = std::time::Instant::now();
        game.write().unwrap().update();
        let elapsed = start.elapsed();
        if elapsed < target_duration {
            std::thread::sleep(target_duration - elapsed);
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy, IntoStaticStr)]
// #[serde(tag = "user", content = "args")]
pub enum User {
    Server,
    Spectator,
    Player(PlayerToken),
}

#[derive(Debug)]
pub enum GameCmdExecutionError {
    NotAuthorized,
    InvalidGameObjectId,
    Other(String),
}