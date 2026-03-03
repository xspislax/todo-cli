use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::time::Duration as StdDuration;
use chrono::{Duration as ChronoDuration, Datelike};

use crate::app::{App, PopupState};
use crate::models::{DeleteTarget, MoveTarget, ViewMode};

pub fn handle_events(app: &mut App) -> Result<bool, Box<dyn std::error::Error>> {
    if !event::poll(StdDuration::from_millis(16))? {
        return Ok(false);
    }

    let Event::Key(key) = event::read()? else {
        return Ok(false);
    };

    if app.is_confirm_delete() {
        return Ok(handle_confirm_delete(app, key));
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('v') {
        if app.is_special_views_popup() {
            app.close_popup();
        } else {
            app.set_popup_state(PopupState::SpecialViews);
            app.folder_index = 0;
        }
        app.request_update();
        return Ok(false);
    }

    Ok(handle_normal_mode(app, key))
}

fn handle_normal_mode(app: &mut App, key: event::KeyEvent) -> bool {
    if app.calendar.show_day_tasks {
        return handle_calendar_day_tasks_mode(app, key);
    }

    if app.calendar.show_calendar {
        return handle_calendar_mode(app, key);
    }

    if app.is_move_popup() {
        return handle_move_popup_mode(app, key);
    }

    match key.code {
        KeyCode::Esc => handle_escape(app),
        KeyCode::Enter => {
            handle_enter(app);
            false
        }
        KeyCode::Up => {
            handle_up(app);
            false
        }
        KeyCode::Down => {
            handle_down(app);
            false
        }
        KeyCode::Right if !app.is_popup_active() => {
            app.set_popup_state(PopupState::FilePreview);
            false
        }
        KeyCode::Backspace => {
            handle_backspace(app);
            false
        }
        KeyCode::Char(c) => {
            handle_char(app, key.modifiers, c);
            false
        }
        _ => false,
    }
}

fn handle_move_popup_mode(app: &mut App, key: event::KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.close_popup();
            false
        }
        KeyCode::Enter => {
            let folder_to_move = app.available_folders.get(app.folder_index).cloned();
            if let Some(folder) = folder_to_move {
                let _ = app.move_task(&folder);
            }
            app.close_popup();
            false
        }
        KeyCode::Up => {
            if app.folder_index > 0 {
                app.folder_index -= 1;
                app.request_update();
            }
            false
        }
        KeyCode::Down => {
            if app.folder_index + 1 < app.available_folders.len() {
                app.folder_index += 1;
                app.request_update();
            }
            false
        }
        _ => false,
    }
}

fn handle_calendar_mode(app: &mut App, key: event::KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.calendar.show_calendar = false;
            app.request_update();
            false
        }
        KeyCode::Enter => {
            app.calendar.show_day_tasks = true;
            app.calendar.show_calendar = false;
            app.file_index = 0;
            app.request_update();
            false
        }
        KeyCode::Left => {
            app.calendar.selected_date -= ChronoDuration::days(1);
            if app.calendar.selected_date.month() != app.calendar.current_date.month() {
                app.calendar.current_date = app.calendar.selected_date;
            }
            app.request_update();
            false
        }
        KeyCode::Right => {
            app.calendar.selected_date += ChronoDuration::days(1);
            if app.calendar.selected_date.month() != app.calendar.current_date.month() {
                app.calendar.current_date = app.calendar.selected_date;
            }
            app.request_update();
            false
        }
        KeyCode::Up => {
            app.calendar.selected_date -= ChronoDuration::days(7);
            if app.calendar.selected_date.month() != app.calendar.current_date.month() {
                app.calendar.current_date = app.calendar.selected_date;
            }
            app.request_update();
            false
        }
        KeyCode::Down => {
            app.calendar.selected_date += ChronoDuration::days(7);
            if app.calendar.selected_date.month() != app.calendar.current_date.month() {
                app.calendar.current_date = app.calendar.selected_date;
            }
            app.request_update();
            false
        }
        _ => false,
    }
}

fn handle_calendar_day_tasks_mode(app: &mut App, key: event::KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.calendar.show_day_tasks = false;
            app.calendar.show_calendar = true;
            app.request_update();
            false
        }
        KeyCode::Up => {
            if app.file_index > 0 {
                app.file_index -= 1;
                app.request_update();
            }
            false
        }
        KeyCode::Down => {
            if app.file_index + 1 < app.calendar.day_tasks.len() {
                app.file_index += 1;
                app.request_update();
            }
            false
        }
        KeyCode::Enter => {
            if app.task_name.is_empty() && !app.calendar.day_tasks.is_empty()
                && let Some(task) = app.calendar.day_tasks.get(app.file_index) {
                    let new_checked = !task.checked;
                    let _ = crate::backend::update_task_checked(&app.config, &task.folder, &task.filename, new_checked);

                    let date_str = format!("{:02}.{:02}.{}",
                        app.calendar.selected_date.day(),
                        app.calendar.selected_date.month(),
                        app.calendar.selected_date.year()
                    );
                    app.calendar.day_tasks = app.get_tasks_for_date(&date_str);
                    app.sort_tasks_for_calendar();
                    app.request_update();
                }
            false
        }
        KeyCode::Char(' ') => {
            if let Some(task) = app.calendar.day_tasks.get(app.file_index) {
                let new_checked = !task.checked;
                let _ = crate::backend::update_task_checked(&app.config, &task.folder, &task.filename, new_checked);

                let date_str = format!("{:02}.{:02}.{}",
                    app.calendar.selected_date.day(),
                    app.calendar.selected_date.month(),
                    app.calendar.selected_date.year()
                );
                app.calendar.day_tasks = app.get_tasks_for_date(&date_str);
                app.sort_tasks_for_calendar();
                app.request_update();
            }
            false
        }
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if !app.calendar.day_tasks.is_empty() {
                app.set_popup_state(PopupState::ConfirmDelete);
                app.delete_target = Some(crate::models::DeleteTarget::Task);
                app.request_update();
            }
            false
        }
        _ => false,
    }
}

fn handle_up(app: &mut App) {
    if app.calendar.show_day_tasks {
        if app.file_index > 0 {
            app.file_index -= 1;
            app.request_update();
        }
    }
    else if app.is_folder_list_popup() || app.is_special_views_popup() || app.is_move_popup() && app.folder_index > 0 {
        app.folder_index -= 1;
        app.request_update();
    }
    else if app.file_index > 0 {
        app.file_index -= 1;
        app.request_update();
    }
}

fn handle_down(app: &mut App) {
    if app.calendar.show_day_tasks {
        if app.file_index + 1 < app.calendar.day_tasks.len() {
            app.file_index += 1;
            app.request_update();
        }
    } else if app.is_folder_list_popup() {
        if app.folder_index + 1 < app.all_folders.len() {
            app.folder_index += 1;
            app.request_update();
        }
    } else if app.is_special_views_popup() {
        if app.folder_index + 1 < app.special_views.len() {
            app.folder_index += 1;
            app.request_update();
        }
    } else if app.is_move_popup() {
        if app.folder_index + 1 < app.available_folders.len() {
            app.folder_index += 1;
            app.request_update();
        }
    } else if app.file_index + 1 < app.tasks.len() {
        app.file_index += 1;
        app.request_update();
    }
}

fn handle_backspace(app: &mut App) {
    if app.is_folder_list_popup() {
        app.folder_name.pop();
        app.request_update();
    } else if !app.is_special_views_popup() && matches!(app.view_mode, ViewMode::Normal) {
        app.task_name.pop();
        app.request_update();
    }
}

fn handle_char(
    app: &mut App,
    modifiers: KeyModifiers,
    c: char,
) {
    if modifiers.contains(KeyModifiers::CONTROL) && c == 'c' {
        if !app.tasks.is_empty() && matches!(app.view_mode, ViewMode::Normal) {
            app.set_popup_state(PopupState::MoveTask);
            app.move_target = Some(MoveTarget::Task);
            app.folder_index = 0;
        }
        return;
    }

    if modifiers.contains(KeyModifiers::CONTROL) && c == 'f' {
        if !app.tasks.is_empty() {
            let _ = app.open_file_in_editor();
        }
        return;
    }

    if modifiers.contains(KeyModifiers::CONTROL) && c == 'd' {
        if !app.is_confirm_delete() {
            app.set_popup_state(PopupState::ConfirmDelete);
            app.delete_target = if app.is_folder_list_popup() {
                Some(crate::models::DeleteTarget::Folder)
            } else {
                Some(crate::models::DeleteTarget::Task)
            };
        }
        return;
    }

    if modifiers.contains(KeyModifiers::CONTROL) && c == 'l' {
        if app.is_folder_list_popup() {
            app.close_popup();
        } else {
            app.set_popup_state(PopupState::FolderList);
            app.folders = app.all_folders.clone();
            app.folder_index = 0;
        }
        app.request_update();
        return;
    }

    if app.is_special_views_popup() {
        return;
    }

    if app.is_folder_list_popup() {
        app.folder_name.push(c);
        app.request_update();
    } else if matches!(app.view_mode, ViewMode::Normal) {
        app.task_name.push(c);
        app.request_update();
    }
}

fn handle_escape(app: &mut App) -> bool {
    if app.is_file_preview() || app.is_move_popup() || app.is_folder_list_popup() || app.is_special_views_popup() {
        app.close_popup();
        false
    }
    else if !matches!(app.view_mode, ViewMode::Normal) {
        app.view_mode = ViewMode::Normal;
        app.file_index = 0;
        app.request_update();
        false
    } else {
        true
    }
}

fn handle_enter(app: &mut App) {
    use chrono::Local;
    use std::fs;
    use std::path::Path;

    if app.is_special_views_popup() {
        if let Some(view) = app.special_views.get(app.folder_index) {
            match view.as_str() {
                "Today" => {
                    app.view_mode = ViewMode::Today;
                    app.calendar.show_day_tasks = false;
                }
                "Next 7 days" => {
                    app.view_mode = ViewMode::NextSevenDays;
                    app.calendar.show_day_tasks = false;
                }
                "Calendar" => {
                    app.calendar.show_calendar = true;
                    app.calendar.show_day_tasks = false;
                    app.calendar.current_date = Local::now().date_naive();
                    app.calendar.selected_date = Local::now().date_naive();
                }
                _ => {}
            }
            app.close_popup();
            app.file_index = 0;
            app.request_update();
        }
    } else if app.is_folder_list_popup() {
        if !app.folder_name.is_empty() {
            let new_folder = app.folder_name.trim().to_string();
            if !new_folder.is_empty() {
                let folder_path = app.config.get_folder_path(&new_folder);
                if !Path::new(&folder_path).exists() {
                    let _ = fs::create_dir_all(&folder_path);
                }
                app.folder_name.clear();
                app.request_update();
            }
        } else if let Some(folder) = app.all_folders.get(app.folder_index) {
            app.selected_folder = Some(folder.clone());
            app.view_mode = ViewMode::Normal;
            app.close_popup();
            app.request_update();
        }
    } else if app.calendar.show_calendar && !app.calendar.show_day_tasks {
        let date_str = format!("{:02}.{:02}.{}",
            app.calendar.selected_date.day(),
            app.calendar.selected_date.month(),
            app.calendar.selected_date.year()
        );

        app.calendar.show_day_tasks = true;
        app.calendar.show_calendar = false;
        app.tasks = app.get_tasks_for_date(&date_str);
        app.file_index = 0;
        app.request_update();
    } else if app.task_name.is_empty() {
        if !app.tasks.is_empty() {
            app.toggle_selected_task();
        }
    } else if matches!(app.view_mode, ViewMode::Normal) {
        app.create_task();
        app.task_name.clear();
        app.request_update();
    }
}

fn handle_confirm_delete(app: &mut App, key: event::KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('y') => {
            if app.calendar.show_day_tasks {
                app.delete_selected_task_from_calendar();
            } else {
                match app.delete_target {
                    Some(DeleteTarget::Task) => app.delete_selected_task(),
                    Some(DeleteTarget::Folder) => app.delete_selected_folder(),
                    None => {}
                }
            }
            app.close_popup();
            false
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            app.close_popup();
            false
        }
        _ => false,
    }
}
