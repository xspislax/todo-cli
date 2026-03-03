use chrono::{Local, Duration, NaiveDate, Weekday, Datelike};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Priority {
    High,
    Medium,
    Low,
}

#[derive(Clone, PartialEq)]
pub enum ViewMode {
    Normal,
    Today,
    NextSevenDays,
    WithoutDate,
}

#[derive(Clone)]
pub enum DeleteTarget {
    Task,
    Folder,
}

#[derive(Clone)]
pub enum MoveTarget {
    Task,
}

#[derive(Clone)]
pub struct Task {
    pub title: String,
    pub date: String,
    pub priority: String,
    pub description: String,
    pub filename: String,
    pub folder: String,
    pub checked: bool,
}

#[derive(Clone)]
pub struct CalendarState {
    pub current_date: NaiveDate,
    pub selected_date: NaiveDate,
    pub tasks_by_date: HashSet<String>,
    pub show_calendar: bool,
    pub show_day_tasks: bool,
    pub day_tasks: Vec<Task>,
}

impl CalendarState {
    pub fn new() -> Self {
        let today = Local::now().date_naive();
        Self {
            current_date: today,
            selected_date: today,
            tasks_by_date: HashSet::new(),
            show_calendar: false,
            show_day_tasks: false,
            day_tasks: Vec::new(),
        }
    }
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 31,
    }
}

pub fn parse_date_token(token: &str) -> Option<String> {
    let today = Local::now().date_naive();
    let cleaned = token.trim().to_lowercase();

    if let Ok(days) = cleaned.parse::<i64>() {
        let target = today + Duration::days(days);
        return Some(format!("{:02}.{:02}.{}", target.day(), target.month(), target.year()));
    }

    if let Ok(date) = NaiveDate::parse_from_str(&cleaned, "%d.%m.%Y") {
        return Some(format!("{:02}.{:02}.{}", date.day(), date.month(), date.year()));
    }

    if cleaned == "tomorrow" {
        let target = today + Duration::days(1);
        return Some(format!("{:02}.{:02}.{}", target.day(), target.month(), target.year()));
    }

    if cleaned == "today" {
        let target = today;
        return Some(format!("{:02}.{:02}.{}", target.day(), target.month(), target.year()));
    }

    let weekday = match cleaned.as_str() {
        "monday" => Some(Weekday::Mon),
        "tuesday" => Some(Weekday::Tue),
        "wednesday" => Some(Weekday::Wed),
        "thursday" => Some(Weekday::Thu),
        "friday" => Some(Weekday::Fri),
        "saturday" => Some(Weekday::Sat),
        "sunday" => Some(Weekday::Sun),
        _ => None,
    };

    if let Some(target_weekday) = weekday {
        let mut date = today;
        while date.weekday() != target_weekday {
            date += Duration::days(1);
        }
        if date == today {
            date += Duration::days(7);
        }
        return Some(format!("{:02}.{:02}.{}", date.day(), date.month(), date.year()));
    }

    None
}
