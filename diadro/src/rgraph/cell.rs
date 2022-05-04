use eframe::{
    egui::Id,
    emath::{Pos2, Rect, Vec2},
    epaint::Shape,
};

use crate::graph::Zoom;

use super::{
    algo::PointAlgoritm, ucell::UnMxEdge, CellType, Contained, Figure, MxCell, MxCellState,
    MxConnectable,
};

impl Figure {
    /// Convert figure to equi::frame::Shape
    pub fn to_shape(&self) -> Shape {
        match self {
            Figure::Vec(vec_shape) => {
                Shape::Vec(vec_shape.iter().map(|shape| shape.to_shape()).collect())
            }
            Figure::LineSegment { points, stroke } => Shape::LineSegment {
                points: *points,
                stroke: stroke.clone(),
            },
            Figure::Path(path) => Shape::Path(path.clone()),
            Figure::Rect(rect) => Shape::Rect(*rect),
            Figure::Text(text) => Shape::Text(text.clone()),
            Figure::Mesh(mesh) => Shape::Mesh(mesh.clone()),
            Figure::QuadraticBezier(qb) => Shape::QuadraticBezier(qb.clone()),
            Figure::CubicBezier(cb) => Shape::CubicBezier(cb.clone()),
        }
    }

    /// Translate shape
    pub fn translate(&mut self, delta: Vec2) {
        match self {
            Figure::Vec(shapes) => {
                for shape in shapes {
                    shape.translate(delta);
                }
            }
            Figure::LineSegment { points, .. } => {
                for p in points {
                    *p += delta;
                }
            }
            Figure::Path(path_shape) => {
                for p in &mut path_shape.points {
                    *p += delta;
                }
            }
            Figure::Rect(rect_shape) => {
                rect_shape.rect = rect_shape.rect.translate(delta);
            }
            Figure::Text(text_shape) => {
                text_shape.pos += delta;
            }
            Figure::Mesh(mesh) => {
                mesh.translate(delta);
            }
            Figure::QuadraticBezier(bezier_shape) => {
                for p in &mut bezier_shape.points {
                    *p += delta;
                }
            }
            Figure::CubicBezier(cubie_curve) => {
                for p in &mut cubie_curve.points {
                    *p += delta;
                }
            }
        }
    }

    /// Zoom shape
    pub fn zoom(&mut self, zoom_factor: f32) {
        match self {
            Figure::Vec(shapes) => {
                for shape in shapes {
                    shape.zoom(zoom_factor);
                }
            }
            Figure::LineSegment { points, .. } => {
                for p in points {
                    *p = p.zoom(zoom_factor);
                }
            }
            Figure::Path(path_shape) => {
                for p in &mut path_shape.points {
                    *p = p.zoom(zoom_factor);
                }
            }
            Figure::Rect(rect_shape) => {
                rect_shape.rect = rect_shape.rect.zoom(zoom_factor);
            }
            Figure::Text(text_shape) => {
                // TODO: fix galley
                text_shape.pos = text_shape.pos.zoom(zoom_factor);
            }
            Figure::Mesh(mesh) => {
                for vtx in &mut mesh.vertices {
                    vtx.pos = vtx.pos.zoom(zoom_factor);
                }
            }
            Figure::QuadraticBezier(bezier_shape) => {
                for p in &mut bezier_shape.points {
                    *p = p.zoom(zoom_factor);
                }
            }
            Figure::CubicBezier(cubie_curve) => {
                for p in &mut cubie_curve.points {
                    *p = p.zoom(zoom_factor);
                }
            }
        }
    }

    /// Function checks if rectangle contains given point and also detemines for
    /// which actions for rectangle at this point are allowed
    /// ### Arguments
    /// * rect -rectangle coordinates
    /// * point - point to locate
    /// * epsilone - tolerance
    /// ### Return
    /// * Contained enum or None
    fn contains_in_rect(rect: Rect, point: Pos2, epsilon: f32) -> Option<Contained> {
        match rect.contains(point) {
            true if point.distance(rect.right_top()) <= epsilon => {
                Some(Contained::ResizeTRtoBL(point))
            }
            true if point.distance(rect.left_top()) <= epsilon => {
                Some(Contained::ResizeTLtoBR(point))
            }
            true if point.distance(rect.left_bottom()) <= epsilon => {
                Some(Contained::ResizeBLtoTR(point))
            }
            true if point.distance(rect.right_bottom()) <= epsilon => {
                Some(Contained::ResizeBRtoTL(point))
            }
            true if point.belong_line(&[rect.right_top(), rect.right_bottom()], epsilon) => {
                Some(Contained::ResizeRtoL(point))
            }
            true if point.belong_line(&[rect.left_top(), rect.left_bottom()], epsilon) => {
                Some(Contained::ResizeLtoR(point))
            }
            true if point.belong_line(&[rect.left_top(), rect.right_top()], epsilon) => {
                Some(Contained::ResizeTtoB(point))
            }
            true if point.belong_line(&[rect.left_bottom(), rect.right_bottom()], epsilon) => {
                Some(Contained::ResizeBtoT(point))
            }
            true => Some(Contained::InArea),
            _ => None,
        }
    }

    /// Check if the figure contains given point
    /// TODO: Transform to Contains trait and implements the trait for each figure independently
    pub fn contains(&self, point: Pos2, epsilon: f32) -> Option<Contained> {
        match self {
            Figure::Vec(shapes) => shapes
                .iter()
                .find_map(|figure| figure.contains(point, epsilon)),
            Figure::Rect(rect) => Self::contains_in_rect(rect.rect, point, epsilon),
            Figure::LineSegment { points, .. } => {
                if point.belong_line(points, epsilon) {
                    Some(Contained::InArea)
                } else {
                    None
                }
            }
            Figure::Path(path) => {
                if point.belong_path(&path.points, epsilon) {
                    Some(Contained::InArea)
                } else {
                    None
                }
            }
            Figure::Text(text) => {
                Self::contains_in_rect(text.visual_bounding_rect(), point, epsilon)
            }
            _ => {
                tracing::error!("Sorry, I don't know how to determine belonging ath the moment");
                None
            }
        }
    }
}

impl MxCell {
    /// Constructs new empty mx_cell
    pub fn new(id: Id) -> Self {
        Self {
            id,
            cell_type: CellType::Connectable(MxConnectable {
                edges: Default::default(),
            }),
            shapes: Default::default(),
            connection_points: Default::default(),
            state: MxCellState::Free,
        }
    }

    /// Construct empty edge
    pub fn new_edge(id: Id) -> Self {
        Self {
            id,
            cell_type: CellType::Edge(UnMxEdge::new(None, None)),
            shapes: Default::default(),
            connection_points: Default::default(),
            state: MxCellState::Free,
        }
    }

    /// Move all the shapes by this many points, in-place.
    pub fn translate(&mut self, delta: Vec2) -> &mut Self {
        self.shapes.iter_mut().for_each(|shape| {
            shape.translate(delta);
        });
        self
    }

    /// Zoom all the shapes using given zoom_factor
    pub fn zoom(&mut self, zoom_factor: f32) -> &mut Self {
        self.shapes.iter_mut().for_each(|shape| {
            shape.zoom(zoom_factor);
        });
        self
    }

    pub fn connection_points(&self) -> &Vec<Pos2> {
        &self.connection_points
    }

    pub fn set_state(&mut self, state: MxCellState) {
        self.state = state;
    }

    /// Find connection point by pos.
    /// ### Arguments
    /// * point - position used to find closest connection point
    /// * epsilon - ± tolerance over which point will be determined
    /// ### Return
    /// * Option of Contained::ConnectionPoint(connection point index)
    #[inline]
    pub fn find_cp(&self, point: Pos2, epsilon: f32) -> Option<Contained> {
        self.connection_points
            .iter()
            .enumerate()
            .find(|(_, cp)| cp.distance(point) <= epsilon)
            .map(|(idx, _)| Contained::ConnectionPoint(idx))
    }

    /// Check if given point belongs to contained figure or figures&
    /// ### Arguments
    /// * point - position used to find closest connection point
    /// * epsilon - ± tolerance over which point will be determined
    /// ### Return
    /// * Option of Contained enum
    pub fn contains(&self, point: Pos2, epsilon: f32) -> Option<Contained> {
        match &self.cell_type {
            CellType::Edge(edge) => edge.contains(point),
            _ => match self.state {
                MxCellState::Selected => self.find_cp(point, epsilon),

                // check all shapes in
                // ! REVERSE order
                _ => self
                    .shapes
                    .iter()
                    .rev()
                    .find_map(|figure| figure.contains(point, epsilon)),
            },
        }
    }
}
