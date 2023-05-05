mod component;

use crate::prelude::*;
pub use component::*;
use std::collections::BTreeMap;
use std::collections::HashSet;

use super::GameObjectBody;


#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Spacecraft {
    pub owner: PlayerToken,
    pub components: BTreeMap<ComponentId, Component>,
    pub body: GameObjectBody,
    central_component: ComponentId,
    pub inertia: f32,
    pub center_of_mass: Vec2,
    pub mass: f32,
    bounds: Vec<Vec2>,
    pub tags: Vec<String>,
    health: f32,
}

// needs a lot of caching
impl Spacecraft {
    /// Receives a verified structure and builds a spacecraft from it
    pub fn build(
        structure: SpacecraftStructure,
        owner: PlayerToken,
        transform: GameObjectBody,
    ) -> Self {
        let mut components = BTreeMap::new();
        let mut central_component = None;
        for (index, component_placeholder) in structure
            .component_placeholders
            .clone()
            .into_iter()
            .enumerate()
        {
            let component_type = component_placeholder.component_type;
            let orientation = component_placeholder.orientation;

            let component = component_type.build(component_placeholder.position, orientation);
            if let ComponentType::Central = component_type {
                central_component = Some(index as ComponentId);
            }
            let component_id = (if component_type.top().is_some() {
                256 + index
            } else {
                index
            }) as ComponentId;
            components.insert(component_id, component);
        }
        let central_component = central_component.unwrap();
        let mut spacecraft = Self {
            owner,
            components,
            body: transform,
            central_component,
            tags: structure.tags,
            ..Default::default()
        };
        spacecraft.reconstruct();
        spacecraft
    }
    pub fn update(&mut self, time: f32) -> Vec<GameObjectEffect> {
        self.reconstruct();

        let mut result = vec![];
        let mut forces = vec![];
        for (_, component) in &mut self.components {
            for component_effect in component.update(time) {
                match component_effect {
                    ComponentEffect::CreateProjectile(
                        projectile_type,
                        position,
                        velocity,
                        rotation,
                    ) => {
                        result.push(GameObjectEffect::LaunchProjectile(
                            projectile_type.construct(
                                GameObjectBody::new(
                                    (position - self.center_of_mass).rotate_rad(self.body.rotation)
                                        + self.body.position,
                                    velocity.rotate_rad(self.body.rotation) + self.body.velocity,
                                    rotation + self.body.rotation,
                                    time,
                                ),
                                self.owner,
                            ),
                        ));
                    }
                    ComponentEffect::ApplyForce(direction) => {
                        let origin = component.body().centered_position();
                        forces.push((origin, direction));
                    }
                }
            }
        }

        forces
            .into_iter()
            .for_each(|x| self.apply_force_local(x.0, x.1, time-self.body.cur_time));

        self.body.update(time);

        result
    }
    pub fn reconstruct(&mut self) {
        self.components.drain_filter(|_, x| x.health() <= 0.);

        let mut construction = BTreeMap::new();
        let mut top_construction = BTreeMap::new();
        for (id, component) in &self.components {
            if component.body().top().is_some() {
                let pos = component.body().position;
                top_construction.insert((pos.x, pos.y), id);
            } else {
                for pos in component.body().occupied_positions() {
                    construction.insert((pos.x, pos.y), id);
                }
            }
        }

        let Some(central_component) = self.components.get(&self.central_component) else {
            self.components.clear();
            self.health = 0.;
            return;
        };

        let mut stack = central_component.body().occupied_positions();
        let dirs = [ivec2(0, 1), ivec2(1, 0), ivec2(-1, 0), ivec2(0, -1)];

        let mut survives = HashSet::new();
        survives.insert(self.central_component);
        while let Some(pos) = stack.pop() {
            for dir in dirs {
                let new_pos = pos + dir;
                if let Some(id) = construction.get(&(new_pos.x, new_pos.y)) {
                    stack.push(new_pos);
                    survives.insert(**id);
                    if let Some(top_id) = top_construction.get(&(new_pos.x, new_pos.y)) {
                        survives.insert(**top_id);
                    }
                }
                construction.remove(&(new_pos.x, new_pos.y));
            }
        }

        self.components.drain_filter(|id, _| !survives.contains(id));
        let points = self
            .components
            .values()
            .map(|x| x.body().corner_points())
            .flatten()
            .collect::<Vec<Vec2>>();
        self.bounds = convex_hull(points)
            .into_iter()
            .map(|ver| ver - self.center_of_mass)
            .collect();
        self.health = self.components.values().map(|x| x.health()).sum();

        // here was a message that it needs fixing i removed it because i didn't find anything...
        let new_center_of_mass_offset = self.center_of_mass() - self.center_of_mass;
        self.body.position +=
            Vec2::from_angle(self.body.rotation).rotate(new_center_of_mass_offset);
        self.center_of_mass += new_center_of_mass_offset;

        self.inertia = self.inertia();
        self.mass = self.compute_mass();
    }

    pub fn compute_mass(&self) -> f32 {
        self.components.iter().map(|(_, x)| x.mass()).sum()
    }

    fn inertia(&self) -> f32 {
        self.components
            .values()
            .map(|x| {
                x.body()
                    .centered_position()
                    .distance(self.center_of_mass)
                    .powi(2)
                    * x.mass()
            })
            .sum()
    }
    /// Applies a force in a local coordinate system
    fn apply_force_local(&mut self, origin: Vec2, direction: Vec2, dt: f32) {
        self.body.acceleration +=
            direction.rotate(Vec2::from_angle(self.body.rotation)) / self.compute_mass() / dt;
        self.body.angular_acceleration +=
            (origin - self.center_of_mass).perp_dot(direction / dt) / self.inertia;
    }
    fn center_of_mass(&self) -> Vec2 {
        let mut result = Vec2::ZERO;
        let mut total_mass = 0.;
        for component in self.components.values() {
            result += component.body().centered_position() * component.mass();
            total_mass += component.mass();
        }
        result / total_mass
    }
    pub fn execute_component_cmd(&mut self, component_id: ComponentId, cmd: ComponentCmd) {
        if let Some(component) = self.components.get_mut(&component_id) {
            component.handle_cmd(cmd);
        }
    }
    pub fn component_position_local(&self, component_body: &ComponentBody) -> Vec2 {
        (component_body.centered_position() - self.center_of_mass).rotate_rad(self.body.rotation)
    }
}

impl Spacecraft {
    pub fn bounds(&self) -> Vec<Vec2> {
        self.bounds.clone()
    }
    pub fn transform_mut(&mut self) -> &mut GameObjectBody {
        &mut self.body
    }
    pub fn transform(&self) -> &GameObjectBody {
        &self.body
    }
    pub fn health(&self) -> f32 {
        self.health
    }

    pub fn mass(&self) -> f32 {
        self.mass
    }

    pub fn apply_damage(&mut self, damage: f32, position: Vec2) -> Vec<(Material, f32)> {
        let position = position - self.body.position;
        let mut result = vec![];
        let mut damage = damage;
        let mut components = self.components.iter_mut().collect::<Vec<_>>();
        components.sort_by(|a, b| {
            let a_position = (a.1.body().centered_position() - self.center_of_mass)
                .rotate_rad(self.body.rotation);
            let b_position = (b.1.body().centered_position() - self.center_of_mass)
                .rotate_rad(self.body.rotation);
            a_position
                .distance(position)
                .partial_cmp(&b_position.distance(position))
                .unwrap()
        });
        // println!("{:?} distance: {:?}", components[0], (components[0].1.body().centered_position()-self.center_of_mass).rotate_rad(self.transform.rotation));
        for (_, component) in components {
            let component_damage = damage.min(component.health());
            component.apply_damage(component_damage);
            damage -= component_damage;
            self.health -= component_damage;
            result.extend(
                component
                    .body()
                    .origin
                    .materials()
                    .into_iter()
                    .map(|(material, amount)| {
                        (
                            material,
                            amount * component_damage / component.body().origin.health(),
                        )
                    })
                    .collect::<Vec<(Material, f32)>>(),
            );
            if damage <= 0. {
                break;
            }
        }
        result
    }

    pub fn destroyed(&self) -> bool {
        self.components.is_empty()
    }

    pub fn owner(&self) -> Option<PlayerToken> {
        Some(self.owner)
    }
    pub fn collides_point(&self, position: Vec2) -> bool {
        self.body.position.distance(position) < 5.
    }
}
