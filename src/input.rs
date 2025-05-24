//! Input handling module for keyboard and mouse events

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, MouseEvent, MouseEventKind},
    terminal::{self, ClearType},
    cursor,
    ExecutableCommand,
};
use std::io::{self, stdout};
use std::time::Duration;

/// Input event types
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    Key(KeyCode),
    Mouse { x: u16, y: u16, kind: MouseEventKind },
    Resize { width: u16, height: u16 },
    Quit,
}

/// Input manager for handling terminal events
pub struct InputManager {
    mouse_enabled: bool,
    last_terminal_size: (u16, u16),
    mouse_position: (u16, u16),
    mouse_visible: bool,
    #[allow(dead_code)]
    raw_mode_enabled: bool,
}

impl InputManager {
    pub fn new() -> io::Result<Self> {
        // Inizializzazione silenziosa senza log
        terminal::enable_raw_mode()?;
        crossterm::execute!(
            stdout(),
            terminal::EnterAlternateScreen,
            cursor::Hide,
            event::EnableMouseCapture
        )?;
        
        let terminal_size = terminal::size().unwrap_or((80, 24));
        
        Ok(Self {
            mouse_enabled: true,
            last_terminal_size: terminal_size,
            mouse_position: (0, 0),
            mouse_visible: true,
            raw_mode_enabled: true,
        })
    }

    pub fn is_mouse_enabled(&self) -> bool {
        self.mouse_enabled
    }

    pub fn set_mouse_enabled(&mut self, enabled: bool) -> io::Result<()> {
        self.mouse_enabled = enabled;
        if enabled {
            crossterm::execute!(stdout(), event::EnableMouseCapture)?;
        } else {
            crossterm::execute!(stdout(), event::DisableMouseCapture)?;
        }
        Ok(())
    }

    pub fn get_mouse_position(&self) -> (u16, u16) {
        self.mouse_position
    }

    pub fn set_mouse_visible(&mut self, visible: bool) {
        self.mouse_visible = visible;
    }

    pub fn is_mouse_visible(&self) -> bool {
        self.mouse_visible
    }

    pub fn poll_event(&mut self, timeout: Duration) -> io::Result<Option<InputEvent>> {
        // Controlla sempre il ridimensionamento prima degli eventi
        let current_size = terminal::size()?;
        if current_size != self.last_terminal_size {
            self.last_terminal_size = current_size;
            return Ok(Some(InputEvent::Resize { 
                width: current_size.0, 
                height: current_size.1 
            }));
        }

        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(KeyEvent { code, modifiers, .. }) => {
                    // Gestione Ctrl+C e Ctrl+D per uscita pulita
                    if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                        match code {
                            KeyCode::Char('c') | KeyCode::Char('d') => {
                                return Ok(Some(InputEvent::Quit));
                            },
                            _ => {}
                        }
                    }
                    
                    match code {
                        KeyCode::Char('q') | KeyCode::Esc => Ok(Some(InputEvent::Quit)),
                        _ => Ok(Some(InputEvent::Key(code))),
                    }
                },
                Event::Mouse(MouseEvent { column, row, kind, .. }) => {
                    // Aggiorna posizione mouse con bounds checking
                    self.mouse_position = (
                        column.min(self.last_terminal_size.0.saturating_sub(1)),
                        row.min(self.last_terminal_size.1.saturating_sub(1))
                    );
                    
                    Ok(Some(InputEvent::Mouse { 
                        x: self.mouse_position.0, 
                        y: self.mouse_position.1, 
                        kind 
                    }))
                },
                Event::Resize(width, height) => {
                    self.last_terminal_size = (width, height);
                    Ok(Some(InputEvent::Resize { width, height }))
                },
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    pub fn clear_screen(&self) -> io::Result<()> {
        // Pulizia più robusta del terminale
        stdout().execute(terminal::Clear(terminal::ClearType::All))?;
        stdout().execute(terminal::Clear(terminal::ClearType::Purge))?; // Purge scrollback
        stdout().execute(cursor::MoveTo(0, 0))?;
        Ok(())
    }

    pub fn refresh_screen(&self) -> io::Result<()> {
        // Force refresh del terminale
        stdout().execute(terminal::Clear(ClearType::All))?;
        stdout().execute(cursor::MoveTo(0, 0))?;
        Ok(())
    }

    pub fn set_cursor_position(&self, x: u16, y: u16) -> io::Result<()> {
        stdout().execute(cursor::MoveTo(x, y))?;
        Ok(())
    }

    pub fn show_cursor(&self) -> io::Result<()> {
        stdout().execute(cursor::Show)?;
        Ok(())
    }

    pub fn hide_cursor(&self) -> io::Result<()> {
        stdout().execute(cursor::Hide)?;
        Ok(())
    }

    pub fn get_terminal_size(&self) -> (u16, u16) {
        self.last_terminal_size
    }

    pub fn force_refresh(&mut self) -> io::Result<()> {
        // Reset completo del terminale
        stdout().execute(terminal::Clear(terminal::ClearType::All))?;
        stdout().execute(terminal::Clear(terminal::ClearType::Purge))?;
        stdout().execute(cursor::MoveTo(0, 0))?;
        stdout().execute(crossterm::style::ResetColor)?;
        
        let size = terminal::size()?;
        if size != self.last_terminal_size {
            self.last_terminal_size = size;
        }
        Ok(())
    }
}

impl Drop for InputManager {
    fn drop(&mut self) {
        // Cleanup silenzioso
        let _ = crossterm::execute!(
            stdout(),
            cursor::Show,
            event::DisableMouseCapture,
            terminal::LeaveAlternateScreen
        );
        let _ = terminal::disable_raw_mode();
    }
}
