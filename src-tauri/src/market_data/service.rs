use super::asset_catalog::{
    AssetDefinition, AssetFetchKind, category_counts as asset_category_counts,
    definition_by_id as asset_definition_by_id, definitions_for_asset,
    tradingview_symbol as asset_tradingview_symbol,
};
use super::catalog::{
    IndexDefinition, category_counts, definition_by_id as index_definition_by_id,
    definitions_for_category, finnhub_symbol, tradingview_symbol as index_tradingview_symbol,
};
use super::models::{
    AssetOverviewResponse, AssetOverviewRow, FinnhubCandleResponse, FinnhubQuote, IndexOverviewRow,
    IndicesOverviewResponse, MarketChartPoint, MarketChartSeriesResponse, MarketItemDetailResponse,
    MarketSnapshot,
};
use super::resolve::{
    default_index_columns, default_market_tabs, normalize_asset, normalize_category,
    provider_label, resolve_asset_provider, resolve_indices_provider,
};
use log::{debug, error, info, warn};
use serde_json::{Map, Value};
use std::{
    collections::HashMap,
    env, fs,
    path::PathBuf,
    process::Command,
    sync::{Mutex, Once, OnceLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

pub(crate) const MARKET_LOG_TARGET: &str = "astraquant::market_data";

static ENV_LOADER: Once = Once::new();
static SNAPSHOT_CACHE: OnceLock<Mutex<HashMap<String, CachedSnapshot>>> = OnceLock::new();
static ALPHA_RATE_LIMITER: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
static ALPHA_DAILY_LIMIT_REACHED: OnceLock<Mutex<bool>> = OnceLock::new();
static FINNHUB_RATE_LIMIT_UNTIL: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
static FINNHUB_CANDLE_ACCESS_DENIED: OnceLock<Mutex<bool>> = OnceLock::new();

#[derive(Clone)]
struct CachedSnapshot {
    snapshot: MarketSnapshot,
    inserted_at: Instant,
}

#[tauri::command]
pub(crate) async fn get_market_snapshot(
    provider: String,
    symbol: String,
    asset_class: String,
) -> Result<MarketSnapshot, String> {
    tauri::async_runtime::spawn_blocking(move || {
        get_market_snapshot_sync(provider, symbol, asset_class)
    })
    .await
    .map_err(|error| format!("Market snapshot worker failed: {error}"))?
}

fn get_market_snapshot_sync(
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
pub(crate) async fn get_indices_overview(
    category: String,
    preferred_provider: Option<String>,
) -> Result<IndicesOverviewResponse, String> {
    tauri::async_runtime::spawn_blocking(move || {
        get_indices_overview_sync(category, preferred_provider)
    })
    .await
    .map_err(|error| format!("Indices overview worker failed: {error}"))?
}

fn get_indices_overview_sync(
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
        "tradingview" => fetch_indices_with_tradingview_loop(&definitions)?,
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
        title_key: "indicesTitle".to_string(),
        description_key: "indicesDescription".to_string(),
        updated_at,
        source_note,
        categories: category_counts(),
        tabs: default_market_tabs(),
        columns: default_index_columns(),
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

#[tauri::command]
pub(crate) async fn get_asset_overview(
    asset: String,
    category: Option<String>,
    preferred_provider: Option<String>,
) -> Result<AssetOverviewResponse, String> {
    tauri::async_runtime::spawn_blocking(move || {
        get_asset_overview_sync(asset, category, preferred_provider)
    })
    .await
    .map_err(|error| format!("Asset overview worker failed: {error}"))?
}

fn get_asset_overview_sync(
    asset: String,
    category: Option<String>,
    preferred_provider: Option<String>,
) -> Result<AssetOverviewResponse, String> {
    load_env();
    info!(
        target: MARKET_LOG_TARGET,
        "get_asset_overview start asset={} preferred_provider={:?}",
        asset,
        preferred_provider
    );

    let selected_asset = normalize_asset(&asset)?;
    let selected_category = category
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "all")
        .map(str::to_string);
    let provider = resolve_asset_provider(&selected_asset, preferred_provider.as_deref())?;
    let client = build_http_client()?;
    let definitions = definitions_for_asset(&selected_asset)
        .ok_or_else(|| format!("Unsupported asset overview request: {}", selected_asset))?;
    let filtered_definitions = if let Some(category_id) = selected_category.as_deref() {
        definitions
            .iter()
            .filter(|definition| definition.category == category_id)
            .collect::<Vec<_>>()
    } else {
        definitions.iter().collect::<Vec<_>>()
    };

    info!(
        target: MARKET_LOG_TARGET,
        "get_asset_overview provider_resolved provider={} asset={} category={:?} definitions={}",
        provider,
        selected_asset,
        selected_category,
        filtered_definitions.len()
    );

    let (rows, unavailable_count) =
        fetch_assets_with_provider(&client, provider, &selected_asset, &filtered_definitions)?;

    let updated_at = rows.iter().find_map(|row| row.as_of.clone());
    let source_note = if unavailable_count == 0 {
        format!("{} aggregated market overview", provider_label(provider))
    } else {
        format!(
            "{} aggregated market overview · {} symbol(s) unavailable",
            provider_label(provider),
            unavailable_count
        )
    };
    let categories = asset_category_counts(&selected_asset);

    let response = AssetOverviewResponse {
        provider: provider.to_string(),
        asset: selected_asset,
        updated_at,
        source_note,
        categories,
        tabs: default_market_tabs(),
        rows,
    };

    info!(
        target: MARKET_LOG_TARGET,
        "get_asset_overview success provider={} asset={} rows={} updated_at={:?}",
        response.provider,
        response.asset,
        response.rows.len(),
        response.updated_at
    );

    Ok(response)
}

#[tauri::command]
pub(crate) async fn get_market_item_detail(
    kind: String,
    item_id: String,
    preferred_provider: Option<String>,
) -> Result<MarketItemDetailResponse, String> {
    tauri::async_runtime::spawn_blocking(move || {
        get_market_item_detail_sync(kind, item_id, preferred_provider)
    })
    .await
    .map_err(|error| format!("Market detail worker failed: {error}"))?
}

fn get_market_item_detail_sync(
    kind: String,
    item_id: String,
    preferred_provider: Option<String>,
) -> Result<MarketItemDetailResponse, String> {
    load_env();
    let selected_kind = kind.trim().to_ascii_lowercase();
    let client = build_http_client()?;

    info!(
        target: MARKET_LOG_TARGET,
        "get_market_item_detail start kind={} item_id={} preferred_provider={:?}",
        selected_kind,
        item_id,
        preferred_provider
    );

    let response = if selected_kind == "indices" {
        let definition = index_definition_by_id(&item_id)
            .ok_or_else(|| format!("Unknown index detail item id: {item_id}"))?;
        let requested_provider = preferred_provider
            .as_deref()
            .map(str::trim)
            .filter(|provider| !provider.is_empty());

        if requested_provider == Some("tradingview") {
            let chart = fetch_tradingview_chart_series(
                "indices",
                definition.id,
                definition.code,
                index_tradingview_symbol(definition.id),
            )?;

            market_detail_from_index(definition, market_snapshot_from_chart_series(chart))
        } else {
            resolve_indices_provider(preferred_provider.as_deref())?;
            let snapshot = fetch_index_snapshot(&client, definition)?;

            market_detail_from_index(definition, snapshot)
        }
    } else {
        let asset = normalize_asset(&selected_kind)?;
        let definition = asset_definition_by_id(&asset, &item_id)
            .ok_or_else(|| format!("Unknown market detail item id: {item_id}"))?;
        let requested_provider = preferred_provider
            .as_deref()
            .map(str::trim)
            .filter(|provider| !provider.is_empty());

        if requested_provider == Some("tradingview") {
            let chart = fetch_tradingview_chart_series(
                &asset,
                definition.id,
                definition.code,
                asset_tradingview_symbol(&asset, definition.id),
            )?;

            market_detail_from_asset(&asset, definition, market_snapshot_from_chart_series(chart))
        } else {
            let provider = resolve_asset_provider(&asset, preferred_provider.as_deref())?;
            let snapshot = fetch_asset_snapshot(&client, provider, &asset, definition)?;

            market_detail_from_asset(&asset, definition, snapshot)
        }
    }?;

    info!(
        target: MARKET_LOG_TARGET,
        "get_market_item_detail success kind={} item_id={} provider={} tradingview_symbol={:?}",
        response.kind,
        response.id,
        response.provider,
        response.tradingview_symbol
    );

    Ok(response)
}

#[tauri::command]
pub(crate) async fn get_market_chart_series(
    kind: String,
    item_id: String,
    preferred_provider: Option<String>,
) -> Result<MarketChartSeriesResponse, String> {
    tauri::async_runtime::spawn_blocking(move || {
        get_market_chart_series_sync(kind, item_id, preferred_provider)
    })
    .await
    .map_err(|error| format!("Market chart worker failed: {error}"))?
}

fn get_market_chart_series_sync(
    kind: String,
    item_id: String,
    preferred_provider: Option<String>,
) -> Result<MarketChartSeriesResponse, String> {
    load_env();
    let selected_kind = kind.trim().to_ascii_lowercase();
    let client = build_http_client()?;

    info!(
        target: MARKET_LOG_TARGET,
        "get_market_chart_series start kind={} item_id={} preferred_provider={:?}",
        selected_kind,
        item_id,
        preferred_provider
    );

    let requested_provider = preferred_provider
        .as_deref()
        .map(str::trim)
        .filter(|provider| !provider.is_empty());

    let response = if selected_kind == "indices" {
        let definition = index_definition_by_id(&item_id)
            .ok_or_else(|| format!("Unknown index chart item id: {item_id}"))?;
        if requested_provider == Some("tradingview") {
            fetch_tradingview_chart_series(
                "indices",
                definition.id,
                definition.code,
                index_tradingview_symbol(definition.id),
            )
        } else {
            let provider = resolve_indices_provider(preferred_provider.as_deref())?;
            fetch_index_chart_series(&client, provider, definition)
        }
    } else {
        let asset = normalize_asset(&selected_kind)?;
        let definition = asset_definition_by_id(&asset, &item_id)
            .ok_or_else(|| format!("Unknown market chart item id: {item_id}"))?;
        if requested_provider == Some("tradingview") {
            fetch_tradingview_chart_series(
                &asset,
                definition.id,
                definition.code,
                asset_tradingview_symbol(&asset, definition.id),
            )
        } else {
            let provider = resolve_asset_provider(&asset, preferred_provider.as_deref())?;
            fetch_asset_chart_series(&client, provider, &asset, definition)
        }
    }?;

    info!(
        target: MARKET_LOG_TARGET,
        "get_market_chart_series success kind={} item_id={} provider={} points={} series_type={}",
        response.kind,
        response.id,
        response.provider,
        response.points.len(),
        response.series_type
    );

    Ok(response)
}

fn market_snapshot_from_chart_series(chart: MarketChartSeriesResponse) -> MarketSnapshot {
    let latest = chart.points.last();
    let previous = if chart.points.len() >= 2 {
        chart.points.get(chart.points.len() - 2)
    } else {
        None
    };
    let latest_close = latest.and_then(|point| point.close.or(point.value));
    let previous_close = previous.and_then(|point| point.close.or(point.value));
    let change = match (latest_close, previous_close) {
        (Some(current), Some(previous_value)) => Some(current - previous_value),
        _ => None,
    };
    let change_percent = match (latest_close, previous_close) {
        (Some(current), Some(previous_value)) if previous_value.abs() >= f64::EPSILON => {
            Some((current - previous_value) / previous_value * 100.0)
        }
        _ => None,
    };

    MarketSnapshot {
        provider: chart.provider,
        symbol: chart.symbol,
        asset_class: chart.kind,
        price: latest_close,
        change,
        change_percent,
        open: latest.and_then(|point| point.open),
        high: latest.and_then(|point| point.high),
        low: latest.and_then(|point| point.low),
        previous_close,
        volume: None,
        currency: chart.currency,
        as_of: latest.map(|point| point.time.clone()),
        technical_rating: chart.technical_rating,
        source_note: chart.source_note,
    }
}

fn fetch_tradingview_chart_series(
    kind: &str,
    id: &str,
    symbol: &str,
    tradingview_symbol: Option<&str>,
) -> Result<MarketChartSeriesResponse, String> {
    let tradingview_symbol = tradingview_symbol
        .ok_or_else(|| format!("Missing TradingView symbol mapping for {kind}/{id}"))?;
    let script_path = resolve_tradingview_bridge_script()?;

    info!(
        target: MARKET_LOG_TARGET,
        "launching tradingview bridge kind={} id={} symbol={} tradingview_symbol={}",
        kind,
        id,
        symbol,
        tradingview_symbol
    );

    let output =
        Command::new("node")
            .arg(&script_path)
            .arg(tradingview_symbol)
            .arg("D")
            .current_dir(script_path.parent().ok_or_else(|| {
                "TradingView bridge script parent directory is missing".to_string()
            })?)
            .output()
            .map_err(|error| format!("Failed to start TradingView bridge with node: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let detail = if !stderr.is_empty() { stderr } else { stdout };
        return Err(format!(
            "TradingView bridge failed for {tradingview_symbol}: {detail}"
        ));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| format!("TradingView bridge returned non-utf8 output: {error}"))?;
    let payload = stdout.trim();

    let mut response = serde_json::from_str::<MarketChartSeriesResponse>(payload)
        .map_err(|error| format!("TradingView bridge response parse failed: {error}"))?;

    response.kind = kind.to_string();
    response.id = id.to_string();
    response.symbol = symbol.to_string();

    Ok(response)
}

fn resolve_tradingview_bridge_script() -> Result<PathBuf, String> {
    let current_dir = env::current_dir().map_err(|error| {
        format!("Unable to resolve current_dir for TradingView bridge: {error}")
    })?;
    let candidates = [
        current_dir.join("scripts").join("tradingview-chart.cjs"),
        current_dir
            .join("..")
            .join("scripts")
            .join("tradingview-chart.cjs"),
    ];

    for path in candidates {
        if fs::metadata(&path).is_ok() {
            return Ok(path);
        }
    }

    Err(
        "TradingView bridge script was not found. Expected scripts/tradingview-chart.cjs."
            .to_string(),
    )
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

fn request_alpha_json(
    client: &reqwest::blocking::Client,
    url: String,
    operation: &str,
    subject: &str,
) -> Result<Value, String> {
    if alpha_daily_limit_reached() {
        return Err(
            "Alpha Vantage daily rate limit already reached for current app session.".to_string(),
        );
    }

    wait_for_alpha_slot();
    let value: Value = request_json(client, url, "alpha-vantage", operation, subject)?;

    if let Some(message) = value
        .get("Error Message")
        .and_then(Value::as_str)
        .or_else(|| value.get("Note").and_then(Value::as_str))
        .or_else(|| value.get("Information").and_then(Value::as_str))
    {
        let sanitized_message = sanitize_provider_message(message);

        if is_alpha_daily_limit_message(&sanitized_message) {
            mark_alpha_daily_limit_reached();
        }

        let full_message =
            format!("Alpha Vantage {operation} request failed for {subject}: {sanitized_message}");
        warn!(target: MARKET_LOG_TARGET, "{full_message}");
        return Err(full_message);
    }

    Ok(value)
}

fn truncate_for_log(value: &str, max_chars: usize) -> String {
    let mut truncated = value.chars().take(max_chars).collect::<String>();

    if value.chars().count() > max_chars {
        truncated.push_str("...");
    }

    truncated.replace('\n', "\\n")
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

fn fetch_indices_with_tradingview_loop(
    definitions: &[&IndexDefinition],
) -> Result<(Vec<IndexOverviewRow>, usize), String> {
    info!(
        target: MARKET_LOG_TARGET,
        "aggregated symbol loop start provider=tradingview symbols={}",
        definitions.len()
    );
    let mut rows = Vec::with_capacity(definitions.len());
    let mut unavailable_count = 0;

    for definition in definitions {
        let snapshot = match index_tradingview_symbol(definition.id) {
            Some(symbol) => match fetch_tradingview_snapshot("indices", definition.id, definition.code, symbol) {
                Ok(snapshot) => Some(snapshot),
                Err(error) => {
                    unavailable_count += 1;
                    warn!(
                        target: MARKET_LOG_TARGET,
                        "aggregated quote unavailable provider=tradingview provider_symbol={} index_code={} error={}",
                        symbol,
                        definition.code,
                        error
                    );
                    None
                }
            },
            None => {
                unavailable_count += 1;
                warn!(
                    target: MARKET_LOG_TARGET,
                    "missing provider symbol mapping provider=tradingview index_id={} code={}",
                    definition.id,
                    definition.code
                );
                None
            }
        };

        rows.push(index_row_from_snapshot(definition, snapshot));
    }

    Ok((rows, unavailable_count))
}

fn fetch_assets_with_provider(
    client: &reqwest::blocking::Client,
    provider: &str,
    asset: &str,
    definitions: &[&AssetDefinition],
) -> Result<(Vec<AssetOverviewRow>, usize), String> {
    let mut rows = Vec::with_capacity(definitions.len());
    let mut unavailable_count = 0;

    for definition in definitions {
        let snapshot = match fetch_asset_snapshot(client, provider, asset, definition) {
            Ok(snapshot) => Some(snapshot),
            Err(error) => {
                unavailable_count += 1;
                warn!(
                    target: MARKET_LOG_TARGET,
                    "asset overview quote unavailable provider={} asset={} symbol={} error={}",
                    provider,
                    asset,
                    definition.code,
                    error
                );
                None
            }
        };

        rows.push(asset_row_from_snapshot(definition, snapshot));
    }

    Ok((rows, unavailable_count))
}

fn fetch_asset_snapshot(
    client: &reqwest::blocking::Client,
    provider: &str,
    asset: &str,
    definition: &AssetDefinition,
) -> Result<MarketSnapshot, String> {
    let cache_key = format!("{}:{}:{}", provider, asset, definition.id);
    let ttl = snapshot_ttl(provider, definition.fetch_kind);

    if let Some(snapshot) = read_cached_snapshot(&cache_key, ttl) {
        return Ok(snapshot);
    }

    let snapshot = match provider {
        "finnhub" => {
            let symbol = definition
                .finnhub_symbol
                .ok_or_else(|| format!("Missing Finnhub symbol mapping for {}", definition.code))?;
            fetch_finnhub(client, symbol, asset)
        }
        "tradingview" => {
            let symbol = asset_tradingview_symbol(asset, definition.id)
                .ok_or_else(|| format!("Missing TradingView symbol mapping for {}", definition.code))?;
            fetch_tradingview_snapshot(asset, definition.id, definition.code, symbol)
        }
        "alpha-vantage" => match definition.fetch_kind {
            AssetFetchKind::Quote => {
                let symbol = definition.alpha_symbol.ok_or_else(|| {
                    format!("Missing Alpha symbol mapping for {}", definition.code)
                })?;
                fetch_alpha_global_quote(client, symbol, asset, definition.currency)
            }
            AssetFetchKind::DigitalDaily => {
                let symbol = definition.alpha_symbol.ok_or_else(|| {
                    format!("Missing Alpha symbol mapping for {}", definition.code)
                })?;
                let market = definition.alpha_market.ok_or_else(|| {
                    format!("Missing Alpha market mapping for {}", definition.code)
                })?;
                fetch_alpha_digital_daily(client, symbol, market)
            }
            AssetFetchKind::CommoditySeries => {
                let function = definition.alpha_function.ok_or_else(|| {
                    format!("Missing Alpha commodity mapping for {}", definition.code)
                })?;
                fetch_alpha_commodity_series(client, function)
            }
        },
        _ => Err(format!("Unsupported market data provider: {provider}")),
    };
    let snapshot = match snapshot {
        Ok(snapshot) => snapshot,
        Err(error) if provider == "finnhub" && is_finnhub_rate_limit_message(&error) => {
            mark_finnhub_rate_limited(Duration::from_secs(60));
            return Err(error);
        }
        Err(error) => return Err(error),
    };

    store_cached_snapshot(cache_key, &snapshot);
    Ok(snapshot)
}

fn fetch_tradingview_snapshot(
    kind: &str,
    id: &str,
    symbol: &str,
    tradingview_symbol: &str,
) -> Result<MarketSnapshot, String> {
    fetch_tradingview_chart_series(kind, id, symbol, Some(tradingview_symbol))
        .map(market_snapshot_from_chart_series)
}

fn fetch_index_chart_series(
    client: &reqwest::blocking::Client,
    provider: &str,
    definition: &IndexDefinition,
) -> Result<MarketChartSeriesResponse, String> {
    match provider {
        "finnhub" => {
            let symbol = finnhub_symbol(definition).ok_or_else(|| {
                format!(
                    "Missing Finnhub chart symbol mapping for {}",
                    definition.code
                )
            })?;

            fetch_finnhub_candle_series(
                client,
                "indices",
                definition.id,
                definition.code,
                symbol,
                Some(definition.currency),
                "Finnhub daily candles",
            )
        }
        _ => Err(format!("Unsupported historical chart provider: {provider}")),
    }
}

fn fetch_asset_chart_series(
    client: &reqwest::blocking::Client,
    provider: &str,
    asset: &str,
    definition: &AssetDefinition,
) -> Result<MarketChartSeriesResponse, String> {
    let result = match provider {
        "finnhub" => {
            let symbol = definition.finnhub_symbol.ok_or_else(|| {
                format!(
                    "Missing Finnhub chart symbol mapping for {}",
                    definition.code
                )
            })?;

            fetch_finnhub_candle_series(
                client,
                asset,
                definition.id,
                definition.code,
                symbol,
                Some(definition.currency),
                "Finnhub daily candles",
            )
        }
        "alpha-vantage" => match definition.fetch_kind {
            AssetFetchKind::Quote => {
                let symbol = definition.alpha_symbol.ok_or_else(|| {
                    format!("Missing Alpha chart symbol mapping for {}", definition.code)
                })?;

                fetch_alpha_daily_chart_series(
                    client,
                    asset,
                    definition.id,
                    definition.code,
                    symbol,
                    definition.currency,
                )
            }
            AssetFetchKind::DigitalDaily => {
                let symbol = definition.alpha_symbol.ok_or_else(|| {
                    format!(
                        "Missing Alpha digital symbol mapping for {}",
                        definition.code
                    )
                })?;
                let market = definition.alpha_market.ok_or_else(|| {
                    format!(
                        "Missing Alpha digital market mapping for {}",
                        definition.code
                    )
                })?;

                fetch_alpha_digital_chart_series(
                    client,
                    asset,
                    definition.id,
                    definition.code,
                    symbol,
                    market,
                )
            }
            AssetFetchKind::CommoditySeries => {
                let function_name = definition.alpha_function.ok_or_else(|| {
                    format!(
                        "Missing Alpha commodity chart mapping for {}",
                        definition.code
                    )
                })?;

                fetch_alpha_commodity_chart_series(
                    client,
                    asset,
                    definition.id,
                    definition.code,
                    function_name,
                )
            }
        },
        _ => Err(format!("Unsupported historical chart provider: {provider}")),
    };

    match result {
        Ok(series) => Ok(series),
        Err(error) if provider == "finnhub" && is_finnhub_rate_limit_message(&error) => {
            mark_finnhub_rate_limited(Duration::from_secs(60));
            Err(error)
        }
        Err(error) if provider == "finnhub" && is_finnhub_candle_access_message(&error) => {
            mark_finnhub_candle_access_denied();
            Err(error)
        }
        Err(error) => Err(error),
    }
}

fn fetch_finnhub_candle_series(
    client: &reqwest::blocking::Client,
    kind: &str,
    id: &str,
    display_symbol: &str,
    provider_symbol: &str,
    currency: Option<&str>,
    source_note: &str,
) -> Result<MarketChartSeriesResponse, String> {
    if finnhub_rate_limit_active() {
        return Err("Finnhub rate limit is active. Please retry later.".to_string());
    }

    if finnhub_candle_access_denied() {
        return Err("Finnhub stock candle access is unavailable for current API key.".to_string());
    }

    let token = env_key(&["FINNHUB_API_KEY", "FINNHUB_TOKEN"])?;
    let to = current_unix_timestamp();
    let from = to.saturating_sub(60 * 60 * 24 * 220);
    let url = format!(
        "https://finnhub.io/api/v1/stock/candle?symbol={}&resolution=D&from={}&to={}&token={}",
        urlencoding::encode(provider_symbol),
        from,
        to,
        urlencoding::encode(&token)
    );

    let payload: FinnhubCandleResponse =
        match request_json(client, url, "finnhub", "stock_candle", provider_symbol) {
            Ok(payload) => payload,
            Err(error) => {
                if is_finnhub_rate_limit_message(&error) {
                    mark_finnhub_rate_limited(Duration::from_secs(60));
                }

                if is_finnhub_candle_access_message(&error) {
                    mark_finnhub_candle_access_denied();
                }

                return Err(error);
            }
        };

    if payload.s != "ok" {
        return Err(format!(
            "Finnhub stock candle response returned status={} for {}",
            payload.s, provider_symbol
        ));
    }

    let points = trim_chart_points(
        payload
            .t
            .iter()
            .enumerate()
            .filter_map(|(index, timestamp)| {
                let open = payload.o.get(index).copied().flatten()?;
                let high = payload.h.get(index).copied().flatten()?;
                let low = payload.l.get(index).copied().flatten()?;
                let close = payload.c.get(index).copied().flatten()?;

                Some(MarketChartPoint {
                    time: timestamp.to_string(),
                    open: Some(open),
                    high: Some(high),
                    low: Some(low),
                    close: Some(close),
                    value: None,
                })
            })
            .collect(),
        180,
    );

    if points.is_empty() {
        return Err(format!(
            "Finnhub stock candle series did not return OHLC data for {}",
            provider_symbol
        ));
    }

    Ok(MarketChartSeriesResponse {
        provider: "finnhub".to_string(),
        kind: kind.to_string(),
        id: id.to_string(),
        symbol: display_symbol.to_string(),
        interval: "1D".to_string(),
        series_type: "candlestick".to_string(),
        source_note: source_note.to_string(),
        currency: currency.map(str::to_string),
        technical_rating: None,
        points,
    })
}

fn fetch_alpha_daily_chart_series(
    client: &reqwest::blocking::Client,
    asset: &str,
    id: &str,
    display_symbol: &str,
    provider_symbol: &str,
    currency: &str,
) -> Result<MarketChartSeriesResponse, String> {
    let api_key = env_key(&["ALPHA_API_KEY"])?;
    let url = format!(
        "https://www.alphavantage.co/query?function=TIME_SERIES_DAILY&symbol={}&outputsize=full&apikey={}",
        urlencoding::encode(provider_symbol),
        urlencoding::encode(&api_key)
    );

    let payload = request_alpha_json(client, url, "time_series_daily", provider_symbol)?;
    let series = payload
        .get("Time Series (Daily)")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            format!("Alpha Vantage TIME_SERIES_DAILY payload missing for {provider_symbol}")
        })?;

    let points = trim_chart_points(
        alpha_daily_series_entries(series)
            .into_iter()
            .filter_map(|(date, entry)| {
                let open = parse_string_f64(entry.get("1. open"))?;
                let high = parse_string_f64(entry.get("2. high"))?;
                let low = parse_string_f64(entry.get("3. low"))?;
                let close = parse_string_f64(entry.get("4. close"))?;

                Some(MarketChartPoint {
                    time: date,
                    open: Some(open),
                    high: Some(high),
                    low: Some(low),
                    close: Some(close),
                    value: None,
                })
            })
            .collect(),
        180,
    );

    if points.is_empty() {
        return Err(format!(
            "Alpha Vantage TIME_SERIES_DAILY returned no OHLC rows for {}",
            provider_symbol
        ));
    }

    Ok(MarketChartSeriesResponse {
        provider: "alpha-vantage".to_string(),
        kind: asset.to_string(),
        id: id.to_string(),
        symbol: display_symbol.to_string(),
        interval: "1D".to_string(),
        series_type: "candlestick".to_string(),
        source_note: "Alpha Vantage TIME_SERIES_DAILY".to_string(),
        currency: Some(currency.to_string()),
        technical_rating: None,
        points,
    })
}

fn fetch_alpha_digital_chart_series(
    client: &reqwest::blocking::Client,
    asset: &str,
    id: &str,
    display_symbol: &str,
    provider_symbol: &str,
    market: &str,
) -> Result<MarketChartSeriesResponse, String> {
    let api_key = env_key(&["ALPHA_API_KEY"])?;
    let url = format!(
        "https://www.alphavantage.co/query?function=DIGITAL_CURRENCY_DAILY&symbol={}&market={}&apikey={}",
        urlencoding::encode(provider_symbol),
        urlencoding::encode(market),
        urlencoding::encode(&api_key)
    );

    let subject = format!("{provider_symbol}/{market}");
    let payload = request_alpha_json(client, url, "digital_currency_daily", &subject)?;
    let series = payload
        .get("Time Series (Digital Currency Daily)")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            format!("Alpha Vantage DIGITAL_CURRENCY_DAILY payload missing for {subject}")
        })?;

    let points = trim_chart_points(
        alpha_daily_series_entries(series)
            .into_iter()
            .filter_map(|(date, entry)| {
                let open = parse_digital_metric(entry, "open", market)?;
                let high = parse_digital_metric(entry, "high", market)?;
                let low = parse_digital_metric(entry, "low", market)?;
                let close = parse_digital_metric(entry, "close", market)?;

                Some(MarketChartPoint {
                    time: date,
                    open: Some(open),
                    high: Some(high),
                    low: Some(low),
                    close: Some(close),
                    value: None,
                })
            })
            .collect(),
        180,
    );

    if points.is_empty() {
        return Err(format!(
            "Alpha Vantage DIGITAL_CURRENCY_DAILY returned no OHLC rows for {}",
            subject
        ));
    }

    Ok(MarketChartSeriesResponse {
        provider: "alpha-vantage".to_string(),
        kind: asset.to_string(),
        id: id.to_string(),
        symbol: display_symbol.to_string(),
        interval: "1D".to_string(),
        series_type: "candlestick".to_string(),
        source_note: "Alpha Vantage DIGITAL_CURRENCY_DAILY".to_string(),
        currency: Some(market.to_string()),
        technical_rating: None,
        points,
    })
}

fn fetch_alpha_commodity_chart_series(
    client: &reqwest::blocking::Client,
    asset: &str,
    id: &str,
    display_symbol: &str,
    function_name: &str,
) -> Result<MarketChartSeriesResponse, String> {
    let api_key = env_key(&["ALPHA_API_KEY"])?;
    let url = format!(
        "https://www.alphavantage.co/query?function={}&interval=daily&apikey={}",
        urlencoding::encode(function_name),
        urlencoding::encode(&api_key)
    );

    let payload = request_alpha_json(client, url, "commodity_series", function_name)?;
    let rows = payload
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("Alpha Vantage commodity payload missing for {function_name}"))?;

    let mut sorted_rows = rows.iter().filter_map(Value::as_object).collect::<Vec<_>>();
    sorted_rows.sort_by(|left, right| {
        let left_date = left.get("date").and_then(Value::as_str).unwrap_or_default();
        let right_date = right
            .get("date")
            .and_then(Value::as_str)
            .unwrap_or_default();
        left_date.cmp(right_date)
    });

    let points = trim_chart_points(
        sorted_rows
            .into_iter()
            .filter_map(|row| {
                let date = row.get("date").and_then(Value::as_str)?;
                let value = parse_string_f64(row.get("value"))?;

                Some(MarketChartPoint {
                    time: date.to_string(),
                    open: None,
                    high: None,
                    low: None,
                    close: None,
                    value: Some(value),
                })
            })
            .collect(),
        180,
    );

    if points.is_empty() {
        return Err(format!(
            "Alpha Vantage commodity series returned no rows for {}",
            function_name
        ));
    }

    Ok(MarketChartSeriesResponse {
        provider: "alpha-vantage".to_string(),
        kind: asset.to_string(),
        id: id.to_string(),
        symbol: display_symbol.to_string(),
        interval: "1D".to_string(),
        series_type: "area".to_string(),
        source_note: "Alpha Vantage commodity daily series".to_string(),
        currency: Some("USD".to_string()),
        technical_rating: None,
        points,
    })
}

fn alpha_daily_series_entries<'a>(
    series: &'a Map<String, Value>,
) -> Vec<(String, &'a Map<String, Value>)> {
    let mut entries = series
        .iter()
        .filter_map(|(date, value)| value.as_object().map(|entry| (date.clone(), entry)))
        .collect::<Vec<_>>();

    entries.sort_by(|left, right| left.0.cmp(&right.0));
    entries
}

fn trim_chart_points(
    mut points: Vec<MarketChartPoint>,
    max_points: usize,
) -> Vec<MarketChartPoint> {
    if points.len() > max_points {
        points.drain(0..points.len().saturating_sub(max_points));
    }

    points
}

fn current_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}

fn snapshot_ttl(provider: &str, fetch_kind: AssetFetchKind) -> Duration {
    match (provider, fetch_kind) {
        ("alpha-vantage", AssetFetchKind::Quote) => Duration::from_secs(60 * 15),
        ("alpha-vantage", AssetFetchKind::DigitalDaily) => Duration::from_secs(60 * 30),
        ("alpha-vantage", AssetFetchKind::CommoditySeries) => Duration::from_secs(60 * 60),
        ("finnhub", _) => Duration::from_secs(60 * 5),
        _ => Duration::from_secs(60 * 10),
    }
}

fn snapshot_cache() -> &'static Mutex<HashMap<String, CachedSnapshot>> {
    SNAPSHOT_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn alpha_rate_limiter() -> &'static Mutex<Option<Instant>> {
    ALPHA_RATE_LIMITER.get_or_init(|| Mutex::new(None))
}

fn alpha_daily_limit_state() -> &'static Mutex<bool> {
    ALPHA_DAILY_LIMIT_REACHED.get_or_init(|| Mutex::new(false))
}

fn finnhub_rate_limit_state() -> &'static Mutex<Option<Instant>> {
    FINNHUB_RATE_LIMIT_UNTIL.get_or_init(|| Mutex::new(None))
}

fn finnhub_candle_access_state() -> &'static Mutex<bool> {
    FINNHUB_CANDLE_ACCESS_DENIED.get_or_init(|| Mutex::new(false))
}

fn alpha_daily_limit_reached() -> bool {
    alpha_daily_limit_state()
        .lock()
        .map(|state| *state)
        .unwrap_or(false)
}

fn mark_alpha_daily_limit_reached() {
    if let Ok(mut state) = alpha_daily_limit_state().lock() {
        *state = true;
    }
}

fn finnhub_rate_limit_active() -> bool {
    let Ok(mut state) = finnhub_rate_limit_state().lock() else {
        return false;
    };

    match *state {
        Some(until) if Instant::now() < until => true,
        Some(_) => {
            *state = None;
            false
        }
        None => false,
    }
}

fn mark_finnhub_rate_limited(cooldown: Duration) {
    if let Ok(mut state) = finnhub_rate_limit_state().lock() {
        *state = Some(Instant::now() + cooldown);
    }
}

fn finnhub_candle_access_denied() -> bool {
    finnhub_candle_access_state()
        .lock()
        .map(|state| *state)
        .unwrap_or(false)
}

fn mark_finnhub_candle_access_denied() {
    if let Ok(mut state) = finnhub_candle_access_state().lock() {
        *state = true;
    }
}

fn read_cached_snapshot(key: &str, ttl: Duration) -> Option<MarketSnapshot> {
    let Ok(cache) = snapshot_cache().lock() else {
        return None;
    };

    let entry = cache.get(key)?;
    if entry.inserted_at.elapsed() <= ttl {
        return Some(entry.snapshot.clone());
    }

    None
}

fn store_cached_snapshot(key: String, snapshot: &MarketSnapshot) {
    let Ok(mut cache) = snapshot_cache().lock() else {
        return;
    };

    cache.insert(
        key,
        CachedSnapshot {
            snapshot: snapshot.clone(),
            inserted_at: Instant::now(),
        },
    );
}

fn wait_for_alpha_slot() {
    loop {
        let now = Instant::now();
        let Ok(mut limiter) = alpha_rate_limiter().lock() else {
            return;
        };

        match *limiter {
            Some(last_request) => {
                let elapsed = now.saturating_duration_since(last_request);
                if elapsed < Duration::from_millis(1_200) {
                    let wait_for = Duration::from_millis(1_200) - elapsed;
                    drop(limiter);
                    std::thread::sleep(wait_for);
                    continue;
                }
            }
            None => {}
        }

        *limiter = Some(now);
        return;
    }
}

fn asset_row_from_snapshot(
    definition: &AssetDefinition,
    snapshot: Option<MarketSnapshot>,
) -> AssetOverviewRow {
    let currency = snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.currency.clone())
        .or_else(|| Some(definition.currency.to_string()));
    let technical_rating = technical_rating(
        snapshot.as_ref(),
    );

    AssetOverviewRow {
        id: definition.id.to_string(),
        category_id: definition.category.to_string(),
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

fn market_detail_from_asset(
    asset: &str,
    definition: &AssetDefinition,
    snapshot: MarketSnapshot,
) -> Result<MarketItemDetailResponse, String> {
    Ok(MarketItemDetailResponse {
        provider: snapshot.provider.clone(),
        kind: asset.to_string(),
        id: definition.id.to_string(),
        symbol: definition.code.to_string(),
        name: definition.name.to_string(),
        region: definition.region.to_string(),
        currency: snapshot
            .currency
            .clone()
            .or_else(|| Some(definition.currency.to_string())),
        price: snapshot.price,
        change: snapshot.change,
        change_percent: snapshot.change_percent,
        open: snapshot.open,
        high: snapshot.high,
        low: snapshot.low,
        previous_close: snapshot.previous_close,
        as_of: snapshot.as_of.clone(),
        source_note: snapshot.source_note.clone(),
        technical_rating: technical_rating(Some(&snapshot)),
        tradingview_symbol: asset_tradingview_symbol(asset, definition.id).map(str::to_string),
    })
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
        snapshot.as_ref(),
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

fn market_detail_from_index(
    definition: &IndexDefinition,
    snapshot: MarketSnapshot,
) -> Result<MarketItemDetailResponse, String> {
    Ok(MarketItemDetailResponse {
        provider: snapshot.provider.clone(),
        kind: "indices".to_string(),
        id: definition.id.to_string(),
        symbol: definition.code.to_string(),
        name: definition.name.to_string(),
        region: definition.region.to_string(),
        currency: snapshot
            .currency
            .clone()
            .or_else(|| Some(definition.currency.to_string())),
        price: snapshot.price,
        change: snapshot.change,
        change_percent: snapshot.change_percent,
        open: snapshot.open,
        high: snapshot.high,
        low: snapshot.low,
        previous_close: snapshot.previous_close,
        as_of: snapshot.as_of.clone(),
        source_note: snapshot.source_note.clone(),
        technical_rating: technical_rating(Some(&snapshot)),
        tradingview_symbol: index_tradingview_symbol(definition.id).map(str::to_string),
    })
}

fn fetch_index_snapshot(
    client: &reqwest::blocking::Client,
    definition: &IndexDefinition,
) -> Result<MarketSnapshot, String> {
    let symbol = finnhub_symbol(definition)
        .ok_or_else(|| format!("Missing Finnhub symbol mapping for {}", definition.code))?;

    fetch_finnhub(client, symbol, "index")
}


fn technical_rating(snapshot: Option<&MarketSnapshot>) -> String {
    if let Some(value) = snapshot.and_then(|snapshot| snapshot.technical_rating.clone()) {
        return value;
    }

    match snapshot.and_then(|snapshot| snapshot.change_percent) {
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
    if finnhub_rate_limit_active() {
        return Err("Finnhub rate limit is active. Please retry later.".to_string());
    }

    let token = env_key(&["FINNHUB_API_KEY", "FINNHUB_TOKEN"])?;
    let url = format!(
        "https://finnhub.io/api/v1/quote?symbol={}&token={}",
        urlencoding::encode(symbol),
        urlencoding::encode(&token)
    );

    let quote: FinnhubQuote = match request_json(client, url, "finnhub", "quote", symbol) {
        Ok(quote) => quote,
        Err(error) => {
            if is_finnhub_rate_limit_message(&error) {
                mark_finnhub_rate_limited(Duration::from_secs(60));
            }

            return Err(error);
        }
    };

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
        technical_rating: None,
        source_note: "Finnhub quote endpoint".to_string(),
    })
}

fn fetch_alpha_global_quote(
    client: &reqwest::blocking::Client,
    symbol: &str,
    asset_class: &str,
    currency: &str,
) -> Result<MarketSnapshot, String> {
    let api_key = env_key(&["ALPHA_API_KEY"])?;
    let url = format!(
        "https://www.alphavantage.co/query?function=GLOBAL_QUOTE&symbol={}&apikey={}",
        urlencoding::encode(symbol),
        urlencoding::encode(&api_key)
    );

    let payload = request_alpha_json(client, url, "global_quote", symbol)?;
    let quote = payload
        .get("Global Quote")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("Alpha Vantage GLOBAL_QUOTE payload missing for {symbol}"))?;

    Ok(MarketSnapshot {
        provider: "alpha-vantage".to_string(),
        symbol: symbol.to_string(),
        asset_class: asset_class.to_string(),
        price: parse_string_f64(quote.get("05. price")),
        change: parse_string_f64(quote.get("09. change")),
        change_percent: parse_percent_f64(quote.get("10. change percent")),
        open: parse_string_f64(quote.get("02. open")),
        high: parse_string_f64(quote.get("03. high")),
        low: parse_string_f64(quote.get("04. low")),
        previous_close: parse_string_f64(quote.get("08. previous close")),
        volume: parse_string_f64(quote.get("06. volume")),
        currency: Some(currency.to_string()),
        as_of: quote
            .get("07. latest trading day")
            .and_then(Value::as_str)
            .map(str::to_string),
        technical_rating: None,
        source_note: "Alpha Vantage GLOBAL_QUOTE".to_string(),
    })
}

fn fetch_alpha_digital_daily(
    client: &reqwest::blocking::Client,
    symbol: &str,
    market: &str,
) -> Result<MarketSnapshot, String> {
    let api_key = env_key(&["ALPHA_API_KEY"])?;
    let url = format!(
        "https://www.alphavantage.co/query?function=DIGITAL_CURRENCY_DAILY&symbol={}&market={}&apikey={}",
        urlencoding::encode(symbol),
        urlencoding::encode(market),
        urlencoding::encode(&api_key)
    );

    let subject = format!("{symbol}/{market}");
    let payload = request_alpha_json(client, url, "digital_currency_daily", &subject)?;
    let series = payload
        .get("Time Series (Digital Currency Daily)")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            format!("Alpha Vantage DIGITAL_CURRENCY_DAILY payload missing for {subject}")
        })?;
    let (latest_date, latest_entry, previous_entry) = first_two_series_entries(series, &subject)?;

    let latest_close = parse_digital_metric(latest_entry, "close", market).ok_or_else(|| {
        format!("Alpha Vantage DIGITAL_CURRENCY_DAILY latest close missing for {subject}")
    })?;
    let previous_close = parse_digital_metric(previous_entry, "close", market);
    let change = previous_close.map(|value| latest_close - value);
    let change_percent = previous_close.and_then(|value| {
        if value.abs() < f64::EPSILON {
            None
        } else {
            Some((latest_close - value) / value * 100.0)
        }
    });

    Ok(MarketSnapshot {
        provider: "alpha-vantage".to_string(),
        symbol: subject,
        asset_class: "crypto".to_string(),
        price: Some(latest_close),
        change,
        change_percent,
        open: parse_digital_metric(latest_entry, "open", market),
        high: parse_digital_metric(latest_entry, "high", market),
        low: parse_digital_metric(latest_entry, "low", market),
        previous_close,
        volume: None,
        currency: Some(market.to_string()),
        as_of: Some(latest_date.to_string()),
        technical_rating: None,
        source_note: "Alpha Vantage DIGITAL_CURRENCY_DAILY".to_string(),
    })
}

fn fetch_alpha_commodity_series(
    client: &reqwest::blocking::Client,
    function_name: &str,
) -> Result<MarketSnapshot, String> {
    let api_key = env_key(&["ALPHA_API_KEY"])?;
    let url = format!(
        "https://www.alphavantage.co/query?function={}&interval=daily&apikey={}",
        urlencoding::encode(function_name),
        urlencoding::encode(&api_key)
    );

    let payload = request_alpha_json(client, url, "commodity_series", function_name)?;
    let rows = payload
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("Alpha Vantage commodity payload missing for {function_name}"))?;

    let mut sorted_rows = rows.iter().filter_map(Value::as_object).collect::<Vec<_>>();
    sorted_rows.sort_by(|left, right| {
        let left_date = left.get("date").and_then(Value::as_str).unwrap_or_default();
        let right_date = right
            .get("date")
            .and_then(Value::as_str)
            .unwrap_or_default();
        right_date.cmp(left_date)
    });

    if sorted_rows.len() < 2 {
        return Err(format!(
            "Alpha Vantage commodity payload did not return enough rows for {function_name}"
        ));
    }

    let latest = sorted_rows
        .first()
        .copied()
        .ok_or_else(|| format!("Alpha Vantage commodity latest row missing for {function_name}"))?;
    let previous = sorted_rows.get(1).copied().ok_or_else(|| {
        format!("Alpha Vantage commodity previous row missing for {function_name}")
    })?;

    let latest_value = parse_string_f64(latest.get("value")).ok_or_else(|| {
        format!("Alpha Vantage commodity latest value missing for {function_name}")
    })?;
    let previous_value = parse_string_f64(previous.get("value"));
    let change = previous_value.map(|value| latest_value - value);
    let change_percent = previous_value.and_then(|value| {
        if value.abs() < f64::EPSILON {
            None
        } else {
            Some((latest_value - value) / value * 100.0)
        }
    });

    Ok(MarketSnapshot {
        provider: "alpha-vantage".to_string(),
        symbol: function_name.to_string(),
        asset_class: "futures".to_string(),
        price: Some(latest_value),
        change,
        change_percent,
        open: None,
        high: None,
        low: None,
        previous_close: previous_value,
        volume: None,
        currency: Some("USD".to_string()),
        as_of: latest
            .get("date")
            .and_then(Value::as_str)
            .map(str::to_string),
        technical_rating: None,
        source_note: "Alpha Vantage commodity daily series".to_string(),
    })
}

fn first_two_series_entries<'a>(
    series: &'a Map<String, Value>,
    subject: &str,
) -> Result<(&'a String, &'a Map<String, Value>, &'a Map<String, Value>), String> {
    let mut entries = series
        .iter()
        .filter_map(|(date, value)| value.as_object().map(|entry| (date, entry)))
        .collect::<Vec<_>>();

    entries.sort_by(|left, right| right.0.cmp(left.0));

    let (latest_date, latest_entry) = entries
        .first()
        .copied()
        .ok_or_else(|| format!("Alpha Vantage daily series returned no entries for {subject}"))?;
    let (_, previous_entry) = entries.get(1).copied().ok_or_else(|| {
        format!("Alpha Vantage daily series returned only one entry for {subject}")
    })?;

    Ok((latest_date, latest_entry, previous_entry))
}

fn parse_string_f64(value: Option<&Value>) -> Option<f64> {
    value.and_then(|value| {
        value
            .as_str()
            .and_then(|value| value.replace(',', "").trim().parse::<f64>().ok())
            .or_else(|| value.as_f64())
    })
}

fn parse_percent_f64(value: Option<&Value>) -> Option<f64> {
    value.and_then(|value| {
        value
            .as_str()
            .and_then(|value| value.trim().trim_end_matches('%').parse::<f64>().ok())
    })
}

fn parse_digital_metric(entry: &Map<String, Value>, metric: &str, market: &str) -> Option<f64> {
    let metric = metric.to_ascii_lowercase();
    let market_marker = format!("({})", market.to_ascii_uppercase());

    for (key, value) in entry {
        let normalized = key.to_ascii_lowercase();
        if normalized.contains(&metric) && normalized.contains(&market_marker.to_ascii_lowercase())
        {
            if let Some(parsed) = parse_string_f64(Some(value)) {
                return Some(parsed);
            }
        }
    }

    entry.iter().find_map(|(key, value)| {
        let normalized = key.to_ascii_lowercase();
        if normalized.contains(&metric) {
            parse_string_f64(Some(value))
        } else {
            None
        }
    })
}

fn is_alpha_daily_limit_message(message: &str) -> bool {
    let normalized = message.to_ascii_lowercase();
    normalized.contains("25 requests per day")
        || normalized.contains("daily rate limit")
        || normalized.contains("standard api rate limit")
}

fn is_finnhub_rate_limit_message(message: &str) -> bool {
    let normalized = message.to_ascii_lowercase();
    (normalized.contains("finnhub") || normalized.contains("api limit reached"))
        && (normalized.contains("http 429")
            || normalized.contains("too many requests")
            || normalized.contains("api limit reached")
            || normalized.contains("remaining limit: 0")
            || normalized.contains("rate limit"))
}

fn is_finnhub_candle_access_message(message: &str) -> bool {
    let normalized = message.to_ascii_lowercase();
    (normalized.contains("finnhub") || normalized.contains("stock_candle"))
        && (normalized.contains("http 403")
            || normalized.contains("403 forbidden")
            || normalized.contains("you don't have access to this resource")
            || normalized.contains("stock candle access is unavailable"))
}

fn sanitize_provider_message(message: &str) -> String {
    let mut sanitized = message.to_string();

    if let Ok(key) = env::var("ALPHA_API_KEY") {
        if !key.trim().is_empty() {
            sanitized = sanitized.replace(&key, "[redacted]");
        }
    }

    sanitized
}
