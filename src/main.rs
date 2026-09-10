use std::fs;

use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};

const MACONDO_BASE_URL: &str = "https://macondo.hackclub.com";
const BATCH_SIZE: i32 = 100;
const OUTPUT_FILE: &str = "projects.json";

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
struct ProjectOwner {
    id: String,
    slack_id: Option<String>,
    username: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
struct MacondoProject {
    created_at: String,
    updated_at: String,

    fruit: Option<String>,
    id: i32,
    name: String,
    owner: ProjectOwner,
    #[serde(rename(deserialize = "streakStatus"))]
    streak_status: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let mut all_projects = Vec::new();
    let mut start_index = 1;

    loop {
        println!("Fetching 100 projects starting from {start_index}");
        let batch = request_batch(start_index).await?;
        println!("Found {} projects", batch.len());
        if batch.is_empty() {
            break;
        }
        all_projects.extend(batch);
        start_index += BATCH_SIZE;
    }

    fs::write(OUTPUT_FILE, serde_json::to_string(&all_projects)?)?;

    Ok(())
}

async fn request_batch(start_index: i32) -> Result<Vec<MacondoProject>> {
    let client = reqwest::ClientBuilder::new().build()?;
    let mut handles: Vec<tokio::task::JoinHandle<Result<Option<MacondoProject>>>> =
        Vec::with_capacity(BATCH_SIZE as usize);

    for i in start_index..(start_index + BATCH_SIZE) {
        let client = client.clone();
        handles.push(tokio::spawn(async move {
            let resp = client.get(project_url(i)).send().await?;
            if resp.status() == reqwest::StatusCode::NOT_FOUND {
                return Ok(None);
            }
            let project: MacondoProject = resp.error_for_status()?.json().await?;
            Ok(Some(project))
        }));
    }

    let mut projects = Vec::with_capacity(BATCH_SIZE as usize);
    for handle in handles {
        if let Some(project) = handle.await?? {
            projects.push(project);
        }
    }
    Ok(projects)
}

fn project_url(id: i32) -> String {
    format!("{MACONDO_BASE_URL}/api/projects/{id}")
}
