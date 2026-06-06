use super::catalog::{
    INDEX_CATEGORY_IDS, IndexDefinition, category_counts, definitions_for_category,
    index_symbol_for_provider,
};
use super::models::{
    FinnhubQuote, IndexOverviewRow, IndicesOverviewResponse, MarketSnapshot, MassiveAggResponse,
    MassiveIndexSnapshot, MassiveIndicesSnapshotResponse, TwelveQuote,
};
use log::{debug, error, info, warn};
use serde_json::Value;
use std::{collections::HashMap, env, path::PathBuf, sync::Once, time::Duration};

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
        "massive" => fetch_massive(&client, &symbol, &asset_class),
        "twelvedata" => fetch_twelve_data(&client, &symbol, &asset_class),
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
        "twelvedata" => fetch_indices_with_twelvedata(&client, &definitions)?,
        "massive" => fetch_indices_with_massive_snapshot(&client, &definitions)?,
        "finnhub" => fetch_indices_with_symbol_loop(&client, &definitions, provider)?,
        _ => return Err(format!("Unsupported market data provider: {provider}")),
    };

    let updated_at = rows.iter().find_map(|row| row.as_of.clone());
    let source_note = if unavailable_count == 0 {
        format!("{} aggregated quotes", provider_label(provider))
    } else {
        format!(
            "{} aggregated quotes · {} symbol(s) unavailable",
            provider_label(provider),
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

fn provider_label(provider: &str) -> &'static str {
    match provider {
        "finnhub" => "Finnhub",
        "massive" => "Massive",
        "twelvedata" => "Twelve Data",
        _ => "Unknown",
    }
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
            "twelvedata" if has_env_key(&["TWELVE_DATA_API_KEY", "TWELVEDATA_API_KEY"]) => {
                info!(
                    target: MARKET_LOG_TARGET,
                    "resolved indices provider requested={} selected=twelvedata",
                    provider
                );
                Ok("twelvedata")
            }
            "massive" if has_env_key(&["MASSIVE_API_KEY", "POLYGON_API_KEY"]) => {
                info!(
                    target: MARKET_LOG_TARGET,
                    "resolved indices provider requested={} selected=massive",
                    provider
                );
                Ok("massive")
            }
            "finnhub" if has_env_key(&["FINNHUB_API_KEY", "FINNHUB_TOKEN"]) => {
                info!(
                    target: MARKET_LOG_TARGET,
                    "resolved indices provider requested={} selected=finnhub",
                    provider
                );
                Ok("finnhub")
            }
            "twelvedata" | "massive" | "finnhub" => {
                let message = format!(
                    "{} API key is not configured for aggregated indices",
                    provider_label(provider)
                );
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

    if has_env_key(&["TWELVE_DATA_API_KEY", "TWELVEDATA_API_KEY"]) {
        info!(
            target: MARKET_LOG_TARGET,
            "resolved indices provider automatically selected=twelvedata"
        );
        return Ok("twelvedata");
    }

    if has_env_key(&["MASSIVE_API_KEY", "POLYGON_API_KEY"]) {
        info!(
            target: MARKET_LOG_TARGET,
            "resolved indices provider automatically selected=massive"
        );
        return Ok("massive");
    }

    if has_env_key(&["FINNHUB_API_KEY", "FINNHUB_TOKEN"]) {
        info!(
            target: MARKET_LOG_TARGET,
            "resolved indices provider automatically selected=finnhub"
        );
        return Ok("finnhub");
    }

    let message =
        "Missing API key. Configure TWELVE_DATA_API_KEY, MASSIVE_API_KEY/POLYGON_API_KEY, or FINNHUB_API_KEY."
            .to_string();
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

fn normalize_symbol(symbol: &str) -> String {
    symbol.trim().to_ascii_uppercase()
}

fn fetch_indices_with_symbol_loop(
    client: &reqwest::blocking::Client,
    definitions: &[&IndexDefinition],
    provider: &str,
) -> Result<(Vec<IndexOverviewRow>, usize), String> {
    info!(
        target: MARKET_LOG_TARGET,
        "aggregated symbol loop start provider={} symbols={}",
        provider,
        definitions.len()
    );
    let mut rows = Vec::with_capacity(definitions.len());
    let mut unavailable_count = 0;

    for definition in definitions {
        let Some(provider_symbol) = index_symbol_for_provider(definition, provider) else {
            warn!(
                target: MARKET_LOG_TARGET,
                "missing provider symbol mapping provider={} index_id={} code={}",
                provider,
                definition.id,
                definition.code
            );
            unavailable_count += 1;
            rows.push(index_row_from_snapshot(definition, None));
            continue;
        };

        let snapshot = match provider {
            "finnhub" => fetch_finnhub(client, provider_symbol, "index"),
            "massive" => fetch_massive(client, provider_symbol, "index"),
            "twelvedata" => fetch_twelve_data(client, provider_symbol, "index"),
            _ => {
                let message = format!("Unsupported market data provider: {provider}");
                warn!(target: MARKET_LOG_TARGET, "{message}");
                Err(message)
            }
        };

        let snapshot = match snapshot {
            Ok(snapshot) => Some(snapshot),
            Err(error) => {
                unavailable_count += 1;
                warn!(
                    target: MARKET_LOG_TARGET,
                    "aggregated quote unavailable provider={} provider_symbol={} index_code={} error={}",
                    provider,
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

fn fetch_indices_with_massive_snapshot(
    client: &reqwest::blocking::Client,
    definitions: &[&IndexDefinition],
) -> Result<(Vec<IndexOverviewRow>, usize), String> {
    let token = env_key(&["MASSIVE_API_KEY", "POLYGON_API_KEY"])?;
    let symbols: Vec<&str> = definitions
        .iter()
        .filter_map(|definition| definition.symbols.massive)
        .collect();

    if symbols.is_empty() {
        warn!(
            target: MARKET_LOG_TARGET,
            "massive snapshot requested with zero mapped symbols; returning empty set"
        );
        return Ok((Vec::new(), 0));
    }

    let url = format!(
        "https://api.massive.com/v3/snapshot/indices?ticker.any_of={}&limit={}&sort=ticker&order=asc&apiKey={}",
        urlencoding::encode(&symbols.join(",")),
        symbols.len(),
        urlencoding::encode(&token)
    );

    let response = request_json::<MassiveIndicesSnapshotResponse>(
        client,
        url,
        "massive",
        "indices snapshot",
        &format!("{} symbols", symbols.len()),
    );

    let response = match response {
        Ok(response) => response,
        Err(error) => {
            warn!(
                target: MARKET_LOG_TARGET,
                "massive indices snapshot failed; falling back to per-symbol previous aggregate error={error}"
            );
            return fetch_indices_with_symbol_loop(client, definitions, "massive");
        }
    };

    let snapshot_map = response
        .results
        .unwrap_or_default()
        .into_iter()
        .filter_map(|snapshot| {
            let ticker = snapshot.ticker.as_deref()?;
            Some((normalize_symbol(ticker), snapshot))
        })
        .collect::<HashMap<_, _>>();

    debug!(
        target: MARKET_LOG_TARGET,
        "massive indices snapshot parsed symbols={}",
        snapshot_map.len()
    );

    let mut unavailable_count = 0;
    let rows = definitions
        .iter()
        .map(|definition| {
            let snapshot = definition
                .symbols
                .massive
                .and_then(|symbol| snapshot_map.get(&normalize_symbol(symbol)));

            match snapshot {
                Some(snapshot) => {
                    if snapshot.error.is_some() || snapshot.message.is_some() {
                        unavailable_count += 1;
                        warn!(
                            target: MARKET_LOG_TARGET,
                            "massive indices snapshot returned symbol error symbol={} error={:?} message={:?}",
                            definition.code,
                            snapshot.error,
                            snapshot.message
                        );
                    }

                    massive_index_row_from_snapshot(definition, snapshot)
                }
                None => {
                    unavailable_count += 1;
                    warn!(
                        target: MARKET_LOG_TARGET,
                        "massive indices snapshot missing symbol in response symbol={}",
                        definition.code
                    );
                    index_row_from_snapshot(definition, None)
                }
            }
        })
        .collect();

    Ok((rows, unavailable_count))
}

fn fetch_indices_with_twelvedata(
    client: &reqwest::blocking::Client,
    definitions: &[&IndexDefinition],
) -> Result<(Vec<IndexOverviewRow>, usize), String> {
    let token = env_key(&["TWELVE_DATA_API_KEY", "TWELVEDATA_API_KEY"])?;
    let symbols: Vec<&str> = definitions
        .iter()
        .filter_map(|definition| definition.symbols.twelvedata)
        .collect();

    if symbols.is_empty() {
        return Ok((Vec::new(), 0));
    }

    let url = format!(
        "https://api.twelvedata.com/quote?symbol={}&apikey={}",
        urlencoding::encode(&symbols.join(",")),
        urlencoding::encode(&token)
    );

    let value: Value = request_json(
        client,
        url,
        "twelvedata",
        "bulk quote",
        &format!("{} symbols", symbols.len()),
    )?;

    match parse_twelvedata_bulk_quotes(value) {
        Ok(snapshots) => {
            let mut unavailable_count = 0;
            let rows = definitions
                .iter()
                .map(|definition| {
                    let snapshot = definition
                        .symbols
                        .twelvedata
                        .and_then(|symbol| snapshots.get(&normalize_symbol(symbol)).cloned());

                    if snapshot.is_none() {
                        unavailable_count += 1;
                        warn!(
                            target: MARKET_LOG_TARGET,
                            "bulk quote missing symbol in response provider=twelvedata symbol={}",
                            definition.code
                        );
                    }

                    index_row_from_snapshot(definition, snapshot)
                })
                .collect();

            Ok((rows, unavailable_count))
        }
        Err(error) => {
            warn!(
                target: MARKET_LOG_TARGET,
                "twelvedata bulk parse failed; falling back to symbol loop error={error}"
            );
            fetch_indices_with_symbol_loop(client, definitions, "twelvedata")
        }
    }
}

fn parse_twelvedata_bulk_quotes(value: Value) -> Result<HashMap<String, MarketSnapshot>, String> {
    let mut snapshots = HashMap::new();

    match value {
        Value::Object(object) => {
            if let Some(message) = object.get("message").and_then(Value::as_str) {
                return Err(format!("Twelve Data returned an error: {message}"));
            }

            if object
                .get("status")
                .and_then(Value::as_str)
                .is_some_and(|status| status.eq_ignore_ascii_case("error"))
            {
                let message = object
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("Unknown Twelve Data error");

                return Err(format!("Twelve Data returned an error: {message}"));
            }

            if object.contains_key("symbol") {
                let quote: TwelveQuote = serde_json::from_value(Value::Object(object))
                    .map_err(|error| format!("Twelve Data quote parse failed: {error}"))?;
                let snapshot = market_snapshot_from_twelve_quote(quote, "index");
                snapshots.insert(normalize_symbol(&snapshot.symbol), snapshot);
                return Ok(snapshots);
            }

            for (symbol_key, quote_value) in object {
                let quote: TwelveQuote = serde_json::from_value(quote_value)
                    .map_err(|error| format!("Twelve Data bulk quote parse failed: {error}"))?;
                let snapshot =
                    market_snapshot_from_twelve_quote_with_fallback(quote, "index", &symbol_key);
                snapshots.insert(normalize_symbol(&snapshot.symbol), snapshot);
            }
        }
        Value::Array(array) => {
            for item in array {
                let quote: TwelveQuote = serde_json::from_value(item)
                    .map_err(|error| format!("Twelve Data bulk quote parse failed: {error}"))?;
                let snapshot = market_snapshot_from_twelve_quote(quote, "index");
                snapshots.insert(normalize_symbol(&snapshot.symbol), snapshot);
            }
        }
        _ => return Err("Unexpected Twelve Data bulk response format".to_string()),
    }

    debug!(
        target: MARKET_LOG_TARGET,
        "parsed twelvedata bulk quotes count={}",
        snapshots.len()
    );

    Ok(snapshots)
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

fn massive_index_row_from_snapshot(
    definition: &IndexDefinition,
    snapshot: &MassiveIndexSnapshot,
) -> IndexOverviewRow {
    let session = snapshot.session.as_ref();
    let change_percent = session.and_then(|session| session.change_percent);

    IndexOverviewRow {
        id: definition.id.to_string(),
        symbol: definition.code.to_string(),
        name: snapshot
            .name
            .clone()
            .unwrap_or_else(|| definition.name.to_string()),
        region: definition.region.to_string(),
        currency: Some(definition.currency.to_string()),
        price: snapshot
            .value
            .or_else(|| session.and_then(|session| session.close)),
        change: session.and_then(|session| session.change),
        change_percent,
        open: session.and_then(|session| session.open),
        high: session.and_then(|session| session.high),
        low: session.and_then(|session| session.low),
        previous_close: session.and_then(|session| session.previous_close.or(session.close)),
        as_of: snapshot.last_updated.map(|timestamp| timestamp.to_string()),
        technical_rating: technical_rating(change_percent),
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

fn fetch_massive(
    client: &reqwest::blocking::Client,
    symbol: &str,
    asset_class: &str,
) -> Result<MarketSnapshot, String> {
    let token = env_key(&["MASSIVE_API_KEY", "POLYGON_API_KEY"])?;
    let url = format!(
        "https://api.massive.com/v2/aggs/ticker/{}/prev?adjusted=true&apiKey={}",
        urlencoding::encode(symbol),
        urlencoding::encode(&token)
    );

    let response: MassiveAggResponse =
        request_json(client, url, "massive", "previous aggregate", symbol)?;

    let result = response
        .results
        .and_then(|mut results| results.pop())
        .ok_or_else(|| {
            let message = "Massive returned no aggregate data for this symbol".to_string();
            warn!(
                target: MARKET_LOG_TARGET,
                "massive previous aggregate empty symbol={symbol}"
            );
            message
        })?;

    let change = match (result.c, result.o) {
        (Some(close), Some(open)) => Some(close - open),
        _ => None,
    };
    let change_percent = match (change, result.o) {
        (Some(change), Some(open)) if open != 0.0 => Some((change / open) * 100.0),
        _ => None,
    };

    Ok(MarketSnapshot {
        provider: "massive".to_string(),
        symbol: symbol.to_string(),
        asset_class: asset_class.to_string(),
        price: result.c,
        change,
        change_percent,
        open: result.o,
        high: result.h,
        low: result.l,
        previous_close: result.o,
        volume: result.v,
        currency: None,
        as_of: result.t.map(|timestamp| timestamp.to_string()),
        source_note: "Massive previous aggregate endpoint".to_string(),
    })
}

fn fetch_twelve_data(
    client: &reqwest::blocking::Client,
    symbol: &str,
    asset_class: &str,
) -> Result<MarketSnapshot, String> {
    let token = env_key(&["TWELVE_DATA_API_KEY", "TWELVEDATA_API_KEY"])?;
    let url = format!(
        "https://api.twelvedata.com/quote?symbol={}&apikey={}",
        urlencoding::encode(symbol),
        urlencoding::encode(&token)
    );

    let quote: TwelveQuote = request_json(client, url, "twelvedata", "quote", symbol)?;

    if let Some(message) = quote.message.clone() {
        error!(
            target: MARKET_LOG_TARGET,
            "twelvedata quote returned error symbol={} message={}",
            symbol,
            message
        );
        return Err(format!("Twelve Data returned an error: {message}"));
    }

    Ok(market_snapshot_from_twelve_quote_with_fallback(
        quote,
        asset_class,
        symbol,
    ))
}

fn market_snapshot_from_twelve_quote(quote: TwelveQuote, asset_class: &str) -> MarketSnapshot {
    market_snapshot_from_twelve_quote_with_fallback(quote, asset_class, "UNKNOWN")
}

fn market_snapshot_from_twelve_quote_with_fallback(
    quote: TwelveQuote,
    asset_class: &str,
    fallback_symbol: &str,
) -> MarketSnapshot {
    MarketSnapshot {
        provider: "twelvedata".to_string(),
        symbol: quote.symbol.unwrap_or_else(|| fallback_symbol.to_string()),
        asset_class: asset_class.to_string(),
        price: parse_number(quote.close),
        change: parse_number(quote.change),
        change_percent: parse_number(quote.percent_change),
        open: parse_number(quote.open),
        high: parse_number(quote.high),
        low: parse_number(quote.low),
        previous_close: parse_number(quote.previous_close),
        volume: parse_number(quote.volume),
        currency: quote.currency,
        as_of: quote.datetime,
        source_note: "Twelve Data quote endpoint".to_string(),
    }
}

fn parse_number(value: Option<String>) -> Option<f64> {
    value.and_then(|value| match value.parse::<f64>() {
        Ok(number) => Some(number),
        Err(error) => {
            warn!(
                target: MARKET_LOG_TARGET,
                "failed to parse numeric field value={} error={error}",
                value
            );
            None
        }
    })
}
