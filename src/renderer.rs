//! Sistema di rendering intelligente con gestione ottimizzata del framebuffer

use crate::{StyledFrameBuffer, Rect, StyledChar};
use std::io::{self, Write, stdout};
use crossterm::{cursor, terminal, ExecutableCommand};
use rayon::prelude::*;
use parking_lot::RwLock;
use std::sync::Arc;
use std::collections::VecDeque;
use std::time::{Instant, Duration};

/// Sistema di paging per regioni del framebuffer
#[allow(dead_code)]
struct FrameBufferPage {
    data: Vec<StyledChar>,
    width: usize,
    height: usize,
    dirty: bool,
    last_access: Instant,
}

impl FrameBufferPage {
    #[allow(dead_code)]
    fn new(width: usize, height: usize) -> Self {
        Self {
            data: vec![StyledChar::default(); width * height],
            width,
            height,
            dirty: true,
            last_access: Instant::now(),
        }
    }
}

impl StyledChar {
    /// Get ANSI style codes for this character
    pub fn get_style_codes(&self) -> String {
        let mut codes = String::new();
        
        if let Some(fg) = self.fg_color {
            codes.push_str(fg.to_ansi_fg());
        }
        
        if let Some(bg) = self.bg_color {
            codes.push_str(bg.to_ansi_bg());
        }
        
        codes
    }
}

/// Gestore rendering con ottimizzazioni intelligenti e paging
pub struct SmartRenderer {
    /// Dimensioni del terminale reale
    terminal_size: (u16, u16),
    /// Dimensioni dell'area di lavoro simulata
    workspace_size: (usize, usize),
    /// Offset per centrare il workspace nel terminale
    workspace_offset: (usize, usize),
    /// Buffer precedente per confronto
    last_buffer: StyledFrameBuffer,
    /// Regioni dirty ottimizzate
    dirty_regions: Vec<Rect>,
    /// Modalità rendering (completo o parziale)
    force_full_refresh: bool,
    /// Sistema di paging per grandi framebuffer
    page_cache: Arc<RwLock<std::collections::HashMap<(usize, usize), FrameBufferPage>>>,
    page_size: usize,
    max_cached_pages: usize,
    /// Buffer di output ottimizzato
    #[allow(dead_code)]
    output_buffer: Arc<RwLock<String>>,
    /// Coda di regioni da renderizzare
    #[allow(dead_code)]
    render_queue: Arc<RwLock<VecDeque<Rect>>>,
}

impl SmartRenderer {
    pub fn new() -> io::Result<Self> {
        let terminal_size = terminal::size()?;
        
        // Calcola workspace ottimale (lascia margini)
        let workspace_width = (terminal_size.0 as usize).saturating_sub(4).max(40);
        let workspace_height = (terminal_size.1 as usize).saturating_sub(4).max(20);
        
        let workspace_offset = (
            (terminal_size.0 as usize - workspace_width) / 2,
            (terminal_size.1 as usize - workspace_height) / 2
        );
        
        let last_buffer = StyledFrameBuffer::new_pooled(workspace_width, workspace_height);
        
        Ok(Self {
            terminal_size,
            workspace_size: (workspace_width, workspace_height),
            workspace_offset,
            last_buffer,
            dirty_regions: Vec::new(),
            force_full_refresh: true,
            page_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
            page_size: 64, // 64x64 pixel pages
            max_cached_pages: 16,
            output_buffer: Arc::new(RwLock::new(String::with_capacity(32768))),
            render_queue: Arc::new(RwLock::new(VecDeque::new())),
        })
    }
    
    /// Aggiorna dimensioni quando il terminale viene ridimensionato
    pub fn update_terminal_size(&mut self, new_size: (u16, u16)) -> io::Result<()> {
        self.terminal_size = new_size;
        
        // Ricalcola workspace
        let new_width = (new_size.0 as usize).saturating_sub(4).max(40);
        let new_height = (new_size.1 as usize).saturating_sub(4).max(20);
        
        self.workspace_offset = (
            (new_size.0 as usize - new_width) / 2,
            (new_size.1 as usize - new_height) / 2
        );
        
        // Ridimensiona buffer se necessario
        if (new_width, new_height) != self.workspace_size {
            self.workspace_size = (new_width, new_height);
            self.last_buffer.resize(new_width, new_height);
            self.force_full_refresh = true;
        }
        
        // Pulisci terminale completamente
        stdout().execute(terminal::Clear(terminal::ClearType::All))?;
        stdout().execute(cursor::MoveTo(0, 0))?;
        
        Ok(())
    }
    
    /// Ottieni dimensioni del workspace
    pub fn get_workspace_size(&self) -> (usize, usize) {
        self.workspace_size
    }
    
    /// Ottieni offset del workspace
    pub fn get_workspace_offset(&self) -> (usize, usize) {
        self.workspace_offset
    }
    
    /// Converti coordinate terminale in coordinate workspace
    pub fn terminal_to_workspace(&self, x: u16, y: u16) -> Option<(usize, usize)> {
        let x = x as usize;
        let y = y as usize;
        
        if x >= self.workspace_offset.0 && y >= self.workspace_offset.1 {
            let workspace_x = x - self.workspace_offset.0;
            let workspace_y = y - self.workspace_offset.1;
            
            if workspace_x < self.workspace_size.0 && workspace_y < self.workspace_size.1 {
                return Some((workspace_x, workspace_y));
            }
        }
        
        None
    }
    
    /// Converti coordinate workspace in coordinate terminale
    pub fn workspace_to_terminal(&self, x: usize, y: usize) -> (u16, u16) {
        (
            (x + self.workspace_offset.0) as u16,
            (y + self.workspace_offset.1) as u16
        )
    }
    
    /// Aggiungi regione dirty
    pub fn mark_dirty(&mut self, rect: Rect) {
        // Clamp il rect ai bounds del workspace
        let clamped_rect = Rect::new(
            rect.x.min(self.workspace_size.0),
            rect.y.min(self.workspace_size.1),
            rect.width.min(self.workspace_size.0.saturating_sub(rect.x)),
            rect.height.min(self.workspace_size.1.saturating_sub(rect.y))
        );
        
        if clamped_rect.width > 0 && clamped_rect.height > 0 {
            self.dirty_regions.push(clamped_rect);
        }
    }
    
    /// Forza refresh completo
    pub fn force_full_refresh(&mut self) {
        self.force_full_refresh = true;
        self.dirty_regions.clear();
        self.mark_dirty(Rect::new(0, 0, self.workspace_size.0, self.workspace_size.1));
    }
    
    /// Rendering intelligente con ottimizzazioni
    pub fn render(&mut self, buffer: &StyledFrameBuffer) -> io::Result<()> {
        if buffer.width != self.workspace_size.0 || buffer.height != self.workspace_size.1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Buffer size mismatch with workspace"
            ));
        }
        
        if self.force_full_refresh {
            self.render_full(buffer)?;
            self.force_full_refresh = false;
        } else {
            self.render_incremental(buffer)?;
        }
        
        // Aggiorna buffer di confronto
        self.last_buffer = buffer.clone();
        self.dirty_regions.clear();
        
        stdout().flush()?;
        Ok(())
    }
    
    /// Rendering con sistema di paging
    pub fn render_paged(&mut self, buffer: &StyledFrameBuffer) -> io::Result<()> {
        if buffer.width != self.workspace_size.0 || buffer.height != self.workspace_size.1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Buffer size mismatch with workspace"
            ));
        }

        if self.force_full_refresh {
            self.render_full_paged(buffer)?;
            self.force_full_refresh = false;
        } else {
            self.render_incremental_paged(buffer)?;
        }

        self.last_buffer = buffer.clone();
        self.dirty_regions.clear();
        
        stdout().flush()?;
        Ok(())
    }

    /// Rendering completo
    fn render_full(&mut self, buffer: &StyledFrameBuffer) -> io::Result<()> {
        // Pulisci terminale
        stdout().execute(terminal::Clear(terminal::ClearType::All))?;
        
        // Disegna bordo workspace
        self.draw_workspace_border()?;
        
        // Renderizza tutto il buffer
        for y in 0..buffer.height {
            for x in 0..buffer.width {
                let styled_char = buffer.get(x, y);
                let (term_x, term_y) = self.workspace_to_terminal(x, y);
                
                stdout().execute(cursor::MoveTo(term_x, term_y))?;
                print!("{}", styled_char.to_string());
            }
        }
        
        Ok(())
    }
    
    /// Rendering completo con paging
    fn render_full_paged(&mut self, buffer: &StyledFrameBuffer) -> io::Result<()> {
        stdout().execute(terminal::Clear(terminal::ClearType::All))?;
        self.draw_workspace_border()?;

        // Suddividi il buffer in pagine
        let pages_x = (buffer.width + self.page_size - 1) / self.page_size;
        let pages_y = (buffer.height + self.page_size - 1) / self.page_size;

        // Crea lista di regioni da renderizzare
        let mut page_regions = Vec::new();
        for page_y in 0..pages_y {
            for page_x in 0..pages_x {
                let start_x = page_x * self.page_size;
                let start_y = page_y * self.page_size;
                let end_x = (start_x + self.page_size).min(buffer.width);
                let end_y = (start_y + self.page_size).min(buffer.height);
                
                page_regions.push(Rect::new(start_x, start_y, end_x - start_x, end_y - start_y));
            }
        }

        // Rendering parallelo delle pagine
        let workspace_offset = self.workspace_offset;
        let page_outputs: Vec<String> = page_regions
            .into_par_iter()
            .map(|page_rect| {
                SmartRenderer::render_page_region_static(buffer, page_rect, workspace_offset)
            })
            .collect();

        // Output sequenziale dei risultati
        for page_output in page_outputs {
            print!("{}", page_output);
        }

        Ok(())
    }
    
    /// Rendering incrementale (solo regioni cambiate)
    fn render_incremental(&mut self, buffer: &StyledFrameBuffer) -> io::Result<()> {
        // Ottimizza regioni dirty (merge regioni adiacenti)
        let optimized_regions = self.optimize_dirty_regions();
        
        for region in optimized_regions {
            self.render_region(buffer, region)?;
        }
        
        Ok(())
    }
    
    /// Rendering incrementale con paging
    fn render_incremental_paged(&mut self, buffer: &StyledFrameBuffer) -> io::Result<()> {
        // Identifica pagine dirty
        let dirty_pages = self.identify_dirty_pages(buffer);
        
        if dirty_pages.is_empty() {
            return Ok(());
        }

        // Rendering parallelo solo delle pagine dirty
        let workspace_offset = self.workspace_offset;
        let page_outputs: Vec<String> = dirty_pages
            .into_par_iter()
            .map(|page_rect| Self::render_page_region_static(buffer, page_rect, workspace_offset))
            .collect();

        // Output sequenziale
        for page_output in page_outputs {
            print!("{}", page_output);
        }

        Ok(())
    }
    
    /// Renderizza una specifica regione
    fn render_region(&mut self, buffer: &StyledFrameBuffer, region: Rect) -> io::Result<()> {
        for y in region.y..(region.y + region.height).min(buffer.height) {
            let mut line_changed = false;
            
            // Verifica se la riga è cambiata
            for x in region.x..(region.x + region.width).min(buffer.width) {
                if buffer.get(x, y) != self.last_buffer.get(x, y) {
                    line_changed = true;
                    break;
                }
            }
            
            if line_changed {
                // Renderizza l'intera riga per performance
                let (start_term_x, term_y) = self.workspace_to_terminal(region.x, y);
                stdout().execute(cursor::MoveTo(start_term_x, term_y))?;
                
                // Ottimizzazione: costruisci stringa completa per la riga
                let mut line_string = String::new();
                for x in region.x..(region.x + region.width).min(buffer.width) {
                    let styled_char = buffer.get(x, y);
                    line_string.push_str(&styled_char.to_string());
                }
                
                print!("{}", line_string);
            }
        }
        
        Ok(())
    }
    
    /// Identifica le pagine che sono cambiate
    fn identify_dirty_pages(&self, buffer: &StyledFrameBuffer) -> Vec<Rect> {
        let mut dirty_pages = Vec::new();
        let pages_x = (buffer.width + self.page_size - 1) / self.page_size;
        let pages_y = (buffer.height + self.page_size - 1) / self.page_size;

        for page_y in 0..pages_y {
            for page_x in 0..pages_x {
                let start_x = page_x * self.page_size;
                let start_y = page_y * self.page_size;
                let end_x = (start_x + self.page_size).min(buffer.width);
                let end_y = (start_y + self.page_size).min(buffer.height);
                
                let page_rect = Rect::new(start_x, start_y, end_x - start_x, end_y - start_y);
                
                // Verifica se la pagina è cambiata con campionamento
                if self.is_page_dirty(buffer, page_rect) {
                    dirty_pages.push(page_rect);
                }
            }
        }

        dirty_pages
    }

    /// Verifica se una pagina è dirty usando campionamento intelligente
    fn is_page_dirty(&self, buffer: &StyledFrameBuffer, page_rect: Rect) -> bool {
        // Campiona alcuni punti della pagina per performance
        let sample_points = [
            (0, 0),
            (page_rect.width / 2, page_rect.height / 2),
            (page_rect.width - 1, page_rect.height - 1),
            (page_rect.width / 4, page_rect.height / 4),
        ];

        for &(dx, dy) in &sample_points {
            let x = page_rect.x + dx;
            let y = page_rect.y + dy;
            
            if x < buffer.width && y < buffer.height &&
               x < self.last_buffer.width && y < self.last_buffer.height {
                if buffer.get(x, y) != self.last_buffer.get(x, y) {
                    return true;
                }
            }
        }
        
        false
    }

    /// Rendering ottimizzato di una regione/pagina (versione statica per parallelismo)
    fn render_page_region_static(buffer: &StyledFrameBuffer, region: Rect, workspace_offset: (usize, usize)) -> String {
        let mut output = String::with_capacity(region.width * region.height * 15);
        
        // Rendering ottimizzato con batching degli stili
        for y in region.y..(region.y + region.height).min(buffer.height) {
            let term_x = (region.x + workspace_offset.0) as u16;
            let term_y = (y + workspace_offset.1) as u16;
            output.push_str(&format!("\x1b[{};{}H", term_y + 1, term_x + 1));
            
            // Batch caratteri con stesso stile
            let mut current_style = None;
            let mut style_batch = String::new();
            
            for x in region.x..(region.x + region.width).min(buffer.width) {
                let styled_char = buffer.get(x, y);
                let char_style = (styled_char.fg_color, styled_char.bg_color);
                
                if current_style != Some(char_style) {
                    // Flush batch precedente
                    if !style_batch.is_empty() {
                        output.push_str(&style_batch);
                        style_batch.clear();
                    }
                    
                    // Nuovo stile
                    output.push_str(&styled_char.get_style_codes());
                    current_style = Some(char_style);
                }
                
                style_batch.push(styled_char.ch);
            }
            
            // Flush finale
            if !style_batch.is_empty() {
                output.push_str(&style_batch);
            }
            
            output.push_str("\x1b[0m");
        }
        
        output
    }

    /// Rendering ottimizzato di una regione/pagina
    #[allow(dead_code)]
    fn render_page_region(&self, buffer: &StyledFrameBuffer, region: Rect) -> String {
        Self::render_page_region_static(buffer, region, self.workspace_offset)
    }

    /// Ottimizza regioni dirty unendo quelle adiacenti
    fn optimize_dirty_regions(&self) -> Vec<Rect> {
        if self.dirty_regions.len() <= 1 {
            return self.dirty_regions.clone();
        }
        
        // Per semplicità, se ci sono troppe regioni, renderizza tutto
        if self.dirty_regions.len() > 20 {
            return vec![Rect::new(0, 0, self.workspace_size.0, self.workspace_size.1)];
        }
        
        // TODO: Implementare merge intelligente delle regioni
        // Per ora ritorna le regioni così come sono
        self.dirty_regions.clone()
    }
    
    /// Ottimizza regioni dirty con clustering intelligente
    #[allow(dead_code)]
    fn optimize_dirty_regions_advanced(&self) -> Vec<Rect> {
        if self.dirty_regions.len() <= 1 {
            return self.dirty_regions.clone();
        }
        
        if self.dirty_regions.len() > 50 {
            // Troppe regioni: rendering completo
            return vec![Rect::new(0, 0, self.workspace_size.0, self.workspace_size.1)];
        }
        
        // Clustering delle regioni adiacenti
        let mut optimized = Vec::new();
        let mut processed = vec![false; self.dirty_regions.len()];
        
        for i in 0..self.dirty_regions.len() {
            if processed[i] {
                continue;
            }
            
            let mut cluster = self.dirty_regions[i];
            processed[i] = true;
            
            // Cerca regioni adiacenti da unire
            let mut found_adjacent = true;
            while found_adjacent {
                found_adjacent = false;
                
                for j in 0..self.dirty_regions.len() {
                    if processed[j] {
                        continue;
                    }
                    
                    let other = self.dirty_regions[j];
                    
                    // Verifica adiacenza e convenineza del merge
                    if self.should_merge_regions(cluster, other) {
                        cluster = self.merge_regions(cluster, other);
                        processed[j] = true;
                        found_adjacent = true;
                    }
                }
            }
            
            optimized.push(cluster);
        }
        
        optimized
    }

    /// Verifica se due regioni dovrebbero essere unite
    #[allow(dead_code)]
    fn should_merge_regions(&self, a: Rect, b: Rect) -> bool {
        // Calcola l'area del bounding box che conterrebbe entrambe
        let merged = self.merge_regions(a, b);
        let merged_area = merged.width * merged.height;
        let combined_area = a.width * a.height + b.width * b.height;
        
        // Unisci solo se l'overhead è ragionevole (max 50% di spreco)
        merged_area <= combined_area * 3 / 2
    }

    /// Unisce due regioni in un bounding box
    #[allow(dead_code)]
    fn merge_regions(&self, a: Rect, b: Rect) -> Rect {
        let min_x = a.x.min(b.x);
        let min_y = a.y.min(b.y);
        let max_x = (a.x + a.width).max(b.x + b.width);
        let max_y = (a.y + a.height).max(b.y + b.height);
        
        Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }
    
    /// Cleanup periodico della cache delle pagine
    pub fn cleanup_page_cache(&mut self) {
        let mut cache = self.page_cache.write();
        
        if cache.len() <= self.max_cached_pages {
            return;
        }
        
        // Rimuovi le pagine più vecchie
        let now = Instant::now();
        cache.retain(|_, page| {
            now.duration_since(page.last_access) < Duration::from_secs(5)
        });
        
        // Se ancora troppo pieno, rimuovi le più vecchie
        if cache.len() > self.max_cached_pages {
            // Raccogli le chiavi da rimuovere separatamente
            let mut pages_to_remove: Vec<_> = cache.iter()
                .map(|(key, page)| (*key, page.last_access))
                .collect();
            
            pages_to_remove.sort_by_key(|(_, last_access)| *last_access);
            
            let to_remove = pages_to_remove.len() - self.max_cached_pages;
            for (key, _) in pages_to_remove.iter().take(to_remove) {
                cache.remove(key);
            }
        }
    }
    
    /// Disegna bordo del workspace
    fn draw_workspace_border(&self) -> io::Result<()> {
        let border_color = "\x1b[36m"; // Cyan
        let reset_color = "\x1b[0m";
        
        // Bordo superiore
        let top_y = self.workspace_offset.1.saturating_sub(1);
        if top_y < self.terminal_size.1 as usize {
            stdout().execute(cursor::MoveTo(
                self.workspace_offset.0.saturating_sub(1) as u16,
                top_y as u16
            ))?;
            print!("{}┌{}┐{}", 
                border_color,
                "─".repeat(self.workspace_size.0),
                reset_color
            );
        }
        
        // Bordi laterali
        for y in 0..self.workspace_size.1 {
            let term_y = (self.workspace_offset.1 + y) as u16;
            
            // Bordo sinistro
            if self.workspace_offset.0 > 0 {
                stdout().execute(cursor::MoveTo(
                    self.workspace_offset.0.saturating_sub(1) as u16,
                    term_y
                ))?;
                print!("{}│{}", border_color, reset_color);
            }
            
            // Bordo destro
            let right_x = (self.workspace_offset.0 + self.workspace_size.0) as u16;
            if right_x < self.terminal_size.0 {
                stdout().execute(cursor::MoveTo(right_x, term_y))?;
                print!("{}│{}", border_color, reset_color);
            }
        }
        
        // Bordo inferiore
        let bottom_y = (self.workspace_offset.1 + self.workspace_size.1) as u16;
        if bottom_y < self.terminal_size.1 {
            stdout().execute(cursor::MoveTo(
                self.workspace_offset.0.saturating_sub(1) as u16,
                bottom_y
            ))?;
            print!("{}└{}┘{}", 
                border_color,
                "─".repeat(self.workspace_size.0),
                reset_color
            );
        }
        
        Ok(())
    }
    
    /// Nascondi cursore
    pub fn hide_cursor(&self) -> io::Result<()> {
        stdout().execute(cursor::Hide)?;
        Ok(())
    }
    
    /// Mostra cursore
    pub fn show_cursor(&self) -> io::Result<()> {
        stdout().execute(cursor::Show)?;
        Ok(())
    }
}
