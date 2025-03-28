use super::events::{self, NotificationEvent};
use anyhow::{Context, Result};
use chrono::{DateTime, NaiveTime, TimeZone, Utc};
use icalendar::{Calendar, CalendarComponent, CalendarDateTime, Component, DatePerhapsTime};
use std::{
    cmp::Reverse,
    io::{Error, ErrorKind},
    str::FromStr,
};

pub async fn get_events_from_file(url: &str) -> Result<Vec<events::NotificationEvent>> {
    let mut result: Vec<events::NotificationEvent> = Vec::new();

    let response = reqwest::get(url).await.map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("couldn't get calendar data: {}", e),
        )
    })?;

    let text = response.text().await.map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("couldn't read response body: {}", e),
        )
    })?;

    let parsed_calendar: Calendar = text.parse().unwrap();

    for component in &parsed_calendar.components {
        if let CalendarComponent::Event(event) = component {
            let dt_start: icalendar::DatePerhapsTime = event
                .get_start()
                .context("couldnt get event")
                .map_err(|e| {
                    Error::new(
                        ErrorKind::Other,
                        format!("couldn't get calendar data: {}", e),
                    )
                })?;

            let dt: DateTime<Utc> = match dt_start {
                DatePerhapsTime::Date(date) => {
                    let naive_datetime = date.and_time(NaiveTime::MIN);
                    DateTime::<Utc>::from_naive_utc_and_offset(naive_datetime, Utc)
                }

                DatePerhapsTime::DateTime(date) => match date {
                    CalendarDateTime::Floating(naive_dt) => {
                        // Floating-Zeiten haben keine Zeitzone. Annahme: UTC verwenden.
                        Utc.from_utc_datetime(&naive_dt)
                    }
                    CalendarDateTime::Utc(utc_dt) => {
                        // Bereits in UTC
                        utc_dt
                    }
                    CalendarDateTime::WithTimezone { date_time, tzid } => {
                        let new_dt = date_time.and_utc();
                        match chrono_tz::Tz::from_str(&tzid) {
                            Ok(tz) => tz.from_utc_datetime(&date_time).to_utc(),
                            Err(_) => new_dt,
                        }
                    }
                },
            };

            let event_title = match event.get_summary() {
                Some(dt) => dt.to_string(),
                None => "Untitled".to_string(),
            };

            let new_event = events::NotificationEvent {
                summary: event_title,
                date: dt,
            };
            result.push(new_event);
        }
    }
    Ok(result)
}

fn filter_events<F>(
    all_events: Vec<events::NotificationEvent>,
    predicate: F,
) -> Result<Vec<events::NotificationEvent>>
where
    F: Fn(&events::NotificationEvent) -> bool,
{
    Ok(all_events.into_iter().filter(predicate).collect())
}

// pub fn get_today_events(
//     all_events: Vec<events::NotificationEvent>,
// ) -> Result<Vec<events::NotificationEvent>> {
//     let today = Utc::now().naive_utc().date();
//     filter_events(all_events, |event| event.date.naive_utc().date() == today)
// }

pub fn get_after_now_events(
    all_events: Vec<events::NotificationEvent>,
) -> Result<Vec<events::NotificationEvent>> {
    let today = Utc::now().naive_utc().date();
    let now = Utc::now();
    filter_events(all_events, |event| {
        event.date.naive_utc().date() == today && event.date > now
    })
}

pub async fn get_next_events(
    amount: i8,
    all_events: Vec<NotificationEvent>,
) -> Result<Vec<events::NotificationEvent>> {
    let mut todays_events = get_after_now_events(all_events).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("couldn't get calendar data: {}", e),
        )
    })?;


    todays_events.sort_by_key(|event| Reverse(event.date));


    let mut results: Vec<events::NotificationEvent> = Vec::new();

    for _ in 0..amount {
        match todays_events.pop() {
            Some(event) => {
                results.push(event);
            }
            None => {
                break;
            }
        }
    }

    Ok(results)
}
