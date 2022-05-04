use eframe::emath::Pos2;

pub trait PointAlgoritm {
    /// Check if point lies on given line
    fn belong_line(&self, line: &[Pos2; 2], epsilon: f32) -> bool;
    fn belong_path(&self, path: &Vec<Pos2>, epsilon: f32) -> bool;
}

impl PointAlgoritm for Pos2 {
    fn belong_line(&self, line: &[Pos2; 2], epsilon: f32) -> bool {
        // Normalization (?)
        let line_delta = line[0] - line[1];
        let point_delta = line[0] - *self;

        if point_delta.x == 0. && point_delta.y == 0. {
            true
        } else if line_delta.x == 0. {
            // Check if point coordinates lies between start and end points
            point_delta.y == 0.
                || (line[0].x - epsilon <= self.x
                    && line[0].x + epsilon >= self.x
                    && line_delta.y.signum() == point_delta.y.signum()
                    && line_delta.y <= point_delta.y)
        } else if line_delta.y == 0. {
            point_delta.x == 0.
                || (line[0].y - epsilon <= self.y
                    && line[0].y + epsilon >= self.y
                    && line_delta.x.signum() == point_delta.x.signum()
                    && line_delta.x <= point_delta.x)
        } else {
            let k = line_delta.y / line_delta.x;
            let b = line[1].y - line[1].x * k;

            // compute x
            let y = k * self.x + b;
            let x = (self.y - b) / k;

            (self.x - x).abs() <= epsilon
                && (self.y - y).abs() <= epsilon
                && (line_delta.x.signum() == point_delta.x.signum() || point_delta.x == 0.)
                && line_delta.x <= point_delta.x - (epsilon * point_delta.x.signum())
                && (line_delta.y.signum() == point_delta.y.signum() || point_delta.y == 0.)
                && line_delta.y <= point_delta.y - (epsilon * point_delta.y.signum())
        }
    }

    fn belong_path(&self, points: &Vec<Pos2>, epsilon: f32) -> bool {
        for idx in 1..points.len() {
            let start = points[idx - 1];
            let end = points[idx];
            if self.belong_line(&[start, end], epsilon) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_find_map() {
        let array = [1, 2, 3, 4, 2, 5, 6, 7, 8, 16, 18, 24];
        let found = array
            .iter()
            .find_map(|v| if v % 2 == 0 { Some(v) } else { None });
        assert_eq!(found, Some(&2));

        let found = array
            .iter()
            .rev()
            .find_map(|v| if v % 2 == 0 { Some(v) } else { None });
        assert_eq!(found, Some(&24));
    }
}
