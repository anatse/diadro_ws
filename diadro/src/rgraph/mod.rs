mod cell;
mod errors;
mod ucell;

use std::collections::HashMap;

use eframe::{
    egui::Id,
    emath::Pos2,
    epaint::{
        CubicBezierShape, Mesh, PathShape, QuadraticBezierShape, RectShape, Stroke, TextShape,
    },
};
use serde::{Deserialize, Serialize};

use self::errors::MxErrors;

#[derive(Serialize, Deserialize, Debug)]
pub struct MxEdge {
    pub start: Option<Id>,
    pub end: Option<Id>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MxConnectable {
    pub edges: Vec<Id>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CellType {
    Edge(MxEdge),
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
#[must_use = "Add a shape to diagram"]
pub struct MxCell {
    pub id: Id,
    pub cell_type: CellType,
    /// Array of shapes which must be enough to describe the figure
    pub shapes: Vec<Figure>,
    pub connection_points: Vec<Pos2>,
}

/// Contains relations between cells
pub struct MxCellIndices {
    cells_map: HashMap<Id, MxCell>,
    keys_ordering: Vec<Id>,
}

impl MxCellIndices {
    pub fn new() -> Self {
        Self {
            cells_map: Default::default(),
            keys_ordering: Default::default(),
        }
    }

    pub fn add_cell(&mut self, mx_cell: MxCell) {
        let id = mx_cell.id;
        self.cells_map.insert(id, mx_cell);
        // Must be added only once
        if !self.keys_ordering.contains(&id) {
            self.keys_ordering.push(id);
        }
    }

    pub fn remove_cell(&mut self, id: Id) {
        self.cells_map.remove(&id);
        if let Some((idx, _)) = self
            .keys_ordering
            .iter()
            .enumerate()
            .find(|(_, fid)| **fid == id)
        {
            self.keys_ordering.remove(idx);
        }
    }

    pub fn move_to_last(&mut self, id: Id) {
        if let Some((idx, _)) = self
            .keys_ordering
            .iter()
            .enumerate()
            .find(|(_, fid)| **fid == id)
        {
            if idx < self.keys_ordering.len() - 1 {
                let mut new_vec = Vec::from(&self.keys_ordering[..idx]);
                new_vec.extend_from_slice(&self.keys_ordering[idx + 1..]);
                new_vec.push(id);
                self.keys_ordering = new_vec;
            }
        }
    }

    pub fn get(&self, id: Id) -> Option<&MxCell> {
        self.cells_map.get(&id)
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut MxCell> {
        self.cells_map.get_mut(&id)
    }

    /// Get list of values by list of ids
    pub fn get_by_keys(&self, ids: &Vec<Id>) -> Vec<&MxCell> {
        ids.iter()
            .map(|id| self.cells_map.get(id))
            .filter_map(|x| x)
            .collect()
    }

    /// Iterate over all values ordered by key_ordering vec
    pub fn iter(&self) -> impl Iterator<Item = &MxCell> {
        self.keys_ordering
            .iter()
            .map(|id| self.cells_map.get(id))
            .filter_map(|cell| cell)
    }

    /// Iterate over all values ordered by key_ordering vec and call mutable function for each mxcell value
    pub fn iter_mut(&mut self, process_cell: impl Fn(Option<&mut MxCell>) -> ()) {
        self.keys_ordering
            .iter()
            .for_each(|id| process_cell(self.cells_map.get_mut(id)))
    }

    pub fn get_by_index(&self, index: usize) -> Option<&MxCell> {
        self.keys_ordering
            .get(index)
            .and_then(|id| self.cells_map.get(id))
    }

    pub fn extract_connectable(&self, id: Id) -> Result<&MxConnectable, MxErrors> {
        match self.get(id) {
            None => Err(MxErrors::MxCellNotFound),
            Some(MxCell {
                cell_type: CellType::Connectable(conn),
                ..
            }) => Ok(conn),
            _ => Err(MxErrors::WrongMxCellType),
        }
    }

    pub fn extract_edge(&self, id: Id) -> Result<&MxEdge, MxErrors> {
        match self.get(id) {
            None => Err(MxErrors::MxCellNotFound),
            Some(MxCell {
                cell_type: CellType::Edge(edge),
                ..
            }) => Ok(edge),
            _ => Err(MxErrors::WrongMxCellType),
        }
    }
}

#[cfg(test)]
mod tests {
    use eframe::{egui::Id, emath::Pos2};

    use super::{MxCell, MxCellIndices};

    #[test]
    fn test_move_to_last() {
        let mut indices = MxCellIndices::new();
        for i in 0..10 {
            indices.add_cell(MxCell::new(Id::new(i)));
        }

        indices.move_to_last(Id::new(2));
        assert!(indices.get_by_index(9).is_some());
        assert_eq!(indices.get_by_index(9).unwrap().id, Id::new(2));
    }

    #[test]
    fn test_iter_mut() {
        let mut indices = MxCellIndices::new();
        for i in 0..10 {
            indices.add_cell(MxCell::new(Id::new(i)));
        }

        // Change each cell - add connection point
        indices.iter_mut(|opt_cell| match opt_cell {
            Some(cell) => {
                cell.connection_points.push(Pos2::ZERO);
            }
            None => {}
        });

        indices
            .iter()
            .for_each(|cell| assert!(cell.connection_points().len() == 1));
    }
}
