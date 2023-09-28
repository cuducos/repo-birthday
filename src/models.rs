use chrono::prelude::*;
use chrono::{NaiveDate, NaiveDateTime};

pub struct FirstCommit {
    pub message: String,
    pub date: NaiveDateTime,
    pub name: String,
    pub owner: String,
}

impl std::fmt::Display for FirstCommit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            self.date,
            self.message.lines().next().unwrap_or(""),
        )
    }
}

fn change_year(original: NaiveDate, year: i32) -> NaiveDate {
    if original.year() == year {
        return original;
    }

    let mut month = original.month();
    let mut day = original.day();
    if month == 2 && day == 29 {
        let is_leap = (year % 4 == 0) && (year % 100 != 0 || year % 400 == 0);
        if !is_leap {
            month = 3;
            day = 1;
        }
    }

    match NaiveDate::from_ymd_opt(year, month, day) {
        Some(date) => date,
        None => panic!("Error moving date {original} to {year}"),
    }
}

impl FirstCommit {
    pub fn next_aniversary(&self) -> NaiveDate {
        let today = Local::now().date_naive();
        let date = self.date.date();
        if today == date {
            return today;
        }

        let mut result = change_year(date, today.year());
        if result < today {
            result = change_year(date, today.year() + 1);
        }

        result
    }

    pub fn days_to_next_aniversary(&self) -> i64 {
        (self.next_aniversary() - Local::now().date_naive()).num_days()
    }

    pub fn age(&self) -> i32 {
        self.next_aniversary().year() - self.date.year()
    }
}
