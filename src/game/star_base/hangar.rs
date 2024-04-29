use crate::prelude::*;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Hangar {
    pub build_speed: f32,
    pub progress: f32,
    pub deploy: bool,
    pub building_queue: VecDeque<SpacecraftStructure>,
}

impl std::fmt::Display for Hangar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Hangar: deploy: {}, in queue: {}, progress: {}/{}",
            self.deploy,
            self.building_queue.len(),
            self.progress,
            if let Some(structure) = self.building_queue.front() {
                structure.build_time()
            } else {
                0.
            }
        )
    }
}

impl Hangar {
    pub fn new() -> Self {
        Self {
            build_speed: 1.,
            progress: 0.,
            deploy: false,
            building_queue: VecDeque::new(),
        }
    }
    pub fn build(&mut self, structure: SpacecraftStructure) {
        self.building_queue.push_back(structure);
    }
    pub fn update(&mut self, dt: f32) -> Vec<HangarEffect> {
        if self.building_queue.is_empty() {
            return vec![];
        }
        let mut result = vec![];

        if self.deploy && self.build_finished() {
            self.deploy = false;
            self.progress = 0.;
            if let Some(structure) = self.building_queue.pop_front() {
                result.push(HangarEffect::Deploy(structure));
            }
        }
        self.progress += self.build_speed * dt;

        result
    }
    pub fn build_finished(&self) -> bool {
        if let Some(structure) = self.building_queue.front() {
            self.progress >= structure.build_time()
        } else {
            true
        }
    }
    pub fn total_finish_time(&self) -> f32 {
        (self
            .building_queue
            .iter()
            .map(|s| s.build_time())
            .sum::<f32>()
            - self.progress)
            / self.build_speed
    }
}

#[derive(Clone, Debug)]
pub enum HangarEffect {
    Deploy(SpacecraftStructure),
}
