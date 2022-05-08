use anyhow::{Context, Result};
use chrono::{DateTime, Local, NaiveDate};
use reqwest::{
    header::{
        ACCEPT, ACCEPT_LANGUAGE, CONNECTION, CONTENT_TYPE, COOKIE, DNT, HOST, ORIGIN, REFERER, TE,
        USER_AGENT,
    },
    Client,
};
use rocket::form::validate::Contains;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct RawEvent {
    #[serde(rename = "isPostponed")]
    #[serde(deserialize_with = "deserialize_string_from_number")]
    pub postponed: String,
    #[serde(rename = "Month")]
    pub month: String,
    #[serde(rename = "Year")]
    pub year: String,
    #[serde(rename = "Day")]
    pub day: String,
    #[serde(rename = "thePlace")]
    pub location: String,
    #[serde(rename = "eventType")]
    pub event_type: String,
    #[serde(rename = "theOpponentString")]
    pub opponent: String,
    #[serde(rename = "isCancelled")]
    #[serde(deserialize_with = "deserialize_string_from_number")]
    pub cancelled: String,
    #[serde(rename = "theTitle")]
    pub name: String,
    #[serde(rename = "homeOrAway")]
    #[serde(deserialize_with = "deserialize_string_from_number")]
    pub home: String,
    #[serde(rename = "theTime")]
    pub time: String,
    #[serde(rename = "rescheddate")]
    pub rescheduled_date: String,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Event {
    pub name: String,
    pub time: DateTime<Local>,
    pub sport: String,
    pub varsity: bool,
    pub opponent: String,
    pub location: String,
    pub rescheduled: bool,
    pub rescheduled_date: Option<NaiveDate>,
    pub cancelled: bool,
}

impl RawEvent {
    pub async fn fetch_this_weeks() -> Result<Vec<RawEvent>> {
        let client = Client::new();
        let resp = client
        .post("https://goffstownathletics.com/main/calendarWeekEvents")
        .header(HOST, "goffstownathletics.com")
        .header(
            USER_AGENT,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:100.0) Gecko/20100101 Firefox/100.0",
        )
        .header(ACCEPT, "application/json, text/javascript, */*; q=0.01")
        .header(ACCEPT_LANGUAGE, "en-US,en;q=0.5")
        .header(
            CONTENT_TYPE,
            "application/x-www-form-urlencoded; charset=UTF-8",
        )
        .header("x-requested-with", "XMLHttpRequest")
        .header(ORIGIN, "https://goffstownathletics.com")
        .header(DNT, "1")
        .header(CONNECTION, "keep-alive")
        .header(REFERER, "https://goffstownathletics.com/main/calendar")
        .header(COOKIE, "wfx_unq=1UFMH2Zpyl68DE6k; cfid=e7e60dae-33cf-4fdf-aabd-38328c1adbc5; cftoken=0; ERD=30B2F5090B2BE8E73FA88560548E6651F6F90073D7B4BBBF1BEA2946CA9FADAA; CALDATE=5/3/2022; CALVIEW=week")
        .header("sec-fetch-dest", "empty")
        .header("sec-fetch-mode", "cors")
        .header("sec-fetch-site", "same-origin")
        .header(TE, "trailers").body("fromMonth=5&fromYear=2022&fromDay=3&toMonth=5&toYear=2022&toDay=7").send().await?;

        let raw_games: Vec<Vec<Value>> = serde_json::from_str(&resp.text().await?)?;
        let mut games = Vec::new();
        for raw_game in raw_games.get(1).unwrap() {
            let game: RawEvent = serde_json::from_value(raw_game.clone()).unwrap();
            games.push(game);
        }

        Ok(games)
    }

    pub fn clean(&mut self) -> Result<Option<Event>> {
        fn str_to_bool(text: &str) -> bool {
            match text {
                "" => false,
                "0" => false,
                "1" => true,
                _ => unreachable!("Failed to convert string to boolean"),
            }
        }

        if self.event_type != "sport"
            || self.name.contains("Middle School")
            || !str_to_bool(&self.home)
        {
            return Ok(None);
        }

        Ok(Some(Event {
            name: self.name.to_owned(),
            time: DateTime::parse_from_str(
                &format!(
                    "{}-{:0>2}-{:0>2} {:0>4} {}",
                    self.year,
                    self.month,
                    self.day,
                    self.time,
                    Local::now().format("%z")
                ),
                "%Y-%m-%d %I:%M %p %z",
            )
            .context("Failed to parse datetime")?
            .with_timezone(&Local),
            sport: self.name.split(" ").last().unwrap().to_string(),
            varsity: self.name.to_lowercase().contains("varsity"),
            opponent: self.opponent.to_owned(),
            location: self.location.to_owned(),
            rescheduled: !self.rescheduled_date.is_empty(),
            cancelled: str_to_bool(&self.cancelled),
            rescheduled_date: if self.rescheduled_date.is_empty() || self.rescheduled_date == "TBA"
            {
                None
            } else {
                Some(NaiveDate::parse_from_str(
                    &self.rescheduled_date,
                    "%m/%d/%Y",
                )?)
            },
        }))
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use chrono::{DateTime, Local};

    use crate::fetch::{Event, RawEvent};

    #[tokio::test]
    async fn fetch_this_weeks() {
        assert!(RawEvent::fetch_this_weeks().await.is_ok());
    }

    #[test]
    fn clean() -> Result<()> {
        assert_eq!(
            RawEvent {
                postponed: String::from("0"),
                month: String::from("5"),
                year: String::from("2022"),
                day: String::from("3"),
                location: String::from("Sanborn Regional High School"),
                event_type: String::from("sport"),
                opponent: String::from("Multiple Opponents"),
                cancelled: String::from("0"),
                name: String::from("Boys-Girls Varsity Outdoor Track"),
                home: String::from("0"),
                time: String::from("4:00 PM"),
                rescheduled_date: String::from(""),
            }
            .clean()?,
            Some(Event {
                name: String::from("Boys-Girls Varsity Outdoor Track"),
                time: DateTime::parse_from_rfc3339("2022-05-03T16:00:00-04:00")?
                    .with_timezone(&Local),
                sport: String::from("Track"),
                varsity: true,
                opponent: String::from("Multiple Opponents"),
                location: String::from("Sanborn Regional High School"),
                rescheduled: false,
                rescheduled_date: None,
                cancelled: false,
            })
        );

        assert_eq!(
            RawEvent {
                postponed: String::from("0"),
                month: String::from("5"),
                year: String::from("2022"),
                day: String::from("3"),
                location: String::from("Goffstown High School"),
                event_type: String::from("school"),
                opponent: String::from(""),
                cancelled: String::from("0"),
                name: String::from("School Board Meeting"),
                home: String::from("0"),
                time: String::from("7:21 PM"),
                rescheduled_date: String::from(""),
            }
            .clean()?,
            None
        );
        Ok(())
    }
}
