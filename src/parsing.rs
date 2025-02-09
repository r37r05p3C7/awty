use std::default::Default;

use color_eyre::eyre::Result;
use kuchikiki::traits::TendrilSink;
use kuchikiki::NodeRef;
use regex::Regex;
use serde::{Deserialize, Serialize};
use ureq::{Agent, Error};

use crate::utils::error;

pub const HOST: &str = "https://f95zone.to";

pub fn logged_in(agent: &Agent) -> bool {
    let url = format!("{HOST}/account");
    match agent.get(&url).call() {
        Ok(_) => true,
        Err(Error::Status(_, _)) => false,
        Err(Error::Transport(err)) => {
            error(&format!("Network error: {err}"));
            std::process::exit(1);
        }
    }
}

pub fn parse_thread(id: &str, agent: &Agent) -> ThreadSlug {
    let url = format!("{HOST}/threads/{id}");

    let res = match agent.get(url.as_str()).call() {
        Ok(res) => res,
        Err(Error::Status(code, res)) => {
            return ThreadSlug::error(
                id,
                &format!("Bad response code: {} {}", code, res.status_text()),
            );
        }
        Err(err) => return ThreadSlug::error(id, &format!("Network error: {err}")),
    };

    let Ok(body) = res.into_string() else {
        return ThreadSlug::error(id, "Failed to read response body");
    };

    let html = kuchikiki::parse_html().one(body);
    let header = match html.select(".p-title").expect("Selector error").next() {
        Some(node_data) => node_data.as_node().to_owned(),
        None => return ThreadSlug::error(id, "Failed to locate thread header"),
    };
    let title = get_title(&header);
    let status = get_status(&header);

    ThreadSlug {
        id: id.to_string(),
        title,
        status,
        ..Default::default()
    }
}

fn get_title(header: &NodeRef) -> String {
    let mut title = String::new();

    let title_node = match header.select_first(".p-title-value") {
        Ok(node) => node.as_node().children(),
        Err(()) => return title,
    };

    // Removes prefixes: "[prefix1] [prefix2] Title [suffix]" -> "Title [suffix]"
    for node in title_node {
        if let Some(elem) = node.clone().into_element_ref() {
            if let Some(class_attr) = elem.attributes.borrow().get("class") {
                if class_attr.contains("labelLink") || class_attr.contains("label-append") {
                    continue;
                }
            }
        }
        title.push_str(&node.as_text().unwrap().borrow());
    }

    // Isolates title text: "Title [suffix]" -> "Title"
    let re_name = Regex::new(r"^\s*(.*?)(?:\s*\[.*?]\s*)*$").expect("Regex init error");
    if let Some(captures) = re_name.captures(&title) {
        if let Some(m) = captures.get(1) {
            title = m.as_str().to_string();
        }
    }

    title
}

fn get_status(header: &NodeRef) -> Status {
    let mut prefixes: Vec<String> = vec![];
    for span in header.select("span").expect("Selector error") {
        let Some(child) = span.as_node().first_child() else {
            continue;
        };
        let Some(text) = child.as_text() else {
            continue;
        };
        prefixes.push(text.borrow().to_string());
    }
    for prefix in &prefixes {
        if let Ok(Some(status)) = Status::from_str(prefix) {
            return status;
        }
    }
    Status::InDevelopment
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ThreadSlug {
    pub id: String,
    pub title: String,
    pub status: Status,
    pub error: Option<String>,
}

impl ThreadSlug {
    pub fn error(id: &str, msg: &str) -> ThreadSlug {
        ThreadSlug {
            id: id.to_string(),
            error: Some(String::from(msg)),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum Status {
    #[default]
    InDevelopment,
    Completed,
    Abandoned,
    OnHold,
}

impl Status {
    pub fn from_str(str: &str) -> Result<Option<Status>> {
        if Regex::new(r"(?i)completed")?.is_match(str) {
            return Ok(Some(Status::Completed));
        }
        if Regex::new(r"(?i)abandoned")?.is_match(str) {
            return Ok(Some(Status::Abandoned));
        }
        if Regex::new(r"(?i)on ?hold")?.is_match(str) {
            return Ok(Some(Status::OnHold));
        }
        Ok(None)
    }
}
