use super::command::{Command, CommandData};
use crate::{BackendError, BoxedError};
use serde::Deserialize;
use serde_json::from_str;
use std::collections::HashMap;

pub fn strip_prefix<'a>(str: &'a str, prefix: &str) -> &'a str {
    if !str.starts_with(prefix) {
        &str[..]
    } else {
        &str[prefix.len()..str.len()]
    }
}

pub fn load_file(path: &str) -> Result<String, BackendError> {
    std::fs::read_to_string(path).map_err(|e| {
        BackendError::from(format!("Failed to read the lua file at `{}`: {}.", path, e))
    })
}

pub fn parse_json<'a, R>(json: &'a str) -> R
where
    R: Deserialize<'a>,
{
    match from_str(&json) {
        Ok(json) => json,
        Err(e) => {
            panic!("Failed to read \"commands.json\": {}", e);
        }
    }
}

pub fn duration_format(duration: chrono::Duration) -> String {
    let mut output = String::from("");

    let days = duration.num_days();
    if days > 0 {
        output += &format!("{} days ", days);
    }
    let hours = duration.num_hours();
    if hours > 0 {
        output += &format!("{} hours ", hours - days * 24);
    }
    let minutes = duration.num_minutes();
    if minutes > 0 && days <= 0 {
        output += &format!("{} minutes ", minutes - hours * 60);
    }
    let seconds = duration.num_seconds();
    if seconds > 0 && hours <= 0 {
        output += &format!("{} seconds", seconds - minutes * 60);
    }

    output
}

pub fn find_command<'a, 'lua>(
    commands: &HashMap<String, Command<'lua>>,
    name: &'a str,
) -> Option<(CommandData<'lua>, Option<Vec<&'a str>>)> {
    let tokens = name.split_whitespace().collect::<Vec<&str>>();
    let mut next_commands = commands;

    for i in 0..tokens.len() {
        if let Some(command) = next_commands.get(tokens[i]) {
            let commands = command.commands.as_ref();

            let next = if i + 1 < tokens.len() {
                Some(tokens[i + 1])
            } else {
                None
            };

            if next.is_some() && commands.is_some() && commands.unwrap().contains_key(next.unwrap())
            {
                next_commands = commands.unwrap();
                continue;
            }

            let data = command.data.as_ref();
            if data.is_some() {
                let mut args: Option<Vec<&str>> = None;
                if tokens.len() - i > 0 {
                    let (_, right) = tokens.split_at(i + 1);
                    if right.len() > 0 {
                        args = Some(right.to_vec())
                    }
                }
                return Some((data.cloned().unwrap(), args));
            } else {
                return None;
            }
        }
    }

    None
}

pub fn reload_command<'a, 'b, 'lua, F>(
    commands: &mut HashMap<String, Command<'lua>>,
    name: &'a str,
    reloader: F,
) -> Result<(), BoxedError>
where
    F: Fn(&mut CommandData<'lua>) -> Result<(), BoxedError>,
{
    let tokens = name.split_whitespace().collect::<Vec<&str>>();
    let mut next_commands = commands;
    let mut i = 0;

    while let Some(command) = next_commands.get_mut(tokens[i]) {
        let commands = command.commands.as_mut();

        if i + 1 < tokens.len()
            && commands
                .as_ref()
                .map(|c| c.contains_key(tokens[i + 1]))
                .unwrap_or(false)
        {
            next_commands = commands.unwrap();
            i += 1;
            continue;
        }

        if let Some(data) = command.data.as_mut() {
            log::info!("Reloading `{}` [path = {}]", name, data.path);
            return reloader(data);
        }

        break;
    }

    Err(BoxedError::from(format!(
        "Command `{}` wasn't found or isn't scripted",
        name
    )))
}
