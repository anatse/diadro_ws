use std::{cell::RefCell, ops, rc::Rc};

use eframe::{
    egui::{Color32, CursorIcon, Id, PointerButton, Pos2, Stroke, Ui, Vec2},
    emath::Rect,
};

pub trait GraphUi {
    fn add_figure(&mut self, figure: Rc<RefCell<Box<dyn GraphFigure>>>);
    fn remove_figure(&mut self, figure_id: Id);
    fn generate_id(&mut self) -> Id;
}

// pub trait GraphUiClone {
//     /// Defines Box<dyn GraphUi> as cloneable
//     fn clone_box(&self) -> Box<dyn GraphUi>;
// }

// impl<T> GraphUiClone for T
// where
//     T: 'static + GraphUi + Clone,
// {
//     fn clone_box(&self) -> Box<dyn GraphUi> {
//         Box::new(self.clone())
//     }
// }

// impl Clone for Box<dyn GraphUi> {
//     fn clone(&self) -> Box<dyn GraphUi> {
//         self.clone_box()
//     }
// }

pub type SelectMode = u8;

pub const SELECT_MODE_NONE: SelectMode = 0;
pub const SELECT_MODE_HOVER: SelectMode = 1;
pub const SELECT_MODE_SELECTED: SelectMode = 2;

/// Trait Shape used to represents any shape to drawing into Graph
/// derived from ShapeClone trait to implement Clone behaviour
pub trait GraphFigure {
    /// Set identifier of the shape
    fn set_id(&mut self, id: Id);
    /// Element identifier
    fn id(&self) -> Id;
    /// Draw shape
    fn draw(&mut self, ui: &mut Ui, zoom_factor: f32, scroll_delta: Vec2);
    /// Select shape
    fn select(&mut self, selected: SelectMode);
    /// Check if given point lay inside the figure
    fn contains(&self, point: Pos2) -> Option<CursorIcon>;
    /// Return selected state
    fn selected(&self) -> SelectMode;
    /// Move figure using given offset
    /// # Arguments
    ///  - pos - cursor position
    ///  - drag_started - drag started cursor position
    fn move_to(&mut self, pos: Pos2, drag_started: Pos2);
    /// Start dragging figure
    fn drag_start(&mut self, hover_pos: Pos2, button: PointerButton, zoom_factor: f32);
    fn dragged_by(&mut self, hover_pos: Pos2, button: PointerButton);
    fn drag_released(&mut self, hover_pos: Pos2, button: PointerButton);
    fn double_click(&mut self);

    /// Rectangle contained figure
    fn rect(&self) -> Rect;

    /// Point which can be used to connect to other figures. Only from these points lines can be drawn
    fn connection_points(&self) -> &Vec<Pos2>;
}

// /// Need to make Box<dyn Shape> cloneable
// pub trait ShapeClone {
//     /// Defines Box<dyn Shape> as cloneable
//     fn clone_box(&self) -> Box<dyn GraphFigure>;
// }

// /// Implements clone_box to make Box<dyn Shape> cloneable
// impl<T> ShapeClone for T
// where
//     T: 'static + GraphFigure + Clone,
// {
//     fn clone_box(&self) -> Box<dyn GraphFigure> {
//         Box::new(self.clone())
//     }
// }

// /// Implements Clone for Box<dyn Shape>
// impl Clone for Box<dyn GraphFigure> {
//     fn clone(&self) -> Box<dyn GraphFigure> {
//         self.clone_box()
//     }
// }

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum DragMode {
    Move(Pos2),
    Extend,
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

#[derive(Clone, Debug)]
pub struct FigureBasics {
    pub fill_color: Color32,
    pub selected_fill_color: Color32,
    pub stroke: Stroke,
    pub selected_stroke: Stroke,
    pub shadow: Shadow,
}

impl Default for FigureBasics {
    fn default() -> Self {
        Self {
            fill_color: Color32::from_rgba_premultiplied(100, 100, 50, 50),
            selected_fill_color: Color32::from_rgba_premultiplied(50, 100, 100, 50),
            stroke: Stroke::new(1., Color32::YELLOW),
            selected_stroke: Default::default(),
            shadow: Default::default(),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShadowPlace {
    Top,
    Bottom,
    Right,
    Left,
}

impl From<ShadowPlace> for u8 {
    fn from(sp: ShadowPlace) -> Self {
        sp.into_u8()
    }
}

impl ShadowPlace {
    #[inline]
    fn check_flag(flags: u8, sp: ShadowPlace) -> bool {
        flags & sp.into_u8() > 0
    }

    pub fn from(flags: u8) -> Vec<ShadowPlace> {
        let mut res = Vec::new();
        if ShadowPlace::check_flag(flags, ShadowPlace::Top) {
            res.push(ShadowPlace::Top);
        }

        if ShadowPlace::check_flag(flags, ShadowPlace::Bottom) {
            res.push(ShadowPlace::Bottom);
        }

        if ShadowPlace::check_flag(flags, ShadowPlace::Right) {
            res.push(ShadowPlace::Right);
        }

        if ShadowPlace::check_flag(flags, ShadowPlace::Left) {
            res.push(ShadowPlace::Left);
        }

        res
    }

    #[inline]
    pub fn into_u8(&self) -> u8 {
        match self {
            ShadowPlace::Top => 1,
            ShadowPlace::Bottom => 1 << 1,
            ShadowPlace::Right => 1 << 2,
            ShadowPlace::Left => 1 << 3,
        }
    }
}

impl ops::BitAnd<u8> for ShadowPlace {
    type Output = u8;

    fn bitand(self, rhs: u8) -> Self::Output {
        self.into_u8() & rhs
    }
}

impl ops::BitOr for ShadowPlace {
    type Output = u8;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.into_u8() | rhs.into_u8()
    }
}

#[repr(packed)]
#[derive(Clone, Debug)]
pub struct Shadow {
    pub shadow_color: Color32,
    pub shadow_place: u8,
}

impl Default for Shadow {
    fn default() -> Self {
        Self {
            shadow_color: Default::default(),
            shadow_place: ShadowPlace::Bottom | ShadowPlace::Right,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::shapes::ShadowPlace;

    #[test]
    fn shadow_place_from() {
        let flags = ShadowPlace::Top | ShadowPlace::Bottom;
        let res = ShadowPlace::from(flags);
        assert_eq!(res.len(), 2);
        assert_eq!(res[0], ShadowPlace::Top);
        assert_eq!(res[1], ShadowPlace::Bottom);
    }

    #[test]
    fn shadow_place_into_u8() {
        let sp = ShadowPlace::Top;
        let res = sp.into_u8();
        assert_eq!(res, 1);
    }

    #[test]
    fn shadow_place_check_flag() {
        let flags = ShadowPlace::Top | ShadowPlace::Bottom;
        assert!(ShadowPlace::check_flag(flags, ShadowPlace::Top));
        assert!(ShadowPlace::check_flag(flags, ShadowPlace::Bottom));
        assert!(!ShadowPlace::check_flag(flags, ShadowPlace::Right));
        assert!(!ShadowPlace::check_flag(flags, ShadowPlace::Left));
    }
}
