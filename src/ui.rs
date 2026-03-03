use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph, List, ListItem, Clear, Table, Row, Cell},
    text::Span
};

use crate::app::{App, truncate};
use crate::models::ViewMode;

pub fn draw(f: &mut Frame, app: &mut App) {
    let size = f.area();

    f.render_widget(Clear, size);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)])
        .split(size);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(main_chunks[0]);

    let view_title = match app.view_mode {
        ViewMode::Normal => format!("Tasks - {}", app.selected_folder.as_deref().unwrap_or("")),
        ViewMode::Today => "Today's Tasks".to_string(),
        ViewMode::NextSevenDays => "Next 7 Days".to_string(),
        ViewMode::WithoutDate => "WithoutDate".to_string(),
    };

    let input_text = if app.task_name.is_empty() {
        Span::raw(" ")
    } else {
        Span::raw(app.task_name.as_str())
    };

    let inputwidet = Paragraph::new(input_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(view_title))
        .style(Style::default().fg(Color::White));

    f.render_widget(inputwidet, left_chunks[0]);

    let content = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1)])
        .split(left_chunks[1]);

    draw_tasks_table(f, app, content[0]);

    match app.popup_state {
        crate::app::PopupState::FolderList => {
            draw_popup(f, app, size);
        }
        crate::app::PopupState::MoveTask => {
            draw_move_popup(f, app, size);
        }
        crate::app::PopupState::SpecialViews => {
            draw_special_views_popup(f, app, size);
        }
        crate::app::PopupState::FilePreview => {
            draw_file_preview(f, app, size);
        }
        crate::app::PopupState::ConfirmDelete => {
            draw_popup_delete(f, size);
        }
        crate::app::PopupState::None => {}
    }

    if app.calendar.show_calendar {
        crate::calendar::draw_calendar_popup(f, app, size);
    }

    if app.calendar.show_day_tasks {
        crate::calendar::draw_calendar_day_tasks_popup(f, app, size);
    }
}

fn draw_move_popup(f: &mut Frame, app: &mut App, size: Rect) {
    let popup_area = centered_rect(30, 35, size);
    f.render_widget(Clear, popup_area);

    let popup_block = Block::default()
        .title("Move task to folder")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(Color::White));

    f.render_widget(popup_block, popup_area);

    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Min(0)
        ]).split(popup_area);

    let folder_items: Vec<ListItem> = app.available_folders
        .iter()
        .map(|d| ListItem::new(d.clone()))
        .collect();

    let folder_list = List::new(folder_items)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
        ).highlight_symbol("▶ ");

    f.render_stateful_widget(folder_list, popup_chunks[0], &mut app.folder_state);
}

fn draw_tasks_table(f: &mut Frame, app: &mut App, area: Rect) {
    let (columns, headers) = match app.view_mode {
        ViewMode::Normal => (
            vec![
                Constraint::Length(5),
                Constraint::Percentage(40),
                Constraint::Length(12),
                Constraint::Length(20),
                Constraint::Percentage(43),
            ],
            vec![" ", "Name", "Date", "Priority", "Description"]
        ),
        ViewMode::Today | ViewMode::NextSevenDays | ViewMode::WithoutDate => (
            vec![
                Constraint::Length(5),
                Constraint::Percentage(30),
                Constraint::Length(12),
                Constraint::Length(15),
                Constraint::Percentage(25),
                Constraint::Length(15),
            ],
            vec![" ", "Name", "Date", "Priority", "Description", "List"]
        ),
    };

    let rows: Vec<Row> = app.tasks
        .iter()
        .map(|task| {
            let checkbox = if task.checked { "[x]" } else { "[ ]" };

            match app.view_mode {
                ViewMode::Normal => {
                    Row::new(vec![
                        Cell::from(checkbox),
                        Cell::from(truncate(&task.title, 45)),
                        Cell::from(task.date.clone()),
                        Cell::from(task.priority.clone()),
                        Cell::from(truncate(&task.description, 85)),
                    ])
                }
                ViewMode::Today | ViewMode::NextSevenDays | ViewMode::WithoutDate => {
                    Row::new(vec![
                        Cell::from(checkbox),
                        Cell::from(truncate(&task.title, 35)),
                        Cell::from(task.date.clone()),
                        Cell::from(task.priority.clone()),
                        Cell::from(truncate(&task.description, 45)),
                        Cell::from(task.folder.clone()),
                    ])
                }
            }
        }).collect();

    let table = Table::new(
        rows,
        columns
    ).header(
        Row::new(headers).style(Style::default().fg(Color::White))
    ).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
    ).row_highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White));

    f.render_stateful_widget(table, area, &mut app.table_state);
}

fn draw_special_views_popup(f: &mut Frame, app: &mut App, size: Rect) {
    let popup_area = centered_rect(30, 13, size);
    f.render_widget(Clear, popup_area);

    let popup_block = Block::default()
        .title("Special Views")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(Color::White));

    f.render_widget(popup_block, popup_area);

    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Min(0)
        ]).split(popup_area);

    let view_items: Vec<ListItem> = app.special_views
        .iter()
        .map(|view| {
            ListItem::new(view.clone())
        })
        .collect();

    let view_list = List::new(view_items)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(view_list, popup_chunks[0], &mut app.folder_state);
}

fn draw_file_preview(f: &mut Frame, app: &mut App, size: Rect) {
    let area = centered_rect(30, 40, size);
    f.render_widget(Clear, area);

    let preview = Paragraph::new(app.file_content.as_str())
        .block(
            Block::default()
                .title("Todo info")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
        ).wrap(ratatui::widgets::Wrap {trim: false});

    f.render_widget(preview, area);
}

fn draw_popup(f: &mut Frame, app: &mut App, size: Rect) {
    let popup_area = centered_rect(30, 35, size);
    f.render_widget(Clear, popup_area);

    let popup_block = Block::default()
        .title("Lists")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(Color::White));

    f.render_widget(popup_block, popup_area);

    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0)
        ]).split(popup_area);

    let popup_input = Paragraph::new(app.folder_name.as_str())
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));

    f.render_widget(popup_input, popup_chunks[0]);

    let folder_items: Vec<ListItem> = app.all_folders
        .iter()
        .map(|d| ListItem::new(d.clone()))
        .collect();

    let folder_list = List::new(folder_items)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
        ).highlight_symbol("▶ ");

    f.render_stateful_widget(folder_list, popup_chunks[1], &mut app.folder_state);
}

fn draw_popup_delete(f: &mut Frame, size: Rect) {
    let area = centered_rect(30, 20, size);
    f.render_widget(Clear, area);

    let block = Paragraph::new(
        "Delete selected item\n\n[y] Yes [n / Esc] Cancel"
    ).block(
        Block::default()
            .title("Confirm delete")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
    ).alignment(Alignment::Center);

    f.render_widget(block, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100-percent_y)/2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100-percent_y)/2)
        ].as_ref())
        .split(r);

    let vertical = popup_layout[1];

    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100-percent_x)/2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100-percent_x)/2)
        ].as_ref())
        .split(vertical);

    horizontal_layout[1]
}
