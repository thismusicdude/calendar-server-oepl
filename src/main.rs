use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use anyhow::{Context, Result};
use std::cmp::Reverse;
use std::fs::File;
use std::io::Write;
use std::{env, fs, path::PathBuf};
use utils::events::NotificationEvent;
mod utils;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Deserialize, Serialize)]
struct ServerConfigFile {
    urls: Vec<String>,
    ammount_of_next_events: i8,
    nothing_todo_message: String,
}

/// loads the ServerConfigFile object from a specific path
fn load_config(file_path: &PathBuf) -> Result<ServerConfigFile> {
    let data = fs::read_to_string(file_path)?;
    let config: ServerConfigFile = serde_json::from_str(&data)?;
    Ok(config)
}

/// gets an absolute path from a local path as PathBuf
fn get_abs_path(path: &str) -> Result<PathBuf> {
    // format!("Hello {}!", name)
    let mut current_dir = env::current_dir()
        .with_context(|| "Couldnt get current working directory of this program")?;
    current_dir.push(path);
    Ok(current_dir)
}

#[post("/api/set_config")]
async fn api_set_calendars(form: web::Json<ServerConfigFile>) -> impl Responder {

    let config_file = match get_abs_path("config.json") {
        Ok(pb) => pb,
        Err(e) => return HttpResponse::NotFound().body(format!["couldnt get config.json path: {}", e]),
    };

    let json_data =
        serde_json::to_string_pretty(&form).expect("could not serialize content into file");

    // ixme: check if every needed field is avaiable

    // set config file to user specified config file
    let mut file = File::create(config_file).expect("could not create file");
    file.write_all(json_data.as_bytes())
        .expect("could not save json data");

    HttpResponse::Ok().body("set config successul")
}

#[get("/api/get_config")]
async fn api_get_calendars() -> impl Responder {
    // get config file path
    let config_file = match get_abs_path("config.json") {
        Ok(pb) => pb,
        Err(e) => return HttpResponse::NotFound().body(format!["couldnt get config.json path: {}", e]),
    };

    // retrieve config file as string
    let config_content =
        fs::read_to_string(config_file).unwrap_or_else(|_| "Couldn't load config file".to_string());

    // send config file to html page
    HttpResponse::Ok()
        .content_type("application/json")
        .body(config_content)
}

#[get("/setup")]
async fn setup() -> impl Responder {
    // retrieve a html file
    let html_path = match get_abs_path("src/html/setup.html") {
        Ok(pb) => pb,
        Err(e) => return HttpResponse::NotFound().body(format!["setup.html file not found: {}", e]),
    };

    let html_content =
        fs::read_to_string(html_path).unwrap_or_else(|_| "Error while loading html page".to_string());

    HttpResponse::Ok()
        .content_type("text/html")
        .body(html_content)
}


#[get("/next_events.json")]
async fn paperdisplay() -> impl Responder {

    // get absolute config json path
    let config_path = match get_abs_path("config.json") {
        Ok(pb) => pb,
        Err(e) => return HttpResponse::NotFound().body(format!["couldnt get absolute path of config.json: {}", e]),
    };

    // get config.json file
    let config_file = match load_config(&config_path) {
        Ok(pb) => pb,
        Err(e) => return HttpResponse::NotFound().body(format!["couldnt get config.json path: {}", e]),
    };

    let mut all_events: Vec<NotificationEvent> = Vec::new();

    // retrieve all events of every given ics url
    for url in config_file.urls {
        let events_from_url = match utils::getevents::get_events_from_file(&url).await {
            Ok(pb) => pb,
            Err(e) => {
                return HttpResponse::NotFound()
                    .body(format!["couldnt load next events from {}\nbecause of {}", url, e]);
            }
        };
        all_events.extend(events_from_url);
    }

    // retrieve "next" events, specified by the ammount in the config file
    let mut next_events =
        match utils::getevents::get_next_events(config_file.ammount_of_next_events, all_events)
            .await
        {
            Ok(pb) => pb,
            Err(e) => return HttpResponse::NotFound().body(format!["couldnt get next events {}:", e]),
        };

    // sort them by date
    next_events.sort_by_key(|event| Reverse(event.date));
    
    // if there is no event, retrieve "nothing_todo_message"
    if next_events.len() == 0 {
        let json_data = json!([{"text": [5, 5, format!["{}", config_file.nothing_todo_message ],"fonts/calibrib50",2,0]}]);
        return HttpResponse::Ok().body(json_data.to_string());
    }

    let mut json_vector: Vec<Value> = Vec::new();

    let mut last_coord = 5;
    
    // generate json template for epaper displays
    for event in next_events {
        json_vector.push(
            json!([{"text": [5, last_coord, format!["{}", event.summary ],"fonts/calibrib50",2,0]}]),
        );
        last_coord = last_coord + 53;
        json_vector.push(json!([{"text": [5, last_coord, format!["at {}", event.date.time().to_string() ],"fonts/calibrib20",2,0]}]));
        last_coord = last_coord + 23;
    }

    match serde_json::to_string_pretty(&json_vector) {
        Ok(json) => return HttpResponse::Ok().body(json),
        Err(e) => return HttpResponse::NotFound().body(format!["couldnt get next events {}", e]),
    };
}


#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(setup)
            .service(paperdisplay)
            .service(api_get_calendars)
            .service(api_set_calendars)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
