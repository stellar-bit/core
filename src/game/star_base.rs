mod hangar;

use crate::game::GameObjectBody;
use crate::prelude::*;
use glam::Vec2;
use hangar::{Hangar, HangarEffect};

const STARBASE_SIZE: f32 = 15.;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct StarBase {
    pub body: GameObjectBody,
    pub owner: PlayerId,
    health: f32,
    pub hangars: Vec<Hangar>,
}

impl StarBase {
    pub fn owner(&self) -> Option<PlayerId> {
        Some(self.owner)
    }
    pub fn apply_damage(&mut self, damage: f32, _position: Vec2) -> Vec<(Material, f32)> {
        self.health -= damage;
        vec![(Material::Nickel, damage / 5.)]
    }
    pub fn health(&self) -> f32 {
        self.health
    }
    pub fn destroyed(&self) -> bool {
        self.health <= 0.
    }
}

impl StarBase {
    pub fn new(position: Vec2, velocity: Vec2, time: f32, owner: PlayerId) -> Self {
        let body = GameObjectBody::new(
            position,
            velocity,
            0.,
            time,
            vec![
                vec2(STARBASE_SIZE, STARBASE_SIZE),
                vec2(-STARBASE_SIZE, STARBASE_SIZE),
                vec2(-STARBASE_SIZE, -STARBASE_SIZE),
                vec2(STARBASE_SIZE, -STARBASE_SIZE),
            ],
        );
        Self {
            body,
            owner,
            health: 2000.,
            hangars: vec![Hangar::new(), Hangar::new()],
        }
    }
    pub fn mass(&self) -> f32 {
        10000.
    }
    pub fn can_build_spacecraft(
        &self,
        structure: &SpacecraftStructure,
        hangar_index: usize,
    ) -> bool {
        structure.valid() && self.hangars.get(hangar_index).is_some()
    }
    pub fn build_spacecraft(&mut self, structure: &SpacecraftStructure, hangar_index: usize) {
        self.hangars[hangar_index].build(structure.clone());
    }
    pub fn deploy_spacecraft(&mut self, hangar_index: usize) {
        let Some(hangar) = self.hangars.get_mut(hangar_index) else {
            return;
        };
        hangar.deploy = true;
    }
    pub fn update(&mut self, time: f32) -> Vec<GameObjectEffect> {
        let dt = time - self.body.cur_time;
        let mut result = vec![];
        for hangar in self.hangars.iter_mut() {
            let hangar_effects = hangar.update(dt);
            for hangar_effect in hangar_effects {
                match hangar_effect {
                    HangarEffect::Deploy(structure) => {
                        let mut spacecraft_transform = self.body.clone();
                        spacecraft_transform.angular_velocity = 0.;
                        spacecraft_transform.position.y += 25.;

                        result.push(GameObjectEffect::SpawnSpacecraft(Spacecraft::build(
                            structure,
                            self.owner,
                            spacecraft_transform,
                        )));
                    }
                }
            }
        }

        result
    }
}
