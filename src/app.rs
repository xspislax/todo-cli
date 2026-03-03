use ratatui::widgets::{ListState, TableState};
use chrono::{Local, Duration, NaiveDate, Datelike};
use std::fs;

use crate::models::{ViewMode, CalendarState, Task, DeleteTarget, MoveTarget};
use crate::config::Config;
use crate::backend;

pub struct App {
    pub task_name: String,
    pub tasks: Vec<Task>,
    pub folder_name: String,
    pub all_folders: Vec<String>,
    pub available_folders: Vec<String>,
    pub folders: Vec<String>,
    pub selected_folder: Option<String>,
    pub folder_index: usize,
    pub show_popup: bool,
    pub show_special_views_popup: bool,
    pub show_file_preview: bool,
    pub show_move_popup: bool,
    pub folder_state: ListState,
    pub file_index: usize,
    pub table_state: TableState,
    pub file_content: String,
    pub confirm_delete: bool,
    pub delete_target: Option<DeleteTarget>,
    pub move_target: Option<MoveTarget>,
    pub view_mode: ViewMode,
    pub special_views: Vec<String>,
    pub calendar: CalendarState,
    pub config: Config,
    pub needs_update: bool,
}

impl App {
    pub fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let mut app = Self {
            task_name: String::new(),
            tasks: Vec::new(),
            folder_name: String::new(),
            all_folders: Vec::new(),
            available_folders: Vec::new(),
            folders: Vec::new(),
            selected_folder: Some(config.features.default_folder.clone()),
            folder_index: 0,
            show_popup: false,
            show_special_views_popup: false,
            show_file_preview: false,
            show_move_popup: false,
            folder_state: ListState::default(),
            file_index: 0,
            table_state: TableState::default(),
            file_content: String::new(),
            confirm_delete: false,
            delete_target: None,
            move_target: None,
            view_mode: ViewMode::Normal,
            special_views: vec!["Today".to_string(), "Next 7 days".to_string(), "Calendar".to_string()],
            calendar: CalendarState::new(),
            config,
            needs_update: true,
        };

        app.update();
        Ok(app)
    }

    pub fn update(&mut self) {
        if !self.needs_update {
            return;
        }

        self.folder_state.select(Some(self.folder_index));
        self.table_state.select(Some(self.file_index));

        if let Ok((_, folders)) = backend::read_base_dir(&self.config) {
            self.all_folders = folders.clone();

            if self.show_popup {
                self.folders = self.all_folders.clone();
            }

            if let Some(current) = &self.selected_folder {
                self.available_folders = folders
                    .into_iter()
                    .filter(|f| f != current)
                    .collect();
            } else {
                self.available_folders = folders;
            }
        }

        match self.view_mode {
            ViewMode::Normal => {
                if let Some(folder) = &self.selected_folder
                    && let Ok(files) = backend::read_folder_files(&self.config, folder) {
                        self.tasks = files
                            .iter()
                            .filter_map(|f| backend::parse_task_file(&self.config, folder, f).ok())
                            .collect();
                    }
                self.sort_tasks();
            }
            ViewMode::Today => {
                self.tasks = self.get_today_tasks();
                self.sort_tasks();
            }
            ViewMode::NextSevenDays => {
                self.tasks = self.get_next_seven_days_tasks();
                self.sort_tasks();
            }
        }

        self.update_calendar_tasks();

        if self.calendar.show_day_tasks {
            let date_str = format!("{:02}.{:02}.{}",
                self.calendar.selected_date.day(),
                self.calendar.selected_date.month(),
                self.calendar.selected_date.year()
            );
            self.calendar.day_tasks = self.get_tasks_for_date(&date_str);
            self.sort_tasks_for_calendar();
        }

        if let Some(task) = self.tasks.get(self.file_index) {
            self.file_content = backend::read_file_content(&self.config, &task.folder, &task.filename).unwrap_or_default();
        }

        self.needs_update = false;
    }

    pub fn request_update(&mut self) {
        self.needs_update = true;
    }

    fn sort_tasks(&mut self) {
        self.tasks.sort_by(|a, b| {
            if a.checked == b.checked {
                if !a.date.is_empty() && !b.date.is_empty()
                    && let (Ok(date_a), Ok(date_b)) = (
                        NaiveDate::parse_from_str(&a.date, "%d.%m.%Y"),
                        NaiveDate::parse_from_str(&b.date, "%d.%m.%Y")
                    )
                {
                    return date_a.cmp(&date_b);
                }
                std::cmp::Ordering::Equal
            } else if !a.checked && b.checked {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });
    }

    pub fn sort_tasks_for_calendar(&mut self) {
        self.calendar.day_tasks.sort_by(|a, b| {
            if a.checked == b.checked {
                if !a.date.is_empty() && !b.date.is_empty()
                    && let (Ok(date_a), Ok(date_b)) = (
                        NaiveDate::parse_from_str(&a.date, "%d.%m.%Y"),
                        NaiveDate::parse_from_str(&b.date, "%d.%m.%Y")
                    )
                {
                    return date_a.cmp(&date_b);
                }
            std::cmp::Ordering::Equal
            } else if !a.checked && b.checked {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });
    }

    fn update_calendar_tasks(&mut self) {
        let mut dates_with_tasks = std::collections::HashSet::new();

        if let Ok((_, folders)) = backend::read_base_dir(&self.config) {
            for folder in folders {
                if let Ok(files) = backend::read_folder_files(&self.config, &folder) {
                    for file in files {
                        if let Ok(task) = backend::parse_task_file(&self.config, &folder, &file)
                            && !task.date.is_empty() {
                                dates_with_tasks.insert(task.date);
                            }
                    }
                }
            }
        }

        self.calendar.tasks_by_date = dates_with_tasks;
    }

    pub fn get_tasks_for_date(&self, date_str: &str) -> Vec<Task> {
        let mut tasks = Vec::new();

        if let Ok((_, folders)) = backend::read_base_dir(&self.config) {
            for folder in folders {
                if let Ok(files) = backend::read_folder_files(&self.config, &folder) {
                    for file in files {
                        if let Ok(task) = backend::parse_task_file(&self.config, &folder, &file)
                        && task.date == date_str
                        {
                            tasks.push(task);
                        }
                    }
                }
            }
        }

        tasks
    }

    fn get_today_tasks(&self) -> Vec<Task> {
        let today = Local::now().date_naive();
        let today_str = format!("{:02}.{:02}.{}", today.day(), today.month(), today.year());
        self.get_tasks_for_date(&today_str)
    }

    fn get_next_seven_days_tasks(&self) -> Vec<Task> {
        let mut tasks = Vec::new();
        let today = Local::now().date_naive();
        let seven_days_later = today + Duration::days(7);

        if let Ok((_, folders)) = backend::read_base_dir(&self.config) {
            for folder in folders {
                if let Ok(files) = backend::read_folder_files(&self.config, &folder) {
                    for file in files {
                        if let Ok(task) = backend::parse_task_file(&self.config, &folder, &file)
                            && let Ok(task_date) = NaiveDate::parse_from_str(&task.date, "%d.%m.%Y")
                            && task_date >= today
                            && task_date <= seven_days_later
                        {
                            tasks.push(task);
                        }
                    }
                }
            }
        }

        tasks
    }

    pub fn toggle_selected_task(&mut self) {
        if let Some(task) = self.tasks.get(self.file_index) {
            let new_checked = !task.checked;
            let _ = backend::update_task_checked(&self.config, &task.folder, &task.filename, new_checked);

            if let Ok(files) = backend::read_folder_files(&self.config, &task.folder) {
                self.tasks = files
                    .iter()
                    .filter_map(|f| backend::parse_task_file(&self.config, &task.folder, f).ok())
                    .collect();
            }
            self.sort_tasks();
            self.request_update();
        }
    }

    pub fn create_task(&mut self) {
        if let Some(folder) = &self.selected_folder {
            let (title, content, priority, date) = backend::parse_task_input(&self.task_name);
            if !title.is_empty() {
                let _ = backend::zapisz_zadanie(
                    &self.config,
                    &title,
                    folder,
                    &content,
                    priority,
                    date.as_deref(),
                );

                if let Ok(files) = backend::read_folder_files(&self.config, folder) {
                    self.tasks = files
                        .iter()
                        .filter_map(|f| backend::parse_task_file(&self.config, folder, f).ok())
                        .collect();
                }
                self.sort_tasks();
                self.request_update();
            }
        }
    }

    pub fn open_file_in_editor(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(task) = self.tasks.get(self.file_index) {
            let filepath = self.config.get_full_path(&task.folder, &task.filename);

            #[cfg(target_os = "windows")]
            {
                use std::process::Command;
                Command::new("cmd")
                    .args(&["/c", "start"])
                    .arg(&filepath)
                    .spawn()?;
            }

            #[cfg(target_os = "macos")]
            {
                use std::process::Command;
                Command::new("open")
                    .arg(&filepath)
                    .spawn()?;
            }

            #[cfg(target_os = "linux")]
            {
                use std::process::Command;
                Command::new("xdg-open")
                    .arg(&filepath)
                    .spawn()?;
            }
        }
        Ok(())
    }

    pub fn move_task(&mut self, target_folder: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(task) = self.tasks.get(self.file_index) {
            let source_path = self.config.get_full_path(&task.folder, &task.filename);
            let target_path = self.config.get_full_path(target_folder, &task.filename);

            fs::copy(&source_path, &target_path)?;
            fs::remove_file(&source_path)?;

            self.file_index = self.file_index.saturating_sub(1);
            self.request_update();
        }
        Ok(())
    }

    pub fn delete_selected_task(&mut self) {
        if let Some(task) = self.tasks.get(self.file_index) {
            let _ = backend::delete_task(&self.config, &task.folder, &task.filename);
            self.file_index = self.file_index.saturating_sub(1);
            self.request_update();
        }
    }
    pub fn delete_selected_task_from_calendar(&mut self) {
        if let Some(task) = self.calendar.day_tasks.get(self.file_index) {
            let _ = backend::delete_task(&self.config, &task.folder, &task.filename);

            let date_str = format!("{:02}.{:02}.{}",
                self.calendar.selected_date.day(),
                self.calendar.selected_date.month(),
                self.calendar.selected_date.year()
            );
            self.calendar.day_tasks = self.get_tasks_for_date(&date_str);
            self.sort_tasks_for_calendar();

            if self.file_index >= self.calendar.day_tasks.len() {
                self.file_index = self.file_index.saturating_sub(1);
            }

            self.request_update();
        }
    }

    pub fn delete_selected_folder(&mut self) {
        if let Some(folder) = self.folders.get(self.folder_index)
            && folder != &self.config.features.default_folder
            {
                let _ = backend::delete_folder(
                    &self.config,
                    folder,
                    &self.config.features.default_folder,
                );

                self.selected_folder = Some(self.config.features.default_folder.clone());
                self.folder_index = 0;
                self.request_update();
        }
    }
}

pub fn truncate(text: &str, max: usize) -> String {
    if text.len() > max {
        format!("{}...", &text[..max])
    } else {
        text.to_string()
    }
}
