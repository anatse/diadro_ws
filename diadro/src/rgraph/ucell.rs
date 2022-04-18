use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use super::MxCell;

pub struct UnMxEdge {
    start: Rc<RefCell<MxCell>>,
    end: Rc<RefCell<MxCell>>,
}

impl UnMxEdge {
    pub fn get_start(&self) -> Ref<'_, MxCell> {
        self.start.borrow()
    }

    pub fn get_end(&self) -> Ref<'_, MxCell> {
        self.end.borrow()
    }

    pub fn get_start_mut(&self) -> RefMut<'_, MxCell> {
        self.start.borrow_mut()
    }

    pub fn get_end_mut(&self) -> RefMut<'_, MxCell> {
        self.end.borrow_mut()
    }

    pub fn get_end_rc(&self) -> Rc<RefCell<MxCell>> {
        self.end.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use eframe::{egui::Id, emath::Pos2};

    use crate::rgraph::MxCell;

    use super::UnMxEdge;

    #[test]
    #[should_panic]
    fn test_refs_wrong() {
        let mx1 = MxCell::new(Id::new(1));
        let mx2 = MxCell::new(Id::new(1));
        let edge = UnMxEdge {
            start: Rc::new(RefCell::new(mx1)),
            end: Rc::new(RefCell::new(mx2)),
        };

        // Firts get start (unsafe part)
        let x = edge.get_start();
        // Here must be panic - runtime borrow checker
        let y = edge.get_start_mut();

        println!("{:?} = {:?}", x, y);
    }

    #[test]
    fn test_refs_properly() {
        let mx1 = MxCell::new(Id::new(1));
        let mx2 = MxCell::new(Id::new(1));
        let edge = UnMxEdge {
            start: Rc::new(RefCell::new(mx1)),
            end: Rc::new(RefCell::new(mx2)),
        };

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
        let edge = UnMxEdge {
            start: Rc::new(RefCell::new(mx1)),
            end: Rc::new(RefCell::new(mx2)),
        };

        // Firts get start (unsafe part)
        let x = edge.get_end_rc();
        println!("{:?}", x);
        // Here must be panic - runtime borrow checker
        let mut y = edge.get_end_mut();
        y.connection_points.push(Pos2::ZERO);

        assert_eq!(x.borrow().connection_points.len(), 1);
    }

    #[test]
    fn test_rc_properly() {
        let mx1 = MxCell::new(Id::new(1));
        let mx2 = MxCell::new(Id::new(1));
        let edge = UnMxEdge {
            start: Rc::new(RefCell::new(mx1)),
            end: Rc::new(RefCell::new(mx2)),
        };

        // Firts get start (unsafe part)
        let x = edge.get_end_rc();
        println!("{:?}", x);

        {
            let mut y = edge.get_end_mut();
            y.connection_points.push(Pos2::ZERO);
        }

        assert_eq!(x.borrow().connection_points.len(), 1);
    }
}
