use std::{cell::RefCell, rc::Rc};

use eframe::emath::{Pos2, Vec2};

use super::{ucell::UnMxEdge, MxCell};

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

pub enum GraphState {
    Tool(MxCell),
    Arrow(UnMxEdge),
    Dragged(MxCell),
    Nothing,
}

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

impl GraphUI {
    pub fn on_hover(&mut self, hover_point: Pos2) {
        match self.state {
            // Do nothing because
            GraphState::Nothing => {
                // Trying to find selected figure
                // let found_cell = self.cells.iter().find_map(|cell| {
                //     let mut_cell =  cell.borrow_mut();

                //     mut_cell.
                // });
            }
            _ => {}
        }
    }
}

pub struct Graph {
    grap_ud: GraphUI,
}

impl Graph {
    /// Tolerance for detect cursor in point
    const POINT_OVER_TOLERANCE: f32 = 7.0;
}
