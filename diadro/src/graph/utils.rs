use eframe::egui::{Pos2, Vec2};
use eframe::emath::{pos2, Rect};

/// Trait describes zoom functional
pub trait Zoom {
    fn zoom(&self, zoom_factor: f32) -> Self;
    fn unzoom(&self, zoom_factor: f32) -> Self;
}

/// Implies zoom functionality for points
impl Zoom for Pos2 {
    fn zoom(&self, zoom_factor: f32) -> Self {
        Pos2::new(self.x * zoom_factor, self.y * zoom_factor)
    }

    fn unzoom(&self, zoom_factor: f32) -> Self {
        Pos2::new(self.x / zoom_factor, self.y / zoom_factor)
    }
}

/// Implies zoom functionality for vecs
impl Zoom for Vec2 {
    fn zoom(&self, zoom_factor: f32) -> Self {
        Vec2::new(self.x * zoom_factor, self.y * zoom_factor)
    }

    fn unzoom(&self, zoom_factor: f32) -> Self {
        Vec2::new(self.x / zoom_factor, self.y / zoom_factor)
    }
}

/// Implies zoom functionality for rectangles
impl Zoom for Rect {
    fn zoom(&self, zoom_factor: f32) -> Self {
        Rect::from_two_pos(self.max.zoom(zoom_factor), self.min.zoom(zoom_factor))
    }

    fn unzoom(&self, zoom_factor: f32) -> Self {
        Rect::from_two_pos(self.max.unzoom(zoom_factor), self.min.unzoom(zoom_factor))
    }
}

/// Trait describes functional for point mathematics
pub trait PointMath {
    fn in_line(&self, line: [Pos2; 2], tolerance: f32) -> bool;
    /// return true if point lays over another point with specified tolerance
    /// ### Arguments
    /// - pos - point to which current point is compared
    /// - tolerance - tolerance
    fn over(&self, pos: Pos2, tolerance: f32) -> bool;
}

/// Implies point mathematics for Pos2
impl PointMath for Pos2 {
    #[inline]
    fn in_line(&self, line: [Pos2; 2], tolerance: f32) -> bool {
        let rect = Rect::from_two_pos(
            Pos2 {
                x: line[0].x - tolerance,
                y: line[0].y - tolerance,
            },
            Pos2 {
                x: line[1].x + tolerance,
                y: line[1].y + tolerance,
            },
        );
        rect.contains(*self)
    }

    #[inline]
    fn over(&self, pos: Pos2, tolerance: f32) -> bool {
        self.distance(pos) < tolerance
    }
}

/// Struct defines two ppoints line
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct TwoPosLine {
    start: Pos2,
    end: Pos2,
    angle: f32,
}

/// Implies some functions for two point lines
#[allow(dead_code)]
impl TwoPosLine {
    /// Compute line angle
    #[inline]
    fn compute_angle(start: Pos2, end: Pos2) -> f32 {
        (end.x - start.x).atan2(end.y - start.y)
    }

    /// Construct line from two piounts array
    pub fn new(points: [Pos2; 2]) -> Self {
        Self {
            start: points[0],
            end: points[1],
            angle: Self::compute_angle(points[0], points[1]),
        }
    }

    /// Move end of line to new point and recompute line angle
    #[inline]
    pub fn move_to(&mut self, end_point: Pos2) {
        self.end = end_point;
        self.angle = Self::compute_angle(self.start, self.end);
    }

    #[inline]
    pub fn set_points(&mut self, points: [Pos2; 2]) {
        self.start = points[0];
        self.end = points[1];
        self.angle = Self::compute_angle(points[0], points[1]);
    }

    /// Compute new point at specified distance from the line end and belonging to the line
    /// ### Arguments
    /// - distance - positive float that specifies distance from end of line
    /// ### Return
    ///   point coordinates
    #[inline]
    pub fn point_from_end(&self, distance: f32) -> Pos2 {
        pos_by_angle(self.end, self.angle, -distance)
    }

    /// Translate current line into new coordinates using deltas
    #[inline]
    pub fn translate(&self, delta: Vec2) -> Self {
        Self::new([
            pos2(self.start.x + delta.x, self.start.y + delta.y),
            pos2(self.end.x + delta.x, self.end.y + delta.y),
        ])
    }

    #[inline]
    pub fn angle(&self) -> f32 {
        self.angle
    }

    #[inline]
    pub fn start(&self) -> Pos2 {
        self.start
    }

    #[inline]
    pub fn end(&self) -> Pos2 {
        self.end
    }

    #[inline]
    pub fn into_points(&self) -> [Pos2; 2] {
        [self.start, self.end]
    }

    #[inline]
    pub fn set_start(&mut self, pos: Pos2) {
        self.start = pos;
    }

    #[inline]
    pub fn center(&self) -> Pos2 {
        pos2(
            (self.start.x + self.end.x) / 2.0,
            (self.start.y + self.end.y) / 2.0,
        )
    }

    /// Function split line by equal parts
    /// ### Arguments
    /// * parts - number of parts to split
    /// ### Return
    /// Ve<Pos> - number of points
    /// ### Example
    /// ```
    /// use eframe::egui::Pos2;
    /// use diadro::graph::TwoPosLine;
    ///
    /// let line = TwoPosLine::new([Pos2::new(0., 0.), Pos2::new(40., 0.)]);
    /// let parts = line.split(4);
    ///
    /// assert_eq!(parts.len(), 5);
    /// assert_eq!(parts[1], Pos2::new(10.0, 0.));
    /// assert_eq!(parts[2], Pos2::new(20.0, 0.));
    /// assert_eq!(parts[3], Pos2::new(30.0, 0.));
    /// assert_eq!(parts[4], Pos2::new(40.0, 0.));
    /// ```
    #[inline]
    pub fn split(&self, parts: usize) -> Vec<Pos2> {
        let f_parts = parts as f32;
        let dx = (self.end.x - self.start.x) / f_parts;
        let dy = (self.end.y - self.start.y) / f_parts;
        let mut res = Vec::with_capacity(parts + 1);
        res.push(self.start);
        for idx in 1..parts {
            res.push(pos2(
                self.start.x + dx * idx as f32,
                self.start.y + dy * idx as f32,
            ));
        }
        res.push(self.end);
        res
    }
}

/// Implies zoom functionality for two points line
impl Zoom for TwoPosLine {
    #[inline]
    fn zoom(&self, zoom_factor: f32) -> Self {
        Self::new([self.start.zoom(zoom_factor), self.end.zoom(zoom_factor)])
    }

    #[inline]
    fn unzoom(&self, zoom_factor: f32) -> Self {
        Self::new([self.start.unzoom(zoom_factor), self.end.unzoom(zoom_factor)])
    }
}

impl From<[Pos2; 2]> for TwoPosLine {
    #[inline]
    fn from(points: [Pos2; 2]) -> Self {
        TwoPosLine::new(points)
    }
}

/// Compute point coordinates based on start point line angle and distance
/// ### Arguments
///  - start - starting point
///  - angle - line angle in radians
///  - distance - distance from start to desired point
/// ### Return
///  new point coordinates
/// ### Example
/// ```
/// use eframe::egui::Pos2;
///
/// let start = Pos2::new(0., 0.);
/// let angle = 3.14159 / 2.0; // 90 degrees
/// let end = diadro::graph::pos_by_angle(start, angle, 10.0);
///
/// assert_eq!(end.x.round(), 10.);
/// ```
#[inline]
pub fn pos_by_angle(start: Pos2, angle: f32, distance: f32) -> Pos2 {
    pos2(
        start.x + distance * angle.sin(),
        start.y + distance * angle.cos(),
    )
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        io::{Cursor, Read},
    };

    use eframe::{
        emath::pos2,
        epaint::{Pos2, Rect, Vec2},
    };

    use crate::graph::utils::PointMath;

    use super::{TwoPosLine, Zoom};

    #[test]
    fn test_shift_line() {
        // By x ordinate
        let line = TwoPosLine::new([pos2(0., 0.), pos2(10., 0.)]);
        let new_pos = line.point_from_end(2.);
        assert_eq!(new_pos.x.round(), 8.0);
        assert_eq!(new_pos.y.round(), 0.0);

        // By y ordinate
        let line = TwoPosLine::new([pos2(0., 0.), pos2(0., 10.)]);
        let new_pos = line.point_from_end(2.);
        assert_eq!(new_pos.x.round(), 0.0);
        assert_eq!(new_pos.y.round(), 8.0);
    }

    #[test]
    fn test_pos_zoom() {
        let pos = Pos2::new(10., 10.);
        let pos = pos.zoom(2.);
        assert_eq!(pos, Pos2::new(20., 20.));
    }

    #[test]
    fn test_vec_zoom() {
        let pos = Vec2::new(10., 10.);
        let pos = pos.zoom(2.);
        assert_eq!(pos, Vec2::new(20., 20.));
    }

    #[test]
    fn test_rect() {
        let zoom_factor = 0.05;
        let rc = Rect::from_two_pos(Pos2::new(2., 2.), Pos2::new(7., 9.));
        let rcz = rc.zoom(zoom_factor);
        assert_eq!(
            rcz,
            Rect::from_two_pos(
                Pos2::new(2. * zoom_factor, 2. * zoom_factor),
                Pos2 {
                    x: 7. * zoom_factor,
                    y: 9. * zoom_factor
                }
            )
        );
    }

    #[test]
    fn test_in_line() {
        let pos = [Pos2::new(5., 5.), Pos2::new(15., 15.)];
        assert!(pos[0].in_line(pos, 5.));
        assert!(pos[1].in_line(pos, 5.));
        assert!(Pos2::new(0., 0.).in_line(pos, 5.));
        assert!(Pos2::new(20., 20.).in_line(pos, 5.));
        assert!(!Pos2::new(25., 25.).in_line(pos, 5.));
    }

    #[test]
    fn test_over() {
        let pos = Pos2::new(5., 5.);
        assert!(pos.over(Pos2::new(5., 5.), 5.));
        assert!(pos.over(Pos2::new(2., 2.), 5.));
        assert!(!pos.over(Pos2::new(20., 20.), 5.));
        assert!(!pos.over(Pos2::new(25., 25.), 5.));
    }

    #[test]
    fn test_line_split() {
        let rect = Rect::from_two_pos(pos2(0., 0.), pos2(40., 40.));
        let line = TwoPosLine::new([rect.left_top(), rect.right_top()]);
        let parts = line.split(4);
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[1], Pos2::new(10.0, 0.));
        assert_eq!(parts[2], Pos2::new(20.0, 0.));
        assert_eq!(parts[3], Pos2::new(30.0, 0.));
        assert_eq!(parts[4], Pos2::new(40.0, 0.));

        let slice = &parts[1..];
        assert_eq!(slice[1], Pos2::new(20.0, 0.));
        assert_eq!(slice[2], Pos2::new(30.0, 0.));
        assert_eq!(slice[3], Pos2::new(40.0, 0.));

        let slice = &parts[..4];
        assert_eq!(slice.len(), 4);
    }

    #[test]
    fn test_read_mxgraph_compressed_xml() {
        use minidom::Element;

        let context = fs::read_to_string("/Users/sbt-sementsov-av/Downloads/Audit2Abyss_arch.xml")
            .expect("File not open");
        let root = context.parse::<Element>();
        let root = root.unwrap();
        for child in root.children() {
            if child.is("diagram", "article") {
                let res = base64::decode(child.text()).expect("Error decoding text");
                let cursor = Cursor::new(res);
                let mut d = flate2::read::DeflateDecoder::new(cursor);
                let mut s = String::new();
                d.read_to_string(&mut s).unwrap();
                let s = urlencoding::decode(&s).expect("error url decoding xml");

                println!("MxGraph content: {}", s);
            }
        }
    }
}
