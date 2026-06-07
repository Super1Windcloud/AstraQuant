use super::catalog::{
    INDEX_CATEGORY_IDS, IndexDefinition, category_counts, definitions_for_category, finnhub_symbol,
};
use super::models::{FinnhubQuote, IndexOverviewRow, IndicesOverviewResponse, MarketSnapshot};
use log::{debug, error, info, warn};
use std::{env, path::PathBuf, sync::Once, time::Duration};

pub(crate) const MARKET_LOG_TARGET: &str = "astraquant::market_data";

static ENV_LOADER: Once = Once::new();

#[tauri::command]
pub(crate) fn get_market_snapshot(
    provider: String,
    symbol: String,
    asset_class: String,
) -> Result<MarketSnapshot, String> {
    load_env();
    info!(
        target: MARKET_LOG_TARGET,
        "get_market_snapshot start provider={} symbol={} asset_class={}",
        provider,
        symbol,
        asset_class
    );

    let client = build_http_client()?;

    let result = match provider.as_str() {
        "finnhub" => fetch_finnhub(&client, &symbol, &asset_class),
        _ => {
            let message = format!("Unsupported market data provider: {provider}");
            warn!(target: MARKET_LOG_TARGET, "{message}");
            Err(message)
        }
    };

    match &result {
        Ok(snapshot) => info!(
            target: MARKET_LOG_TARGET,
            "get_market_snapshot success provider={} symbol={} price={:?} as_of={:?}",
            snapshot.provider,
            snapshot.symbol,
            snapshot.price,
            snapshot.as_of
        ),
        Err(error) => error!(
            target: MARKET_LOG_TARGET,
            "get_market_snapshot failed provider={} symbol={} error={}",
            provider,
            symbol,
            error
        ),
    }

    result
}

#[tauri::command]
pub(crate) fn get_indices_overview(
    category: String,
    preferred_provider: Option<String>,
) -> Result<IndicesOverviewResponse, String> {
    load_env();
    info!(
        target: MARKET_LOG_TARGET,
        "get_indices_overview start category={} preferred_provider={:?}",
        category,
        preferred_provider
    );

    let selected_category = normalize_category(&category);
    let provider = resolve_indices_provider(preferred_provider.as_deref())?;
    let client = build_http_client()?;
    let definitions = definitions_for_category(&selected_category);
    info!(
        target: MARKET_LOG_TARGET,
        "get_indices_overview provider_resolved provider={} category={} definitions={}",
        provider,
        selected_category,
        definitions.len()
    );

    let (rows, unavailable_count) = match provider {
        "finnhub" => fetch_indices_with_symbol_loop(&client, &definitions)?,
        _ => return Err(format!("Unsupported market data provider: {provider}")),
    };

    let updated_at = rows.iter().find_map(|row| row.as_of.clone());
    let source_note = if unavailable_count == 0 {
        "Finnhub aggregated quotes".to_string()
    } else {
        format!(
            "Finnhub aggregated quotes · {} symbol(s) unavailable",
            unavailable_count
        )
    };

    let response = IndicesOverviewResponse {
        provider: provider.to_string(),
        category: selected_category,
        updated_at,
        source_note,
        categories: category_counts(),
        rows,
    };

    info!(
        target: MARKET_LOG_TARGET,
        "get_indices_overview success provider={} category={} rows={} updated_at={:?}",
        response.provider,
        response.category,
        response.rows.len(),
        response.updated_at
    );

    Ok(response)
}

fn build_http_client() -> Result<reqwest::blocking::Client, String> {
    debug!(
        target: MARKET_LOG_TARGET,
        "creating reqwest blocking client timeout_seconds=12"
    );
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(12))
        .build()
        .map_err(|error| {
            error!(
                target: MARKET_LOG_TARGET,
                "failed to create reqwest blocking client error={error}"
            );
            format!("Failed to create HTTP client: {error}")
        })
}

fn load_env() {
    ENV_LOADER.call_once(|| {
        match dotenvy::dotenv() {
            Ok(path) => info!(
                target: MARKET_LOG_TARGET,
                "loaded dotenv from default search path={}",
                path.display()
            ),
            Err(error) => debug!(
                target: MARKET_LOG_TARGET,
                "dotenv default load skipped error={error}"
            ),
        }

        if let Ok(current_dir) = env::current_dir() {
            let candidates: [PathBuf; 2] = [
                current_dir.join(".env"),
                current_dir.join("..").join(".env"),
            ];

            for path in candidates {
                if path.exists() {
                    match dotenvy::from_path(&path) {
                        Ok(_) => info!(
                            target: MARKET_LOG_TARGET,
                            "loaded dotenv from candidate path={}",
                            path.display()
                        ),
                        Err(error) => warn!(
                            target: MARKET_LOG_TARGET,
                            "failed to load dotenv candidate path={} error={error}",
                            path.display()
                        ),
                    }
                }
            }
        } else {
            warn!(
                target: MARKET_LOG_TARGET,
                "unable to resolve current_dir while loading dotenv candidates"
            );
        }
    });
}

fn env_key(names: &[&str]) -> Result<String, String> {
    for name in names {
        if let Ok(value) = env::var(name) {
            if !value.trim().is_empty() {
                debug!(
                    target: MARKET_LOG_TARGET,
                    "resolved api key env var name={name}"
                );
                return Ok(value);
            }
        }
    }

    let message = format!("Missing API key. Expected one of: {}", names.join(", "));
    warn!(target: MARKET_LOG_TARGET, "{message}");
    Err(message)
}

fn has_env_key(names: &[&str]) -> bool {
    names.iter().any(|name| {
        env::var(name)
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
    })
}

fn read_json_response<T: serde::de::DeserializeOwned>(
    response: reqwest::blocking::Response,
    provider: &str,
    operation: &str,
    subject: &str,
) -> Result<T, String> {
    let status = response.status();
    let body = response.text().map_err(|error| {
        let message =
            format!("{provider} {operation} response body read failed for {subject}: {error}");
        error!(target: MARKET_LOG_TARGET, "{message}");
        message
    })?;

    debug!(
        target: MARKET_LOG_TARGET,
        "http response provider={provider} operation={operation} subject={subject} status={} body_bytes={}",
        status,
        body.len()
    );

    if !status.is_success() {
        let body_preview = truncate_for_log(&body, 400);
        let message = format!(
            "{provider} {operation} returned HTTP {status} for {subject}. body={body_preview}"
        );
        error!(target: MARKET_LOG_TARGET, "{message}");
        return Err(message);
    }

    serde_json::from_str(&body).map_err(|error| {
        let body_preview = truncate_for_log(&body, 400);
        let message = format!(
            "{provider} {operation} response parse failed for {subject}: {error}. body={body_preview}"
        );
        error!(target: MARKET_LOG_TARGET, "{message}");
        message
    })
}

fn request_json<T: serde::de::DeserializeOwned>(
    client: &reqwest::blocking::Client,
    url: String,
    provider: &str,
    operation: &str,
    subject: &str,
) -> Result<T, String> {
    debug!(
        target: MARKET_LOG_TARGET,
        "http request start provider={provider} operation={operation} subject={subject}"
    );

    let response = client.get(url).send().map_err(|error| {
        let message = format!("{provider} {operation} request failed for {subject}: {error}");
        error!(target: MARKET_LOG_TARGET, "{message}");
        message
    })?;

    read_json_response(response, provider, operation, subject)
}

fn truncate_for_log(value: &str, max_chars: usize) -> String {
    let mut truncated = value.chars().take(max_chars).collect::<String>();

    if value.chars().count() > max_chars {
        truncated.push_str("...");
    }

    truncated.replace('\n', "\\n")
}

fn resolve_indices_provider(preferred: Option<&str>) -> Result<&'static str, String> {
    if let Some(provider) = preferred
        .map(str::trim)
        .filter(|provider| !provider.is_empty())
    {
        info!(
            target: MARKET_LOG_TARGET,
            "resolving preferred indices provider requested={provider}"
        );
        return match provider {
            "finnhub" if has_env_key(&["FINNHUB_API_KEY", "FINNHUB_TOKEN"]) => {
                info!(
                    target: MARKET_LOG_TARGET,
                    "resolved indices provider requested={} selected=finnhub",
                    provider
                );
                Ok("finnhub")
            }
            "finnhub" => {
                let message =
                    "Finnhub API key is not configured for aggregated indices".to_string();
                warn!(target: MARKET_LOG_TARGET, "{message}");
                Err(message)
            }
            _ => {
                let message = format!("Unsupported market data provider: {provider}");
                warn!(target: MARKET_LOG_TARGET, "{message}");
                Err(message)
            }
        };
    }

    if has_env_key(&["FINNHUB_API_KEY", "FINNHUB_TOKEN"]) {
        info!(
            target: MARKET_LOG_TARGET,
            "resolved indices provider automatically selected=finnhub"
        );
        return Ok("finnhub");
    }

    let message = "Missing API key. Configure FINNHUB_API_KEY or FINNHUB_TOKEN.".to_string();
    error!(target: MARKET_LOG_TARGET, "{message}");
    Err(message)
}

fn normalize_category(category: &str) -> String {
    let normalized = category.trim().to_ascii_lowercase();

    if INDEX_CATEGORY_IDS.contains(&normalized.as_str()) {
        normalized
    } else {
        warn!(
            target: MARKET_LOG_TARGET,
            "unknown indices category requested={} defaulting_to=all",
            category
        );
        "all".to_string()
    }
}

fn fetch_indices_with_symbol_loop(
    client: &reqwest::blocking::Client,
    definitions: &[&IndexDefinition],
) -> Result<(Vec<IndexOverviewRow>, usize), String> {
    info!(
        target: MARKET_LOG_TARGET,
        "aggregated symbol loop start provider=finnhub symbols={}",
        definitions.len()
    );
    let mut rows = Vec::with_capacity(definitions.len());
    let mut unavailable_count = 0;

    for definition in definitions {
        let Some(provider_symbol) = finnhub_symbol(definition) else {
            warn!(
                target: MARKET_LOG_TARGET,
                "missing provider symbol mapping provider=finnhub index_id={} code={}",
                definition.id,
                definition.code
            );
            unavailable_count += 1;
            rows.push(index_row_from_snapshot(definition, None));
            continue;
        };

        let snapshot = fetch_finnhub(client, provider_symbol, "index");

        let snapshot = match snapshot {
            Ok(snapshot) => Some(snapshot),
            Err(error) => {
                unavailable_count += 1;
                warn!(
                    target: MARKET_LOG_TARGET,
                    "aggregated quote unavailable provider=finnhub provider_symbol={} index_code={} error={}",
                    provider_symbol,
                    definition.code,
                    error
                );
                None
            }
        };

        rows.push(index_row_from_snapshot(definition, snapshot));
    }

    Ok((rows, unavailable_count))
}

fn index_row_from_snapshot(
    definition: &IndexDefinition,
    snapshot: Option<MarketSnapshot>,
) -> IndexOverviewRow {
    let currency = snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.currency.clone())
        .or_else(|| Some(definition.currency.to_string()));
    let technical_rating = technical_rating(
        snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.change_percent),
    );

    IndexOverviewRow {
        id: definition.id.to_string(),
        symbol: definition.code.to_string(),
        name: definition.name.to_string(),
        region: definition.region.to_string(),
        currency,
        price: snapshot.as_ref().and_then(|snapshot| snapshot.price),
        change: snapshot.as_ref().and_then(|snapshot| snapshot.change),
        change_percent: snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.change_percent),
        open: snapshot.as_ref().and_then(|snapshot| snapshot.open),
        high: snapshot.as_ref().and_then(|snapshot| snapshot.high),
        low: snapshot.as_ref().and_then(|snapshot| snapshot.low),
        previous_close: snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.previous_close),
        as_of: snapshot.and_then(|snapshot| snapshot.as_of),
        technical_rating,
    }
}

fn technical_rating(change_percent: Option<f64>) -> String {
    match change_percent {
        Some(value) if value >= 2.0 => "Strong buy".to_string(),
        Some(value) if value >= 0.4 => "Buy".to_string(),
        Some(value) if value <= -2.0 => "Strong sell".to_string(),
        Some(value) if value <= -0.4 => "Sell".to_string(),
        _ => "Neutral".to_string(),
    }
}

fn fetch_finnhub(
    client: &reqwest::blocking::Client,
    symbol: &str,
    asset_class: &str,
) -> Result<MarketSnapshot, String> {
    let token = env_key(&["FINNHUB_API_KEY", "FINNHUB_TOKEN"])?;
    let url = format!(
        "https://finnhub.io/api/v1/quote?symbol={}&token={}",
        urlencoding::encode(symbol),
        urlencoding::encode(&token)
    );

    let quote: FinnhubQuote = request_json(client, url, "finnhub", "quote", symbol)?;

    Ok(MarketSnapshot {
        provider: "finnhub".to_string(),
        symbol: symbol.to_string(),
        asset_class: asset_class.to_string(),
        price: quote.c,
        change: quote.d,
        change_percent: quote.dp,
        open: quote.o,
        high: quote.h,
        low: quote.l,
        previous_close: quote.pc,
        volume: None,
        currency: None,
        as_of: quote.t.map(|timestamp| timestamp.to_string()),
        source_note: "Finnhub quote endpoint".to_string(),
    })
}
