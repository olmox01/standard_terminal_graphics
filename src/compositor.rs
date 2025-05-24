//! Compositing system for layered rendering

use crate::{StyledFrameBuffer, Rect};

/// Layer for compositing
pub struct Layer {
    pub buffer: StyledFrameBuffer,
    pub position: (usize, usize),
    pub visible: bool,
    pub z_order: i32,
}

impl Layer {
    pub fn new(width: usize, height: usize, x: usize, y: usize) -> Self {
        Self {
            buffer: StyledFrameBuffer::new(width, height),
            position: (x, y),
            visible: true,
            z_order: 0,
        }
    }
}

/// Compositor for managing multiple layers
pub struct Compositor {
    layers: Vec<Layer>,
    output_buffer: StyledFrameBuffer,
}

impl Compositor {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            layers: Vec::new(),
            output_buffer: StyledFrameBuffer::new(width, height),
        }
    }

    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
        self.layers.sort_by_key(|l| l.z_order);
    }

    pub fn compose(&mut self) -> &StyledFrameBuffer {
        self.output_buffer.clear();

        for layer in &self.layers {
            if layer.visible {
                let dst_rect = Rect::new(
                    layer.position.0,
                    layer.position.1,
                    layer.buffer.width,
                    layer.buffer.height,
                );
                
                let src_rect = Rect::new(0, 0, layer.buffer.width, layer.buffer.height);
                self.output_buffer.blit(&layer.buffer, src_rect, dst_rect.x, dst_rect.y);
            }
        }

        &self.output_buffer
    }

    pub fn get_layer_mut(&mut self, index: usize) -> Option<&mut Layer> {
        self.layers.get_mut(index)
    }
}
