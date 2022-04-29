mod algo;
mod cell;
mod errors;
mod graph;
mod ucell;

pub use self::ucell::UnMxEdge;

use eframe::{
    egui::Id,
    emath::Pos2,
    epaint::{
        CubicBezierShape, Mesh, PathShape, QuadraticBezierShape, RectShape, Stroke, TextShape,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MxConnectable {
    pub edges: Vec<Id>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CellType {
    Edge(UnMxEdge),
    Connectable(MxConnectable),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Figure {
    Vec(Vec<Figure>),
    LineSegment { points: [Pos2; 2], stroke: Stroke },
    Path(PathShape),
    Rect(RectShape),
    Text(TextShape),
    Mesh(Mesh),
    QuadraticBezier(QuadraticBezierShape),
    CubicBezier(CubicBezierShape),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MxCellState {
    Selected,
    Hovered,
    Dragging,
    Free,
}

#[derive(Serialize, Deserialize, Debug)]
#[must_use = "Add a shape to diagram"]
pub struct MxCell {
    pub id: Id,
    pub cell_type: CellType,
    /// Array of shapes which must be enough to describe the figure
    pub shapes: Vec<Figure>,
    pub connection_points: Vec<Pos2>,
    pub state: MxCellState,
}

pub enum Contained {
    /// In area
    InArea,
    /// In connection point
    ConnectionPoint(usize),
    /// Left to Right
    ResizeLtoR(Pos2),
    /// Right to Left
    ResizeRtoL(Pos2),
    /// Top to Bottom
    ResizeTtoB(Pos2),
    /// Bottom to Top
    ResizeBtoT(Pos2),
    /// TopLeft to BottomRight
    ResizeTLtoBR(Pos2),
    /// BottomRight to TopLeft
    ResizeBRtoTL(Pos2),
    /// TopRight to BottomLeft
    ResizeTRtoBL(Pos2),
    /// BottomLeft to TopRight
    ResizeBLtoTR(Pos2),
}
