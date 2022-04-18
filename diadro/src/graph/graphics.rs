use std::{cell::RefCell, rc::Rc};

use eframe::{
    egui::{CursorIcon, Id, InnerResponse, Painter, PointerButton, Sense, Ui, Visuals},
    emath::Vec2,
    epaint::{Color32, Pos2, Stroke},
};

use super::{
    arrow::{ArrowFigure, ConnectionPoint},
    shapes::{GraphUi, SELECT_MODE_HOVER, SELECT_MODE_SELECTED},
    utils::PointMath,
    GraphFigure, RectFigure,
};

/// Tolerance for detect cursor in point
const POINT_OVER_TOLERANCE: f32 = 7.0;

// #[derive(Clone)]
pub struct GraphicsData {
    /// Last used identifier. Used to generate identifiers
    last_id: usize,
    /// Selected tool - type of added figure or relation between its
    selected_tool: Option<Box<dyn GraphFigure>>,
    /// Selected figure index - index in figures vector
    selected_figure_idx: Option<usize>,
    /// List of figures in diagram
    figures: Vec<Rc<RefCell<Box<dyn GraphFigure>>>>,
    /// Screen parameters% scroll delta - defines offset [x; y] for whole screen
    scroll_delta: Vec2,
    /// Zoom factor
    zoom_factor: f32,
    /// Flag defines selection behaviour
    /// - True - elements can selected by mouse pointer and further can receive events
    select_enabled: bool,
    /// Flag used to determine drag state
    is_dragged: bool,
    /// Edges
    edges: Vec<ArrowFigure>,
    /// Currently dragged arrow (edge)
    dragged_edge: Option<ArrowFigure>,
    /// Color for drawing connection point
    edge_point_color: Color32,
    /// Color for drawing selected connection point
    selected_edge_point_stroke: Stroke,
    /// Figure currently selected by dragging edge
    selected_by_edge_figure_idx: Option<usize>,
}

impl Default for GraphicsData {
    fn default() -> Self {
        Self {
            last_id: Default::default(),
            selected_tool: Some(Box::new(RectFigure::default())),
            selected_figure_idx: Default::default(),
            figures: Default::default(),
            scroll_delta: Default::default(),
            zoom_factor: 1.0,
            select_enabled: Default::default(),
            is_dragged: false,
            edges: Default::default(),
            dragged_edge: None,
            edge_point_color: Color32::YELLOW,
            selected_edge_point_stroke: Stroke::new(1., Color32::YELLOW),
            selected_by_edge_figure_idx: None,
        }
    }
}

/// Implies functions for graphics data
impl GraphUi for GraphicsData {
    /// Add figure
    fn add_figure(&mut self, figure: Rc<RefCell<Box<dyn GraphFigure>>>) {
        self.figures.push(figure);
    }

    /// Remove figure
    fn remove_figure(&mut self, figure_id: eframe::egui::Id) {
        self.figures
            .iter()
            .position(|r| RefCell::borrow(r).id() == figure_id)
            .iter()
            .for_each(|found| {
                self.figures.remove(*found);
            });
    }

    /// Generate new figure id
    fn generate_id(&mut self) -> eframe::egui::Id {
        self.last_id += 1;
        Id::new(self.last_id)
    }
}

impl GraphicsData {
    /// Function selects element by cursor coordinates
    /// # Arguments
    ///  - point - cursor coordinates
    pub fn select_by_point(&mut self, point: Pos2) -> Option<CursorIcon> {
        let mut cursor: Option<CursorIcon> = None;

        if self.select_enabled {
            self.selected_figure_idx = None;
            let mut index = 0;
            for r in self.figures.iter_mut() {
                let s = RefCell::borrow(r).selected();
                r.borrow_mut().select(s & !SELECT_MODE_HOVER);

                if let Some(ci) = RefCell::borrow(r).contains(point) {
                    self.selected_figure_idx = Some(index);
                    cursor = Some(ci);
                }

                index += 1;
            }

            if let Some(idx) = self.selected_figure_idx {
                let mut figure = RefCell::borrow_mut(&self.figures[idx]);
                let prev = figure.selected();
                figure.select(prev | SELECT_MODE_HOVER);
            }
        } else if let Some(arrow) = self.dragged_edge.as_mut() {
            // Used whe trying to drag arrow/edge
            self.selected_by_edge_figure_idx = None;
            arrow.disconnect_end();

            // When drag an arrow then
            for (idx, ref_fig) in self.figures.iter().enumerate() {
                let fig = RefCell::borrow(ref_fig);

                // Skip arrow start figure
                if arrow
                    .get_start_connection()
                    .as_ref()
                    .map(|con| con.get_figure())
                    .filter(|ff| Rc::ptr_eq(ff, ref_fig))
                    .is_some()
                {
                    continue;
                }

                if fig.contains(point).is_some() {
                    self.selected_by_edge_figure_idx = Some(idx);
                }

                // Draw connection points for the figure if end of arrow located inside the figure
                let connection_points = fig.connection_points();
                for (idx, c_pos) in connection_points.iter().enumerate() {
                    if point.over(*c_pos, POINT_OVER_TOLERANCE) {
                        arrow.connect_end(ConnectionPoint::new(ref_fig.clone(), idx));
                        break;
                    }
                }
            }
        }

        cursor
    }

    /// Drawing scene include all figures, lines, connection points and other
    fn draw(&mut self, ui: &mut Ui) {
        for r in self.figures.iter_mut() {
            RefCell::borrow_mut(r).draw(ui, self.zoom_factor, self.scroll_delta);
        }

        for a in self.edges.iter_mut() {
            a.draw(ui, self.zoom_factor, self.scroll_delta);
        }

        if self.is_dragged {
            if let Some(fig) = self.selected_tool.as_mut() {
                fig.draw(ui, self.zoom_factor, self.scroll_delta);
            }
        }

        if let Some(edge) = self.dragged_edge.as_mut() {
            edge.draw(ui, self.zoom_factor, self.scroll_delta);
        }
    }

    /// Drawing one connection point
    #[inline]
    fn draw_edge_point(&self, point: Pos2, painter: &Painter) {
        painter.circle_filled(point, 3.5, self.edge_point_color);
    }

    /// Drawing selected connection point
    #[inline]
    fn draw_selected_edge_point(&self, point: Pos2, painter: &Painter) {
        painter.circle_stroke(point, 5., self.selected_edge_point_stroke);
    }
}

/// Defines all graphics diagram operations
pub struct Graphics {
    /// Graphics data
    graphics_data: GraphicsData,
}

impl Graphics {
    pub fn new() -> Self {
        Self {
            graphics_data: GraphicsData::default(),
        }
    }
}

/// Implies graphics/digram operations
impl Graphics {
    /// Determines if point located over connection points
    /// ### Arguments
    /// * point - point for which location will be determined
    /// ### Return
    /// <usize, Pos2> - connection point index and point
    fn point_in_edge_controls(&self, point: Pos2) -> Option<(usize, Pos2)> {
        if let Some(figure) = self.graphics_data.selected_figure_idx.and_then(|idx| {
            self.graphics_data
                .figures
                .get(idx)
                .map(|ref_fig| ref_fig.as_ref().borrow())
                .filter(|fig| fig.selected() & SELECT_MODE_SELECTED > 0)
        }) {
            let c_points = figure.connection_points();
            for (idx, pt) in c_points.iter().enumerate() {
                if point.over(*pt, POINT_OVER_TOLERANCE) {
                    return Some((idx, point));
                }
            }
        }

        None
    }

    /// Return currently selected figure
    #[inline]
    fn selected_figure(&self) -> Option<&Rc<RefCell<Box<dyn GraphFigure>>>> {
        self.graphics_data
            .figures
            .iter()
            .find(|fig| RefCell::borrow(fig).selected() & SELECT_MODE_SELECTED > 0)
    }

    /// Draw controls to add out edges over selected figure. Each control represents as a circle with plus symbol inside
    fn draw_edge_controls(&mut self, ui: &mut Ui) {
        if let Some(fig) = self.selected_figure() {
            // Draw only for selected figures
            let fig = RefCell::borrow(&fig);
            // let rect = fig.borrow().rect();
            let points = fig.connection_points();
            let painter = ui.painter();
            for point in points {
                self.graphics_data.draw_edge_point(*point, painter);
            }
        }

        if let Some(fig) = self
            .graphics_data
            .selected_by_edge_figure_idx
            .and_then(|idx| self.graphics_data.figures.get(idx))
        {
            let painter = ui.painter();
            for point in fig.as_ref().borrow().connection_points() {
                self.graphics_data.draw_edge_point(*point, painter);
            }
        }
    }

    /// Draw whole canvas
    pub fn ui(&mut self, ui: &mut Ui) -> InnerResponse<()> {
        let ctx = ui.ctx();
        ctx.set_visuals(Visuals::light());

        // Compute size
        let size = ui.available_size_before_wrap();
        // Allocate the space.
        let mut response = ui
            .allocate_response(size, Sense::click_and_drag())
            .on_hover_cursor(CursorIcon::Default);

        // Zoom factor computing
        let zd = self.graphics_data.zoom_factor + ui.input().zoom_delta() - 1.;
        if zd != self.graphics_data.zoom_factor && zd > 0. {
            self.graphics_data.zoom_factor = zd;
        }

        if response.hovered() {
            if let Some(hp) = response.hover_pos() {
                if let Some(cursor) = self.graphics_data.select_by_point(hp) {
                    response = response.on_hover_cursor(cursor);
                }

                if let Some((_, point)) = self.point_in_edge_controls(hp) {
                    response = response.on_hover_cursor(CursorIcon::Default);
                    // Draw cidx point for current figure
                    self.graphics_data
                        .draw_selected_edge_point(point, ui.painter());
                }
            }
        }

        if response.double_clicked() {
            if let Some(selected_figure) = self
                .graphics_data
                .selected_figure_idx
                .and_then(|idx| self.graphics_data.figures.get_mut(idx))
            {
                selected_figure.borrow_mut().double_click();
            }
        }

        if response.clicked() {
            // Clear selection
            self.graphics_data
                .figures
                .iter_mut()
                .filter(|fig| RefCell::borrow(fig).selected() & SELECT_MODE_SELECTED > 0)
                .for_each(|fig| {
                    let selected = RefCell::borrow(fig).selected();
                    RefCell::borrow_mut(fig).select(selected & !SELECT_MODE_SELECTED)
                });

            if let Some(selected_figure) = self
                .graphics_data
                .selected_figure_idx
                .and_then(|idx| self.graphics_data.figures.get_mut(idx))
            {
                let selected = RefCell::borrow(selected_figure).selected();
                RefCell::borrow_mut(selected_figure).select(selected | SELECT_MODE_SELECTED);
            }
        }

        // Process drag started event
        if response.drag_started() {
            let hover_pos = response.hover_pos().unwrap_or_default();
            if let Some((cpoint, _)) = self.point_in_edge_controls(hover_pos) {
                let mut edge =
                    ArrowFigure::new([hover_pos, hover_pos], self.graphics_data.generate_id());

                if let Some(fig) = self.selected_figure() {
                    edge.connect_start(ConnectionPoint::new(Rc::clone(fig), cpoint));
                }

                self.graphics_data.dragged_edge = Some(edge);
            } else if let Some(selected_figure) = self
                .graphics_data
                .selected_figure_idx
                .and_then(|idx| self.graphics_data.figures.get_mut(idx))
            {
                selected_figure.borrow_mut().drag_start(
                    hover_pos,
                    PointerButton::Primary,
                    self.graphics_data.zoom_factor,
                );
            } else if let Some(fig) = self.graphics_data.selected_tool.as_mut() {
                fig.drag_start(
                    hover_pos,
                    PointerButton::Primary,
                    self.graphics_data.zoom_factor,
                );
                self.graphics_data.is_dragged = true;
            }
            self.graphics_data.select_enabled = false;
        }

        if response.dragged_by(PointerButton::Primary) {
            let hover_pos = response.hover_pos().unwrap_or_default();
            if let Some(edge) = self.graphics_data.dragged_edge.as_mut() {
                edge.set_end_pos(hover_pos);
            } else if let Some(selected_figure) = self
                .graphics_data
                .selected_figure_idx
                .and_then(|idx| self.graphics_data.figures.get_mut(idx))
            {
                selected_figure
                    .borrow_mut()
                    .dragged_by(hover_pos, PointerButton::Primary)
            } else if let Some(fig) = self.graphics_data.selected_tool.as_mut() {
                fig.dragged_by(hover_pos, PointerButton::Primary);
            }
        }

        if response.drag_released() {
            let hover_pos = response.hover_pos().unwrap_or_default();
            if let Some(mut edge) = self.graphics_data.dragged_edge.take() {
                edge.set_end_pos(hover_pos);
                self.graphics_data.edges.push(edge.clone());
            } else if let Some(selected_figure) = self
                .graphics_data
                .selected_figure_idx
                .and_then(|idx| self.graphics_data.figures.get_mut(idx))
            {
                selected_figure
                    .borrow_mut()
                    .drag_released(hover_pos, PointerButton::Primary)
            } else if self.graphics_data.selected_tool.is_some() {
                let fig = self.graphics_data.selected_tool.take();
                let mut f = fig.unwrap();
                f.set_id(self.graphics_data.generate_id());
                f.drag_released(hover_pos, PointerButton::Primary);
                self.graphics_data.add_figure(Rc::new(RefCell::new(f)));
                self.graphics_data.selected_tool = Some(Box::new(RectFigure::default()));
            }
            self.graphics_data.select_enabled = true;
            self.graphics_data.is_dragged = false;
            self.graphics_data.selected_by_edge_figure_idx = None;
        }

        let scroll_delta = ui.input().scroll_delta;
        if scroll_delta != Vec2::ZERO {
            self.graphics_data.scroll_delta = scroll_delta;
        }

        self.graphics_data.draw(ui);
        self.draw_edge_controls(ui);

        InnerResponse {
            inner: (),
            response,
        }
    }
}
