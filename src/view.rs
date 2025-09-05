use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::model::App;

fn user_list(app: &mut App, frame: &mut Frame, area: Rect) {
    let mut items = vec![];

    for user in app.users_vm.users.iter() {
        let text = format!("{} ({}) - {}", user.display_name, user.id, user.username);
        items.push(ListItem::new(text));
    }

    let user_list = List::new(items)
        .block(Block::default().title("Users").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    let mut list_state = ListState::default();
    list_state.select(Some(app.users_vm.selected_idx));

    frame.render_stateful_widget(user_list.clone(), area, &mut list_state);
}

fn mfa_list(app: &mut App, frame: &mut Frame, area: Rect) {
    let mut items = vec![];

    for device in app.mfa_vm.devices.iter() {
        let text = format!("{} {}", device.id, device.name);
        items.push(ListItem::new(text));
    }

    let device_list = List::new(items)
        .block(Block::default().title("MFA Devices").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    let mut list_state = ListState::default();
    list_state.select(Some(app.mfa_vm.selected_idx));

    frame.render_stateful_widget(device_list.clone(), area, &mut list_state);
}

pub fn render(app: &mut App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal) // or Vertical if you want top/bottom
        .constraints([
            Constraint::Percentage(33), // 1/3 of the screen
            Constraint::Percentage(67), // remaining 2/3
        ])
        .split(frame.area());

    user_list(app, frame, chunks[0]);
    mfa_list(app, frame, chunks[1]);
}
