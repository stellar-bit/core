mod block;
mod engine;
mod orientation;
mod weapon;

use crate::prelude::*;
use glam::IVec2;
pub use orientation::*;
use std::fmt::Debug;

use self::{block::Block, engine::Engine, weapon::Weapon};

#[derive(Serialize, Deserialize, Debug, Clone)]
// #[serde(tag = "cmd", content = "args")]
pub enum ComponentCmd {
    SetActive(bool),
    SetPower(f32),
    SetRotation(f32),
    SelfDestruct,
}

pub type ComponentId = u16;

pub trait ComponentWrapper {
    fn body(&self) -> &ComponentBody;
    fn update(&mut self, dt: f32) -> Vec<ComponentEffect>;
    fn mass(&self) -> f32;
    fn health(&self) -> f32;
    fn handle_cmd(&mut self, cmd: ComponentCmd) {}
    fn apply_damage(&mut self, damage: f32);
}

#[derive(Serialize, Deserialize, Debug, Clone)]
// #[serde(tag = "type")]
pub enum Component {
    Block(Block),
    Engine(Engine),
    Weapon(Weapon),
}

impl Component {
    pub fn body(&self) -> &ComponentBody {
        match self {
            Component::Block(block) => &block.body,
            Component::Engine(engine) => &engine.body,
            Component::Weapon(weapon) => &weapon.body,
        }
    }
    pub fn update(&mut self, dt: f32) -> Vec<ComponentEffect> {
        match self {
            Component::Block(block) => block.update(dt),
            Component::Engine(engine) => engine.update(dt),
            Component::Weapon(weapon) => weapon.update(dt),
        }
    }
    pub fn mass(&self) -> f32 {
        match self {
            Component::Block(block) => block.mass(),
            Component::Engine(engine) => engine.mass(),
            Component::Weapon(weapon) => weapon.mass(),
        }
    }
    pub fn health(&self) -> f32 {
        match self {
            Component::Block(block) => block.health(),
            Component::Engine(engine) => engine.health(),
            Component::Weapon(weapon) => weapon.health(),
        }
    }
    pub fn handle_cmd(&mut self, cmd: ComponentCmd) {
        match self {
            Component::Block(block) => block.handle_cmd(cmd),
            Component::Engine(engine) => engine.handle_cmd(cmd),
            Component::Weapon(weapon) => weapon.handle_cmd(cmd),
        }
    }
    pub fn apply_damage(&mut self, damage: f32) {
        match self {
            Component::Block(block) => block.apply_damage(damage),
            Component::Engine(engine) => engine.apply_damage(damage),
            Component::Weapon(weapon) => weapon.apply_damage(damage),
        }
    }
}

/// The body represents static properties of a component
/// Position specifies a point in grid along which is component rotated when orientation is set
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ComponentBody {
    pub position: IVec2,
    pub orientation: Orientation,
    pub origin: ComponentType,
}

impl ComponentBody {
    pub fn scale(&self) -> UVec2 {
        self.origin.scale()
    }
    pub fn top(&self) -> Option<TopComponentProperties> {
        self.origin.top()
    }
    pub fn centered_position(&self) -> Vec2 {
        self.position.as_vec2()
            + (self.scale() - UVec2::ONE)
                .as_ivec2()
                .orient(self.orientation)
                .as_vec2()
                / 2.
    }
    pub fn occupied_positions(&self) -> Vec<IVec2> {
        let mut result = vec![];

        if let Some(top) = self.top() {
            for pos in &top.occupies {
                result.push(self.position + pos.orient(self.orientation))
            }
        } else {
            for w in 0..self.scale().x {
                for h in 0..self.scale().y {
                    result.push(self.position + uvec2(w, h).as_ivec2().orient(self.orientation))
                }
            }
        }
        result
    }
    pub fn corner_points(&self) -> Vec<Vec2> {
        vec![
            vec2(-0.5, -0.5),
            vec2(-0.5, 0.5),
            vec2(self.origin.scale().x as f32 - 0.5, -0.5),
            self.origin.scale().as_vec2() - vec2(0.5, 0.5),
        ]
        .into_iter()
        .map(|v| v.orient(self.orientation) + self.position.as_vec2())
        .collect()
    }
}

pub enum ComponentEffect {
    CreateProjectile(ProjectileType, Vec2, Vec2, f32), // position, velocity, rotation
    ApplyForce(Vec2), // for now only direction is sufficient, for more detail we might consider to add origin too
}

#[derive(Clone, Serialize, Deserialize, Debug, Copy, PartialEq, Eq)]
pub enum ComponentType {
    Central,
    SteelBlock,
    RaptorEngine,
    LaserWeapon,
    MissileLauncher,
}

impl ComponentType {
    pub fn build(&self, position: IVec2, orientation: Orientation) -> Component {
        let body = ComponentBody {
            position,
            orientation,
            origin: *self,
        };
        let health = self.health();
        match self {
            ComponentType::Central => Component::Block(Block::new(body, 10., health)),
            ComponentType::SteelBlock => Component::Block(Block::new(body, 50., health)),
            ComponentType::RaptorEngine => Component::Engine(Engine::new(
                body,
                10.,
                health,
                200.,
                50000.,
                0.01,
                vec2(-0.8, 0.),
            )),
            ComponentType::LaserWeapon => Component::Weapon(Weapon::new(
                body,
                10.,
                health,
                5.,
                ProjectileType::Laser,
                50.,
                vec2(0.48, 0.),
                PI,
            )),
            ComponentType::MissileLauncher => Component::Weapon(Weapon::new(
                body,
                40.,
                health,
                20.,
                ProjectileType::Missile,
                30.,
                vec2(1.48, 0.),
                PI / 2.,
            )),
        }
    }
    pub fn health(&self) -> f32 {
        match &self {
            ComponentType::Central => 200.,
            ComponentType::SteelBlock => 100.,
            ComponentType::RaptorEngine => 150.,
            ComponentType::LaserWeapon => 50.,
            ComponentType::MissileLauncher => 100.,
        }
    }
    pub fn scale(&self) -> UVec2 {
        match self {
            ComponentType::Central => uvec2(1, 1),
            ComponentType::SteelBlock => uvec2(1, 1),
            ComponentType::RaptorEngine => uvec2(2, 1),
            ComponentType::LaserWeapon => uvec2(1, 1),
            ComponentType::MissileLauncher => uvec2(2, 1),
        }
    }
    pub fn build_time(&self) -> f32 {
        match self {
            ComponentType::Central => 0.,
            ComponentType::SteelBlock => 1.,
            ComponentType::RaptorEngine => 2.,
            ComponentType::LaserWeapon => 2.,
            ComponentType::MissileLauncher => 3.,
        }
    }
    pub fn top(&self) -> Option<TopComponentProperties> {
        match self {
            ComponentType::Central => None,
            ComponentType::SteelBlock => None,
            ComponentType::RaptorEngine => None,
            ComponentType::LaserWeapon => Some(TopComponentProperties {
                occupies: vec![ivec2(0, 0)],
            }),
            ComponentType::MissileLauncher => Some(TopComponentProperties {
                occupies: vec![
                    ivec2(0, 0),
                    ivec2(1, 0),
                    ivec2(0, 1),
                    ivec2(1, 1),
                    ivec2(1, -1),
                    ivec2(0, -1),
                ],
            }),
        }
    }
    pub fn materials(&self) -> Vec<(Material, f32)> {
        match self {
            ComponentType::Central => vec![(Material::Gold, 25.)],
            ComponentType::SteelBlock => vec![(Material::Steel, 20.)],
            ComponentType::RaptorEngine => vec![(Material::Copper, 10.), (Material::Steel, 10.)],
            ComponentType::LaserWeapon => vec![(Material::Steel, 5.), (Material::Silver, 5.)],
            ComponentType::MissileLauncher => vec![
                (Material::Steel, 10.),
                (Material::Silver, 10.),
                (Material::Gold, 5.),
            ],
        }
    }
}

pub struct TopComponentProperties {
    pub occupies: Vec<IVec2>,
}
