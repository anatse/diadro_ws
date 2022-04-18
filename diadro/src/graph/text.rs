use std::{
    borrow::{Borrow, Cow},
    sync::Arc,
};

use eframe::{
    egui::{Event, Id, Key, PointerButton, Sense, Ui},
    emath::Align2,
    epaint::{text::cursor::CCursor, Color32, FontId, Galley, Pos2, Rect, Rounding, Stroke},
};

const ADJ_RATIO: f32 = 1.3;

/// Text operations
#[derive(Clone, Debug)]
pub struct TextOps {
    text: Cow<'static, str>,
    font: FontId,
    adj_ratio: f32,
    rect: Option<Rect>,
    edit_frame: bool,
    padding: f32,
    cursor_pos: usize,
    alignment: Align2,
}

impl TextOps {
    pub fn new(text: &'static str) -> Self {
        Self {
            text: Cow::Borrowed(text),
            font: FontId::proportional(32.),
            adj_ratio: ADJ_RATIO,
            rect: None,
            edit_frame: true,
            padding: 10.,
            cursor_pos: text.chars().count(),
            alignment: Align2::CENTER_CENTER,
        }
    }

    #[allow(dead_code)]
    pub fn adj_ratio(mut self, r: f32) -> Self {
        self.adj_ratio = r;
        self
    }

    #[allow(dead_code)]
    pub fn pading(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    #[allow(dead_code)]
    pub fn edit_frame(mut self, edit_frame: bool) -> Self {
        self.edit_frame = edit_frame;
        self
    }

    #[allow(dead_code)]
    pub fn set_text(&mut self, text: String, ui: &mut Ui, color: Color32) {
        self.text = Cow::Owned(text);
        // Compute new font size
        match self
            .rect
            .map(|rect| self.compute_font_size(ui, rect, color))
        {
            None => {}
            Some(size) => self.font.size = size,
        }
    }

    fn compute_font_size(&self, ui: &mut Ui, rc: Rect, color: Color32) -> f32 {
        let text = self.text.clone().into_owned();
        let galley = ui
            .painter()
            .layout(text.clone(), self.font.clone(), color, rc.width());

        let width = rc.width() / self.adj_ratio;
        let height = rc.height() / self.adj_ratio;

        // First change adopt font to height
        let width_ratio = width / galley.rect.width();
        let height_ratio = height / galley.rect.height();

        self.font.size * width_ratio.min(height_ratio)
    }

    /// Function divide all color component (red, green, blue) to divider
    /// alpha will become 1 - opaque
    fn color_divide_no_alpha(color: Color32, divider: f32) -> Color32 {
        Color32::from_rgb(
            (color.r() as f32 / divider) as u8,
            (color.g() as f32 / divider) as u8,
            (color.b() as f32 / divider) as u8,
        )
    }

    /// Drawing text into specified rctangle with specified color
    /// ### Arguments
    /// - rc  -rectangle to drawing in
    /// - uui - egui object to drawing
    /// - color - text color
    pub fn draw(
        &mut self,
        rc: Rect,
        ui: &mut Ui,
        id: Id,
        color: Color32,
        bg_color: Color32,
        edited: &mut bool,
    ) {
        // Skip drawing if region too small
        if rc.width() < 2. || rc.height() < 2. {
            return;
        }

        // Load already adjasted font if possible
        let font = match self.rect {
            Some(rect) if rc == rect => Some(&self.font),
            Some(rect) if rc.aspect_ratio() == rect.aspect_ratio() => Some(&self.font),
            _ => None,
        };

        let width = rc.width();

        // Compute galley
        let galley = match font {
            Some(font) => {
                ui.painter()
                    .layout(self.text.clone().into_owned(), font.clone(), color, width)
            }

            None => {
                let font_size = self.compute_font_size(ui, rc, color);
                if font_size < 1.0 {
                    // Font is too small to be drawing
                    return;
                }

                self.font.size = font_size;
                self.rect = Some(rc);

                ui.painter().layout(
                    self.text.clone().into_owned(),
                    self.font.clone(),
                    color,
                    width,
                )
            }
        };

        let rect = self
            .alignment
            .anchor_rect(Rect::from_min_size(rc.center(), galley.size()));

        if *edited {
            self.edit(
                rc,
                ui,
                color,
                bg_color,
                id,
                edited,
                galley.clone(),
                rect.min,
            );
        }

        ui.painter_at(rc).galley(rect.min, galley);
    }

    /// Function add edit functional to text control
    /// ### Arguments
    /// - rect - rectangle within text will be edit
    /// - ui - egui object for drawing primitives
    /// - color - color
    /// - bg_color - background color
    /// - id - identifier of parent figure
    /// - editable - mutable flag used to store editable state& Can be changed inside this
    ///   function. Usually this flag changed to true outside text control
    /// - galley - pre-computed text galley
    /// - galley_pos - offset for galley dependent on text alignment
    fn edit(
        &mut self,
        rect: Rect,
        ui: &mut Ui,
        color: Color32,
        bg_color: Color32,
        id: Id,
        editable: &mut bool,
        galley: Arc<Galley>,
        galley_pos: Pos2,
    ) {
        let rect = rect.shrink(self.padding);
        let resp = ui.interact(rect, id, Sense::click());

        // Drawing frame if needed
        if self.edit_frame {
            // Shrink rect using padding
            let light_fg_color = Self::color_divide_no_alpha(bg_color, 0.8);
            let dark_fg_color = Self::color_divide_no_alpha(bg_color, 1.3);

            let bg_stroke = Stroke::new(1., light_fg_color);
            let fg_stroke = Stroke::new(1., dark_fg_color);

            let dark_color = Self::color_divide_no_alpha(bg_color, 1.1);

            ui.painter().rect_filled(rect, Rounding::none(), dark_color);

            ui.painter()
                .line_segment([rect.left_bottom(), rect.left_top()], fg_stroke);
            ui.painter()
                .line_segment([rect.left_top(), rect.right_top()], fg_stroke);
            ui.painter()
                .line_segment([rect.right_top(), rect.right_bottom()], bg_stroke);
            ui.painter()
                .line_segment([rect.right_bottom(), rect.left_bottom()], bg_stroke);
        }

        self.draw_cursor(ui, &galley, galley_pos);

        if resp.clicked_elsewhere() {
            ui.memory().lock_focus(id, false);
            *editable = false;
        }

        let add_text = ui
            .input()
            .events
            .iter()
            .fold(self.text.clone().into_owned(), |s, ev| match ev {
                Event::Text(text) => self.insert_text(s, text),
                Event::Paste(text) => self.insert_text(s, text),
                Event::Key {
                    key: Key::Backspace,
                    pressed: true,
                    ..
                } => {
                    let res = self.remove_char_at(s, self.cursor_pos);
                    self.cursor_pos = if self.cursor_pos > 0 {
                        self.cursor_pos - 1
                    } else {
                        self.cursor_pos
                    };
                    res
                }
                Event::Key {
                    key: Key::Delete,
                    pressed: true,
                    ..
                } => self.remove_char_at(s, self.cursor_pos + 1),
                Event::Key {
                    key: Key::Enter,
                    pressed: true,
                    ..
                } => self.insert_text(s, "\n"),
                Event::Key {
                    key, pressed: true, ..
                } => self.key_process(*key, &galley),
                Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed: true,
                    ..
                } => {
                    let cursor = galley.cursor_from_pos(*pos - galley_pos);
                    self.cursor_pos = cursor.ccursor.index;
                    s
                }
                _ => {
                    // tracing::debug!("Event: {:?}", e);
                    s
                }
            });

        if self.text.borrow() != add_text {
            self.set_text(add_text, ui, color);
        }
    }

    /// Insert text at cursor
    /// ### Arguments
    /// - s - string to which new text will be inserted
    /// - text - text to insert
    /// ### Returns
    /// - built string
    fn insert_text(&mut self, s: String, text: &str) -> String {
        let count = s.chars().count();
        let s = if count > self.cursor_pos {
            let chars: Vec<char> = s.chars().collect();
            let split = chars.split_at(self.cursor_pos);
            let mut s = String::from_iter(split.0);
            s.push_str(text);
            s.push_str(String::from_iter(split.1).as_str());
            s
        } else {
            s + text
        };
        self.cursor_pos += text.chars().count();
        s
    }

    /// Removes char at position
    /// ### Arguments
    /// - s - string from which char will be removed
    /// - pos - posoition from which char will be removed
    /// new string
    fn remove_char_at(&self, s: String, pos: usize) -> String {
        let count = s.chars().count();
        if pos > count {
            s
        } else if count == 0 || pos == 0 {
            s
        } else {
            let mut sc = s.chars().collect::<Vec<char>>();
            if count == pos {
                sc.truncate(sc.len() - 1);
                String::from_iter(sc)
            } else {
                let split = sc.split_at(pos);
                let mut s = String::from_iter(&split.0[..split.0.len() - 1]);
                s.push_str(String::from_iter(split.1).as_str());
                s
            }
        }
    }

    /// Process all keys used to move cursor over the text
    fn key_process(&mut self, key: Key, galley: &Arc<Galley>) -> String {
        let chars_count = self.text.chars().count();
        let pos = match key {
            Key::ArrowLeft if self.cursor_pos > 0 => self.cursor_pos - 1,
            Key::ArrowRight if self.cursor_pos < chars_count => self.cursor_pos + 1,
            Key::Home if self.cursor_pos > 0 => 0,
            Key::End if self.cursor_pos < chars_count => chars_count,
            Key::ArrowUp => {
                let ccursor = CCursor::new(self.cursor_pos);
                let cursor = galley.from_ccursor(ccursor);
                let cursor = galley.cursor_up_one_row(&cursor);
                cursor.ccursor.index
            }
            Key::ArrowDown => {
                let ccursor = CCursor::new(self.cursor_pos);
                let cursor = galley.from_ccursor(ccursor);
                let cursor = galley.cursor_down_one_row(&cursor);
                cursor.ccursor.index
            }
            _ => self.cursor_pos,
        };

        self.cursor_pos = pos;

        self.text.clone().into_owned()
    }

    fn draw_cursor(&mut self, ui: &mut Ui, galley: &Arc<Galley>, galley_pos: Pos2) {
        let ccursor = CCursor::new(self.cursor_pos);
        let cursor = galley.from_ccursor(ccursor);
        let cursor_pos = galley
            .pos_from_cursor(&cursor)
            .translate(galley_pos.to_vec2());
        let stroke = ui.visuals().selection.stroke;
        ui.painter().line_segment(
            [cursor_pos.right_top(), cursor_pos.right_bottom()],
            (ui.visuals().text_cursor_width, stroke.color),
        );
    }
}
