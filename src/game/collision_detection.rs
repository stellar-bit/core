use super::*;

pub fn sat_collision_detect(convex_hull1: &Vec<Vec2>, convex_hull2: &Vec<Vec2>) -> bool {
    let num_vertices1 = convex_hull1.len();
    let num_vertices2 = convex_hull2.len();

    // Iterate through all axes of the first convex hull
    for i in 0..num_vertices1 {
        let axis = (convex_hull1[(i + 1) % num_vertices1] - convex_hull1[i])
            .rotate_rad(-90.)
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
            .rotate_rad(-90.)
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
