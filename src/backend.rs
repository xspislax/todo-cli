use std::fs;
use std::io;
use std::path::Path;
use crate::models::{Task, Priority};
use crate::config::Config;
use std::fmt::Write;

pub fn read_base_dir(config: &Config) -> io::Result<(Vec<String>, Vec<String>)> {
    let data_path = Path::new(&config.features.data_path);

    if !data_path.exists() {
        fs::create_dir_all(data_path)?;
        return Ok((Vec::new(), Vec::new()));
    }

    let mut files = Vec::new();
    let mut folders = Vec::new();

    for entry in fs::read_dir(data_path)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        let name = entry.file_name()
            .to_string_lossy()
            .to_string();

        if meta.is_dir() {
            folders.push(name);
        } else {
            files.push(name);
        }
    }

    Ok((files, folders))
}

pub fn read_folder_files(config: &Config, folder: &str) -> io::Result<Vec<String>> {
    let mut files = Vec::new();
    let path_str = config.get_folder_path(folder);
    let path = Path::new(&path_str);

    if path.exists() && path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.metadata()?.is_file() {
                files.push(
                    entry.file_name()
                        .to_string_lossy()
                        .to_string()
                );
            }
        }
    }

    Ok(files)
}

pub fn read_file_content(config: &Config, folder: &str, file: &str) -> io::Result<String> {
    let path_str = config.get_full_path(folder, file);
    let path = Path::new(&path_str);
    fs::read_to_string(path)
}

pub fn zapisz_zadanie(
    config: &Config,
    task_title: &str,
    folder: &str,
    filedata: &str,
    priority: Option<Priority>,
    date: Option<&str>
) -> std::io::Result<()> {
    let filename = task_title.replace(' ', "_");
    let filepath_str = config.get_full_path(folder, &format!("{filename}.md"));
    let filepath = Path::new(&filepath_str);

    if let Some(parent) = filepath.parent() && !parent.exists() {
            fs::create_dir_all(parent)?;
    }

    let priority_text = match priority {
        Some(Priority::High) => "high",
        Some(Priority::Medium) => "medium",
        Some(Priority::Low) => "low",
        None => "None",
    };

    let mut content = String::new();
    write!(content, "# {task_title}\n\n").unwrap();
    write!(content, "Checked: false\n\n").unwrap();

    if let Some(d) = date {
        write!(content, "Date: {d}\n\n").unwrap();
    }

    if !filedata.is_empty() {
        write!(content, "Description: {filedata}\n\n").unwrap();
    }

    writeln!(content, "Priority: {priority_text}\n").unwrap();

    fs::write(filepath, content)?;
    Ok(())
}

pub fn update_task_checked(config: &Config, folder: &str, filename: &str, checked: bool) -> io::Result<()> {
    let filepath_str = config.get_full_path(folder, filename);
    let filepath = Path::new(&filepath_str);

    let content = fs::read_to_string(filepath)?;

    let mut new_content = String::new();
    let mut checked_line_updated = false;

    for line in content.lines() {
        if line.starts_with("Checked:") {
            writeln!(new_content, "Checked: {checked}").unwrap();
            checked_line_updated = true;
        } else {
            new_content.push_str(line);
            new_content.push('\n');
        }
    }

    if !checked_line_updated {
        new_content = format!("Checked: {checked}\n{content}");
    }

    fs::write(filepath, new_content)?;
    Ok(())
}

pub fn delete_task(config: &Config, folder: &str, filename: &str) -> io::Result<()> {
    let path_str = config.get_full_path(folder, filename);
    let path = Path::new(&path_str);
    fs::remove_file(path)?;
    Ok(())
}

pub fn delete_folder(config: &Config, folder: &str, default_folder: &str) -> io::Result<()> {
    if folder == default_folder {
        return Ok(())
    }
    let path_str = config.get_folder_path(folder);
    let path = Path::new(&path_str);
    fs::remove_dir_all(path)?;
    Ok(())
}

pub fn parse_task_file(config: &Config, folder: &str, file: &str) -> io::Result<Task> {
    let content = read_file_content(config, folder, file)?;

    let mut title = String::new();
    let mut date = String::new();
    let mut priority = String::new();
    let mut description = String::new();
    let mut checked = false;

    for line in content.lines() {
        if line.starts_with("# ") {
            title = line.replace("# ", "");
        } else if line.starts_with("Checked:") {
            let value = line.replace("Checked:", "").trim().to_lowercase();
            checked = value == "true";
        } else if line.starts_with("Date:") {
            date = line.replace("Date:", "").trim().to_string();
        } else if line.starts_with("Description:") {
            description = line.replace("Description:", "").trim().to_string();
        } else if line.starts_with("Priority:") {
            priority = line.replace("Priority:", "").trim().to_string();
        }
    }

    Ok(Task {
        title,
        date,
        priority,
        description,
        filename: file.to_string(),
        folder: folder.to_string(),
        checked,
    })
}

pub fn parse_task_input(input: &str) -> (String, String, Option<Priority>, Option<String>) {
    use crate::models::parse_date_token;

    let mut task_date: Option<String> = None;
    let mut priority: Option<Priority> = None;

    let mut date_processed = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '@' {
            let mut token = String::new();
            while let Some(&next) = chars.peek() {
                if next.is_whitespace() {
                    break;
                }
                token.push(next);
                chars.next();
            }

            if let Some(parsed) = parse_date_token(&token) {
                task_date = Some(parsed);
            }
        } else {
            date_processed.push(c);
        }
    }

    let mut priority_processed = date_processed;

    let priority_flags = [
        ("!high", Priority::High),
        ("!h", Priority::High),
        ("!medium", Priority::Medium),
        ("!m", Priority::Medium),
        ("!low", Priority::Low),
        ("!l", Priority::Low),
    ];

    for (flag, prio) in &priority_flags {
        if priority_processed.contains(flag) {
            priority = Some(*prio);
            priority_processed = priority_processed.replace(flag, "");
            break;
        }
    }

    let cleaned = priority_processed
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    let (title, description) = if let Some(pos) = cleaned.find('.') {
        let (t, d) = cleaned.split_at(pos);
        (t.trim().to_string(), d[1..].trim().to_string())
    } else {
        (cleaned, String::new())
    };

    (
        title,
        description,
        priority,
        task_date,
    )
}
