//! Desktop Environment Demo - Ambiente desktop completo e ottimizzato
//! 
//! Features:
//! - Window Manager con finestre multiple correttamente allineate
//! - Terminali multipli funzionanti
//! - Visualizzatore immagini
//! - Simulatore video
//! - Taskbar proporzionata
//! - FPS bloccato a 60 con controllo prestazioni

use standard_terminal_graphics::{
    StyledFrameBuffer, FrameBuffer, Rect, Color, FrameTimer,
    input::{InputManager, InputEvent},
    renderer::SmartRenderer,
    StyledChar, image_to_braille_fb,
};
use image::DynamicImage;
use std::time::Duration;
use std::io::{self};
use crossterm::{event::KeyCode, event::MouseEventKind};

#[derive(Debug, Clone, Copy, PartialEq)]
enum WindowType {
    Terminal,
    ImageViewer,
    VideoPlayer,
    FileManager,
    TextEditor,
}

#[derive(Debug, Clone)]
struct Window {
    id: usize,
    title: String,
    rect: Rect,
    window_type: WindowType,
    content: StyledFrameBuffer,
    focused: bool,
    minimized: bool,
    z_order: i32,
    terminal_lines: Vec<String>,
    current_line: String,
    cursor_pos: usize,
    video_frame: usize,
    image_data: Option<FrameBuffer>,
    closed: bool,  // Flag per evitare riaperture automatiche
}

impl Window {
    fn new(id: usize, title: String, rect: Rect, window_type: WindowType) -> Self {
        let mut window = Self {
            id,
            title,
            rect,
            window_type,
            content: StyledFrameBuffer::new(rect.width, rect.height),
            focused: false,
            minimized: false,
            z_order: 0,
            terminal_lines: vec!["$ Welcome to Terminal".to_string()],
            current_line: String::new(),
            cursor_pos: 0,
            video_frame: 0,
            image_data: None,
            closed: false,
        };
        window.update_content();
        window
    }

    fn update_content(&mut self) {
        if self.closed || self.minimized {
            return;
        }
        
        // Aggiorna solo se necessario per ridurre overhead
        self.content.clear();
        self.draw_window_frame();
        
        match self.window_type {
            WindowType::Terminal => self.draw_terminal_content(),
            WindowType::ImageViewer => self.draw_image_content(),
            WindowType::VideoPlayer => self.draw_video_content(),
            WindowType::FileManager => self.draw_file_manager_content(),
            WindowType::TextEditor => self.draw_text_editor_content(),
        }
    }

    fn draw_window_frame(&mut self) {
        let border_color = if self.focused { Color::Yellow } else { Color::Gray };
        let bg_color = if self.focused { Color::Blue } else { Color::Black };
        
        // Bordo finestra - dimensioni corrette
        let frame_rect = Rect::new(0, 0, self.rect.width, self.rect.height);
        self.content.draw_border(frame_rect, Some(border_color), None);
        
        // Barra del titolo - allineamento corretto
        let title_rect = Rect::new(1, 1, self.rect.width.saturating_sub(2), 1);
        if title_rect.width > 0 {
            self.content.draw_rect(title_rect, ' ', Some(Color::White), Some(bg_color));
            
            // Titolo con padding corretto
            let title_text = if self.title.len() > title_rect.width.saturating_sub(6) {
                format!("{}...", &self.title[..title_rect.width.saturating_sub(9)])
            } else {
                self.title.clone()
            };
            
            let title_x = 2; // Allineamento a sinistra con padding
            self.content.draw_text(title_x, 1, &title_text, Some(Color::White), Some(bg_color));
            
            // Pulsanti finestra - posizione corretta
            if self.rect.width > 6 {
                let close_x = self.rect.width.saturating_sub(3);
                self.content.draw_text(close_x, 1, "✕", Some(Color::Red), Some(bg_color));
                
                if self.rect.width > 8 {
                    let minimize_x = self.rect.width.saturating_sub(5);
                    self.content.draw_text(minimize_x, 1, "─", Some(Color::Yellow), Some(bg_color));
                }
            }
        }
    }

    fn draw_terminal_content(&mut self) {
        let content_area = Rect::new(2, 3, 
            self.rect.width.saturating_sub(4), 
            self.rect.height.saturating_sub(5)
        );
        
        if content_area.width == 0 || content_area.height == 0 {
            return;
        }
        
        // Sfondo terminale - PULISCI TUTTO il contenuto prima
        self.content.draw_rect(content_area, ' ', Some(Color::Green), Some(Color::Black));
        
        // Calcola quante linee possiamo mostrare
        let max_lines = content_area.height;
        let start_line = if self.terminal_lines.len() > max_lines {
            self.terminal_lines.len() - max_lines
        } else {
            0
        };
        
        // Mostra cronologia comandi - ASSICURATI che ogni riga sia completa
        for (i, line) in self.terminal_lines[start_line..].iter().enumerate() {
            if i < max_lines.saturating_sub(1) {
                // Riempi la riga con spazi per assicurarti che sia completa
                let mut display_line = if line.len() > content_area.width {
                    format!("{}...", &line[..content_area.width.saturating_sub(3)])
                } else {
                    line.clone()
                };
                
                // Aggiungi spazi per riempire completamente la larghezza
                while display_line.len() < content_area.width {
                    display_line.push(' ');
                }
                
                self.content.draw_text(
                    content_area.x,
                    content_area.y + i,
                    &display_line,
                    Some(Color::Green),
                    Some(Color::Black)
                );
            }
        }
        
        // Linea corrente con cursore - posizione corretta
        let current_y = content_area.y + (max_lines.saturating_sub(1));
        if current_y < content_area.y + content_area.height {
            let mut prompt = format!("$ {}", self.current_line);
            
            // Tronca o riempi per avere esattamente la larghezza corretta
            if prompt.len() > content_area.width {
                prompt = format!("{}...", &prompt[..content_area.width.saturating_sub(3)]);
            }
            while prompt.len() < content_area.width {
                prompt.push(' ');
            }
            
            self.content.draw_text(
                content_area.x,
                current_y,
                &prompt,
                Some(Color::Green),
                Some(Color::Black)
            );
            
            // Cursore lampeggiante con posizione corretta
            let cursor_x = content_area.x + 2 + self.cursor_pos.min(content_area.width.saturating_sub(3));
            if cursor_x < content_area.x + content_area.width {
                self.content.set(
                    cursor_x,
                    current_y,
                    StyledChar::new('█').with_fg(Color::Green).with_bg(Color::Black)
                );
            }
        }
    }

    fn draw_image_content(&mut self) {
        let content_area = Rect::new(2, 3, 
            self.rect.width.saturating_sub(4), 
            self.rect.height.saturating_sub(5)
        );
        
        if content_area.width == 0 || content_area.height == 0 {
            return;
        }
        
        if let Some(ref image_fb) = self.image_data {
            // Centra l'immagine con bounds checking rigoroso
            let safe_width = image_fb.width.min(content_area.width);
            let safe_height = image_fb.height.min(content_area.height);
            
            let img_x = content_area.x + (content_area.width.saturating_sub(safe_width)) / 2;
            let img_y = content_area.y + (content_area.height.saturating_sub(safe_height)) / 2;
            
            // Disegna l'immagine con controllo rigoroso dei bounds
            for y in 0..safe_height {
                for x in 0..safe_width {
                    let dst_x = img_x + x;
                    let dst_y = img_y + y;
                    
                    if dst_x < self.content.width && dst_y < self.content.height {
                        let ch = image_fb.get(x, y);
                        self.content.set(
                            dst_x,
                            dst_y,
                            StyledChar::new(ch).with_fg(Color::White)
                        );
                    }
                }
            }
        } else {
            // Placeholder semplice senza caratteri speciali
            let msg1 = "Image Viewer";
            let msg2 = "Loading...";
            
            if content_area.width > msg1.len() && content_area.height > 2 {
                self.content.draw_text(
                    content_area.x + (content_area.width - msg1.len()) / 2,
                    content_area.y + 2,
                    msg1,
                    Some(Color::Cyan),
                    None
                );
                
                if content_area.height > 4 {
                    self.content.draw_text(
                        content_area.x + (content_area.width - msg2.len()) / 2,
                        content_area.y + 4,
                        msg2,
                        Some(Color::Yellow),
                        None
                    );
                }
            }
        }
    }

    fn draw_video_content(&mut self) {
        let content_area = Rect::new(2, 3, 
            self.rect.width.saturating_sub(4), 
            self.rect.height.saturating_sub(5)
        );
        
        if content_area.width == 0 || content_area.height == 0 {
            return;
        }
        
        // Pattern video semplificato per evitare problemi di rendering
        let frame_pattern = self.video_frame % 4;
        let chars = ['.', 'o', 'O', '#'];
        
        for y in 0..content_area.height {
            for x in 0..content_area.width {
                let pattern_idx = (x + y + frame_pattern) % 4;
                let color = match pattern_idx {
                    0 => Color::Blue,
                    1 => Color::Cyan,
                    2 => Color::White,
                    _ => Color::Gray,
                };
                
                self.content.set(
                    content_area.x + x,
                    content_area.y + y,
                    StyledChar::new(chars[pattern_idx]).with_fg(color)
                );
            }
        }
        
        // Controlli video semplificati
        if content_area.height > 0 {
            let controls_y = content_area.y + content_area.height - 1;
            let controls = "Play Stop";
            
            if controls.len() <= content_area.width {
                self.content.draw_text(
                    content_area.x,
                    controls_y,
                    controls,
                    Some(Color::White),
                    Some(Color::Black)
                );
            }
            
            // Info frame semplificato
            let frame_info = format!("{}", self.video_frame);
            let info_x = content_area.x + content_area.width.saturating_sub(frame_info.len());
            
            if info_x >= content_area.x && frame_info.len() <= content_area.width {
                self.content.draw_text(
                    info_x,
                    controls_y,
                    &frame_info,
                    Some(Color::Yellow),
                    Some(Color::Black)
                );
            }
        }
    }

    fn draw_file_manager_content(&mut self) {
        let content_area = Rect::new(2, 3, 
            self.rect.width.saturating_sub(4), 
            self.rect.height.saturating_sub(5)
        );
        
        if content_area.width == 0 || content_area.height == 0 {
            return;
        }
        
        // Lista file semplificata senza caratteri speciali
        let files = [
            "documents/",
            "downloads/",
            "pictures/",
            "config.txt",
            "readme.md",
            "script.sh",
        ];
        
        for (i, file) in files.iter().enumerate() {
            if i >= content_area.height {
                break;
            }
            
            let display_file = if file.len() > content_area.width {
                &file[..content_area.width]
            } else {
                file
            };
            
            let color = if file.ends_with('/') { Color::Blue } else { Color::White };
            
            self.content.draw_text(
                content_area.x,
                content_area.y + i,
                display_file,
                Some(color),
                None
            );
        }
    }

    fn draw_text_editor_content(&mut self) {
        let content_area = Rect::new(2, 3, 
            self.rect.width.saturating_sub(4), 
            self.rect.height.saturating_sub(5)
        );
        
        if content_area.width == 0 || content_area.height == 0 {
            return;
        }
        
        // Contenuto editor semplificato
        let lines = [
            "Text Editor",
            "",
            "Line 1: Hello World",
            "Line 2: Sample text",
            "Line 3: More content",
            "",
            "Press F2 for new editor",
        ];
        
        for (i, line) in lines.iter().enumerate() {
            if i >= content_area.height {
                break;
            }
            
            let display_line = if line.len() > content_area.width {
                &line[..content_area.width]
            } else {
                line
            };
            
            self.content.draw_text(
                content_area.x,
                content_area.y + i,
                display_line,
                Some(Color::White),
                None
            );
        }
        
        // Cursore editor
        if content_area.height > 2 {
            self.content.set(
                content_area.x,
                content_area.y + 2,
                StyledChar::new('|').with_fg(Color::Yellow)
            );
        }
    }

    fn handle_terminal_input(&mut self, ch: char) {
        match ch {
            '\n' => {
                let command = self.current_line.trim().to_string();
                self.terminal_lines.push(format!("$ {}", command));
                
                // Simula risposta comando con output realistico
                match command.as_str() {
                    "ls" => {
                        self.terminal_lines.push("documents/  downloads/  pictures/".to_string());
                        self.terminal_lines.push("config.txt  readme.md  script.sh".to_string());
                    },
                    "pwd" => self.terminal_lines.push("/home/user/desktop".to_string()),
                    "date" => self.terminal_lines.push("Mon Jan 15 14:30:25 UTC 2024".to_string()),
                    "clear" => self.terminal_lines.clear(),
                    "help" => {
                        self.terminal_lines.push("Available commands:".to_string());
                        self.terminal_lines.push("ls, pwd, date, clear, help, exit".to_string());
                    },
                    "exit" => {
                        self.terminal_lines.push("Terminal session ended.".to_string());
                        self.closed = true;
                    },
                    "" => {}, // Comando vuoto
                    _ => self.terminal_lines.push(format!("bash: {}: command not found", command)),
                }
                
                // Limita cronologia per prestazioni
                if self.terminal_lines.len() > 100 {
                    self.terminal_lines.drain(0..50);
                }
                
                self.current_line.clear();
                self.cursor_pos = 0;
            },
            '\u{8}' => { // Backspace
                if self.cursor_pos > 0 {
                    self.current_line.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                }
            },
            _ => {
                if ch.is_ascii_graphic() || ch == ' ' {
                    if self.current_line.len() < 80 { // Limita lunghezza input
                        self.current_line.insert(self.cursor_pos, ch);
                        self.cursor_pos += 1;
                    }
                }
            }
        }
        self.update_content();
    }

    fn advance_video_frame(&mut self) {
        self.video_frame = (self.video_frame + 1) % 240; // Ciclo più breve per prestazioni
    }

    fn is_click_on_close_button(&self, x: usize, y: usize) -> bool {
        if self.rect.width > 6 {
            let close_x = self.rect.x + self.rect.width.saturating_sub(3);
            let close_y = self.rect.y + 1;
            x == close_x && y == close_y
        } else {
            false
        }
    }

    fn is_click_on_minimize_button(&self, x: usize, y: usize) -> bool {
        if self.rect.width > 8 {
            let minimize_x = self.rect.x + self.rect.width.saturating_sub(5);
            let minimize_y = self.rect.y + 1;
            x == minimize_x && y == minimize_y
        } else {
            false
        }
    }
}

enum DragMode {
    None,
    Move { window_id: usize, offset: (isize, isize) },
    Resize { window_id: usize, anchor: (usize, usize) },
}

struct DesktopEnvironment {
    windows: Vec<Window>,
    next_window_id: usize,
    focused_window: Option<usize>,
    main_fb: StyledFrameBuffer,
    input_manager: InputManager,
    renderer: SmartRenderer,
    frame_timer: FrameTimer,
    running: bool,
    mouse_position: (usize, usize),
    last_mouse_position: (usize, usize),
    show_mouse: bool,
    fps_counter: f32,
    frame_count: u32,
    taskbar_height: usize,
    last_fps_update: std::time::Instant,
    // Drag & Drop
    dragging_window: Option<usize>,
    drag_offset: (isize, isize),
    drag_mode: DragMode,
    drag_start: Option<(usize, usize)>,
    // Ottimizzazioni
    windows_dirty: Vec<bool>,
    taskbar_dirty: bool,
    need_full_refresh: bool,
}

impl DesktopEnvironment {
    fn new() -> io::Result<Self> {
        let input_manager = InputManager::new()?;
        let renderer = SmartRenderer::new()?;
        let workspace_size = renderer.get_workspace_size();
        
        let main_fb = StyledFrameBuffer::new(workspace_size.0, workspace_size.1);
        
        let mut desktop = Self {
            windows: Vec::new(),
            next_window_id: 0,
            focused_window: None,
            main_fb,
            input_manager,
            renderer,
            frame_timer: FrameTimer::new(60),
            running: true,
            mouse_position: (workspace_size.0 / 2, workspace_size.1 / 2),
            last_mouse_position: (workspace_size.0 / 2, workspace_size.1 / 2),
            show_mouse: true,
            fps_counter: 60.0,
            frame_count: 0,
            taskbar_height: 2,
            last_fps_update: std::time::Instant::now(),
            dragging_window: None,
            drag_offset: (0, 0),
            drag_mode: DragMode::None,
            drag_start: None,
            windows_dirty: Vec::new(),
            taskbar_dirty: true,
            need_full_refresh: true,
        };
        
        desktop.create_initial_windows();
        Ok(desktop)
    }

    fn create_initial_windows(&mut self) {
        let workspace_size = self.renderer.get_workspace_size();
        let usable_width = workspace_size.0;
        let usable_height = workspace_size.1 - self.taskbar_height - 2;
        
        // Layout griglia 2x2 con spaziatura
        let window_width = (usable_width / 2).saturating_sub(6);
        let window_height = (usable_height / 2).saturating_sub(4);
        
        // Finestre non sovrapposte
        self.create_window("Terminal 1", WindowType::Terminal, 
            Rect::new(2, 2, window_width, window_height));
        
        let img_window = self.create_window("Image Viewer", WindowType::ImageViewer, 
            Rect::new(window_width + 6, 2, window_width, window_height));
        self.load_sample_image(img_window);
        
        self.create_window("Video Player", WindowType::VideoPlayer, 
            Rect::new(2, window_height + 6, window_width, window_height));
        
        self.create_window("File Manager", WindowType::FileManager, 
            Rect::new(window_width + 6, window_height + 6, window_width, window_height));
    }

    fn create_window(&mut self, title: &str, window_type: WindowType, rect: Rect) -> usize {
        let id = self.next_window_id;
        self.next_window_id += 1;
        
        // Assicurati che la finestra non esca dai bounds dello schermo
        let safe_rect = Rect::new(
            rect.x.min(self.main_fb.width.saturating_sub(10)),
            rect.y.min(self.main_fb.height.saturating_sub(self.taskbar_height + 5)),
            rect.width.min(self.main_fb.width.saturating_sub(rect.x)),
            rect.height.min(self.main_fb.height.saturating_sub(rect.y + self.taskbar_height))
        );
        
        let mut window = Window::new(id, title.to_string(), safe_rect, window_type);
        window.z_order = self.windows.len() as i32;
        
        self.windows.push(window);
        self.windows_dirty.push(true);
        self.focused_window = Some(id);
        self.update_focus();
        self.taskbar_dirty = true;
        self.need_full_refresh = true; // Forza refresh completo per nuove finestre
        
        id
    }

    fn load_sample_image(&mut self, window_id: usize) {
        // Genera un'immagine di esempio
        let img = self.create_sample_image();
        if let Ok(braille_fb) = image_to_braille_fb(&img, 30, 15) {
            if let Some(window) = self.windows.iter_mut().find(|w| w.id == window_id) {
                window.image_data = Some(braille_fb);
                window.update_content();
            }
        }
    }

    fn handle_mouse_click(&mut self, workspace_x: usize, workspace_y: usize, kind: MouseEventKind) {
        match kind {
            MouseEventKind::Down(_) => {
                // Se già in drag, il secondo click termina l'operazione
                match self.drag_mode {
                    DragMode::Move { window_id, .. } => {
                        // Secondo click: sposta la finestra
                        if let Some((start_x, start_y)) = self.drag_start {
                            let dx = workspace_x as isize - start_x as isize;
                            let dy = workspace_y as isize - start_y as isize;
                            if let Some(window) = self.windows.iter_mut().find(|w| w.id == window_id) {
                                let new_x = (window.rect.x as isize + dx).max(0)
                                    .min(self.main_fb.width.saturating_sub(window.rect.width) as isize) as usize;
                                let new_y = (window.rect.y as isize + dy).max(0)
                                    .min(self.main_fb.height.saturating_sub(window.rect.height + self.taskbar_height) as isize) as usize;
                                window.rect.x = new_x;
                                window.rect.y = new_y;
                                self.mark_full_refresh();
                            }
                        }
                        self.drag_mode = DragMode::None;
                        self.drag_start = None;
                        return;
                    }
                    DragMode::Resize { window_id, anchor } => {
                        // Secondo click: ridimensiona la finestra SOLO sui due bordi che combaciano con l'angolo selezionato
                        if let Some((_start_x, _start_y)) = self.drag_start {
                            if let Some(window) = self.windows.iter_mut().find(|w| w.id == window_id) {
                                let left = window.rect.x;
                                let top = window.rect.y;
                                let right = window.rect.x + window.rect.width - 1;
                                let bottom = window.rect.y + window.rect.height - 1;

                                let (anchor_x, anchor_y) = anchor;
                                let (target_x, target_y) = (workspace_x, workspace_y);

                                let mut new_x = left;
                                let mut new_y = top;
                                let mut new_w = window.rect.width;
                                let mut new_h = window.rect.height;

                                // Determina l'angolo selezionato e aggiorna solo i bordi corrispondenti
                                match (anchor_x == left, anchor_y == top, anchor_x == right, anchor_y == bottom) {
                                    // Top-left
                                    (true, true, false, false) => {
                                        new_x = target_x.min(right - 1);
                                        new_y = target_y.min(bottom - 1);
                                        new_w = right.saturating_sub(new_x) + 1;
                                        new_h = bottom.saturating_sub(new_y) + 1;
                                    }
                                    // Top-right
                                    (false, true, true, false) => {
                                        new_y = target_y.min(bottom - 1);
                                        let new_right = target_x.max(left + 1);
                                        new_w = new_right.saturating_sub(left) + 1;
                                        new_x = left;
                                        new_h = bottom.saturating_sub(new_y) + 1;
                                    }
                                    // Bottom-left
                                    (true, false, false, true) => {
                                        new_x = target_x.min(right - 1);
                                        let new_bottom = target_y.max(top + 1);
                                        new_w = right.saturating_sub(new_x) + 1;
                                        new_y = top;
                                        new_h = new_bottom.saturating_sub(top) + 1;
                                    }
                                    // Bottom-right
                                    (false, false, true, true) => {
                                        let new_right = target_x.max(left + 1);
                                        let new_bottom = target_y.max(top + 1);
                                        new_x = left;
                                        new_y = top;
                                        new_w = new_right.saturating_sub(left) + 1;
                                        new_h = new_bottom.saturating_sub(top) + 1;
                                    }
                                    _ => {}
                                }

                                // Applica limiti minimi
                                new_w = new_w.max(2);
                                new_h = new_h.max(2);

                                window.rect.x = new_x;
                                window.rect.y = new_y;
                                window.rect.width = new_w;
                                window.rect.height = new_h;
                                window.content.resize(window.rect.width, window.rect.height);
                                window.update_content();
                                self.mark_full_refresh();
                            }
                        }
                        self.drag_mode = DragMode::None;
                        self.drag_start = None;
                        return;
                    }
                    DragMode::None => {}
                }

                // Prima: verifica click su pulsanti chiusura/minimizza
                for window in &mut self.windows {
                    if window.closed {
                        continue;
                    }
                    // Pulsante chiudi
                    if window.is_click_on_close_button(workspace_x, workspace_y) {
                        window.closed = true;
                        if Some(window.id) == self.focused_window {
                            self.focused_window = None;
                            for other_window in &self.windows {
                                if !other_window.closed {
                                    self.focused_window = Some(other_window.id);
                                    break;
                                }
                            }
                        }
                        self.mark_full_refresh();
                        return;
                    }
                    // Pulsante minimizza (porta in secondo piano)
                    if window.is_click_on_minimize_button(workspace_x, workspace_y) {
                        window.minimized = !window.minimized;
                        self.mark_full_refresh();
                        return;
                    }
                }

                // Se non in drag, verifica click su angolo o bordo
                for window in &self.windows {
                    if window.closed || window.minimized {
                        continue;
                    }
                    // Angoli per resize (4 pixel)
                    let corners = [
                        (window.rect.x, window.rect.y),
                        (window.rect.x + window.rect.width - 1, window.rect.y),
                        (window.rect.x, window.rect.y + window.rect.height - 1),
                        (window.rect.x + window.rect.width - 1, window.rect.y + window.rect.height - 1),
                    ];
                    for &(cx, cy) in &corners {
                        if workspace_x == cx && workspace_y == cy {
                            self.drag_mode = DragMode::Resize { window_id: window.id, anchor: (cx, cy) };
                            self.drag_start = Some((workspace_x, workspace_y));
                            return;
                        }
                    }
                    // Bordo (escludi angoli)
                    let on_left = workspace_x == window.rect.x && workspace_y > window.rect.y && workspace_y < window.rect.y + window.rect.height - 1;
                    let on_right = workspace_x == window.rect.x + window.rect.width - 1 && workspace_y > window.rect.y && workspace_y < window.rect.y + window.rect.height - 1;
                    let on_top = workspace_y == window.rect.y && workspace_x > window.rect.x && workspace_x < window.rect.x + window.rect.width - 1;
                    let on_bottom = workspace_y == window.rect.y + window.rect.height - 1 && workspace_x > window.rect.x && workspace_x < window.rect.x + window.rect.width - 1;
                    if on_left || on_right || on_top || on_bottom {
                        self.drag_mode = DragMode::Move { window_id: window.id, offset: (workspace_x as isize - window.rect.x as isize, workspace_y as isize - window.rect.y as isize) };
                        self.drag_start = Some((workspace_x, workspace_y));
                        return;
                    }
                }
                
                // Click normale su finestra per focus e Z-order (porta in primo piano)
                let mut clicked_window = None;
                let mut max_z = -1;

                for window in &self.windows {
                    if !window.closed && !window.minimized &&
                        window.rect.contains(workspace_x, workspace_y) &&
                        window.z_order > max_z {
                        clicked_window = Some(window.id);
                        max_z = window.z_order;
                    }
                }

                if let Some(window_id) = clicked_window {
                    // Porta la finestra cliccata in primo piano
                    let max_z = self.windows.iter()
                        .filter(|w| !w.closed && !w.minimized)
                        .map(|w| w.z_order)
                        .max()
                        .unwrap_or(0);

                    // Aggiorna z_order della finestra selezionata
                    if let Some(window) = self.windows.iter_mut().find(|w| w.id == window_id) {
                        window.z_order = max_z + 1;
                    }
                    // Aggiorna z_order di tutte le altre finestre per mantenere l'ordine relativo
                    let mut sorted_ids: Vec<_> = self.windows.iter()
                        .filter(|w| !w.closed && !w.minimized && w.id != window_id)
                        .map(|w| (w.z_order, w.id))
                        .collect();
                    sorted_ids.sort_by_key(|&(z, _)| z);
                    for (new_z, &(_, id)) in sorted_ids.iter().enumerate() {
                        if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
                            w.z_order = new_z as i32;
                        }
                    }

                    self.focused_window = Some(window_id);
                    self.update_focus();
                    self.mark_window_dirty(window_id);
                    self.need_full_refresh = true;
                }
            },
            MouseEventKind::Up(_) => {
                // Fine drag
                if self.dragging_window.is_some() {
                    self.dragging_window = None;
                    self.mark_full_refresh();
                }
            },
            MouseEventKind::Moved => {
                // Drag finestra
                if let Some(window_id) = self.dragging_window {
                    if let Some(window) = self.windows.iter_mut().find(|w| w.id == window_id) {
                        let new_x = (workspace_x as isize - self.drag_offset.0)
                            .max(0)
                            .min(self.main_fb.width.saturating_sub(window.rect.width) as isize) as usize;
                        let new_y = (workspace_y as isize - self.drag_offset.1)
                            .max(0)
                            .min(self.main_fb.height.saturating_sub(window.rect.height + self.taskbar_height) as isize) as usize;
                        
                        if new_x != window.rect.x || new_y != window.rect.y {
                            window.rect.x = new_x;
                            window.rect.y = new_y;
                            self.mark_full_refresh();
                        }
                    }
                }
            },
            _ => {}
        }
    }

    fn mark_full_refresh(&mut self) {
        self.need_full_refresh = true;
        self.renderer.force_full_refresh();
        self.windows_dirty.fill(true);
        self.taskbar_dirty = true;
    }

    fn mark_window_dirty(&mut self, window_id: usize) {
        if let Some(index) = self.windows.iter().position(|w| w.id == window_id) {
            if index < self.windows_dirty.len() {
                self.windows_dirty[index] = true;
            }
        }
    }

    fn render(&mut self) -> io::Result<()> {
        // Verifica se serve un refresh
        let mouse_moved = self.mouse_position != self.last_mouse_position;
        let has_dirty_windows = self.windows_dirty.iter().any(|&dirty| dirty);
        
        if !self.need_full_refresh && !mouse_moved && !has_dirty_windows && !self.taskbar_dirty {
            return Ok(());
        }
        
        // Aggiorna regioni dirty
        if self.need_full_refresh {
            self.renderer.mark_dirty(Rect::new(0, 0, self.main_fb.width, self.main_fb.height));
        } else {
            // Mark dirty solo le regioni necessarie
            for (i, &dirty) in self.windows_dirty.iter().enumerate() {
                if dirty && i < self.windows.len() {
                    let window = &self.windows[i];
                    if !window.closed && !window.minimized {
                        self.renderer.mark_dirty(window.rect);
                    }
                }
            }
            
            if self.taskbar_dirty {
                let taskbar_rect = Rect::new(
                    0, 
                    self.main_fb.height.saturating_sub(self.taskbar_height),
                    self.main_fb.width, 
                    self.taskbar_height
                );
                self.renderer.mark_dirty(taskbar_rect);
            }
        }
        
        // Background
        if self.need_full_refresh {
            let bg_char = StyledChar::new('·').with_fg(Color::Blue).with_bg(Color::Cyan);
            self.main_fb.clear_with(bg_char);
            
            // Pattern desktop
            for y in (0..self.main_fb.height).step_by(4) {
                for x in (0..self.main_fb.width).step_by(8) {
                    self.main_fb.set(x, y, StyledChar::new('░').with_fg(Color::White).with_bg(Color::Cyan));
                }
            }
        }
        
        // Rendering finestre
        self.render_windows_with_proper_z_order();
        
        // Taskbar
        self.draw_taskbar();
        
        // Cursore mouse
        self.draw_mouse_cursor();
        
        // Rendering finale
        self.renderer.render(&self.main_fb)?;
        
        // Reset flags
        self.need_full_refresh = false;
        self.taskbar_dirty = false;
        self.windows_dirty.fill(false);
        self.last_mouse_position = self.mouse_position;
        
        Ok(())
    }

    fn render_windows_with_proper_z_order(&mut self) {
        // Ordina finestre per z-order CRESCENTE (dal basso verso l'alto)
        let mut window_indices: Vec<_> = (0..self.windows.len()).collect();
        window_indices.sort_by_key(|&i| self.windows[i].z_order);
        
        // Renderizza in ordine di z-order per sovrapposizione corretta
        for &i in &window_indices {
            if i < self.windows.len() && !self.windows[i].closed && !self.windows[i].minimized {
                self.blit_window_with_clipping(i);
            }
        }
    }

    fn blit_window_with_clipping(&mut self, window_index: usize) {
        if window_index >= self.windows.len() {
            return;
        }
        
        let window = &self.windows[window_index];
        
        // Clipping rigoroso con bounds checking completo
        let fb_width = self.main_fb.width;
        let fb_height = self.main_fb.height.saturating_sub(self.taskbar_height);
        
        let start_x = window.rect.x.min(fb_width);
        let start_y = window.rect.y.min(fb_height);
        let end_x = (window.rect.x + window.content.width).min(fb_width);
        let end_y = (window.rect.y + window.content.height).min(fb_height);
        
        // Verifica che i bounds siano validi
        if start_x >= end_x || start_y >= end_y {
            return;
        }
        
        // Copia pixel per pixel con controllo rigoroso
        for dst_y in start_y..end_y {
            for dst_x in start_x..end_x {
                let src_x = dst_x.saturating_sub(window.rect.x);
                let src_y = dst_y.saturating_sub(window.rect.y);
                
                if src_x < window.content.width && src_y < window.content.height &&
                   dst_x < fb_width && dst_y < fb_height {
                    let src_char = window.content.get(src_x, src_y);
                    self.main_fb.set(dst_x, dst_y, src_char);
                }
            }
        }
    }

    fn draw_mouse_cursor(&mut self) {
        let (mouse_x, mouse_y) = self.mouse_position;
        if (mouse_x as usize) < self.main_fb.width && (mouse_y as usize) < self.main_fb.height {
            self.main_fb.set(
                mouse_x as usize, 
                mouse_y as usize, 
                StyledChar::new('▲').with_fg(Color::Yellow).with_bg(Color::Red)
            );
        }
    }

    fn draw_taskbar(&mut self) {
        let taskbar_y = self.main_fb.height.saturating_sub(self.taskbar_height);
        let taskbar_rect = Rect::new(0, taskbar_y, self.main_fb.width, self.taskbar_height);
        
        // Sfondo taskbar
        self.main_fb.draw_rect(taskbar_rect, ' ', Some(Color::White), Some(Color::Gray));
        
        // Pulsanti finestre semplificati
        let mut x_offset = 1;
        
        for window in &self.windows {
            if window.closed {
                continue;
            }
            
            // Calcola spazio disponibile per i pulsanti
            let available_width = self.main_fb.width.saturating_sub(25); // Riserva spazio per info
            
            if x_offset >= available_width {
                break; // Non c'è più spazio
            }
            
            let button_width = 12; // Larghezza fissa per consistenza
            
            if x_offset + button_width > available_width {
                break;
            }
            
            let bg = if Some(window.id) == self.focused_window { 
                Color::Blue 
            } else if window.minimized { 
                Color::Yellow 
            } else { 
                Color::Gray 
            };
            
            // Titolo troncato per evitare overflow
            let title = if window.title.len() > 10 {
                format!("{}...", &window.title[..7])
            } else {
                window.title.clone()
            };
            
            // Disegna pulsante con dimensioni fisse
            let button_rect = Rect::new(x_offset, taskbar_y, button_width, 1);
            self.main_fb.draw_rect(button_rect, ' ', Some(Color::White), Some(bg));
            
            // Centra il titolo nel pulsante
            let title_x = x_offset + (button_width.saturating_sub(title.len())) / 2;
            self.main_fb.draw_text(
                title_x,
                taskbar_y,
                &title,
                Some(Color::White),
                Some(bg)
            );
            
            x_offset += button_width + 1;
        }
        
        // Info sistema SEMPLIFICATO per evitare disallineamenti
        let active_count = self.windows.iter().filter(|w| !w.closed).count();
        let fps_rounded = self.fps_counter.round() as u32;
        
        // Info compatta senza caratteri speciali
        let info = format!("W:{} {}fps F1:T F2:E", active_count, fps_rounded);
        
        // Posiziona info sempre alla fine della taskbar
        if info.len() < self.main_fb.width {
            let info_x = self.main_fb.width.saturating_sub(info.len());
            self.main_fb.draw_text(
                info_x,
                taskbar_y,
                &info,
                Some(Color::Black),
                Some(Color::White)
            );
        }
    }

    fn create_sample_image(&self) -> DynamicImage {
        let width = 60u32;
        let height = 60u32;
        let mut img_buffer = Vec::with_capacity((width * height) as usize);
        
        // Calcoli ottimizzati pre-computati
        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;
        let time_factor = (self.frame_count / 30) as f32 * 0.2; // Ridotto aggiornamento
        
        for y in 0..height {
            let dy = y as f32 - center_y;
            for x in 0..width {
                let dx = x as f32 - center_x;
                let distance = (dx * dx + dy * dy).sqrt();
                let angle = dy.atan2(dx);
                
                // Pattern spirale ottimizzato con meno calcoli trigonometrici
                let spiral = ((distance * 0.15 + angle * 2.5 + time_factor).sin() * 127.0 + 128.0) as u8;
                img_buffer.push(spiral);
            }
        }
        
        DynamicImage::ImageLuma8(
            image::GrayImage::from_raw(width, height, img_buffer).unwrap()
        )
    }

    fn update_focus(&mut self) {
        for window in &mut self.windows {
            window.focused = Some(window.id) == self.focused_window && !window.closed;
        }
    }

    fn handle_input(&mut self) -> io::Result<()> {
        if let Some(event) = self.input_manager.poll_event(Duration::from_millis(1))? {
            match event {
                InputEvent::Quit => {
                    self.running = false;
                },
                InputEvent::Key(KeyCode::F(1)) => {
                    let workspace_size = self.renderer.get_workspace_size();
                    let offset_x = (self.windows.len() % 3) * 15;
                    let offset_y = (self.windows.len() % 3) * 8;
                    let x = (10 + offset_x).min(workspace_size.0.saturating_sub(40));
                    let y = (5 + offset_y).min(workspace_size.1.saturating_sub(self.taskbar_height + 15));
                    
                    self.create_window(
                        &format!("Terminal {}", self.next_window_id),
                        WindowType::Terminal,
                        Rect::new(x, y, 35, 12)
                    );
                },
                InputEvent::Key(KeyCode::F(2)) => {
                    let workspace_size = self.renderer.get_workspace_size();
                    let offset_x = (self.windows.len() % 2) * 25;
                    let offset_y = (self.windows.len() % 2) * 10;
                    let x = (15 + offset_x).min(workspace_size.0.saturating_sub(50));
                    let y = (3 + offset_y).min(workspace_size.1.saturating_sub(self.taskbar_height + 20));
                    
                    self.create_window(
                        &format!("Editor {}", self.next_window_id),
                        WindowType::TextEditor,
                        Rect::new(x, y, 45, 18)
                    );
                },
                InputEvent::Key(KeyCode::Char(ch)) => {
                    // Input per terminale focalizzato
                    if let Some(focused_id) = self.focused_window {
                        if let Some(window) = self.windows.iter_mut()
                            .find(|w| w.id == focused_id && w.window_type == WindowType::Terminal && !w.closed) {
                            window.handle_terminal_input(ch);
                        }
                    }
                },
                InputEvent::Key(KeyCode::Backspace) => {
                    if let Some(focused_id) = self.focused_window {
                        if let Some(window) = self.windows.iter_mut()
                            .find(|w| w.id == focused_id && w.window_type == WindowType::Terminal && !w.closed) {
                            window.handle_terminal_input('\u{8}');
                        }
                    }
                },
                InputEvent::Key(KeyCode::Enter) => {
                    if let Some(focused_id) = self.focused_window {
                        if let Some(window) = self.windows.iter_mut()
                            .find(|w| w.id == focused_id && w.window_type == WindowType::Terminal && !w.closed) {
                            window.handle_terminal_input('\n');
                        }
                    }
                },
                InputEvent::Mouse { x, y, kind } => {
                    // Converti coordinate terminale in workspace
                    if let Some((workspace_x, workspace_y)) = self.renderer.terminal_to_workspace(x, y) {
                        self.mouse_position = (workspace_x, workspace_y);
                        self.handle_mouse_click(workspace_x, workspace_y, kind);
                    }
                },
                InputEvent::Resize { width, height } => {
                    self.renderer.update_terminal_size((width, height))?;
                    let new_workspace_size = self.renderer.get_workspace_size();
                    
                    // Ridimensiona main framebuffer
                    self.main_fb.resize(new_workspace_size.0, new_workspace_size.1);
                    
                    // Aggiorna finestre che escono dai bounds
                    for window in &mut self.windows {
                        let mut rect_changed = false;
                        
                        if window.rect.x + window.rect.width > new_workspace_size.0 {
                            window.rect.width = new_workspace_size.0.saturating_sub(window.rect.x).max(10);
                            rect_changed = true;
                        }
                        
                        if window.rect.y + window.rect.height > new_workspace_size.1.saturating_sub(self.taskbar_height) {
                            window.rect.height = new_workspace_size.1
                                .saturating_sub(window.rect.y + self.taskbar_height)
                                .max(5);
                            rect_changed = true;
                        }
                        
                        if rect_changed {
                            window.content.resize(window.rect.width, window.rect.height);
                            window.update_content();
                        }
                    }
                    
                    self.mark_full_refresh();
                },
                _ => {}
            }
        }
        Ok(())
    }

    fn update(&mut self) {
        // Aggiorna FPS counter
        if self.last_fps_update.elapsed().as_secs() >= 1 {
            self.fps_counter = self.frame_timer.get_fps();
            self.last_fps_update = std::time::Instant::now();
        }
        
        // Aggiorna frame counter per animazioni
        self.frame_count = self.frame_timer.get_frame_count() as u32;

        // Aggiorna posizione mouse (force dirty se è cambiata)
        if self.mouse_position != self.last_mouse_position {
            self.need_full_refresh = true;
        }

        // Prima raccogli le azioni da eseguire per evitare borrow multipli
        let mut video_to_update = Vec::new();
        let mut image_to_update = Vec::new();
        for (idx, window) in self.windows.iter().enumerate() {
            if window.closed {
                continue;
            }
            match window.window_type {
                WindowType::VideoPlayer => {
                    video_to_update.push(idx);
                }
                WindowType::ImageViewer => {
                    image_to_update.push(idx);
                }
                _ => {}
            }
        }

        // Aggiorna finestre video e raccogli id dirty
        let mut dirty_ids = Vec::new();
        for &idx in &video_to_update {
            if let Some(window) = self.windows.get_mut(idx) {
                window.advance_video_frame();
                window.update_content();
                dirty_ids.push(window.id);
            }
        }
        // Aggiorna finestre immagine e raccogli id dirty
        for &idx in &image_to_update {
            let img = self.create_sample_image();
            let (w, h) = {
                let window = &self.windows[idx];
                (window.rect.width.saturating_sub(4), window.rect.height.saturating_sub(5))
            };
            if let Ok(fb) = image_to_braille_fb(&img, w, h) {
                if let Some(window) = self.windows.get_mut(idx) {
                    window.image_data = Some(fb);
                    window.update_content();
                    dirty_ids.push(window.id);
                }
            }
        }
        // Ora marca dirty fuori dal borrow mutabile su self.windows
        for id in dirty_ids {
            self.mark_window_dirty(id);
        }

        // Pulisci finestre chiuse e aggiorna focus se necessario
        let mut indices_to_remove = Vec::new();
        for (i, window) in self.windows.iter().enumerate() {
            if window.closed {
                indices_to_remove.push(i);
            }
        }
        for &i in indices_to_remove.iter().rev() {
            if i < self.windows.len() {
                self.windows.remove(i);
                if i < self.windows_dirty.len() {
                    self.windows_dirty.remove(i);
                }
            }
        }
        if let Some(focused_id) = self.focused_window {
            if !self.windows.iter().any(|w| w.id == focused_id) {
                self.focused_window = self.windows.first().map(|w| w.id);
                self.update_focus();
            }
        }
        while self.windows_dirty.len() < self.windows.len() {
            self.windows_dirty.push(true);
        }
        while self.windows_dirty.len() > self.windows.len() {
            self.windows_dirty.pop();
        }
        // Aggiorna la posizione last_mouse_position all'ultima del ciclo
        self.last_mouse_position = self.mouse_position;
    }

    fn run(&mut self) -> io::Result<()> {
        self.renderer.hide_cursor()?;
        
        while self.running {
            let frame_start = std::time::Instant::now();
            
            self.handle_input()?;
            self.update();
            self.render()?;
            
            // Frame timing
            self.frame_timer.wait_for_next_frame();
            
            // Debug performance
            if frame_start.elapsed() > Duration::from_millis(25) {
                // Frame lento, forza refresh
                if self.frame_count % 20 == 0 {
                    self.mark_full_refresh();
                }
            }
        }
        
        self.renderer.show_cursor()?;
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let mut desktop = DesktopEnvironment::new()?;
    let result = desktop.run();
    
    // Nessun log di chiusura che interferirebbe con il terminale
    result
}
