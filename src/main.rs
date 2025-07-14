use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use std::env;
use futures::stream::{FuturesUnordered, StreamExt};

const HEADER_API_KEY: &str = "x-api-key";
const PAGE_SIZE: usize = 20;

#[derive(Debug, Deserialize)]
struct ApiResponse {
    #[serde(rename = "pageInfo")]
    page_info: PageInfo,
    results: Vec<RequestItem>,
}

#[derive(Debug, Deserialize)]
struct PageInfo {
    page: u32,
    pages: u32,
}

#[derive(Debug, Deserialize)]
struct RequestItem {
    id: u32,
    #[serde(rename = "externalServiceId")]
    external_service_id: Option<u32>,
    #[serde(rename = "mediaType")]
    media_type: MediaType,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum MediaType {
    Tv,
    Movie,
}

#[derive(Debug, Clone)]
struct SeerOptions {
    name: String,
    url: String,
    headers: HeaderMap,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (overseerr_url, overseerr_key) = get_env_pair("OVERSEERR")?;
    let (jellyseerr_url, jellyseerr_key) = get_env_pair("JELLYSEERR")?;
    let (sonarr_url, sonarr_key) = get_env_pair("SONARR")?;
    let (radarr_url, radarr_key) = get_env_pair("RADARR")?;

    let seers: Vec<SeerOptions> = vec![
        SeerOptions {
            name: "Overseerr".to_string(),
            url: overseerr_url,
            headers: build_headers(&overseerr_key)?,
        },
        SeerOptions {
            name: "Jellyseerr".to_string(),
            url: jellyseerr_url,
            headers: build_headers(&jellyseerr_key)?,
        },
    ];

    let sonarr_headers = build_headers(&sonarr_key)?;
    let radarr_headers = build_headers(&radarr_key)?;

    let client = reqwest::Client::new();

    for seer in &seers {
        process_seer(
            &client,
            seer,
            &sonarr_url,
            &sonarr_headers,
            &radarr_url,
            &radarr_headers,
        )
        .await?;
    }

    Ok(())
}

fn get_env_pair(name: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let url = env::var(format!("{}_URL", name))?;
    let key = env::var(format!("{}_KEY", name))?;
    Ok((url, key))
}

fn build_headers(api_key: &str) -> Result<HeaderMap, Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(HEADER_API_KEY, HeaderValue::from_str(api_key)?);
    Ok(headers)
}

async fn process_seer(
    client: &reqwest::Client,
    seer: &SeerOptions,
    sonarr_url: &str,
    sonarr_headers: &HeaderMap,
    radarr_url: &str,
    radarr_headers: &HeaderMap,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{} processing started...", seer.name);
    let mut current_page = 1;

    loop {
        let url = format!("{}/media?take={}&skip={}", seer.url, PAGE_SIZE, (current_page - 1) * PAGE_SIZE);

        let resp = client.get(&url).headers(seer.headers.clone()).send().await?;

        if !resp.status().is_success() {
            eprintln!("Request to {} failed with status: {}", url, resp.status());
            break;
        }

        let api_response: ApiResponse = resp.json().await?;

        let mut is_media_deleted = false;

        let mut futures = FuturesUnordered::new();
        
        for item in api_response.results {
            let client = client.clone();
            let seer = seer.clone();
            let sonarr_url = sonarr_url.to_string();
            let radarr_url = radarr_url.to_string();
            let sonarr_headers = sonarr_headers.clone();
            let radarr_headers = radarr_headers.clone();
        
            futures.push(async move {
                let should_delete = match item.external_service_id {
                    Some(external_id) => {
                        let (arr_url, arr_headers) = match item.media_type {
                            MediaType::Tv => (format!("{}/series/{}", sonarr_url, external_id), &sonarr_headers),
                            MediaType::Movie => (format!("{}/movie/{}", radarr_url, external_id), &radarr_headers),
                        };
                        !check_arr_item(&client, &arr_url, arr_headers).await.unwrap_or(false)
                    }
                    None => true,
                };
        
                if should_delete {
                    let delete_url = format!("{}/media/{}", seer.url, item.id);
                    let del_resp = client.delete(&delete_url).headers(seer.headers.clone()).send().await;
        
                    match del_resp {
                        Ok(resp) if resp.status().is_success() => {
                            println!("Deleted {:?} media {} from {}", item.media_type, item.id, seer.name);
                            true
                        }
                        Ok(resp) => {
                            eprintln!("Failed to delete {:?} {} from {} (status: {})", item.media_type, item.id, seer.name, resp.status());
                            false
                        }
                        Err(e) => {
                            eprintln!("Failed to delete {:?} {} from {} (error: {})", item.media_type, item.id, seer.name, e);
                            false
                        }
                    }
                } else {
                    false
                }
            });
        }
        
        while let Some(deleted) = futures.next().await {
            if deleted {
                is_media_deleted = true;
            }
        }

        if api_response.page_info.page >= api_response.page_info.pages {
            break;
        }

        if !is_media_deleted {
            current_page += 1;
        }
    }

    println!("{} processing finished.", seer.name);
    Ok(())
}

async fn check_arr_item(
    client: &reqwest::Client,
    url: &str,
    headers: &HeaderMap,
) -> Result<bool, reqwest::Error> {
    let resp = client.get(url).headers(headers.clone()).send().await?;
    Ok(resp.status().is_success())
}
