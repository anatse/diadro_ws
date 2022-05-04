use super::shapes::{
    FigureBasics, SelectMode, SELECT_MODE_HOVER, SELECT_MODE_NONE, SELECT_MODE_SELECTED,
};
use super::text::TextOps;
use super::utils::{PointMath, TwoPosLine};
use super::Zoom;
use super::{DragMode, GraphFigure};
use eframe::egui::CursorIcon;
use eframe::{
    egui::{Id, PointerButton, Ui},
    epaint::{Color32, Pos2, Rect, Rounding, Vec2},
};

#[derive(Clone, Debug)]
pub struct RectFigure {
    id: Id,
    rect: Rect,
    fb: FigureBasics,
    selected: SelectMode,
    zoom_factor: f32,
    scroll_delta: Vec2,
    drag_mode: DragMode,
    text: TextOps,
    text_edit: bool,
    connection_points: Vec<Pos2>,
}

impl Default for RectFigure {
    fn default() -> Self {
        Self {
            id: Id::new(1),
            rect: Rect {
                min: Pos2::ZERO,
                max: Pos2::ZERO,
            },
            selected: SELECT_MODE_NONE,
            zoom_factor: 1.0,
            scroll_delta: Vec2::ZERO,
            drag_mode: DragMode::Extend,
            text: TextOps::new("New figure Проверка переноса строк"),
            text_edit: false,
            fb: Default::default(),
            connection_points: Default::default(),
        }
    }
}

const MARGIN: f32 = 10.;

impl GraphFigure for RectFigure {
    fn set_id(&mut self, id: Id) {
        self.id = id;
    }

    fn id(&self) -> Id {
        self.id
    }

    fn draw(&mut self, ui: &mut Ui, zoom_factor: f32, scroll_delta: Vec2) {
        self.zoom(zoom_factor, scroll_delta);

        ui.painter().rect(
            self.rect,
            Rounding::from(10.),
            Color32::TRANSPARENT,
            self.fb.stroke,
        );

        let bg_color = match self.selected {
            x if x & SELECT_MODE_HOVER > 0 => self.fb.selected_fill_color,
            _ => self.fb.fill_color,
        };

        ui.painter()
            .rect_filled(self.rect.expand(-1.0), Rounding::from(10.), bg_color);

        self.text.draw(
            self.rect,
            ui,
            self.id(),
            Color32::BLUE,
            bg_color,
            &mut self.text_edit,
        );

        if self.selected & SELECT_MODE_SELECTED > 0 {
            self.draw_resize_controls(ui);
        }
    }

    fn select(&mut self, selected: SelectMode) {
        self.selected = selected;
    }

    fn contains(&self, point: Pos2) -> Option<CursorIcon> {
        match self.rect.contains(point) {
            true if point.over(self.rect.right_top(), MARGIN)
                || point.over(self.rect.left_bottom(), MARGIN) =>
            {
                Some(CursorIcon::ResizeNeSw)
            }
            true if point.over(self.rect.left_top(), MARGIN)
                || point.over(self.rect.right_bottom(), MARGIN) =>
            {
                Some(CursorIcon::ResizeNwSe)
            }
            true if point.in_line([self.rect.right_top(), self.rect.right_bottom()], MARGIN)
                || point.in_line([self.rect.left_top(), self.rect.left_bottom()], MARGIN) =>
            {
                Some(CursorIcon::ResizeHorizontal)
            }
            true if point.in_line([self.rect.left_top(), self.rect.right_top()], MARGIN)
                || point.in_line([self.rect.left_bottom(), self.rect.right_bottom()], MARGIN) =>
            {
                Some(CursorIcon::ResizeVertical)
            }
            true => Some(CursorIcon::Default),
            _ => None,
        }
    }

    fn selected(&self) -> SelectMode {
        self.selected
    }

    fn move_to(&mut self, pos: Pos2, drag_started: Pos2) {
        let offset = pos - drag_started;
        self.rect = self.rect.translate(offset);
        for point in &mut self.connection_points {
            *point += offset;
        }
    }

    fn drag_start(&mut self, hover_pos: Pos2, _button: PointerButton, zoom_factor: f32) {
        match self.selected {
            x if x & SELECT_MODE_HOVER > 0 || x & SELECT_MODE_SELECTED > 0 => {
                self.drag_mode = match self.contains(hover_pos) {
                    Some(CursorIcon::ResizeHorizontal)
                        if hover_pos
                            .in_line([self.rect.left_top(), self.rect.left_bottom()], MARGIN) =>
                    {
                        DragMode::ResizeLtoR(hover_pos)
                    }
                    Some(CursorIcon::ResizeHorizontal)
                        if hover_pos
                            .in_line([self.rect.right_top(), self.rect.right_bottom()], MARGIN) =>
                    {
                        DragMode::ResizeRtoL(hover_pos)
                    }
                    Some(CursorIcon::ResizeVertical)
                        if hover_pos
                            .in_line([self.rect.left_top(), self.rect.right_top()], MARGIN) =>
                    {
                        DragMode::ResizeTtoB(hover_pos)
                    }
                    Some(CursorIcon::ResizeVertical)
                        if hover_pos.in_line(
                            [self.rect.left_bottom(), self.rect.right_bottom()],
                            MARGIN,
                        ) =>
                    {
                        DragMode::ResizeBtoT(hover_pos)
                    }
                    Some(CursorIcon::ResizeNwSe)
                        if hover_pos.over(self.rect.left_top(), MARGIN) =>
                    {
                        DragMode::ResizeTLtoBR(hover_pos)
                    }
                    Some(CursorIcon::ResizeNwSe)
                        if hover_pos.over(self.rect.right_bottom(), MARGIN) =>
                    {
                        DragMode::ResizeBRtoTL(hover_pos)
                    }
                    Some(CursorIcon::ResizeNeSw)
                        if hover_pos.over(self.rect.right_top(), MARGIN) =>
                    {
                        DragMode::ResizeTRtoBL(hover_pos)
                    }
                    Some(CursorIcon::ResizeNeSw)
                        if hover_pos.over(self.rect.left_bottom(), MARGIN) =>
                    {
                        DragMode::ResizeBLtoTR(hover_pos)
                    }
                    _ => DragMode::Move(hover_pos),
                };
            }
            _ => {
                self.drag_mode = DragMode::Extend;
                self.rect = Rect::from_two_pos(hover_pos, hover_pos);
            }
        }

        self.zoom_factor = zoom_factor;
    }

    fn dragged_by(&mut self, hover_pos: Pos2, _button: PointerButton) {
        match self.drag_mode {
            DragMode::Move(drag_started) => {
                self.move_to(hover_pos, drag_started);
                self.drag_mode = DragMode::Move(hover_pos);
            }
            DragMode::Extend => {
                self.rect.set_bottom(hover_pos.y);
                self.rect.set_right(hover_pos.x);
            }
            DragMode::ResizeLtoR(_) => {
                self.rect.set_left(hover_pos.x);
            }
            DragMode::ResizeRtoL(_) => {
                self.rect.set_right(hover_pos.x);
            }
            DragMode::ResizeTtoB(_) => {
                self.rect.set_top(hover_pos.y);
            }
            DragMode::ResizeBtoT(_) => {
                self.rect.set_bottom(hover_pos.y);
            }
            DragMode::ResizeTLtoBR(_) => {
                self.rect.set_left(hover_pos.x);
                self.rect.set_top(hover_pos.y);
            }
            DragMode::ResizeBRtoTL(_) => {
                self.rect.set_right(hover_pos.x);
                self.rect.set_bottom(hover_pos.y);
            }
            DragMode::ResizeTRtoBL(_) => {
                self.rect.set_right(hover_pos.x);
                self.rect.set_top(hover_pos.y);
            }
            DragMode::ResizeBLtoTR(_) => {
                self.rect.set_left(hover_pos.x);
                self.rect.set_bottom(hover_pos.y);
            }
        }

        // Compute connection points if empty
        if self.connection_points.is_empty() {
            // Three point on each side
            self.compute_connection_points();
        }
    }

    fn drag_released(&mut self, hover_pos: Pos2, _button: PointerButton) {
        self.dragged_by(hover_pos, _button);
    }

    fn double_click(&mut self) {
        self.text_edit = true;
    }

    fn rect(&self) -> Rect {
        self.rect
    }

    fn connection_points(&self) -> &Vec<Pos2> {
        &self.connection_points
    }
}

impl RectFigure {
    fn zoom(&mut self, zoom_factor: f32, scroll_delta: Vec2) {
        self.rect = self.rect.zoom(zoom_factor / self.zoom_factor);
        self.zoom_factor = zoom_factor;
        if self.scroll_delta != scroll_delta {
            self.rect = self.rect.translate(scroll_delta);
            self.scroll_delta = scroll_delta;
        }

        self.compute_connection_points();
    }

    fn compute_connection_points(&mut self) {
        if self.rect.size() != Vec2::ZERO {
            self.connection_points = Vec::with_capacity(16);
            let line = TwoPosLine::new([self.rect.left_top(), self.rect.right_top()]);
            self.connection_points.extend(line.split(4));
            let line = TwoPosLine::new([self.rect.right_top(), self.rect.right_bottom()]);
            self.connection_points
                .extend_from_slice(&line.split(4)[1..]);
            let line = TwoPosLine::new([self.rect.right_bottom(), self.rect.left_bottom()]);
            self.connection_points
                .extend_from_slice(&line.split(4)[1..]);
            let line = TwoPosLine::new([self.rect.left_bottom(), self.rect.left_top()]);
            self.connection_points
                .extend_from_slice(&line.split(4)[1..4]);
        }
    }

    fn draw_resize_controls(&self, _ui: &mut Ui) {
        // let rect = self.rect;
        // let margin = MARGIN;
        // let rect_pos = Pos2::new(rect.min.x - MARGIN / 2., rect.min.y - MARGIN / 2.);
        // let rect_size = rect.size();
        // let rect_right = rect_pos.x + rect_size.x;
        // let rect_bottom = rect_pos.y + rect_size.y;
        // let rect_center = rect_pos + rect_size / 2.0;

        // let nw = rect_pos;
        // let n = Pos2::new(rect_center.x, nw.y);
        // let ne = Pos2::new(rect_right, nw.y);
        // let e = Pos2::new(ne.x, rect_center.y);
        // let se = Pos2::new(rect_right, rect_bottom);
        // let s = Pos2::new(rect_center.x, se.y);
        // let sw = Pos2::new(nw.x, se.y);
        // let w = Pos2::new(nw.x, rect_center.y);

        // let nw_rect = Rect::from_two_pos(nw, nw + Vec2::new(margin, margin));
        // let n_rect = Rect::from_two_pos(n, n + Vec2::new(margin, margin));
        // let ne_rect = Rect::from_two_pos(ne, ne + Vec2::new(margin, margin));
        // let e_rect = Rect::from_two_pos(e, e + Vec2::new(margin, margin));
        // let se_rect = Rect::from_two_pos(se, se + Vec2::new(margin, margin));
        // let s_rect = Rect::from_two_pos(s, s + Vec2::new(margin, margin));
        // let sw_rect = Rect::from_two_pos(sw, sw + Vec2::new(margin, margin));
        // let w_rect = Rect::from_two_pos(w, w + Vec2::new(margin, margin));

        // [
        //     nw_rect, n_rect, ne_rect, e_rect, se_rect, s_rect, sw_rect, w_rect,
        // ]
        // .iter()
        // .for_each(|r| {
        //     ui.painter()
        //         .rect_filled(*r, Rounding::none(), Color32::WHITE);
        // });
    }
}
