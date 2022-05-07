use anyhow::Result;
use fetch::{Event, RawEvent};
use rocket::{serde::json::Json, Config};
use serde::Serialize;

mod fetch;

#[macro_use]
extern crate rocket;

#[derive(Serialize)]
pub struct APIResult<T: Serialize> {
    pub ok: bool,
    pub err: Option<String>,
    pub data: Option<T>,
}

impl<T: Serialize> APIResult<T> {
    pub fn from_result(result: Result<T>) -> APIResult<T> {
        match result {
            Ok(r) => APIResult {
                ok: true,
                err: None,
                data: Some(r),
            },
            Err(x) => APIResult {
                ok: false,
                err: Some(x.to_string()),
                data: None,
            },
        }
    }
}

#[get("/current-week", format = "json")]
async fn current_week() -> Json<APIResult<Vec<Event>>> {
    let raw_events = RawEvent::fetch_this_weeks().await;
    if raw_events.is_err() {
        return Json(APIResult {
            ok: false,
            err: Some(raw_events.unwrap_err().to_string()),
            data: None,
        });
    }
    let mut games = Vec::new();
    for mut raw_event in raw_events.unwrap() {
        let cleaned_event = raw_event.clean();
        if cleaned_event.is_err() {
            return Json(APIResult {
                ok: false,
                err: Some(cleaned_event.unwrap_err().to_string()),
                data: None,
            });
        } else {
            if cleaned_event.as_ref().unwrap().is_some() {
                games.push(cleaned_event.unwrap().unwrap());
            }
        }
    }
    Json(APIResult::from_result(Ok(games)))
}

#[launch]
fn rocket() -> _ {
    let config = Config::figment().merge(("address", "0.0.0.0"));
    rocket::custom(config).mount("/", routes![current_week])
}
