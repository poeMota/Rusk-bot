use crate::config::{write_file, CONFIG, DATA_PATH};
use reqwest::Error;
use scraper::{Html, Selector};
use std::{collections::HashMap, env, path::PathBuf};

#[derive(Debug)]
pub enum ConnectionError {
    NotAllowedUrl(String),
    ReqwestError(Error),
    UnexpetedOtherError(String),
    AuthError(String),
    StatusCodeError(String, Error),
}

pub async fn unload_content(url: String) -> Result<String, ConnectionError> {
    let _config = CONFIG.read().await;

    if url.contains("..") {
        return Err(ConnectionError::NotAllowedUrl(
            "\"..\" in path are not allowed".to_string(),
        ));
    }

    let env_url = match env::var("URL") {
        Ok(value) => value,
        Err(e) => return Err(ConnectionError::UnexpetedOtherError(e.to_string())),
    };
    let valideted_url = format!("{}/{}", env_url, url)
        .replace("//", "/")
        .replace(" ", "%C2%A0");
    let mut response = reqwest::Client::new().get(valideted_url);

    let need_auth = match env::var("NEEDAUTH") {
        Ok(need) => need == "true".to_string(),
        Err(_) => false,
    };

    if need_auth {
        let username = match env::var("LOGIN") {
            Ok(need) => need,
            Err(e) => return Err(ConnectionError::AuthError(e.to_string())),
        };

        let password = match env::var("PASSWORD") {
            Ok(need) => Some(need),
            Err(e) => return Err(ConnectionError::AuthError(e.to_string())),
        };

        response = response.basic_auth(username, password);
    }

    match response.send().await {
        Ok(content) => match content.error_for_status() {
            Ok(content) => match content.text().await {
                Ok(content) => return Ok(content),
                Err(e) => return Err(ConnectionError::ReqwestError(e)),
            },
            Err(e) => Err(ConnectionError::ReqwestError(e)),
        },
        Err(e) => return Err(ConnectionError::StatusCodeError(url, e)),
    }
}

pub async fn unload_to_file(url: String) -> Result<PathBuf, ConnectionError> {
    let content = unload_content(url.clone()).await?;
    let path = DATA_PATH.join(format!("temp/{}", url.split("/").last().unwrap()));

    write_file(&path, content);
    Ok(path)
}

pub async fn get_user_id(name: String) -> String {
    let config = CONFIG.read().await;
    let not_found = "Not Found".to_string();

    let url = format!("{}name={}", config.userid_api_url, name);
    let data = match reqwest::Client::new().get(url).send().await {
        Ok(content) => match content.text().await {
            Ok(text) => text,
            Err(_) => return not_found,
        },
        Err(_) => return not_found,
    }
    .replace("{", "")
    .replace("}", "")
    .replace("\"", "");

    for i in data.split(",") {
        if i.contains("userId") {
            match i.split(":").last() {
                Some(string) => return string.to_string(),
                None => return not_found,
            };
        }
    }
    not_found
}

pub async fn file_dates(url: String) -> Result<HashMap<String, String>, ConnectionError> {
    let response = unload_content(url).await?;

    let document = Html::parse_document(&response);
    let link_selector = Selector::parse("a").unwrap();

    let mut result = HashMap::new();

    for element in document.select(&link_selector) {
        if let Some(text) = element.text().next() {
            if let Some(sibling) = element
                .next_sibling()
                .and_then(|node| node.value().as_text())
            {
                let date_part = sibling.trim().split("  ").next().unwrap_or("");
                result.insert(text.to_string(), date_part.to_string());
            }
        }
    }

    Ok(result)
}
