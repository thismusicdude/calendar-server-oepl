// inspired by
// https://github.com/rust-dd/google-calendar-cli/blob/main/src/util/calendar.rs
// get events with google api
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use google_calendar3::{
    CalendarHub,
    hyper_rustls::{self, HttpsConnector},
    hyper_util::{self, client::legacy::connect::HttpConnector},
    yup_oauth2::{self, ApplicationSecret},
};

use chrono::{DateTime, Datelike, Duration, Local, Timelike};

use dirs::document_dir;

// Referenz ist immutable
async fn secret_parsing(path: &Path) -> Result<ApplicationSecret> {
    let secret_key = yup_oauth2::read_application_secret(path)
        .await
        // use with context from anyhow
        // ( || {} ) => Closure (anonyme Funktion) ohne Eingabeparameter
        .with_context(|| format!("could not read google secret file, check {:?}", path))?;
    Ok(secret_key)
}

pub fn abs_path(path: &str) -> Result<PathBuf, Box<dyn Error>> {
    // ok_or converts Option to Result
    let mut abs_path = document_dir().ok_or("Failed to determine home directory")?;
    abs_path.push(path);
    Ok(abs_path)
}


// TODO: Implement different oauth2 process, so we can access the url
// And redirect the user throught a web-interface to it
async fn auth(
    secret_load_path: &str,
    verification_store_path: &str,
) -> Result<CalendarHub<HttpsConnector<HttpConnector>>, Box<dyn Error>> {
    let secret_abs_path = abs_path(secret_load_path)?;
    println!(
        "Loading secret file from {}",
        secret_abs_path.to_str().unwrap()
    );

    // if path does not exist, create directory
    if let Some(parent) = secret_abs_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let auth_builder = match secret_parsing(&secret_abs_path).await {
        Ok(secret) => yup_oauth2::InstalledFlowAuthenticator::builder(
            secret,
            yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
        ),
        Err(_) => {
            // do manuall request
            // urn:ietf:wg:oauth:2.0:oob is for "Out of Band" verification
            let secret: yup_oauth2::ApplicationSecret = ApplicationSecret {
                auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
                client_secret: "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".to_string(),
                token_uri: "https://accounts.google.com/o/oauth2/token".to_string(),
                redirect_uris: vec!["urn:ietf:wg:oauth:2.0:oob".to_string()],
                client_id:
                    "84326943465-s2bisj2q7da2tujvv7l8bbghvkp9nem7.apps.googleusercontent.com"
                        .to_string(),
                auth_provider_x509_cert_url: Some(
                    "https://www.googleapis.com/oauth2/v1/certs".to_string(),
                ),
                project_id: None,
                client_email: None,
                client_x509_cert_url: None,
            };
            yup_oauth2::InstalledFlowAuthenticator::builder(
                secret,
                yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
            )
        }
    };

    let store_abs_path = abs_path(verification_store_path)?;
    println!("Saving Auth file to {}", store_abs_path.to_str().unwrap());

    let auth = auth_builder
        .persist_tokens_to_disk(&store_abs_path)
        .build()
        .await?;

    let scopes = &[
        "https://www.googleapis.com/auth/calendar.calendars.readonly",
        "https://www.googleapis.com/auth/calendar.readonly",
        "https://www.googleapis.com/auth/calendar.events.readonly",
    ];

    // return current token
    match auth.token(scopes).await {
        Ok(_) => {}
        Err(e) => println!("Authentication error: {:?}", e),
    }

    // verstehen?
    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .unwrap()
                .https_or_http()
                .enable_http1()
                .build(),
        );

    let hub = CalendarHub::new(client, auth);
    Ok(hub)
}

fn get_start_of_the_week() -> DateTime<Local> {
    let now = Local::now();
    let days_to_subtract = now.weekday().num_days_from_monday() as i64;
    let start_of_the_week = now - Duration::days(days_to_subtract);
    start_of_the_week
}

fn get_calendar_lists() -> Vec<String> {
    vec!["primary".to_string()]
}

pub async fn print_next_tasks(calendarid: &str)  {
    let hub = match auth("client_secret.json", "client_save_auth.json").await {
        Ok(hub) => hub,
        Err(e) => {
            eprintln!("Error during authentication - {}", e);
            return ;
        }
    };

    let now = Local::now();
    let evening = now
        .with_hour(23)
        .unwrap()
        .with_minute(59)
        .unwrap()
        .with_second(59)
        .unwrap()
        .to_utc();

    let events = hub
        .events()
        .list(calendarid)
        .time_min(now.to_utc())
        .time_max(evening)
        .single_events(true)
        .doit()
        .await;

    match events {
        Ok((_, events)) => {
            if let Some(items) = events.items {
                for event in items {
                    println!("{}", event.summary.as_ref().unwrap().to_string());
                }
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}
