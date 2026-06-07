use super::asset_catalog::{
    AssetDefinition, AssetFetchKind, category_counts as asset_category_counts,
    definition_by_id as asset_definition_by_id, definitions_for_asset, supported_assets,
    tradingview_symbol as asset_tradingview_symbol,
};
use super::catalog::{
    INDEX_CATEGORY_IDS, IndexDefinition, category_counts,
    definition_by_id as index_definition_by_id, definitions_for_category, finnhub_symbol,
    tradingview_symbol as index_tradingview_symbol,
};
use super::models::{
    AssetOverviewResponse, AssetOverviewRow, FinnhubQuote, IndexOverviewRow,
    IndicesOverviewResponse, MarketItemDetailResponse, MarketSnapshot, MarketTableColumn,
    MarketViewTab,
};
use log::{debug, error, info, warn};
use serde_json::{Map, Value};
use std::{
    collections::HashMap,
    env,
    path::PathBuf,
    sync::{Mutex, Once, OnceLock},
    time::{Duration, Instant},
};

pub(crate) const MARKET_LOG_TARGET: &str = "astraquant::market_data";

static ENV_LOADER: Once = Once::new();
static SNAPSHOT_CACHE: OnceLock<Mutex<HashMap<String, CachedSnapshot>>> = OnceLock::new();
static ALPHA_RATE_LIMITER: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();

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
    preferred_provider: Option<String>,
) -> Result<AssetOverviewResponse, String> {
    tauri::async_runtime::spawn_blocking(move || get_asset_overview_sync(asset, preferred_provider))
        .await
        .map_err(|error| format!("Asset overview worker failed: {error}"))?
}

fn get_asset_overview_sync(
    asset: String,
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
    let provider = resolve_asset_provider(&selected_asset, preferred_provider.as_deref())?;
    let client = build_http_client()?;
    let definitions = definitions_for_asset(&selected_asset)
        .ok_or_else(|| format!("Unsupported asset overview request: {}", selected_asset))?;

    info!(
        target: MARKET_LOG_TARGET,
        "get_asset_overview provider_resolved provider={} asset={} definitions={}",
        provider,
        selected_asset,
        definitions.len()
    );

    let (rows, unavailable_count) =
        fetch_assets_with_provider(&client, provider, &selected_asset, definitions)?;

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
        let provider = resolve_indices_provider(preferred_provider.as_deref())?;
        let snapshot = fetch_index_snapshot(&client, definition)?;

        market_detail_from_index(definition, snapshot, provider)
    } else {
        let asset = normalize_asset(&selected_kind)?;
        let definition = asset_definition_by_id(&asset, &item_id)
            .ok_or_else(|| format!("Unknown market detail item id: {item_id}"))?;
        let provider = resolve_asset_provider(&asset, preferred_provider.as_deref())?;
        let snapshot = fetch_asset_snapshot(&client, provider, &asset, definition)?;

        market_detail_from_asset(&asset, definition, snapshot, provider)
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

fn request_alpha_json(
    client: &reqwest::blocking::Client,
    url: String,
    operation: &str,
    subject: &str,
) -> Result<Value, String> {
    wait_for_alpha_slot();
    let value: Value = request_json(client, url, "alpha-vantage", operation, subject)?;

    if let Some(message) = value
        .get("Error Message")
        .and_then(Value::as_str)
        .or_else(|| value.get("Note").and_then(Value::as_str))
        .or_else(|| value.get("Information").and_then(Value::as_str))
    {
        let full_message =
            format!("Alpha Vantage {operation} request failed for {subject}: {message}");
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

fn resolve_asset_provider(asset: &str, preferred: Option<&str>) -> Result<&'static str, String> {
    if let Some(provider) = preferred
        .map(str::trim)
        .filter(|provider| !provider.is_empty())
    {
        info!(
            target: MARKET_LOG_TARGET,
            "resolving preferred asset provider asset={} requested={provider}",
            asset
        );

        return match provider {
            "alpha-vantage" if has_env_key(&["ALPHA_API_KEY"]) => {
                if asset_supports_provider(asset, "alpha-vantage") {
                    Ok("alpha-vantage")
                } else {
                    Err(format!(
                        "Alpha Vantage aggregated overview is not supported for asset {asset}"
                    ))
                }
            }
            "alpha-vantage" => Err("Alpha Vantage API key is not configured".to_string()),
            "finnhub" if has_env_key(&["FINNHUB_API_KEY", "FINNHUB_TOKEN"]) => {
                if asset_supports_provider(asset, "finnhub") {
                    Ok("finnhub")
                } else {
                    Err(format!(
                        "Finnhub aggregated overview is not supported for asset {asset}"
                    ))
                }
            }
            "finnhub" => Err("Finnhub API key is not configured".to_string()),
            _ => Err(format!("Unsupported market data provider: {provider}")),
        };
    }

    if asset_supports_provider(asset, "alpha-vantage") && has_env_key(&["ALPHA_API_KEY"]) {
        info!(
            target: MARKET_LOG_TARGET,
            "resolved asset provider automatically selected=alpha-vantage asset={asset}"
        );
        return Ok("alpha-vantage");
    }

    if asset_supports_provider(asset, "finnhub")
        && has_env_key(&["FINNHUB_API_KEY", "FINNHUB_TOKEN"])
    {
        info!(
            target: MARKET_LOG_TARGET,
            "resolved asset provider automatically selected=finnhub asset={asset}"
        );
        return Ok("finnhub");
    }

    Err(format!(
        "Missing API key. Configure a supported provider for asset {}.",
        asset
    ))
}

fn asset_supports_provider(asset: &str, provider: &str) -> bool {
    matches!(
        (asset, provider),
        ("stocks", "alpha-vantage")
            | ("stocks", "finnhub")
            | ("etf", "alpha-vantage")
            | ("etf", "finnhub")
            | ("crypto", "alpha-vantage")
            | ("futures", "alpha-vantage")
    )
}

fn provider_label(provider: &str) -> &'static str {
    match provider {
        "alpha-vantage" => "Alpha Vantage",
        "finnhub" => "Finnhub",
        _ => "Unknown provider",
    }
}

fn default_market_tabs() -> Vec<MarketViewTab> {
    vec![
        MarketViewTab {
            id: "overview".to_string(),
            label_key: "indicesTabOverview".to_string(),
        },
        MarketViewTab {
            id: "performance".to_string(),
            label_key: "indicesTabPerformance".to_string(),
        },
        MarketViewTab {
            id: "technicals".to_string(),
            label_key: "indicesTabTechnicals".to_string(),
        },
    ]
}

fn default_index_columns() -> Vec<MarketTableColumn> {
    vec![
        MarketTableColumn {
            id: "symbol".to_string(),
            label_key: "indicesTableSymbol".to_string(),
            align: "left".to_string(),
        },
        MarketTableColumn {
            id: "price".to_string(),
            label_key: "indicesTablePrice".to_string(),
            align: "right".to_string(),
        },
        MarketTableColumn {
            id: "change_percent".to_string(),
            label_key: "indicesTableChangePct".to_string(),
            align: "right".to_string(),
        },
        MarketTableColumn {
            id: "change".to_string(),
            label_key: "indicesTableChange".to_string(),
            align: "right".to_string(),
        },
        MarketTableColumn {
            id: "high".to_string(),
            label_key: "indicesTableHigh".to_string(),
            align: "right".to_string(),
        },
        MarketTableColumn {
            id: "low".to_string(),
            label_key: "indicesTableLow".to_string(),
            align: "right".to_string(),
        },
        MarketTableColumn {
            id: "technical_rating".to_string(),
            label_key: "indicesTableTechRating".to_string(),
            align: "right".to_string(),
        },
    ]
}

fn normalize_asset(asset: &str) -> Result<String, String> {
    let normalized = asset.trim().to_ascii_lowercase();

    if supported_assets().contains(&normalized.as_str()) {
        Ok(normalized)
    } else {
        Err(format!("Unsupported market overview asset: {asset}"))
    }
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

fn fetch_assets_with_provider(
    client: &reqwest::blocking::Client,
    provider: &str,
    asset: &str,
    definitions: &[AssetDefinition],
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
    }?;

    store_cached_snapshot(cache_key, &snapshot);
    Ok(snapshot)
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
        snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.change_percent),
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
    provider: &str,
) -> Result<MarketItemDetailResponse, String> {
    Ok(MarketItemDetailResponse {
        provider: provider.to_string(),
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
        technical_rating: technical_rating(snapshot.change_percent),
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

fn market_detail_from_index(
    definition: &IndexDefinition,
    snapshot: MarketSnapshot,
    provider: &str,
) -> Result<MarketItemDetailResponse, String> {
    Ok(MarketItemDetailResponse {
        provider: provider.to_string(),
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
        technical_rating: technical_rating(snapshot.change_percent),
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
