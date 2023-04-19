#![feature(btree_drain_filter)]
#![feature(hash_drain_filter)]

pub mod game;
pub mod network;

pub const GAME_BUFFER_SIZE: usize = 2 << 15;
pub const CONSTRUCTION_SIZE: usize = 30;

pub mod prelude {
    pub use super::game;
    pub use super::network;

    pub use bincode;
    use rand::distributions::Standard;
    use serde::de::DeserializeOwned;
    pub use serde_json;
    pub fn serialize_str<T: Serialize>(data: &T) -> Result<String, std::io::Error> {
        serde_yaml::to_string(data).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
    pub fn serialize_bytes<T: Serialize>(data: &T) -> Result<Vec<u8>, std::io::Error> {
        bincode::serialize(data).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
    pub fn deserialize_str<T: DeserializeOwned>(data: &str) -> Result<T, std::io::Error> {
        serde_yaml::from_str(data).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
    pub fn deserialize_bytes<T: DeserializeOwned>(data: &[u8]) -> Result<T, std::io::Error> {
        bincode::deserialize(data).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    pub use serde::{self, Deserialize, Serialize};

    pub use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
    pub use std::f32::consts::PI;
    pub use std::sync::{Arc, Mutex, RwLock};
    pub use std::time;

    pub use chrono;
    pub use rand;

    pub use super::Interval;

    pub use network::{ClientRequest, NetworkError, ServerResponse};

    pub use game::{
        collision_detection::convex_hull, Asteroid, Component, ComponentCmd, ComponentId,
        ComponentPlaceholder, ComponentType, ComponentWrapper, Game, GameCmd, GameEvent,
        GameObject, GameObjectId, Material, Orientation, Player, PlayerToken, Projectile,
        ProjectileType, Spacecraft, SpacecraftStructure, StarBase, Transform, User,
    };
    pub use glam::{ivec2, uvec2, vec2, IVec2, UVec2, Vec2};

    pub fn now() -> time::Duration {
        time::Duration::from_millis(chrono::Local::now().timestamp_millis() as u64)
    }

    pub fn normalize_radians(mut rot: f32) -> f32 {
        if rot < 0. {
            rot = 2. * PI + rot;
        }
        rot %= 2. * PI;
        if rot > PI {
            rot = rot - 2. * PI;
        }
        rot
    }

    pub fn swap_max<T: PartialOrd>(a: &mut T, b: &mut T) {
        if a < b {
            std::mem::swap(a, b);
        }
    }

    pub fn swap_min<T: PartialOrd>(a: &mut T, b: &mut T) {
        if a > b {
            std::mem::swap(a, b);
        }
    }

    pub trait InsertRandomKey<K, V> {
        fn insert_with_unique_key(&mut self, value: V) -> K;
    }

    impl<K: std::hash::Hash + std::cmp::Eq + Clone, V> InsertRandomKey<K, V> for HashMap<K, V>
    where
        Standard: rand::distributions::Distribution<K>,
    {
        fn insert_with_unique_key(&mut self, value: V) -> K {
            let mut key = rand::random();
            while self.contains_key(&key) {
                key = rand::random();
            }
            self.insert(key.clone(), value);
            key
        }
    }

    pub trait Vec2Ext {
        fn rotate_rad(self, radians: f32) -> Self;
        fn angle(self) -> f32;
        fn random_direction() -> Vec2 {
            Vec2::from_angle(rand::random::<f32>() * 2.0 * PI)
        }
        fn random_unit_circle() -> Vec2 {
            Vec2::random_direction() * rand::random::<f32>().sqrt()
        }
    }

    impl Vec2Ext for Vec2 {
        fn rotate_rad(self, radians: f32) -> Self {
            self.rotate(Vec2::from_angle(radians))
        }
        fn angle(self) -> f32 {
            if self == Vec2::ZERO {
                return 0.0;
            }
            Vec2::X.angle_between(self)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn radians_normalization() {
        let rotation = 3.28;
        let target_rotation = rotation - 2. * PI;

        assert_eq!(normalize_radians(rotation), target_rotation);

        let rotation = 6.98;
        let target_rotation = rotation - 2. * PI;

        assert_eq!(normalize_radians(rotation), target_rotation);

        let rotation = 6.98 + PI;
        let target_rotation = rotation - 4. * PI;

        assert_eq!(normalize_radians(rotation), target_rotation);

        // low precision
        // assert_eq!(normalize_radians(rotation), target_rotation);

        let rotation = -3.28;
        let target_rotation = 2. * PI + rotation;

        assert_eq!(normalize_radians(rotation), target_rotation);

        let rotation = -6.98;
        let target_rotation = rotation + 4. * PI;

        assert_eq!(rotation, target_rotation);
    }
}

use std::time;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Interval {
    last: time::Duration,
    interval: time::Duration,
}

impl Interval {
    pub fn new(interval: time::Duration) -> Self {
        Self {
            last: prelude::now(),
            interval,
        }
    }
    pub fn check(&mut self) -> bool {
        let now  = prelude::now();
        if now-self.last > self.interval {
            self.last = now;
            true
        } else {
            false
        }
    }
}
