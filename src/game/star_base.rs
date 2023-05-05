mod hangar;

use crate::game::GameObjectBody;
use crate::prelude::*;
use glam::Vec2;
use hangar::{Hangar, HangarEffect};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct StarBase {
    pub body: GameObjectBody,
    pub owner: PlayerToken,
    health: f32,
    pub hangars: Vec<Hangar>,
}

impl StarBase {
    pub fn transform_mut(&mut self) -> &mut GameObjectBody {
        &mut self.body
    }
    pub fn bounds(&self) -> Vec<Vec2> {
        vec![
            vec2(10., 10.),
            vec2(-10., 10.),
            vec2(-10., -10.),
            vec2(10., -10.),
        ]
    }
    pub fn transform(&self) -> &GameObjectBody {
        &self.body
    }
    pub fn owner(&self) -> Option<PlayerToken> {
        Some(self.owner)
    }
    pub fn apply_damage(&mut self, damage: f32, _position: Vec2) -> Vec<(Material, f32)> {
        self.health -= damage;
        vec![(Material::Gold, damage / 5.)]
    }
    pub fn health(&self) -> f32 {
        self.health
    }
    pub fn destroyed(&self) -> bool {
        self.health <= 0.
    }
    pub fn collides_point(&self, position: Vec2) -> bool {
        self.body.position.distance(position) < 10.
    }
}

impl StarBase {
    pub fn new(transform: GameObjectBody, owner: PlayerToken) -> Self {
        Self {
            body: transform,
            owner,
            health: 10000.,
            hangars: vec![Hangar::new(), Hangar::new()],
        }
    }
    pub fn mass(&self) -> f32 {
        100000.
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
    pub fn update(&mut self, dt: f32) -> Vec<StarBaseEffect> {
        let mut result = vec![];
        for hangar in self.hangars.iter_mut() {
            let hangar_effects = hangar.update(dt);
            for hangar_effect in hangar_effects {
                match hangar_effect {
                    HangarEffect::Deploy(structure) => {
                        let mut spacecraft_transform = self.body.clone();
                        spacecraft_transform.angular_velocity = 0.;
                        spacecraft_transform.position.y += 15.;

                        result.push(StarBaseEffect::SpawnSpacecraft(Spacecraft::build(
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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum StarBaseEffect {
    SpawnSpacecraft(Spacecraft),
}
