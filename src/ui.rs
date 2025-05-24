//! User interface components and widgets

use crate::{StyledFrameBuffer, Rect, Color};

/// Base trait for UI widgets
pub trait Widget {
    fn render(&self, buffer: &mut StyledFrameBuffer);
    fn get_rect(&self) -> Rect;
    fn handle_input(&mut self, event: &crate::input::InputEvent) -> bool;
}

/// Simple button widget
pub struct Button {
    rect: Rect,
    text: String,
    focused: bool,
    pressed: bool,
}

impl Button {
    pub fn new(rect: Rect, text: String) -> Self {
        Self {
            rect,
            text,
            focused: false,
            pressed: false,
        }
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
}

impl Widget for Button {
    fn render(&self, buffer: &mut StyledFrameBuffer) {
        let bg_color = if self.pressed {
            Color::Red
        } else if self.focused {
            Color::Blue
        } else {
            Color::Gray
        };

        buffer.draw_rect(self.rect, ' ', Some(Color::White), Some(bg_color));
        buffer.draw_border(self.rect, Some(Color::White), None);
        
        if self.rect.width > 2 && self.rect.height > 0 {
            let text_x = self.rect.x + (self.rect.width - self.text.len().min(self.rect.width - 2)) / 2;
            let text_y = self.rect.y + self.rect.height / 2;
            buffer.draw_text(text_x, text_y, &self.text, Some(Color::White), Some(bg_color));
        }
    }

    fn get_rect(&self) -> Rect {
        self.rect
    }

    fn handle_input(&mut self, _event: &crate::input::InputEvent) -> bool {
        false
    }
}

/// UI manager for handling multiple widgets
pub struct UIManager {
    widgets: Vec<Box<dyn Widget>>,
    focused_widget: Option<usize>,
}

impl UIManager {
    pub fn new() -> Self {
        Self {
            widgets: Vec::new(),
            focused_widget: None,
        }
    }

    pub fn add_widget(&mut self, widget: Box<dyn Widget>) {
        self.widgets.push(widget);
        if self.focused_widget.is_none() {
            self.focused_widget = Some(0);
        }
    }

    pub fn render(&self, buffer: &mut StyledFrameBuffer) {
        for widget in &self.widgets {
            widget.render(buffer);
        }
    }
}
