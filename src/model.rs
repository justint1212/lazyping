use crate::ping::PingSession;
use crate::update::{self, refresh};
use crate::view::render;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;

#[derive(Debug)]
pub enum View {
    Users,
    Devices,
}

/// The main application which holds the state and logic of the application.
pub struct App {
    /// Is the application running?
    running: bool,

    /// Which view is selected
    pub selected: View,

    pub users_vm: UsersViewModel,

    pub selected_user_id: String,

    pub mfa_vm: MfaViewModel,
}

impl Default for App {
    fn default() -> Self {
        App {
            running: false,
            selected: View::Users,
            users_vm: UsersViewModel::default(),
            selected_user_id: "".to_string(),
            mfa_vm: MfaViewModel::default(),
        }
    }
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        let ping_session = PingSession::new("amadmin", "amadmin").unwrap();
        refresh(&mut self, &ping_session);
        while self.running {
            terminal.draw(|frame| render(&mut self, frame))?;
            match event::read()? {
                // NOTE: only support keyboard for now
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    update::on_key_event(&mut self, key, &ping_session)
                }
                _ => {}
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct User {
    pub id: String,
    pub username: String,
    pub display_name: String,
}

pub struct UsersViewModel {
    pub users: Vec<User>,
    pub selected_idx: usize,
}

impl Default for UsersViewModel {
    fn default() -> Self {
        UsersViewModel {
            users: vec![],
            selected_idx: 0,
        }
    }
}

#[derive(Debug)]
pub struct Device {
    pub id: String,
    pub name: String,
}

pub struct MfaViewModel {
    pub devices: Vec<Device>,
    pub selected_idx: usize,
}

impl Default for MfaViewModel {
    fn default() -> Self {
        MfaViewModel {
            devices: vec![],
            selected_idx: 0,
        }
    }
}
