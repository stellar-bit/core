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
use rand::{SeedableRng, Rng};
use rand_chacha::ChaChaRng;
pub use spacecraft::Spacecraft;
pub use spacecraft::{
    Component, ComponentCmd, ComponentId, ComponentType, Orientation,
};
pub use star_base::{StarBase};
pub use {projectile::Projectile, projectile::ProjectileType};

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::time::Duration;

use strum::IntoStaticStr;

pub use spacecraft_structure::{ComponentPlaceholder, SpacecraftStructure};

use self::collision_detection::{CollisionInfo, check_sharp_collision};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GameSync {
    pub last_update: Duration,
    pub frame: usize,
}

impl GameSync {
    pub fn new() -> Self {
        Self {
            last_update: now(),
            frame: 0,
        }
    }
    pub fn update(&mut self) {
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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Game {
    pub sync: GameSync,
    pub players: HashMap<PlayerToken, Player>, // public keys as public keys
    #[serde(skip)]
    pub cmds_history: Vec<ExecutedGameCmd>,
    pub game_objects: HashMap<GameObjectId, GameObject>,
    #[serde(skip)]
    pub events: Vec<GameEvent>,
    pub time_elapsed: f32,
    rng: ChaChaRng,
    log: Vec<String>
}

impl Game {
    pub fn new() -> Self {
        Self {
            sync: GameSync::new(),
            players: HashMap::new(),
            cmds_history: vec![],
            game_objects: HashMap::new(),
            events: vec![],
            time_elapsed: 0.,
            rng: ChaChaRng::from_entropy(),
            log: vec![]
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.sync.update();
        self.events = vec![];

        self.time_elapsed += dt;

        let time_measure = time::Instant::now();
        self.update_collisions();
        self.update_game_objects();
    }

    fn update_game_objects(&mut self) {
        let _ = self.game_objects
            .extract_if(|_, game_object| game_object.health() <= 0.).collect::<Vec<_>>();

        let mut effects = vec![];
        for game_object in self.game_objects.values_mut() {
            effects.extend(game_object.update(self.time_elapsed));
        }

        for effect in effects {
            self.handle_game_object_effect(effect);
        }

        self.apply_gravity();
    }
    pub fn apply_gravity(&mut self) {
        // let mut asteroids = self
        //     .game_objects
        //     .values_mut()
        //     .filter_map(|game_object| match game_object {
        //         GameObject::Asteroid(asteroid) => Some(asteroid),
        //         _ => None,
        //     })
        //     .collect::<Vec<_>>();

        // for i in 0..asteroids.len() {
        //     for j in i + 1..asteroids.len() {
        //         let distance = asteroids[i]
        //             .body
        //             .position
        //             .distance(asteroids[j].body.position);
        //         let force = 100. / distance.powi(2).max(5.);
        //         let direction =
        //             (asteroids[j].body.position - asteroids[i].body.position).normalize();
        //         asteroids[i].body.acceleration += direction * force;
        //         asteroids[j].body.acceleration -= direction * force;
        //     }
        // }
    }
    pub fn execute_cmd(&mut self, user: User, cmd: GameCmd) -> Result<(), GameCmdExecutionError> {
        self.cmds_history.push(ExecutedGameCmd {
            user,
            cmd: cmd.clone(),
            time: now(),
        });
        match cmd {
            GameCmd::AddLogMessage(msg) => {
                if user != User::Server {
                    return Err(GameCmdExecutionError::NotAuthorized);
                }
                self.log.push(msg);
            }
            GameCmd::SpawnStarBase(player_id, position, velocity) => {
                if user != User::Server {
                    return Err(GameCmdExecutionError::NotAuthorized);
                }
                if !self.players.contains_key(&player_id) {
                    return Err(GameCmdExecutionError::InvalidId);
                }
                self.game_objects
                    .insert_with_unique_key(GameObject::StarBase(StarBase::new(
                        position,
                        velocity,
                        self.time_elapsed,
                        player_id,
                    )), &mut self.rng);
            }
            GameCmd::SpawnRandomAsteroid(pos, vel) => {
                if user != User::Server {
                    return Err(GameCmdExecutionError::NotAuthorized);
                }
                let new_asteroid = Asteroid::new(
                    pos, vel, self.time_elapsed,
                    self.rng.gen::<f32>() * 5. + 2.,
                    self.rng.gen(),
                );
                self.game_objects
                    .insert_with_unique_key(GameObject::Asteroid(new_asteroid), &mut self.rng);
            }
            GameCmd::BuildSpacecraft(game_object_id, spacecraft_structure, hangar_index) => {
                let Some(GameObject::StarBase(star_base)) = self.game_objects.get_mut(&game_object_id) else {
                    return Err(GameCmdExecutionError::InvalidId);
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
                } else {
                    return Err(GameCmdExecutionError::Other(
                        "Couldn't build spacecraft".to_string(),
                    ));
                }
            }
            GameCmd::DeploySpacecraft(game_object_id, hangar_index) => {
                let Some(GameObject::StarBase(star_base)) = self.game_objects.get_mut(&game_object_id) else {
                    return Err(GameCmdExecutionError::InvalidId);
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
                    return Err(GameCmdExecutionError::InvalidId);
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
                if user != User::Server {
                    return Err(GameCmdExecutionError::NotAuthorized);
                }

                self.players.insert(player_id, Player::new());
            }
        }
        Ok(())
    }

    fn handle_game_object_effect(&mut self, effect: GameObjectEffect) {
        match effect {
            GameObjectEffect::LaunchProjectile(projectile) => {
                self.events
                    .push(GameEvent::ProjectileLaunched(projectile.clone()));
                self.game_objects
                    .insert_with_unique_key(GameObject::Projectile(projectile), &mut self.rng);
            }
            GameObjectEffect::SpawnSpacecraft(spacecraft) => {
                self.events
                    .push(GameEvent::SpacecraftDeployed(spacecraft.clone()));
                self.game_objects
                    .insert_with_unique_key(GameObject::Spacecraft(spacecraft), &mut self.rng);
            }
        }
    }

    fn update_collisions(&mut self) {
        let mut destroyed_game_objects: Vec<(GameObjectId, GameObjectId)> = vec![];

        let game_object_ids = self.game_objects.keys().copied().collect::<Vec<_>>();
        
        // -------------------COLLISIONS------------------- //

        let mut collisions_pq = BinaryHeap::new();

        macro_rules! add_collisions {
            ($obj_1_id:expr, $obj_2_id:expr) => {
                let owner1 = self.game_objects[&$obj_1_id].owner();
                let owner2 = self.game_objects[&$obj_2_id].owner();
                if owner1.is_none() || owner1 != owner2 {
                    if let Some(collision) = self.check_sharp_object_collision($obj_1_id, $obj_2_id) {
                        collisions_pq.push(Reverse(collision));
                    }
                    if let Some(collision) = self.check_sharp_object_collision($obj_2_id, $obj_1_id) {
                        collisions_pq.push(Reverse(collision));
                    }
                }
            };
        }

        //---------------- x-bound check optimization --------------------

        #[derive(PartialOrd, PartialEq, Copy, Clone, Default)]
        struct XBound(f32, f32, GameObjectId);
        impl Eq for XBound {}
        impl Ord for XBound {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.0.partial_cmp(&other.0).unwrap()
            }
        } 

        macro_rules! compute_x_bound {
            ($id: expr) => {
                {
                    let mut body = self.game_objects[$id].body().clone();
                    let mut xs = body.bounds.clone().into_iter().map(|p| body.relative_to_world(p).x).collect::<Vec<_>>();

                    body.update_fixed(self.time_elapsed);

                    xs.extend(body.bounds.clone().into_iter().map(|p| body.relative_to_world(p).x));

                    (*xs.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&-1.), *xs.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.))
                }
            }
        }

        let mut x_bounds = game_object_ids.iter().map(|id| {
            let (min, max) = compute_x_bound!(id);
            XBound(min, max, *id)
        }).collect::<Vec<_>>();

        let mut bounds_left_bt = BTreeSet::from_iter(x_bounds.clone());
        let mut bounds_right_bt = BTreeSet::from_iter(x_bounds.iter().map(|bound| XBound(bound.1, bound.0, bound.2)));
        
        let id_map = game_object_ids.iter().enumerate().map(|(i, id)| (*id, i)).collect::<HashMap<_, _>>();

        game_object_ids.iter().enumerate().for_each(|(i, id)| {
            let x_range = x_bounds[i]..XBound(x_bounds[i].1, 0., 0);

            for other_bound in bounds_left_bt.range(x_range.clone()).chain(bounds_right_bt.range(x_range)) {
                add_collisions!(*id, other_bound.2);
            }
        });

        while let Some(Reverse(col)) = collisions_pq.pop() {
            if !self.handle_collision(col) {
                continue;
            }

            let ids = [col.sharp_obj.0, col.other_obj.0];
            for i in 0..2 {
                if self.game_objects[&ids[i]].health() <= 0. {
                    destroyed_game_objects.push((ids[i], ids[(i+1)%2]));
                }
                let new_bound = compute_x_bound!(&ids[i]);
                let new_bound = XBound(new_bound.0, new_bound.1, ids[i]);

                let old_bound = x_bounds[id_map[&ids[i]]];
                x_bounds[id_map[&ids[i]]] = new_bound;

                bounds_left_bt.remove(&old_bound);
                bounds_right_bt.remove(&XBound(old_bound.1, old_bound.0, old_bound.2));

                bounds_left_bt.insert(new_bound);
                bounds_right_bt.insert(XBound(new_bound.1, new_bound.0, new_bound.2));

                let x_range = new_bound..XBound(new_bound.1, 0., 0);

                for other_bound in bounds_left_bt.range(x_range.clone()).chain(bounds_right_bt.range(x_range)) {
                    add_collisions!(ids[i], other_bound.2);
                }
            }
        }
        // -------------------END COLLISIONS------------------- //

        for (destroyed, destroyer) in destroyed_game_objects {
            self.events.push(GameEvent::GameObjectDestroyed(
                self.game_objects.get(&destroyed).unwrap().clone(),
                self.game_objects.get(&destroyer).unwrap().clone(),
            ));
        }
    }

    /// Returns true if collision was valid and handled
    pub fn handle_collision(&mut self, col: CollisionInfo) -> bool {
        let (sharp_obj_id, sharp_obj_stamp, sharp_obj_point) = col.sharp_obj;
        let (other_obj_id, other_obj_stamp, other_obj_line) = col.other_obj;

        if sharp_obj_stamp != self.game_objects[&sharp_obj_id].body().updated ||
            other_obj_stamp != self.game_objects[&other_obj_id].body().updated ||
            self.game_objects[&sharp_obj_id].health() <= 0. ||
            self.game_objects[&other_obj_id].health() <= 0. {
            return false;
        }
        for eff in self.game_objects.get_mut(&sharp_obj_id).unwrap().update_fixed(col.time) {
            self.handle_game_object_effect(eff);
        }
        for eff in self.game_objects.get_mut(&other_obj_id).unwrap().update_fixed(col.time) {
            self.handle_game_object_effect(eff);
        }

        let sharp_obj = self.game_objects.get(&sharp_obj_id).unwrap();
        let other_obj = self.game_objects.get(&other_obj_id).unwrap();

        if other_obj.body().bounds.len() == 0 {
            return false;
        }

        let col_line = (other_obj.body().point_position(other_obj_line%other_obj.body().bounds.len()), other_obj.body().point_position((other_obj_line+1)%other_obj.body().bounds.len()));
        let point_of_collision = sharp_obj.body().point_position(sharp_obj_point);

        let normal = (col_line.0-col_line.1).perp().normalize();

        let mass1 = sharp_obj.mass();
        let mass2 = other_obj.mass();

        let bounciness = sharp_obj.bounciness() * other_obj.bounciness();

        let relative_velocity = other_obj.body().velocity-sharp_obj.body().velocity;

        let impulse_numerator = -(1.+bounciness) * (relative_velocity).dot(normal);
        let impulse_denominator = (1./mass1) + (1./mass2);
        let impulse = impulse_numerator/impulse_denominator;

        let destructive_power = sharp_obj.destructive_power() * other_obj.destructive_power();
        let damage = (1.-bounciness) * relative_velocity.dot(normal).abs().powi(2) * destructive_power;

        let sharp_obj_owner = sharp_obj.owner();
        let other_obj_owner = other_obj.owner();

        let sharp_obj = self.game_objects.get_mut(&sharp_obj_id).unwrap();
        sharp_obj.body_mut().velocity -= impulse * normal / mass1;
        sharp_obj.body_mut().position += normal*0.005;

        let material_gain = sharp_obj.apply_damage(damage*mass2, point_of_collision);
        if let Some(player_id) = other_obj_owner {
            let player = self.players.get_mut(&player_id).unwrap();
            player.give_materials(material_gain);
        }

        let other_obj = self.game_objects.get_mut(&other_obj_id).unwrap();

        other_obj.body_mut().velocity += impulse * normal / mass2;
        other_obj.body_mut().position -= normal*0.005;

        let material_gain = other_obj.apply_damage(damage*mass1, point_of_collision);
        if let Some(player_id) = sharp_obj_owner {
            let player = self.players.get_mut(&player_id).unwrap();
            player.give_materials(material_gain);
        }

        true
    }

    pub fn check_sharp_object_collision(&self, sharp_obj_id: GameObjectId, other_obj_id: GameObjectId) -> Option<CollisionInfo> {
        let mut sharp_body = self.game_objects[&sharp_obj_id].body().clone();
        let mut other_body = self.game_objects[&other_obj_id].body().clone();

        let cur_time = sharp_body.cur_time.max(other_body.cur_time);

        sharp_body.position += sharp_body.velocity*(cur_time-sharp_body.cur_time);
        other_body.position += other_body.velocity*(cur_time-other_body.cur_time);

        sharp_body.velocity -= other_body.velocity;
        other_body.velocity = Vec2::ZERO;

        // we rotate extra 1 (any number) to avoid vertical and horizontal lines
        let extra_rot = 1.;
        let sharp_points = sharp_body.bounds.clone().into_iter().map(|x| sharp_body.relative_to_world(x).rotate_rad(extra_rot)).collect();
        let other_points = other_body.bounds.clone().into_iter().map(|x| other_body.relative_to_world(x).rotate_rad(extra_rot)).collect();
        sharp_body.velocity = sharp_body.velocity.rotate_rad(extra_rot);

        check_sharp_collision(sharp_points, other_points, sharp_body.velocity, self.time_elapsed-cur_time).map(|(dt, sharp_point, other_line)| {
            CollisionInfo {
                time: cur_time+dt,
                sharp_obj: (sharp_obj_id, sharp_body.updated, sharp_point),
                other_obj: (other_obj_id, other_body.updated, other_line)
            }
        })
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

#[derive(Serialize, Deserialize, Clone, Debug)]
// #[serde(tag = "cmd", content = "args")]
pub enum GameCmd {
    SpawnRandomAsteroid(Vec2, Vec2),
    BuildSpacecraft(GameObjectId, SpacecraftStructure, usize),
    ExecuteComponentCmd(GameObjectId, ComponentId, ComponentCmd),
    DeploySpacecraft(GameObjectId, usize),
    AddPlayer(PlayerToken),
    SpawnStarBase(PlayerToken, Vec2, Vec2),
    AddLogMessage(String)
}

pub fn run_game(game: Arc<RwLock<Game>>, tick_rate: u32) {
    let target_duration = Duration::from_secs_f32(1. / tick_rate as f32);
    loop {
        let start = std::time::Instant::now();
        game.write().unwrap().update(target_duration.as_secs_f32());
        let elapsed = start.elapsed();
        if elapsed < target_duration {
            std::thread::sleep(target_duration - elapsed);
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy, IntoStaticStr, PartialEq)]
// #[serde(tag = "user", content = "args")]
pub enum User {
    Server,
    Spectator,
    Player(PlayerToken),
}

#[derive(Debug)]
pub enum GameCmdExecutionError {
    NotAuthorized,
    InvalidId,
    Other(String),
}
