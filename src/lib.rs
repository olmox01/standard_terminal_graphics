//! Libreria di base per rendering immagini e framebuffer Unicode Braille in terminale.
//! Fornisce:
//! - Struttura FrameBuffer per contenere caratteri Unicode (Braille)
//! - Funzione per convertire immagini in framebuffer Braille
//! - Funzione per stampare il framebuffer su terminale
//! - Sistema di gestione schede e compositing
//! - Input handling e elementi UI interattivi
//! - Sistema di animazioni
//! - Rendering intelligente ottimizzato

use image::{DynamicImage, GrayImage};
use std::time::{Duration, Instant};
use parking_lot::Mutex;
use once_cell::sync::Lazy;

pub mod input;
pub mod ui;
pub mod animation;
pub mod compositor;
pub mod renderer;

/// FrameBuffer: matrice di caratteri Unicode (es. Braille)
#[derive(Debug, Clone)]
pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    pub data: Vec<char>,
}

impl FrameBuffer {
    /// Crea un nuovo framebuffer vuoto
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![' '; width * height],
        }
    }

    /// Crea un framebuffer riempito con un carattere specifico
    pub fn filled(width: usize, height: usize, fill_char: char) -> Self {
        Self {
            width,
            height,
            data: vec![fill_char; width * height],
        }
    }

    /// Imposta un carattere in posizione (x, y)
    pub fn set(&mut self, x: usize, y: usize, ch: char) {
        if x < self.width && y < self.height {
            self.data[y * self.width + x] = ch;
        }
    }

    /// Ottiene il carattere in posizione (x, y)
    pub fn get(&self, x: usize, y: usize) -> char {
        if x < self.width && y < self.height {
            self.data[y * self.width + x]
        } else {
            ' '
        }
    }

    /// Pulisce il framebuffer riempiendolo di spazi
    pub fn clear(&mut self) {
        self.data.fill(' ');
    }

    /// Ritorna una stringa rappresentante il framebuffer
    pub fn to_string(&self) -> String {
        let mut result = String::with_capacity(self.width * self.height + self.height);
        for y in 0..self.height {
            for x in 0..self.width {
                result.push(self.get(x, y));
            }
            if y < self.height - 1 {
                result.push('\n');
            }
        }
        result
    }

    /// Stampa il framebuffer su terminale
    pub fn print(&self) {
        print!("{}", self.to_string());
    }

    /// Copia una porzione di altro framebuffer in questo
    pub fn blit(&mut self, src: &FrameBuffer, src_x: usize, src_y: usize, 
                dst_x: usize, dst_y: usize, width: usize, height: usize) {
        for y in 0..height {
            for x in 0..width {
                if src_x + x < src.width && src_y + y < src.height {
                    let ch = src.get(src_x + x, src_y + y);
                    self.set(dst_x + x, dst_y + y, ch);
                }
            }
        }
    }

    /// Converte in StyledFrameBuffer
    pub fn to_styled(&self) -> StyledFrameBuffer {
        let mut styled = StyledFrameBuffer::new(self.width, self.height);
        for y in 0..self.height {
            for x in 0..self.width {
                styled.set(x, y, StyledChar::new(self.get(x, y)));
            }
        }
        styled
    }

    /// Crea un nuovo framebuffer utilizzando il pool di memoria
    pub fn new_pooled(width: usize, height: usize) -> Self {
        let size = width * height;
        
        let data = {
            let mut pool = BUFFER_POOL.lock();
            if let Some(mut reused_buffer) = pool.pop() {
                reused_buffer.clear();
                reused_buffer.resize(size, ' ');
                reused_buffer
            } else {
                vec![' '; size]
            }
        };

        Self { width, height, data }
    }

    /// Rilascia il buffer al pool per il riutilizzo
    pub fn release_to_pool(mut self) {
        if self.data.capacity() >= self.data.len() && self.data.capacity() <= 1024 * 1024 {
            let mut pool = BUFFER_POOL.lock();
            if pool.len() < 16 {
                self.data.clear();
                pool.push(self.data);
            }
        }
    }

    /// Blit parallelo per buffer grandi
    pub fn blit_parallel(&mut self, src: &FrameBuffer, src_x: usize, src_y: usize, 
                        dst_x: usize, dst_y: usize, width: usize, height: usize) {
        if width * height < 1000 {
            // Per buffer piccoli usa la versione sequenziale
            self.blit(src, src_x, src_y, dst_x, dst_y, width, height);
            return;
        }

        // Versione sicura senza unsafe code
        for y_offset in 0..height {
            let src_row = src_y + y_offset;
            let dst_row = dst_y + y_offset;
            
            if src_row < src.height && dst_row < self.height {
                for x_offset in 0..width {
                    let src_col = src_x + x_offset;
                    let dst_col = dst_x + x_offset;
                    
                    if src_col < src.width && dst_col < self.width {
                        let src_char = src.get(src_col, src_row);
                        self.set(dst_col, dst_row, src_char);
                    }
                }
            }
        }
    }
}

/// Rappresenta un'area rettangolare
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl Rect {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.x && x < self.x + self.width && 
        y >= self.y && y < self.y + self.height
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width &&
        self.x + self.width > other.x &&
        self.y < other.y + other.height &&
        self.y + self.height > other.y
    }
}

/// Colore per elementi UI
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Gray,
    Reset,
}

impl Color {
    pub fn to_ansi_fg(&self) -> &'static str {
        match self {
            Color::Black => "\x1b[30m",
            Color::Red => "\x1b[31m",
            Color::Green => "\x1b[32m",
            Color::Yellow => "\x1b[33m",
            Color::Blue => "\x1b[34m",
            Color::Magenta => "\x1b[35m",
            Color::Cyan => "\x1b[36m",
            Color::White => "\x1b[37m",
            Color::Gray => "\x1b[90m",
            Color::Reset => "\x1b[0m",
        }
    }

    pub fn to_ansi_bg(&self) -> &'static str {
        match self {
            Color::Black => "\x1b[40m",
            Color::Red => "\x1b[41m",
            Color::Green => "\x1b[42m",
            Color::Yellow => "\x1b[43m",
            Color::Blue => "\x1b[44m",
            Color::Magenta => "\x1b[45m",
            Color::Cyan => "\x1b[46m",
            Color::White => "\x1b[47m",
            Color::Gray => "\x1b[100m",
            Color::Reset => "\x1b[0m",
        }
    }
}

/// Carattere con attributi di colore
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StyledChar {
    pub ch: char,
    pub fg_color: Option<Color>,
    pub bg_color: Option<Color>,
}

impl StyledChar {
    pub fn new(ch: char) -> Self {
        Self {
            ch,
            fg_color: None,
            bg_color: None,
        }
    }

    pub fn with_fg(mut self, color: Color) -> Self {
        self.fg_color = Some(color);
        self
    }

    pub fn with_bg(mut self, color: Color) -> Self {
        self.bg_color = Some(color);
        self
    }

    pub fn to_string(&self) -> String {
        // Rendering ultra-ottimizzato per evitare disallineamenti
        if self.fg_color.is_none() && self.bg_color.is_none() {
            // Solo carattere per massima performance
            return self.ch.to_string();
        }
        
        let mut result = String::with_capacity(16);
        
        // Applica colori solo se necessario
        if let Some(fg) = self.fg_color {
            result.push_str(fg.to_ansi_fg());
        }
        if let Some(bg) = self.bg_color {
            result.push_str(bg.to_ansi_bg());
        }
        
        result.push(self.ch);
        
        // Reset pulito per evitare bleeding
        if self.fg_color.is_some() || self.bg_color.is_some() {
            result.push_str("\x1b[0m");
        }
        
        result
    }
}

impl Default for StyledChar {
    fn default() -> Self {
        Self::new(' ')
    }
}

/// FrameBuffer avanzato con supporto colori e stili
#[derive(Debug, Clone)]
pub struct StyledFrameBuffer {
    pub width: usize,
    pub height: usize,
    pub data: Vec<StyledChar>,
    dirty_regions: Vec<Rect>,
}

impl StyledFrameBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![StyledChar::default(); width * height],
            dirty_regions: Vec::new(),
        }
    }

    /// Crea un nuovo styled framebuffer utilizzando il pool
    pub fn new_pooled(width: usize, height: usize) -> Self {
        let size = width * height;
        
        let data = {
            let mut pool = STYLED_BUFFER_POOL.lock();
            if let Some(mut reused_buffer) = pool.pop() {
                reused_buffer.clear();
                reused_buffer.resize(size, StyledChar::default());
                reused_buffer
            } else {
                vec![StyledChar::default(); size]
            }
        };

        Self { 
            width, 
            height, 
            data,
            dirty_regions: Vec::with_capacity(8)
        }
    }

    pub fn set(&mut self, x: usize, y: usize, styled_char: StyledChar) {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            if self.data[index] != styled_char {
                self.data[index] = styled_char;
                self.mark_dirty(Rect::new(x, y, 1, 1));
            }
        }
    }

    pub fn get(&self, x: usize, y: usize) -> StyledChar {
        if x < self.width && y < self.height {
            self.data[y * self.width + x]
        } else {
            StyledChar::default()
        }
    }

    pub fn clear(&mut self) {
        self.data.fill(StyledChar::default());
        self.mark_dirty(Rect::new(0, 0, self.width, self.height));
    }

    pub fn clear_with(&mut self, styled_char: StyledChar) {
        self.data.fill(styled_char);
        self.mark_dirty(Rect::new(0, 0, self.width, self.height));
    }

    fn mark_dirty(&mut self, rect: Rect) {
        self.dirty_regions.push(rect);
    }

    pub fn get_dirty_regions(&self) -> &[Rect] {
        &self.dirty_regions
    }

    pub fn clear_dirty(&mut self) {
        self.dirty_regions.clear();
    }

    /// Copia una porzione di altro framebuffer in questo
    pub fn blit(&mut self, src: &StyledFrameBuffer, src_rect: Rect, dst_x: usize, dst_y: usize) {
        for y in 0..src_rect.height {
            for x in 0..src_rect.width {
                if src_rect.x + x < src.width && src_rect.y + y < src.height {
                    let styled_char = src.get(src_rect.x + x, src_rect.y + y);
                    self.set(dst_x + x, dst_y + y, styled_char);
                }
            }
        }
    }

    /// Renderizza solo le righe cambiate con controllo preciso dei caratteri
    pub fn render_partial(&self, last_buffer: &StyledFrameBuffer) -> String {
        if self.width != last_buffer.width || self.height != last_buffer.height {
            return self.to_string();
        }

        let mut result = String::with_capacity(512);
        
        // Confronta e renderizza solo le righe cambiate
        for y in 0..self.height {
            let mut row_changed = false;
            
            // Verifica veloce se la riga è cambiata
            for x in 0..self.width {
                if self.get(x, y) != last_buffer.get(x, y) {
                    row_changed = true;
                    break;
                }
            }
            
            if row_changed {
                // Posiziona cursore all'inizio della riga
                result.push_str(&format!("\x1b[{};1H", y + 1));
                
                // Renderizza la riga completa con gestione colori ottimizzata
                let mut current_fg: Option<Color> = None;
                let mut current_bg: Option<Color> = None;
                
                // IMPORTANTE: Renderizza SEMPRE tutta la larghezza della riga
                for x in 0..self.width {
                    let styled_char = self.get(x, y);
                    
                    // Cambia colori solo quando necessario
                    if styled_char.fg_color != current_fg {
                        current_fg = styled_char.fg_color;
                        if let Some(fg) = current_fg {
                            result.push_str(fg.to_ansi_fg());
                        } else {
                            result.push_str("\x1b[39m");
                        }
                    }
                    
                    if styled_char.bg_color != current_bg {
                        current_bg = styled_char.bg_color;
                        if let Some(bg) = current_bg {
                            result.push_str(bg.to_ansi_bg());
                        } else {
                            result.push_str("\x1b[49m");
                        }
                    }
                    
                    result.push(styled_char.ch);
                }
                
                // Pulisci il resto della riga per evitare caratteri fantasma
                result.push_str("\x1b[0K"); // Clear to end of line
                // Reset colori alla fine della riga
                result.push_str("\x1b[0m");
            }
        }
        
        result
    }

    pub fn to_string(&self) -> String {
        let mut result = String::with_capacity(self.width * self.height * 4);
        
        // Rendering ottimizzato senza escape sequences ridondanti
        let mut current_fg: Option<Color> = None;
        let mut current_bg: Option<Color> = None;
        
        for y in 0..self.height {
            for x in 0..self.width {
                let styled_char = self.get(x, y);
                
                // Cambia colori solo quando necessario
                if styled_char.fg_color != current_fg {
                    current_fg = styled_char.fg_color;
                    if let Some(fg) = current_fg {
                        result.push_str(fg.to_ansi_fg());
                    } else if current_fg.is_none() && (self.has_colors_in_row(y, x) || y == 0) {
                        result.push_str("\x1b[39m"); // Reset foreground solo se necessario
                    }
                }
                
                if styled_char.bg_color != current_bg {
                    current_bg = styled_char.bg_color;
                    if let Some(bg) = current_bg {
                        result.push_str(bg.to_ansi_bg());
                    } else if current_bg.is_none() && (self.has_colors_in_row(y, x) || y == 0) {
                        result.push_str("\x1b[49m"); // Reset background solo se necessario
                    }
                }
                
                result.push(styled_char.ch);
            }
            
            // Reset colori e newline SOLO se non è l'ultima riga
            if y < self.height - 1 {
                // Reset colori solo se erano stati impostati
                if current_fg.is_some() || current_bg.is_some() {
                    result.push_str("\x1b[0m");
                    current_fg = None;
                    current_bg = None;
                }
                result.push('\n');
            }
        }
        
        // Reset finale solo se necessario
        if current_fg.is_some() || current_bg.is_some() {
            result.push_str("\x1b[0m");
        }
        
        result
    }

    // Helper per verificare se ci sono colori nella riga
    fn has_colors_in_row(&self, y: usize, start_x: usize) -> bool {
        for x in start_x..self.width {
            let styled_char = self.get(x, y);
            if styled_char.fg_color.is_some() || styled_char.bg_color.is_some() {
                return true;
            }
        }
        false
    }

    /// Disegna testo con controllo rigoroso delle dimensioni
    pub fn draw_text(&mut self, x: usize, y: usize, text: &str, fg_color: Option<Color>, bg_color: Option<Color>) {
        if y >= self.height || x >= self.width {
            return;
        }
        
        // Calcola spazio disponibile con precisione
        let max_chars = self.width - x;
        let mut char_count = 0;
        
        for ch in text.chars() {
            if char_count >= max_chars {
                break;
            }
            
            let pos_x = x + char_count;
            if pos_x >= self.width {
                break;
            }
            
            // Evita caratteri di controllo che possono causare disallineamenti
            let safe_char = if ch.is_control() || ch as u32 > 127 {
                '?'
            } else {
                ch
            };
            
            let styled_char = StyledChar {
                ch: safe_char,
                fg_color,
                bg_color,
            };
            self.set(pos_x, y, styled_char);
            char_count += 1;
        }
    }

    /// Disegna rettangolo con bounds checking rigoroso
    pub fn draw_rect(&mut self, rect: Rect, ch: char, fg_color: Option<Color>, bg_color: Option<Color>) {
        let styled_char = StyledChar {
            ch,
            fg_color,
            bg_color,
        };

        // Calcola bounds sicuri
        let start_x = rect.x.min(self.width);
        let start_y = rect.y.min(self.height);
        let end_x = (rect.x + rect.width).min(self.width);
        let end_y = (rect.y + rect.height).min(self.height);
        
        for y in start_y..end_y {
            for x in start_x..end_x {
                self.set(x, y, styled_char);
            }
        }
    }

    /// Disegna bordo con dimensioni verificate
    pub fn draw_border(&mut self, rect: Rect, fg_color: Option<Color>, _bg_color: Option<Color>) {
        if rect.width < 2 || rect.height < 2 || 
           rect.x >= self.width || rect.y >= self.height {
            return;
        }
        
        let color = fg_color.unwrap_or(Color::White);
        
        // Calcola bounds sicuri
        let right = (rect.x + rect.width - 1).min(self.width - 1);
        let bottom = (rect.y + rect.height - 1).min(self.height - 1);
        
        // Verifica che i bounds siano validi
        if right <= rect.x || bottom <= rect.y {
            return;
        }

        // Caratteri bordo semplificati per compatibilità
        let corner = '+';
        let horizontal = '-';
        let vertical = '|';

        // Angoli
        self.set(rect.x, rect.y, StyledChar::new(corner).with_fg(color));
        self.set(right, rect.y, StyledChar::new(corner).with_fg(color));
        self.set(rect.x, bottom, StyledChar::new(corner).with_fg(color));
        self.set(right, bottom, StyledChar::new(corner).with_fg(color));

        // Linee orizzontali
        for x in (rect.x + 1)..right {
            if x < self.width {
                self.set(x, rect.y, StyledChar::new(horizontal).with_fg(color));
                self.set(x, bottom, StyledChar::new(horizontal).with_fg(color));
            }
        }

        // Linee verticali
        for y in (rect.y + 1)..bottom {
            if y < self.height {
                self.set(rect.x, y, StyledChar::new(vertical).with_fg(color));
                self.set(right, y, StyledChar::new(vertical).with_fg(color));
            }
        }
    }

    /// Cursore mouse ottimizzato
    pub fn draw_mouse_cursor(&mut self, x: usize, y: usize, visible: bool) {
        if !visible || x >= self.width || y >= self.height {
            return;
        }
        
        // Cursore semplice per evitare problemi di rendering
        let cursor_style = StyledChar::new('*')
            .with_fg(Color::Yellow)
            .with_bg(Color::Red);
        
        self.set(x, y, cursor_style);
    }

    /// Forza un refresh completo del framebuffer
    pub fn force_refresh(&mut self) {
        self.mark_dirty(Rect::new(0, 0, self.width, self.height));
    }

    /// Ridimensiona il framebuffer mantenendo il contenuto esistente
    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        if new_width == self.width && new_height == self.height {
            return;
        }

        let mut new_data = vec![StyledChar::default(); new_width * new_height];
        
        // Copia i dati esistenti con clipping sicuro
        let copy_width = new_width.min(self.width);
        let copy_height = new_height.min(self.height);
        
        for y in 0..copy_height {
            for x in 0..copy_width {
                let old_char = self.get(x, y);
                new_data[y * new_width + x] = old_char;
            }
        }
        
        self.width = new_width;
        self.height = new_height;
        self.data = new_data;
        self.force_refresh();
    }
    
    /// Verifica se un carattere è diverso dalla posizione precedente
    pub fn is_different_from(&self, other: &StyledFrameBuffer, x: usize, y: usize) -> bool {
        if x >= self.width || y >= self.height ||
           x >= other.width || y >= other.height {
            return true;
        }
        
        self.get(x, y) != other.get(x, y)
    }
    
    /// Confronta buffer e restituisce regioni cambiate
    pub fn get_changed_regions(&self, other: &StyledFrameBuffer) -> Vec<Rect> {
        let mut regions = Vec::new();
        
        if self.width != other.width || self.height != other.height {
            // Se dimensioni diverse, tutto è cambiato
            return vec![Rect::new(0, 0, self.width, self.height)];
        }
        
        // Scansione a blocchi per efficienza
        let block_size = 8;
        
        for block_y in (0..self.height).step_by(block_size) {
            for block_x in (0..self.width).step_by(block_size) {
                let mut block_changed = false;
                
                let end_x = (block_x + block_size).min(self.width);
                let end_y = (block_y + block_size).min(self.height);
                
                // Verifica se il blocco è cambiato
                'block_check: for y in block_y..end_y {
                    for x in block_x..end_x {
                        if self.is_different_from(other, x, y) {
                            block_changed = true;
                            break 'block_check;
                        }
                    }
                }
                
                if block_changed {
                    regions.push(Rect::new(
                        block_x,
                        block_y,
                        end_x - block_x,
                        end_y - block_y
                    ));
                }
            }
        }
        
        regions
    }
}

/// Errori che possono verificarsi durante la conversione
#[derive(Debug)]
pub enum ConversionError {
    InvalidDimensions,
    ImageTooLarge,
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::InvalidDimensions => write!(f, "Dimensioni non valide"),
            ConversionError::ImageTooLarge => write!(f, "Immagine troppo grande"),
        }
    }
}

impl std::error::Error for ConversionError {}

/// Converte un blocco 2x4 pixel in un carattere Unicode Braille
fn pixels_to_braille(block: &[u8]) -> char {
    // Mappa dei punti Braille: [1, 2, 3, 7, 4, 5, 6, 8]
    let mut code = 0x2800;
    let mapping = [0, 1, 2, 6, 3, 4, 5, 7];
    for (i, &px) in block.iter().enumerate() {
        if px > 128 {
            code |= 1 << mapping[i];
        }
    }
    std::char::from_u32(code).unwrap_or(' ')
}

/// Converte un blocco 2x4 pixel in Braille con soglia personalizzabile
fn pixels_to_braille_with_threshold(block: &[u8], threshold: u8) -> char {
    let mut code = 0x2800;
    let mapping = [0, 1, 2, 6, 3, 4, 5, 7];
    for (i, &px) in block.iter().enumerate() {
        if px > threshold {
            code |= 1 << mapping[i];
        }
    }
    std::char::from_u32(code).unwrap_or(' ')
}

/// Ridimensiona e converte un'immagine in scala di grigi
fn load_and_resize_image(img: &DynamicImage, max_width: u32, max_height: u32) -> GrayImage {
    let img = img.to_luma8();
    let (w, h) = img.dimensions();
    
    if w == 0 || h == 0 {
        return GrayImage::new(1, 1);
    }
    
    let scale_x = max_width as f32 / w as f32;
    let scale_y = max_height as f32 / h as f32;
    let scale = scale_x.min(scale_y).min(1.0);
    let new_w = ((w as f32 * scale) as u32).max(1);
    let new_h = ((h as f32 * scale) as u32).max(1);
    
    image::imageops::resize(&img, new_w, new_h, image::imageops::FilterType::Triangle)
}

/// Converte un'immagine in un framebuffer Braille
pub fn image_to_braille_fb(img: &DynamicImage, max_width: usize, max_height: usize) -> Result<FrameBuffer, ConversionError> {
    if max_width == 0 || max_height == 0 {
        return Err(ConversionError::InvalidDimensions);
    }
    
    // Ogni carattere Braille rappresenta 2x4 pixel
    let img = load_and_resize_image(img, (max_width * 2) as u32, (max_height * 4) as u32);
    let (w, h) = img.dimensions();
    let fb_w = (w as usize + 1) / 2;
    let fb_h = (h as usize + 3) / 4;
    let mut fb = FrameBuffer::new(fb_w, fb_h);

    for by in 0..fb_h {
        for bx in 0..fb_w {
            let mut block = [0u8; 8];
            for dy in 0..4 {
                for dx in 0..2 {
                    let px = if (bx * 2 + dx) < w as usize && (by * 4 + dy) < h as usize {
                        img.get_pixel((bx * 2 + dx) as u32, (by * 4 + dy) as u32).0[0]
                    } else {
                        0
                    };
                    block[dx + dy * 2] = px;
                }
            }
            let ch = pixels_to_braille(&block);
            fb.set(bx, by, ch);
        }
    }
    Ok(fb)
}

/// Converte un'immagine in framebuffer Braille con soglia personalizzabile
pub fn image_to_braille_fb_with_threshold(
    img: &DynamicImage, 
    max_width: usize, 
    max_height: usize,
    threshold: u8
) -> Result<FrameBuffer, ConversionError> {
    if max_width == 0 || max_height == 0 {
        return Err(ConversionError::InvalidDimensions);
    }
    
    let img = load_and_resize_image(img, (max_width * 2) as u32, (max_height * 4) as u32);
    let (w, h) = img.dimensions();
    let fb_w = (w as usize + 1) / 2;
    let fb_h = (h as usize + 3) / 4;
    let mut fb = FrameBuffer::new(fb_w, fb_h);

    for by in 0..fb_h {
        for bx in 0..fb_w {
            let mut block = [0u8; 8];
            for dy in 0..4 {
                for dx in 0..2 {
                    let px = if (bx * 2 + dx) < w as usize && (by * 4 + dy) < h as usize {
                        img.get_pixel((bx * 2 + dx) as u32, (by * 4 + dy) as u32).0[0]
                    } else {
                        0
                    };
                    block[dx + dy * 2] = px;
                }
            }
            let ch = pixels_to_braille_with_threshold(&block, threshold);
            fb.set(bx, by, ch);
        }
    }
    Ok(fb)
}

/// Sistema di gestione frame rate semplificato
pub struct FrameTimer {
    target_fps: u32,
    frame_duration: Duration,
    last_frame: Instant,
    frame_count: u64,
}

impl FrameTimer {
    pub fn new(target_fps: u32) -> Self {
        let target_fps = target_fps.max(1).min(120); // Clamp tra 1 e 120 FPS
        Self {
            target_fps,
            frame_duration: Duration::from_nanos(1_000_000_000 / target_fps as u64),
            last_frame: Instant::now(),
            frame_count: 0,
        }
    }

    pub fn wait_for_next_frame(&mut self) {
        let elapsed = self.last_frame.elapsed();
        
        if elapsed < self.frame_duration {
            let sleep_time = self.frame_duration - elapsed;
            std::thread::sleep(sleep_time);
        }
        
        self.last_frame = Instant::now();
        self.frame_count += 1;
    }

    pub fn get_fps(&self) -> f32 {
        let elapsed = self.last_frame.elapsed();
        if elapsed.as_secs_f32() > 0.001 {
            (1.0 / elapsed.as_secs_f32()).min(self.target_fps as f32)
        } else {
            self.target_fps as f32
        }
    }

    pub fn get_target_fps(&self) -> u32 {
        self.target_fps
    }

    pub fn get_frame_count(&self) -> u64 {
        self.frame_count
    }
}

// Global buffer pools for memory reuse
static BUFFER_POOL: Lazy<Mutex<Vec<Vec<char>>>> = Lazy::new(|| Mutex::new(Vec::new()));
static STYLED_BUFFER_POOL: Lazy<Mutex<Vec<Vec<StyledChar>>>> = Lazy::new(|| Mutex::new(Vec::new()));

#[cfg(test)]
mod tests {
    use super::*;
    use image::DynamicImage;

    #[test]
    fn test_framebuffer_set_get() {
        let mut fb = FrameBuffer::new(4, 2);
        fb.set(1, 1, 'A');
        assert_eq!(fb.get(1, 1), 'A');
        assert_eq!(fb.get(0, 0), ' ');
    }

    #[test]
    fn test_framebuffer_clear() {
        let mut fb = FrameBuffer::new(2, 2);
        fb.set(0, 0, 'X');
        fb.clear();
        assert_eq!(fb.get(0, 0), ' ');
    }

    #[test]
    fn test_framebuffer_to_string() {
        let mut fb = FrameBuffer::new(2, 2);
        fb.set(0, 0, 'A');
        fb.set(1, 1, 'B');
        let result = fb.to_string();
        assert_eq!(result, "A \n B");
    }

    #[test]
    fn test_image_to_braille_fb() {
        let img = DynamicImage::new_luma8(4, 8);
        let fb = image_to_braille_fb(&img, 2, 2).unwrap();
        assert_eq!(fb.width, 2);
        assert_eq!(fb.height, 2);
    }

    #[test]
    fn test_invalid_dimensions() {
        let img = DynamicImage::new_luma8(4, 8);
        assert!(image_to_braille_fb(&img, 0, 2).is_err());
    }

    #[test]
    fn test_blit() {
        let mut src = FrameBuffer::new(2, 2);
        src.set(0, 0, 'A');
        src.set(1, 1, 'B');
        
        let mut dst = FrameBuffer::new(4, 4);
        dst.blit(&src, 0, 0, 1, 1, 2, 2);
        
        assert_eq!(dst.get(1, 1), 'A');
        assert_eq!(dst.get(2, 2), 'B');
    }

    #[test]
    fn test_styled_char() {
        let styled = StyledChar::new('A').with_fg(Color::Red).with_bg(Color::Blue);
        let output = styled.to_string();
        assert!(output.contains('A'));
        assert!(output.contains("\x1b[31m")); // Red foreground
        assert!(output.contains("\x1b[44m")); // Blue background
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(5, 5, 10, 10);
        assert!(rect.contains(10, 10));
        assert!(!rect.contains(0, 0));
        assert!(!rect.contains(20, 20));
    }

    #[test]
    fn test_styled_framebuffer() {
        let mut fb = StyledFrameBuffer::new(10, 10);
        let styled_char = StyledChar::new('X').with_fg(Color::Red);
        fb.set(5, 5, styled_char);
        assert_eq!(fb.get(5, 5).ch, 'X');
        assert_eq!(fb.get(5, 5).fg_color, Some(Color::Red));
    }

    #[test]
    fn test_frame_timer() {
        let timer = FrameTimer::new(60);
        assert_eq!(timer.target_fps, 60);
        // Non testiamo wait_for_next_frame per evitare rallentamenti nei test
    }
}