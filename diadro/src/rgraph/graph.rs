use std::{cell::RefCell, rc::Rc};

use eframe::emath::{Pos2, Vec2};

use super::{ucell::UnMxEdge, MxCell};

#[allow(dead_code)]
pub(crate) struct Transform {
    pub(crate) scroll_delta: Vec2,
    pub(crate) zoom_factor: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            scroll_delta: Vec2::ZERO,
            zoom_factor: 1.,
        }
    }
}

#[allow(dead_code)]
pub enum GraphState {
    Tool(MxCell),
    Arrow(UnMxEdge),
    Dragged(MxCell),
    Nothing,
}

#[allow(dead_code)]
pub struct GraphUI {
    last_id: usize,
    transform: Transform,
    state: GraphState,
    cells: Vec<Rc<RefCell<MxCell>>>,
    edges: Vec<UnMxEdge>,
}

impl Default for GraphUI {
    fn default() -> Self {
        Self {
            last_id: Default::default(),
            transform: Default::default(),
            state: GraphState::Nothing,
            cells: Default::default(),
            edges: Default::default(),
        }
    }
}

#[allow(dead_code)]
impl GraphUI {
    /// Tolerance for detect cursor in point
    const EPSILON: f32 = 5.0;

    pub fn on_hover(&mut self, hover_point: Pos2) {
        match self.state {
            // Do nothing because
            GraphState::Nothing => {
                // Trying to find selected figure
                let _ = self.cells.iter().find_map(|cell| {
                    let mut_cell = cell.borrow_mut();
                    mut_cell.contains(hover_point, Self::EPSILON)
                });
            }
            _ => {}
        }
    }
}

#[allow(dead_code)]
pub struct Graph {
    grap_ud: GraphUI,
}

impl Graph {}
