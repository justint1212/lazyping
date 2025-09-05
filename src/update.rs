use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    model::{App, View},
    ping::PingSession,
};

/// handles the key events and updates the state of [`app`].
pub fn on_key_event(app: &mut App, key: KeyEvent, session: &PingSession) {
    match app.selected {
        crate::model::View::Users => handle_key_users(app, key, session),
        crate::model::View::Devices => handle_key_mfa(app, key),
    };

    match (key.modifiers, key.code) {
        (_, KeyCode::Esc | KeyCode::Char('q')) | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
            app.quit()
        }
        // add other key handlers here.
        _ => {}
    }
}

fn handle_key_users(app: &mut App, key: KeyEvent, session: &PingSession) {
    let state = &mut app.users_vm;
    match (key.modifiers, key.code) {
        (_, KeyCode::Char('k')) => {
            state.selected_idx = (state.selected_idx + 1) % state.users.len();
        }

        (_, KeyCode::Char('j')) => {
            // gigabrain hack because the component only supports unsigned ints and sometimes
            // this thing goes negative. :3c
            let idx = if state.selected_idx == 0 {
                state.users.len() - 1
            } else {
                state.selected_idx - 1
            };
            state.selected_idx = idx;
        }

        (_, KeyCode::Char(' ')) => {
            app.selected_user_id = state.users.get(state.selected_idx).unwrap().id.clone();
            app.selected = View::Devices;
            load_devices(app, session);
        }

        // add other key handlers here.
        _ => {}
    }
}

fn handle_key_mfa(app: &mut App, key: KeyEvent) {
    let state = &mut app.mfa_vm;
    match (key.modifiers, key.code) {
        (_, KeyCode::Char('k')) => {
            state.selected_idx = (state.selected_idx + 1) % state.devices.len();
        }

        (_, KeyCode::Char('j')) => {
            let idx = if state.selected_idx == 0 {
                state.devices.len() - 1
            } else {
                state.selected_idx - 1
            };
            state.selected_idx = idx;
        }

        (_, KeyCode::Char('h')) => {
            app.selected = View::Users; // go back to userse
        }

        // add other key handlers here.
        _ => {}
    }
}

pub fn refresh(app: &mut App, session: &PingSession) {
    app.users_vm.users.clear();
    if let Ok(users) = session.list_users() {
        app.users_vm.users.extend(users);
    }
}

pub fn load_devices(app: &mut App, session: &PingSession) {
    app.mfa_vm.devices.clear();
    if let Ok(devices) = session.oath_list_devices(&app.selected_user_id) {
        app.mfa_vm.devices.extend(devices);
    }
}
