use icalendar::{Calendar, Component, Event, EventLike};

use crate::models::FirstCommit;

pub fn calendar_from(username: &str, commits: &[FirstCommit]) -> Calendar {
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
