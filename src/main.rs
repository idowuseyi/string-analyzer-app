use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::env;
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Clone)]
pub struct StringProperties {
    pub length: usize,
    pub is_palindrome: bool,
    pub unique_characters: usize,
    pub word_count: usize,
    pub sha256_hash: String,
    pub character_frequency_map: HashMap<char, usize>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StringData {
    pub id: String,
    pub value: String,
    pub properties: StringProperties,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateStringRequest {
    pub value: String,
}

#[derive(Deserialize, Serialize, Default)]
pub struct FilterQuery {
    pub is_palindrome: Option<bool>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub word_count: Option<usize>,
    pub contains_character: Option<String>,
}

#[derive(Serialize)]
pub struct StringsResponse {
    pub data: Vec<StringData>,
    pub count: usize,
    pub filters_applied: FilterQuery,
}

#[derive(Serialize)]
pub struct NaturalLanguageResponse {
    pub data: Vec<StringData>,
    pub count: usize,
    pub interpreted_query: InterpretedQuery,
}

#[derive(Serialize)]
pub struct InterpretedQuery {
    pub original: String,
    pub parsed_filters: FilterQuery,
}

type AppState = Arc<Mutex<HashMap<String, StringData>>>;

fn analyze_string(s: &str) -> StringProperties {
    let length = s.len();
    let is_palindrome = s.to_lowercase().chars().collect::<Vec<_>>() == s.to_lowercase().chars().rev().collect::<Vec<_>>();
    let unique_characters = s.chars().filter(|c| c.is_alphabetic()).collect::<std::collections::HashSet<_>>().len();
    let word_count = s.split_whitespace().count();
    let sha256_hash = Sha256::digest(s.as_bytes()).iter().map(|b| format!("{:02x}", b)).collect::<String>();
    let mut character_frequency_map = HashMap::new();
    for c in s.chars().filter(|c| c.is_alphabetic()) {
        *character_frequency_map.entry(c).or_insert(0) += 1;
    }
    StringProperties {
        length,
        is_palindrome,
        unique_characters,
        word_count,
        sha256_hash: sha256_hash.clone(),
        character_frequency_map,
    }
}

async fn create_string(
    State(state): State<AppState>,
    Json(payload): Json<CreateStringRequest>,
) -> Result<(StatusCode, Json<StringData>), StatusCode> {
    let properties = analyze_string(&payload.value);
    let id = properties.sha256_hash.clone();
    let mut db = state.lock().await;
    if db.contains_key(&id) {
        return Err(StatusCode::CONFLICT);
    }
    let data = StringData {
        id: id.clone(),
        value: payload.value,
        properties,
        created_at: Utc::now(),
    };
    db.insert(id, data.clone());
    Ok((StatusCode::CREATED, Json(data)))
}

async fn get_string(
    State(state): State<AppState>,
    Path(value): Path<String>,
) -> Result<Json<StringData>, StatusCode> {
    let properties = analyze_string(&value);
    let id = properties.sha256_hash.clone();
    let db = state.lock().await;
    if let Some(data) = db.get(&id) {
        Ok(Json(data.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn get_all_strings(
    State(state): State<AppState>,
    Query(query): Query<FilterQuery>,
) -> Result<Json<StringsResponse>, StatusCode> {
    let db = state.lock().await;
    let mut filtered = Vec::new();
    for (_, data) in db.iter() {
        let mut matches = true;
        if let Some(pal) = query.is_palindrome {
            if data.properties.is_palindrome != pal {
                matches = false;
            }
        }
        if let Some(min_l) = query.min_length {
            if data.properties.length < min_l {
                matches = false;
            }
        }
        if let Some(max_l) = query.max_length {
            if data.properties.length > max_l {
                matches = false;
            }
        }
        if let Some(wc) = query.word_count {
            if data.properties.word_count != wc {
                matches = false;
            }
        }
        if let Some(ch) = &query.contains_character {
            if ch.len() != 1 {
                // invalid, but we'll handle in main probably
            } else {
                let ch = ch.chars().next().unwrap();
                if !data.value.chars().any(|c| c == ch) {
                    matches = false;
                }
            }
        }
        if matches {
            filtered.push(data.clone());
        }
    }
    let count = filtered.len();
    if let Some(ch) = &query.contains_character {
        if ch.len() != 1 {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    Ok(Json(StringsResponse {
        data: filtered,
        count,
        filters_applied: query,
    }))
}

fn parse_natural_language_query(query: &str) -> Result<FilterQuery, String> {
    let words: Vec<&str> = query.split_whitespace().collect();
    let mut filters = FilterQuery::default();

    let mut i = 0;
    while i < words.len() {
        match words[i].to_lowercase().as_str() {
            "all" => {
                // skip
                i += 1;
            }
            "single" => {
                if i + 1 < words.len() && words[i + 1].to_lowercase() == "word" {
                    filters.word_count = Some(1);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "palindrome" | "palindromic" => {
                if let Some(prev) = words.get(i - 1) {
                    if prev.to_lowercase() != "non" {
                        filters.is_palindrome = Some(true);
                    }
                } else {
                    filters.is_palindrome = Some(true);
                }
                i += 1;
            }
            "longer" => {
                if i + 2 < words.len() && words[i + 1].to_lowercase() == "than" {
                    if let Ok(num) = words[i + 2].parse::<usize>() {
                        filters.min_length = Some(num + 1);
                        i += 3;
                    } else {
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            }
            "containing" | "contain" => {
                if i + 3 < words.len() && words[i + 1].to_lowercase() == "the" && words[i + 2].to_lowercase() == "letter" {
                    let ch = words[i + 3];
                    if ch.len() == 1 {
                        filters.contains_character = Some(ch.to_string());
                        i += 4;
                    } else {
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            }
            "first" => {
                if i + 1 < words.len() && words[i + 1].to_lowercase() == "vowel" {
                    filters.contains_character = Some("a".to_string());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }

    Ok(filters)
}

async fn filter_by_natural_language(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<NaturalLanguageResponse>, StatusCode> {
    let query = params.get("query").ok_or(StatusCode::BAD_REQUEST)?;
    let parsed = parse_natural_language_query(query).map_err(|_| StatusCode::BAD_REQUEST)?;
    let mut filtered = Vec::new();
    let db = state.lock().await;
    for (_, data) in db.iter() {
        let mut matches = true;
        if let Some(pal) = parsed.is_palindrome {
            if data.properties.is_palindrome != pal {
                matches = false;
            }
        }
        if let Some(min_l) = parsed.min_length {
            if data.properties.length < min_l {
                matches = false;
            }
        }
        if let Some(max_l) = parsed.max_length {
            if data.properties.length > max_l {
                matches = false;
            }
        }
        if let Some(wc) = parsed.word_count {
            if data.properties.word_count != wc {
                matches = false;
            }
        }
        if let Some(ch) = &parsed.contains_character {
            if ch.len() != 1 {
            } else {
                let ch = ch.chars().next().unwrap();
                if !data.value.chars().any(|c| c == ch) {
                    matches = false;
                }
            }
        }
        if matches {
            filtered.push(data.clone());
        }
    }
    let count = filtered.len();
    Ok(Json(NaturalLanguageResponse {
        data: filtered,
        count,
        interpreted_query: InterpretedQuery {
            original: query.to_string(),
            parsed_filters: parsed,
        },
    }))
}

async fn delete_string(
    State(state): State<AppState>,
    Path(value): Path<String>,
) -> StatusCode {
    let properties = analyze_string(&value);
    let id = properties.sha256_hash;
    let mut db = state.lock().await;
    if db.remove(&id).is_some() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

#[tokio::main]
async fn main() {

    // Get port from env, default to 8080
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port).parse::<SocketAddr>().unwrap();

    let state: AppState = Arc::new(Mutex::new(HashMap::new()));

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/kaithhealth", get(|| async { "OK" }))
        .route("/strings", post(create_string))
        .route("/strings/:value", get(get_string))
        .route("/strings", get(get_all_strings))
        .route("/strings/filter-by-natural-language", get(filter_by_natural_language))
        .route("/strings/:value", delete(delete_string))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Server running on http://localhost:8080");
    axum::serve(listener, app).await.unwrap();
}
