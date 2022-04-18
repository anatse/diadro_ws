use eframe::{
    egui::Id,
    emath::{Pos2, Vec2},
    epaint::Shape,
};

use crate::graph::Zoom;

use super::{CellType, Figure, MxCell, MxConnectable, MxEdge};

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
        }
    }

    /// Construct empty edge
    pub fn new_edge(id: Id) -> Self {
        Self {
            id,
            cell_type: CellType::Edge(MxEdge {
                start: None,
                end: None,
            }),
            shapes: Default::default(),
            connection_points: Default::default(),
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
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_relations() {
        let mut cells = vec![MxCell::new(Id::new("test1")), MxCell::new(Id::new("test2"))];
        let mut edges = vec![
            MxCell::new_edge(Id::new("edge1")),
            MxCell::new_edge(Id::new("edge2")),
        ];
        if let Some(edge) = edges.get_mut(0) {
            match &mut edge.cell_type {
                CellType::Edge(e) => {
                    e.start = Some(cells[0].id);
                }
                _ => {}
            }
        }
        cells[0].connection_points.push(Pos2::ZERO);

        let edge_id = edges[0].id;
        let mut indices = MxCellIndices::new();
        cells.into_iter().for_each(|cell| indices.add_cell(cell));
        edges.into_iter().for_each(|cell| indices.add_cell(cell));

        let edge = indices.extract_edge(edge_id).unwrap();
        let con = indices.get(edge.start.unwrap()).unwrap();
        assert_eq!(1, con.connection_points().len());
    }

    // #[test]
    // fn test_mx_relations() {
    // let cells = vec![MxCell::new(Id::new("test1")), MxCell::new(Id::new("test2"))].into_iter().map(|c| Rc::new(RefCell::new(c)))
    //     .collect::<Vec<Rc<RefCell<MxCell>>>>();
    // let edges = vec![MxCell::new_edge(Id::new("edge1")), MxCell::new_edge(Id::new("edge2"))].into_iter().map(|c| Rc::new(RefCell::new(c)))
    //     .collect::<Vec<Rc<RefCell<MxCell>>>>();

    // let mut ct = RefCell::borrow_mut(edges[0].as_ref());
    // match &mut ct.cell_type {
    //     crate::rgraph::CellType::Edge(edge) => {
    //         let c_ref = Rc::new(RefCell::borrow(&cells[0]));
    //         edge.end = Some(Rc::new();
    //     }
    //     crate::rgraph::CellType::Connectable(con) => todo!(),
    // }
    // }

    use eframe::{egui::Id, emath::Pos2};

    use crate::rgraph::{CellType, MxCell, MxCellIndices};
}
