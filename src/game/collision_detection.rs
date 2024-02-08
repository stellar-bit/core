use super::*;

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub struct CollisionInfo {
    pub time: f32,
    pub sharp_obj: (GameObjectId, usize, usize),
    pub other_obj: (GameObjectId, usize, usize),
}

impl Eq for CollisionInfo {}

impl Ord for CollisionInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub fn sat_collision_detect(convex_hull1: &Vec<Vec2>, convex_hull2: &Vec<Vec2>) -> bool {
    let num_vertices1 = convex_hull1.len();
    let num_vertices2 = convex_hull2.len();

    // Iterate through all axes of the first convex hull
    for i in 0..num_vertices1 {
        let axis = (convex_hull1[(i + 1) % num_vertices1] - convex_hull1[i])
            .rotate_rad(-PI/2.0)
            .normalize();
        let (min1, max1) = project(convex_hull1, axis);
        let (min2, max2) = project(convex_hull2, axis);

        // Check if projections overlap
        if max1 < min2 || max2 < min1 {
            return false;
        }
    }

    // Iterate through all axes of the second convex hull
    for i in 0..num_vertices2 {
        let axis = (convex_hull2[(i + 1) % num_vertices2] - convex_hull2[i])
            .rotate_rad(-PI/2.0)
            .normalize();
        let (min1, max1) = project(convex_hull1, axis);
        let (min2, max2) = project(convex_hull2, axis);

        // Check if projections overlap
        if max1 < min2 || max2 < min1 {
            return false;
        }
    }

    return true;
}

/// Checks whether obj 1 collides with obj 2 with one of its corners
/// Don't give me vertical lines pls or I'll do some weird stuff
pub fn check_sharp_collision(
    sharp_points: Vec<Vec2>,
    other_points: Vec<Vec2>,
    velocity: Vec2,
    max_t: f32,
) -> Option<(f32, usize, usize)> {
    let mut collision: Option<(f32, usize, usize)> = None;

    for (i, p) in sharp_points.into_iter().enumerate() {
        for j in 0..other_points.len() {
            let a = other_points[j];
            let b = other_points[(j + 1) % other_points.len()];
            let v = velocity;

            let slope_1 = (b.y - a.y) / (b.x - a.x);
            let y_1 = a.y - a.x * slope_1;
            let slope_2 = (v.y) / (v.x);
            let y_2 = p.y - p.x * slope_2;

            let intercept = (y_2 - y_1) / (slope_1 - slope_2);

            if intercept < a.x.min(b.x) || intercept > a.x.max(b.x) {
                continue;
            }

            let time = (intercept - p.x) / (v.x);

            if time.is_nan() || time <= 0. || time >= max_t {
                continue;
            }

            if let Some(cur_answer) = &mut collision {
                if time < cur_answer.0 {
                    *cur_answer = (time, i, j);
                }
            } else {
                collision = Some((time, i, j));
            }
        }
    }

    collision
}

// Helper function to find the minimum and maximum extent of a shape when projected onto an axis
fn project(convex_hull: &Vec<Vec2>, axis: Vec2) -> (f32, f32) {
    let mut min = axis.dot(convex_hull[0]);
    let mut max = min;

    for i in 1..convex_hull.len() {
        let projection = axis.dot(convex_hull[i]);
        if projection < min {
            min = projection;
        }
        if projection > max {
            max = projection;
        }
    }

    (min, max)
}

pub fn convex_hull(mut points: Vec<Vec2>) -> Vec<Vec2> {
    // let points be the list of points
    // let stack = empty_stack()

    // find the lowest y-coordinate and leftmost point, called P0
    // sort points by polar angle with P0, if several points have the same polar angle then only keep the farthest

    // for point in points:
    //     # pop the last point from the stack if we turn clockwise to reach this point
    //     while count stack > 1 and ccw(next_to_top(stack), top(stack), point) <= 0:
    //         pop stack
    //     push point to stack
    // end

    // return stack

    let mut stack: Vec<Vec2> = Vec::new();

    points.sort_by(|a, b| {
        if a.y == b.y {
            a.x.partial_cmp(&b.x).unwrap()
        } else {
            a.y.partial_cmp(&b.y).unwrap()
        }
    });
    points.dedup();

    let p0 = points[0];

    points.sort_by(|a, b| {
        let a = (*a - p0).angle();
        let b = (*b - p0).angle();
        a.partial_cmp(&b).unwrap()
    });

    for point in points {
        while stack.len() > 1
            && (stack[stack.len() - 1] - stack[stack.len() - 2])
                .angle_between(point - stack[stack.len() - 2])
                <= 0.
        {
            stack.pop();
        }
        stack.push(point);
    }

    stack
}
