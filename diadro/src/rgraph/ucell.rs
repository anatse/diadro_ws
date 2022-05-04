use std::{
    cell::{Ref, RefCell, RefMut},
    f32::consts::PI,
    fmt::Debug,
    rc::Rc,
};

use eframe::{
    egui::{Id, Ui},
    emath::{pos2, Pos2, Vec2},
    epaint::{Color32, PathShape, Shape, Stroke},
};
use serde::de::{Deserialize, Visitor};
use serde::{
    de::MapAccess,
    ser::{Serialize, SerializeStruct},
};

use crate::graph::Zoom;

use super::{algo::PointAlgoritm, Contained, MxCell};

/// Defines edge with reference to figures at the start and end of edge
pub struct UnMxEdge {
    /// Start figure. TODO: Change MxCell to MxVertex
    start: Option<Rc<RefCell<MxCell>>>,
    start_point: Option<usize>,
    /// Edn figure
    end: Option<Rc<RefCell<MxCell>>>,
    end_point: Option<usize>,
    /// Line points include start and end. This points must be computed every time when line changes
    points: Vec<Pos2>,
    /// Default epsilon (tolerance)
    epsilon: f32,

    zoom_factor: f32,
    scroll_delta: Vec2,
    stroke: Stroke,
    arrow_start: bool,
    arrow_end: bool,
}

impl Debug for UnMxEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnMxEdge")
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}

/// Serialize edge as pair of ids of tthe connected figures
impl Serialize for UnMxEdge {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("UnMxEdge", 2)?;
        if let Some(start) = &self.start {
            state.serialize_field("start", &start.borrow().id)?;
        }

        if let Some(start_point) = &self.start_point {
            state.serialize_field("start_point", &start_point)?;
        }

        if let Some(end) = &self.end {
            state.serialize_field("end", &end.borrow().id)?;
        }

        if let Some(end_point) = &self.end_point {
            state.serialize_field("end_point", &end_point)?;
        }

        if !self.points.is_empty() {
            state.serialize_field("points", &self.points)?;
        }

        state.serialize_field("epsilon", &self.epsilon)?;
        state.serialize_field("stroke", &self.stroke)?;

        state.serialize_field("arrow_start", &self.arrow_start)?;
        state.serialize_field("arrow_end", &self.arrow_end)?;

        state.end()
    }
}

struct UnMxEdgeVisitor;

/// Part of the deserialization UnMxEdge
impl<'de> Visitor<'de> for UnMxEdgeVisitor {
    type Value = UnMxEdge;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("`start` or `end`")
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut start: Option<MxCell> = None;
        let mut start_point: Option<usize> = None;
        let mut end: Option<MxCell> = None;
        let mut end_point: Option<usize> = None;
        let mut points: Vec<Pos2> = vec![];
        let mut epsilon: f32 = UnMxEdge::EPSILON;
        let mut stroke = UnMxEdge::default_stroke();
        let mut arrow_start = false;
        let mut arrow_end = false;

        while let Some(key) = map.next_key()? {
            match key {
                "start" => {
                    let value: u64 = map.next_value()?;
                    let end_id: Option<Id> = match serde_json::from_str(value.to_string().as_str())
                    {
                        Ok(v) => Some(v),
                        Err(err) => {
                            tracing::error!("Error deserialing UnMxCell: {}", err);
                            None
                        }
                    };
                    start = end_id.map(MxCell::new);
                }
                "start_point" => {
                    start_point = Some(map.next_value()?);
                }
                "end" => {
                    let value: u64 = map.next_value()?;
                    let end_id: Option<Id> = match serde_json::from_str(value.to_string().as_str())
                    {
                        Ok(v) => Some(v),
                        Err(err) => {
                            tracing::error!("Error deserialing UnMxCell: {}", err);
                            None
                        }
                    };
                    end = end_id.map(MxCell::new);
                }
                "end_point" => {
                    end_point = Some(map.next_value()?);
                }
                "points" => {
                    let value: Vec<Pos2> = map.next_value()?;
                    points.extend(&value);
                }
                "epsilon" => {
                    epsilon = map.next_value()?;
                }
                "stroke" => {
                    stroke = map.next_value()?;
                }
                "arrow_start" => arrow_start = map.next_value()?,
                "arrow_end" => arrow_end = map.next_value()?,
                _ => {}
            }
        }

        Ok(UnMxEdge {
            start: start.map(|v| Rc::new(RefCell::new(v))),
            start_point,
            end: end.map(|v| Rc::new(RefCell::new(v))),
            end_point,
            points,
            epsilon,
            zoom_factor: 1.,
            scroll_delta: Vec2::ZERO,
            stroke,
            arrow_start,
            arrow_end,
        })
    }
}

/// Part of the deserialization UnMxEdge
impl<'de> Deserialize<'de> for UnMxEdge {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["start", "end"];
        deserializer.deserialize_struct("name", FIELDS, UnMxEdgeVisitor)
    }
}

pub enum EdgeVertex {
    Cell(Rc<RefCell<MxCell>>, usize),
    Pos(Pos2),
}

/// Main implementation of edge connected by reference
impl UnMxEdge {
    /// Tolerance to determine belonging point to line
    const EPSILON: f32 = 3.;

    pub fn default_stroke() -> Stroke {
        Stroke::new(1., Color32::YELLOW)
    }

    /// Create new edge from start and end figures.
    pub fn new(start: Option<Rc<RefCell<MxCell>>>, end: Option<Rc<RefCell<MxCell>>>) -> Self {
        Self {
            start,
            start_point: None,
            end,
            end_point: None,
            points: vec![],
            epsilon: Self::EPSILON,
            zoom_factor: 1.,
            scroll_delta: Vec2::ZERO,
            stroke: Self::default_stroke(),
            arrow_start: false,
            arrow_end: false,
        }
    }

    pub fn from_vertices(start: EdgeVertex, end: EdgeVertex) -> Self {
        match (start, end) {
            (EdgeVertex::Cell(s, sp), EdgeVertex::Cell(e, ep)) => Self {
                start: Some(s),
                start_point: Some(sp),
                end: Some(e),
                end_point: Some(ep),
                points: vec![],
                epsilon: Self::EPSILON,
                zoom_factor: 1.,
                scroll_delta: Vec2::ZERO,
                stroke: Self::default_stroke(),
                arrow_start: false,
                arrow_end: false,
            },
            (EdgeVertex::Cell(s, sp), EdgeVertex::Pos(pos)) => Self {
                start: Some(s),
                start_point: Some(sp),
                end: None,
                end_point: None,
                points: vec![pos2(f32::NAN, f32::NAN), pos],
                epsilon: Self::EPSILON,
                zoom_factor: 1.,
                scroll_delta: Vec2::ZERO,
                stroke: Self::default_stroke(),
                arrow_start: false,
                arrow_end: false,
            },
            (EdgeVertex::Pos(pos), EdgeVertex::Cell(e, ep)) => Self {
                start: None,
                start_point: None,
                end: Some(e),
                end_point: Some(ep),
                points: vec![pos, pos2(f32::NAN, f32::NAN)],
                epsilon: Self::EPSILON,
                zoom_factor: 1.,
                scroll_delta: Vec2::ZERO,
                stroke: Self::default_stroke(),
                arrow_start: false,
                arrow_end: false,
            },
            (EdgeVertex::Pos(spos), EdgeVertex::Pos(epos)) => Self {
                start: None,
                start_point: None,
                end: None,
                end_point: None,
                points: vec![spos, epos],
                epsilon: Self::EPSILON,
                zoom_factor: 1.,
                scroll_delta: Vec2::ZERO,
                stroke: Self::default_stroke(),
                arrow_start: false,
                arrow_end: false,
            },
        }
    }

    /// Return immutable reference to the start figure
    pub fn get_start(&self) -> Option<Ref<'_, MxCell>> {
        self.start.as_ref().map(|s| s.borrow())
    }

    /// Return immutable reference to the end figure
    pub fn get_end(&self) -> Option<Ref<'_, MxCell>> {
        self.end.as_ref().map(|s| s.borrow())
    }

    /// Return mutable reference to the start fogure
    pub fn get_start_mut(&self) -> Option<RefMut<'_, MxCell>> {
        self.start.as_ref().map(|s| s.borrow_mut())
    }

    /// Return mutable reference to the end figure
    pub fn get_end_mut(&self) -> Option<RefMut<'_, MxCell>> {
        self.end.as_ref().map(|e| e.borrow_mut())
    }

    /// Return reference to the end figure
    pub fn get_end_rc(&self) -> Option<Rc<RefCell<MxCell>>> {
        self.end.as_ref().cloned()
    }

    /// Set start figure fot the edge
    pub fn set_start(&mut self, mx_cell: Rc<RefCell<MxCell>>, point: usize) {
        self.start = Some(mx_cell);
        self.start_point = Some(point);
        self.compute_points();
    }

    /// Set end figure fot the edge
    pub fn set_end(&mut self, mx_cell: Rc<RefCell<MxCell>>, point: usize) {
        self.end = Some(mx_cell);
        self.end_point = Some(point);
        self.compute_points();
    }
}

/// Implies geometry logic
impl UnMxEdge {
    /// Get position from figure and connection point
    #[inline]
    fn get_figure_pos(mx: &Option<Rc<RefCell<MxCell>>>, pos: Option<usize>) -> Option<Pos2> {
        match (mx, pos) {
            (Some(c), Some(p)) => Some(c.borrow().connection_points[p]),
            _ => None,
        }
    }

    /// Compute line points
    pub fn compute_points(&mut self) {
        let last = self.points.len() - 1;
        let start =
            Self::get_figure_pos(&self.start, self.start_point).unwrap_or_else(|| self.points[0]);

        let end =
            Self::get_figure_pos(&self.end, self.end_point).unwrap_or_else(|| self.points[last]);

        self.points[0] = start;
        self.points[last] = end;
    }

    /// Check is line contains given point& Return type of containing. Possible values:
    ///  - Contained::InArea - point lies on line
    ///  - Contained::ConnectionPoint - point lies on special connection point
    /// ### Arguments
    /// * point - point to check
    /// ### Return
    /// return Non if not contains otherwise return type of containing
    pub fn contains(&self, point: Pos2) -> Option<Contained> {
        if self.points.is_empty() {
            return None;
        }

        if point.distance(self.points[0]) <= self.epsilon {
            return Some(Contained::ConnectionPoint(0));
        }

        if self.points.len() == 1 {
            return None;
        }

        for idx in 1..self.points.len() {
            let start = self.points[idx - 1];
            let end = self.points[idx];

            // Check for connection point
            if point.distance(end) <= self.epsilon {
                return Some(Contained::ConnectionPoint(idx));
            }

            if point.belong_line(&[start, end], self.epsilon) {
                return Some(Contained::InArea);
            }
        }

        None
    }
}

/// Implies drawing functionsl
impl UnMxEdge {
    const ARROW_WING_ANGLE: f32 = 15.;
    const ARROW_WING_SIZE: f32 = 20.;

    #[inline]
    fn compute_angle(start: Pos2, end: Pos2) -> f32 {
        (end.x - start.x).atan2(end.y - start.y)
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
    fn pos_by_angle(start: Pos2, angle: f32, distance: f32) -> Pos2 {
        pos2(
            start.x + distance * angle.sin(),
            start.y + distance * angle.cos(),
        )
    }

    pub fn draw(&mut self, ui: &mut Ui, zoom_factor: f32, scroll_delta: Vec2) {
        // Recompute points each time when drawing
        self.compute_points();

        let transformed: Vec<Pos2> = self
            .points
            .iter()
            .map(|p| {
                let np = p.zoom(zoom_factor / self.zoom_factor);
                if self.scroll_delta != scroll_delta {
                    np + scroll_delta
                } else {
                    np
                }
            })
            .collect();

        self.zoom_factor = zoom_factor;
        self.scroll_delta = scroll_delta;

        let last = transformed.len() - 1;
        let start_line = [transformed[0], transformed[1]];
        let end_line = [transformed[last - 1], transformed[last]];

        ui.painter()
            .add(Shape::Path(PathShape::line(transformed, self.stroke)));

        if self.arrow_start {
            ui.painter().add(Shape::convex_polygon(
                Self::arrow_for_line(start_line, Self::ARROW_WING_ANGLE, Self::ARROW_WING_SIZE),
                self.stroke.color,
                self.stroke,
            ));
        }

        if self.arrow_end {
            ui.painter().add(Shape::convex_polygon(
                Self::arrow_for_line(end_line, Self::ARROW_WING_ANGLE, Self::ARROW_WING_SIZE),
                self.stroke.color,
                self.stroke,
            ));
        }
    }

    #[inline]
    fn arrow_for_line(line: [Pos2; 2], angle_grad: f32, distance: f32) -> Vec<Pos2> {
        let start = line[0];
        let end = line[1];
        let line_angle = Self::compute_angle(start, end);
        let rotate = PI;

        let angle = angle_grad * PI / 180.;
        let left_angle = line_angle + angle + rotate;
        let left_pos = Self::pos_by_angle(end, left_angle, distance);
        let right_angle = line_angle - angle + rotate;
        let right_pos = Self::pos_by_angle(end, right_angle, distance);
        let center_pos = Self::pos_by_angle(end, line_angle, -distance / 1.5);
        vec![end, left_pos, center_pos, right_pos, end]
    }
}

#[cfg(test)]
mod tests {
    use super::UnMxEdge;
    use crate::rgraph::{Contained, MxCell};
    use eframe::{
        egui::Id,
        emath::{pos2, Pos2},
    };
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn serialization_test() {
        let mx1 = MxCell::new(Id::new(1));
        let mx2 = MxCell::new(Id::new(2));
        let mut edge = UnMxEdge::new(
            Some(Rc::new(RefCell::new(mx1))),
            Some(Rc::new(RefCell::new(mx2))),
        );
        edge.points = vec![pos2(1., 2.)];

        let json = serde_json::to_string(&edge).unwrap();
        assert_eq!(
            r#"{"start":15326068958072818760,"end":16069757468406242631,"points":[{"x":1.0,"y":2.0}],"epsilon":3.0,"stroke":{"width":1.0,"color":[255,255,0,255]},"arrow_start":false,"arrow_end":false}"#,
            json
        );

        let edge_de: UnMxEdge = serde_json::from_str(&json).unwrap();
        assert_eq!(
            edge_de.start.as_ref().map(|v| v.borrow().id),
            edge.start.as_ref().map(|v| v.borrow().id)
        );

        assert_eq!(edge_de.points, edge.points);
        assert_eq!(
            edge_de.start.as_ref().map(|v| v.borrow().id),
            edge.start.as_ref().map(|v| v.borrow().id)
        );

        let json_2 = serde_json::to_string(&edge_de).unwrap();
        assert_eq!(json, json_2);
    }

    #[test]
    #[should_panic]
    fn test_refs_wrong() {
        let mx1 = MxCell::new(Id::new(1));
        let mx2 = MxCell::new(Id::new(1));
        let edge = UnMxEdge::new(
            Some(Rc::new(RefCell::new(mx1))),
            Some(Rc::new(RefCell::new(mx2))),
        );

        // Firts get start (unsafe part)
        let x = edge.get_start();
        // Here must be panic - runtime borrow checker
        let y = edge.get_start_mut();

        println!("{:?}/{:?}", x, y);
    }

    #[test]
    fn test_refs_properly() {
        let mx1 = MxCell::new(Id::new(1));
        let mx2 = MxCell::new(Id::new(1));
        let edge = UnMxEdge::new(
            Some(Rc::new(RefCell::new(mx1))),
            Some(Rc::new(RefCell::new(mx2))),
        );

        // Firts get start (unsafe part)
        {
            let x = edge.get_start();
            println!("{:?}", x);
        }

        let y = edge.get_start_mut();
        println!("{:?}", y);
    }

    #[test]
    #[should_panic]
    fn test_rc_wrong() {
        let mx1 = MxCell::new(Id::new(1));
        let mx2 = MxCell::new(Id::new(1));
        let edge = UnMxEdge::new(
            Some(Rc::new(RefCell::new(mx1))),
            Some(Rc::new(RefCell::new(mx2))),
        );

        // Firts get start (unsafe part)
        let x = edge.get_end_rc();
        // Here must be panic - runtime borrow checker
        let mut y = edge.get_end_mut().unwrap();
        y.connection_points.push(Pos2::ZERO);

        assert_eq!(x.unwrap().borrow().connection_points.len(), 1);
    }

    #[test]
    fn test_rc_properly() {
        let mx1 = MxCell::new(Id::new(1));
        let mx2 = MxCell::new(Id::new(1));
        let edge = UnMxEdge::new(
            Some(Rc::new(RefCell::new(mx1))),
            Some(Rc::new(RefCell::new(mx2))),
        );

        // Firts get start (unsafe part)
        let x = edge.get_end_rc();
        {
            let mut y = edge.get_end_mut().unwrap();
            y.connection_points.push(Pos2::ZERO);
        }

        assert_eq!(x.unwrap().borrow().connection_points.len(), 1);
    }

    #[test]
    fn test_contains() {
        let mx1 = MxCell::new(Id::new(1));
        let mx2 = MxCell::new(Id::new(2));
        let mut edge = UnMxEdge::new(
            Some(Rc::new(RefCell::new(mx1))),
            Some(Rc::new(RefCell::new(mx2))),
        );

        // For vertical line
        edge.points = vec![pos2(1., 2.), pos2(1., 5.)];
        assert!(edge.contains(pos2(1., 3.)).is_some());
        assert!(edge.contains(pos2(3., 5.)).is_some());
        assert!(edge.contains(pos2(0., 5.)).is_some());
        assert!(edge.contains(pos2(4.1, 5.)).is_none());

        // For horizontal line
        edge.points = vec![pos2(1., 2.), pos2(10., 2.)];
        assert!(edge.contains(pos2(1., 3.)).is_some());
        assert!(edge.contains(pos2(3., 5.)).is_some());
        assert!(edge.contains(pos2(0., 2.)).is_some());
        assert!(edge.contains(pos2(4.1, 6.)).is_none());

        // For oblique (angle != 90 or 0) line
        edge.points = vec![pos2(1., 1.), pos2(100., 100.)];
        assert!(edge.contains(pos2(1., 1.)).is_some());
        assert!(edge.contains(pos2(100., 100.)).is_some());
        assert!(edge.contains(pos2(53., 50.)).is_some());
        assert!(edge.contains(pos2(100., 98.)).is_some());
        assert!(edge.contains(pos2(104., 97.)).is_none());

        // For oblique (angle != 90 or 0) line
        edge.points = vec![pos2(1., 1.), pos2(200., 100.)];
        assert!(edge.contains(pos2(2., 1.)).is_some());
        assert!(edge.contains(pos2(199., 100.)).is_some());
        assert!(edge.contains(pos2(102., 50.)).is_some());
        assert!(edge.contains(pos2(77., 35.)).is_none());

        edge.points = vec![pos2(1., 1.), pos2(30., 10.)];
        assert!(edge.contains(pos2(3., 1.)).is_some());
        assert!(edge.contains(pos2(29., 10.)).is_some());
        assert!(edge.contains(pos2(31., 10.)).is_some());
    }

    #[test]
    fn test_multiline_contains() {
        let mx1 = MxCell::new(Id::new(1));
        let mx2 = MxCell::new(Id::new(2));
        let mut edge = UnMxEdge::new(
            Some(Rc::new(RefCell::new(mx1))),
            Some(Rc::new(RefCell::new(mx2))),
        );

        edge.points = vec![pos2(1., 2.), pos2(1., 5.), pos2(3., 30.)];
        assert!(edge.contains(pos2(1., 5.)).is_some());
        assert!(edge.contains(pos2(2., 15.)).is_some());
    }

    #[test]
    fn test_multiline_contains_cp() {
        let mx1 = MxCell::new(Id::new(1));
        let mx2 = MxCell::new(Id::new(2));
        let mut edge = UnMxEdge::new(
            Some(Rc::new(RefCell::new(mx1))),
            Some(Rc::new(RefCell::new(mx2))),
        );

        edge.points = vec![pos2(1., 2.), pos2(1., 50.), pos2(3., 90.)];
        let idx = edge.contains(pos2(1., 48.)).map(|cp| match cp {
            Contained::ConnectionPoint(idx) => idx,
            _ => 0xff,
        });

        assert_eq!(idx, Some(1));
    }
}
