use ratatui::{
    Frame,
    layout::{Rect, Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Modifier},
    widgets::{Block, BorderType, Borders, Paragraph, Clear, Cell, Row, Table},
    text::Line
};
use chrono::{Local, Datelike};

use crate::app::{App, truncate};
use crate::models::days_in_month;

pub fn draw_calendar_popup(f: &mut Frame, app: &mut App, size: Rect) {
    let border_type = match app.config.view.border_types.to_lowercase().as_str() {
        #[allow(clippy::match_same_arms)]
        "rounded" => BorderType::Rounded,
        "thick" => BorderType::Thick,
        "double" => BorderType::Double,
        "plain" => BorderType::Plain,
        "quadrant" => BorderType::QuadrantOutside,
        _ => BorderType::Rounded,
    };
    let popup_area = centered_rect(30, 35, size);

    f.render_widget(Clear, popup_area);
    let popup_block = Block::default()
        .title(format!("Calendar - {}", app.calendar.current_date.format("%B %Y"))).title_alignment(ratatui::layout::HorizontalAlignment::Center)
        .borders(Borders::ALL)
        .border_type(border_type)
        .style(Style::default().fg(Color::White));

    f.render_widget(popup_block, popup_area);

    let vertical_padding = (popup_area.height.saturating_sub(12)) / 2;
    let horizontal_padding = (popup_area.width.saturating_sub(40)) / 2;
    let centered_area = Layout::default().direction(Direction::Vertical).constraints([
            Constraint::Length(vertical_padding),
            Constraint::Length(10),
            Constraint::Length(vertical_padding),
    ]).split(popup_area)[1];

    let centered_area = Layout::default().direction(Direction::Horizontal).constraints([
            Constraint::Length(horizontal_padding),
            Constraint::Length(40),
            Constraint::Length(horizontal_padding),
        ]).split(centered_area)[1];
    let calendar_layout = Layout::default().direction(Direction::Vertical).constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ]).split(centered_area);

    let weekdays = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let mut header_cells = Vec::new();
    for weekday in &weekdays {
        header_cells.push(Cell::from(*weekday).style(Style::default().fg(Color::Yellow)));
    }
    let header_row = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD));

    let first_day = chrono::NaiveDate::from_ymd_opt(
        app.calendar.current_date.year(),
        app.calendar.current_date.month(),
        1
    ).unwrap();

    let days_in_month = days_in_month(app.calendar.current_date.year(), app.calendar.current_date.month());
    let start_weekday = first_day.weekday().number_from_monday() as usize;

    let mut rows = Vec::new();
    let mut current_week = Vec::new();

    for _ in 1..start_weekday {
        current_week.push(Cell::from("  "));
    }

    for day in 1..=days_in_month {
        let date = chrono::NaiveDate::from_ymd_opt(
            app.calendar.current_date.year(),
            app.calendar.current_date.month(),
            day
        ).unwrap();

        let date_str = format!("{:02}.{:02}.{}", day, date.month(), date.year());
        let has_tasks = app.calendar.tasks_by_date.contains(&date_str);

        let day_str = if has_tasks {
            format!("{}{}", day, "•")
        } else {
            format!("{day:2}")
        };

        let mut cell_style = Style::default();

        if date == app.calendar.selected_date {
            cell_style = cell_style.bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD);
        } else if date == Local::now().date_naive() {
            cell_style = cell_style.fg(Color::Cyan);
        } else if has_tasks {
            cell_style = cell_style.fg(Color::Green);
        }

        current_week.push(Cell::from(day_str).style(cell_style));

        if current_week.len() == 7 {
            rows.push(Row::new(current_week.clone()));
            current_week.clear();
        }
    }

    while current_week.len() < 7 && !current_week.is_empty() {
        current_week.push(Cell::from("  "));
    }
    if !current_week.is_empty() {
        rows.push(Row::new(current_week));
    }

    let widths = vec![Constraint::Length(5); 7];
    let calendar_table = Table::new(rows, widths).header(header_row).block(Block::default().borders(Borders::NONE));

    f.render_widget(calendar_table, calendar_layout[1]);

    let nav_text = Paragraph::new("←/→ day • ↑/↓ week • Enter select • Esc close").style(Style::default().fg(Color::DarkGray)).alignment(Alignment::Center);

    f.render_widget(nav_text, calendar_layout[2]);
}

pub fn draw_calendar_day_tasks_popup(f: &mut Frame, app: &mut App, size: Rect) {
    let border_type = match app.config.view.border_types.to_lowercase().as_str() {
        #[allow(clippy::match_same_arms)]
        "rounded" => BorderType::Rounded,
        "thick" => BorderType::Thick,
        "double" => BorderType::Double,
        "plain" => BorderType::Plain,
        "quadrant" => BorderType::QuadrantOutside,
        _ => BorderType::Rounded,
    };
    let popup_area = centered_rect(70, 70, size);
    f.render_widget(Clear, popup_area);

    let title = format!("Tasks for {}", app.calendar.selected_date.format("%d.%m.%Y"));

    let popup_block = Block::default()
        .title(title).title_alignment(ratatui::layout::HorizontalAlignment::Center)
        .borders(Borders::ALL)
        .border_type(border_type)
        .style(Style::default().fg(Color::White));

    f.render_widget(popup_block, popup_area);

    let content_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Min(0),
            Constraint::Percentage(10),
        ])
        .split(popup_area)[1];

    let content_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(5),
            Constraint::Min(0),
            Constraint::Percentage(5),
        ])
        .split(content_area)[1];

    let tasks_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(content_area);

    if app.calendar.day_tasks.is_empty() {
        let no_tasks = Paragraph::new("No tasks for this day")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default());
        f.render_widget(no_tasks, tasks_layout[0]);
    } else {
        let rows: Vec<Row> = app.calendar.day_tasks
            .iter()
            .map(|task| {
                let checkbox = if task.checked { "[x]" } else { "[ ]" };
                Row::new(vec![
                    Cell::from(checkbox),
                    Cell::from(truncate(&task.title, 40)),
                    Cell::from(task.priority.clone()),
                    Cell::from(truncate(&task.description, 40)),
                    Cell::from(task.folder.clone()),
                ])
            }).collect();

        let table = Table::new(
            rows,
            vec![
                Constraint::Length(5),
                Constraint::Percentage(35),
                Constraint::Length(15),
                Constraint::Percentage(30),
                Constraint::Length(15),
            ]
        ).header(
            Row::new(vec![" ", "Name", "Priority", "Description", "List"])
                .style(Style::default().fg(Color::White))
        ).block(Block::default().borders(Borders::NONE))
        .row_highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);

        f.render_stateful_widget(table, tasks_layout[0], &mut app.table_state);
    }

    let nav_text = Paragraph::new(vec![
        Line::from("↑/↓ navigate • Enter/Space toggle • Ctrl+D delete"),
        Line::from("Esc back to calendar"),
    ])
    .style(Style::default().fg(Color::DarkGray))
    .alignment(Alignment::Center);

    f.render_widget(nav_text, tasks_layout[1]);
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
