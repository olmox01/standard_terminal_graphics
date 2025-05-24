//! Animation system for smooth transitions and effects

use crate::{StyledFrameBuffer, FrameBuffer};
use std::time::{Duration, Instant};

/// Base trait for animations
pub trait Animation {
    fn update(&mut self, delta_time: Duration) -> bool; // Returns true if animation is finished
    fn apply(&self, buffer: &mut StyledFrameBuffer);
}

/// Frame sequence animation
pub struct FrameSequence {
    frames: Vec<FrameBuffer>,
    current_frame: usize,
    frame_duration: Duration,
    last_frame_time: Instant,
    looping: bool,
    finished: bool,
}

impl FrameSequence {
    pub fn new(frames: Vec<FrameBuffer>, fps: u32) -> Self {
        Self {
            frames,
            current_frame: 0,
            frame_duration: Duration::from_nanos(1_000_000_000 / fps as u64),
            last_frame_time: Instant::now(),
            looping: true,
            finished: false,
        }
    }

    pub fn set_looping(&mut self, looping: bool) {
        self.looping = looping;
    }
}

impl Animation for FrameSequence {
    fn update(&mut self, _delta_time: Duration) -> bool {
        if self.finished || self.frames.is_empty() {
            return true;
        }

        if self.last_frame_time.elapsed() >= self.frame_duration {
            self.current_frame += 1;
            self.last_frame_time = Instant::now();

            if self.current_frame >= self.frames.len() {
                if self.looping {
                    self.current_frame = 0;
                } else {
                    self.current_frame = self.frames.len() - 1;
                    self.finished = true;
                    return true;
                }
            }
        }

        false
    }

    fn apply(&self, buffer: &mut StyledFrameBuffer) {
        if let Some(frame) = self.frames.get(self.current_frame) {
            let styled_frame = frame.to_styled();
            for y in 0..buffer.height.min(styled_frame.height) {
                for x in 0..buffer.width.min(styled_frame.width) {
                    buffer.set(x, y, styled_frame.get(x, y));
                }
            }
        }
    }
}

/// Animation manager
pub struct AnimationManager {
    animations: Vec<Box<dyn Animation>>,
}

impl AnimationManager {
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
        }
    }

    pub fn add_animation(&mut self, animation: Box<dyn Animation>) {
        self.animations.push(animation);
    }

    pub fn update(&mut self, delta_time: Duration) {
        self.animations.retain_mut(|anim| !anim.update(delta_time));
    }

    pub fn apply_all(&self, buffer: &mut StyledFrameBuffer) {
        for animation in &self.animations {
            animation.apply(buffer);
        }
    }
}
