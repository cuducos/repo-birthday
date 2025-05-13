use anyhow::Result;
use icalendar::{Calendar, Component, Event, EventLike};

use crate::models::FirstCommit;

pub fn calendar_from(username: &str, commits: &mut Vec<FirstCommit>) -> Calendar {
    let mut calendar = Calendar::new();
    let calendar =
        calendar.name(format!("{}'s  GitHub repository anniversaries", username).as_str());
    for commit in commits {
        let age = commit.age();
        let pluralized = if age == 1 { "year" } else { "years" };
        let title = format!(
            "🎂 {}/{} ({} {} old)",
            commit.owner, commit.name, age, pluralized
        );
        let event = Event::new()
            .all_day(commit.next_anniversary())
            .summary(title.as_str())
            .done();
        calendar.push(event);
    }
    calendar.done()
}

pub fn to_ical(username: &str, calendar: Calendar) -> Result<String> {
    let filename = format!("{}.ical", username);
    if std::path::Path::new(&filename).exists() {
        panic!("File {} already exists", filename);
    }
    std::fs::write(&filename, calendar.to_string())?;
    Ok(filename)
}
