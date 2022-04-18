use std::{borrow::Cow, cell::RefCell, f32::consts::PI, rc::Rc};

use eframe::{
    egui::{CursorIcon, Id, PointerButton, Ui},
    emath::{Pos2, Rect, Vec2},
    epaint::{PathShape, Shape},
};

use crate::graph::pos_by_angle;

use super::{
    shapes::{FigureBasics, SelectMode, SELECT_MODE_NONE, SELECT_MODE_SELECTED},
    utils::{PointMath, TwoPosLine},
    GraphFigure, Zoom,
};

/// Defines connections point
#[derive(Clone)]
pub struct ConnectionPoint {
    /// Figure to which this point is connected
    figure: Rc<RefCell<Box<dyn GraphFigure>>>,
    /// Index within figure's connection points array
    connection_point: usize,
}

impl ConnectionPoint {
    /// Construct new connection point
    pub fn new(figure: Rc<RefCell<Box<dyn GraphFigure>>>, connection_point: usize) -> Self {
        Self {
            figure,
            connection_point,
        }
    }

    /// Return connetion point position
    pub fn get_connection_pos(&self) -> Option<Pos2> {
        let fig = self.figure.borrow();
        fig.connection_points()
            .get(self.connection_point)
            .map(|p| *p)
    }

    /// Return figure top which this connections point is connected
    pub fn get_figure(&self) -> &Rc<RefCell<Box<dyn GraphFigure>>> {
        &self.figure
    }
}

/// Defines edge figure
#[derive(Clone)]
#[allow(dead_code)]
pub struct ArrowFigure {
    /// Identifier
    id: Id,
    /// Start and end position
    line: TwoPosLine,
    wing_size: f32,
    size: f32,

    start_arrow: bool,
    end_arrow: bool,

    zoom_factor: f32,
    scroll_delta: Vec2,
    fb: FigureBasics,
    text: Option<Cow<'static, str>>,
    selected: bool,
    start_figure: Option<ConnectionPoint>,
    end_figure: Option<ConnectionPoint>,
}

impl std::fmt::Debug for ArrowFigure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArrowFigure")
            .field("id", &self.id)
            .field("line", &self.line)
            .field("wing_size", &self.wing_size)
            .field("size", &self.size)
            .field("zoom_factor", &self.zoom_factor)
            .field("scroll_delta", &self.scroll_delta)
            .field("fb", &self.fb)
            .field("text", &self.text)
            .field("selected", &self.selected)
            .field("start_figure", &self.start_figure.is_some())
            .field("end_figure", &self.end_figure.is_some())
            .finish()
    }
}

#[allow(dead_code)]
impl ArrowFigure {
    pub fn new(line: impl Into<TwoPosLine>, id: Id) -> Self {
        Self {
            id,
            line: line.into(),
            wing_size: 20.,
            size: 15.,
            start_arrow: false,
            end_arrow: true,
            zoom_factor: 1.,
            scroll_delta: Vec2::ZERO,
            fb: Default::default(),
            text: None,
            selected: false,
            start_figure: None,
            end_figure: None,
        }
    }

    pub fn start_arrow(&self) -> bool {
        self.start_arrow
    }

    pub fn end_arrow(&self) -> bool {
        self.end_arrow
    }

    pub fn set_start_arrow(&mut self, flag: bool) {
        self.start_arrow = flag;
    }

    pub fn set_end_arrow(&mut self, flag: bool) {
        self.end_arrow = flag;
    }

    pub fn connect_start(&mut self, figure: ConnectionPoint) {
        self.start_figure = Some(figure);
    }

    pub fn disconnect_start(&mut self) {
        self.start_figure = None;
    }

    pub fn connect_end(&mut self, figure: ConnectionPoint) {
        self.end_figure = Some(figure);
    }

    pub fn disconnect_end(&mut self) {
        self.end_figure = None;
    }

    pub fn set_end_pos(&mut self, pos: Pos2) {
        self.line.move_to(pos);
    }

    pub fn get_start_connection(&self) -> &Option<ConnectionPoint> {
        &self.start_figure
    }

    pub fn get_end_connection(&self) -> &Option<ConnectionPoint> {
        &self.end_figure
    }

    /// Function compute nearest point on the rectangle's edges centers for the given point.
    /// ### Arguments
    /// * `rect` - rectangle to check
    /// * `pos` - point to check
    /// ### Returns
    /// * `Pos2` - nearest point on the rectangle's edges centers
    pub fn compute_nearest_point_to_rect(rect: Rect, _point: Pos2) -> Pos2 {
        // Fill rect's edge centers
        let connection_points = [
            rect.center_top() + Vec2 { x: 0., y: -20. },
            rect.center_bottom() + Vec2 { x: 0., y: 20. },
            rect.right_center() + Vec2 { x: 20., y: 0. },
            rect.left_center() + Vec2 { x: -20., y: 0. },
        ];
        let mut distance = f32::MAX;
        let mut min_pos = Pos2::ZERO;
        for point in connection_points {
            let d1 = point.distance(point);
            if d1 < distance {
                distance = d1;
                min_pos = point;
            }
        }
        min_pos
    }

    /// Function computes start point of line. If start point of line is connected to figure then it will be computed based on point in that figure.
    /// If no figures connected then it will be computed based on line's begin.
    pub fn compute_start_point(&self) -> Pos2 {
        if let Some(ref figure) = self.start_figure {
            figure
                .get_connection_pos()
                .unwrap_or_else(|| self.line.start())
        } else {
            self.line.start()
        }
    }

    pub fn compute_end_point(&self) -> Pos2 {
        if let Some(ref figure) = self.end_figure {
            figure
                .get_connection_pos()
                .unwrap_or_else(|| self.line.end())
        } else {
            self.line.end()
        }
    }

    #[inline]
    fn arrow_for_line(&self, angle_grad: f32, distance: f32) -> Vec<Pos2> {
        let line_angle = self.line.angle();
        let rotate = PI;

        // line_angle - angle + 180
        let angle = angle_grad * PI / 180.;
        let left_angle = line_angle + angle + rotate;
        let left_pos = pos_by_angle(self.line.end(), left_angle, distance);
        let right_angle = line_angle - angle + rotate;
        let right_pos = pos_by_angle(self.line.end(), right_angle, distance);
        let center_pos = self.line.point_from_end(distance / 1.5);
        vec![
            self.line.end(),
            left_pos,
            center_pos,
            right_pos,
            self.line.end(),
        ]
    }

    /// Drawing lines between two points: start and end. To determine start and end points there are
    /// self.start_figure and self.end_figure must be used.
    /// Function must use only vertical and hozintal lines to draw
    /// Draw only line, do not drawing arrows
    fn compute_lines_points(&mut self, zoom_factor: f32, scroll_delta: Vec2) -> Vec<Pos2> {
        // Compute line's start and end points
        self.line = self.line.zoom(zoom_factor / self.zoom_factor);
        self.zoom_factor = zoom_factor;
        if self.scroll_delta != scroll_delta {
            self.line = self.line.translate(scroll_delta);
            self.scroll_delta = scroll_delta;
        }

        // Compute real line's start and end points
        self.line
            .set_points([self.compute_start_point(), self.compute_end_point()]);
        self.line.into_points().to_vec()
    }
}

impl GraphFigure for ArrowFigure {
    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn id(&self) -> Id {
        self.id
    }

    fn draw(&mut self, ui: &mut Ui, zoom_factor: f32, scroll_delta: Vec2) {
        // Compute start and end points if defined start and end connections
        let line_points = self.compute_lines_points(zoom_factor, scroll_delta);
        ui.painter()
            .add(Shape::Path(PathShape::line(line_points, self.fb.stroke)));

        ui.painter().add(Shape::convex_polygon(
            self.arrow_for_line(15., 20.).to_vec(),
            self.fb.fill_color,
            self.fb.stroke,
        ));
    }

    fn select(&mut self, selected: SelectMode) {
        self.selected = if selected & SELECT_MODE_SELECTED > 0 {
            true
        } else {
            false
        };
    }

    fn contains(&self, point: Pos2) -> Option<CursorIcon> {
        if point.in_line(self.line.into_points(), 2.) {
            Some(CursorIcon::Grab)
        } else {
            None
        }
    }

    fn selected(&self) -> SelectMode {
        if self.selected {
            SELECT_MODE_SELECTED
        } else {
            SELECT_MODE_NONE
        }
    }

    fn move_to(&mut self, pos: Pos2, drag_started: Pos2) {
        let offset = pos - drag_started;
        self.line = self.line.translate(offset);
    }

    fn drag_start(&mut self, _hover_pos: Pos2, _button: PointerButton, _zoom_factor: f32) {
        todo!()
    }

    fn dragged_by(&mut self, _hover_pos: Pos2, _button: PointerButton) {
        todo!()
    }

    fn drag_released(&mut self, _hover_pos: Pos2, _button: PointerButton) {
        todo!()
    }

    fn double_click(&mut self) {
        todo!()
    }

    fn rect(&self) -> Rect {
        todo!()
    }

    fn connection_points(&self) -> &Vec<Pos2> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_arrow_figure_compute_nearest_point() {}
}
