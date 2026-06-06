use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, env, path::PathBuf, time::Duration};
use tauri::{self, Manager};

#[tauri::command]
fn show_window(window: tauri::Window) -> Result<(), String> {
    if window.is_visible().unwrap() {
        return Ok(());
    }

    window
        .show()
        .map_err(|error| format!("Failed to show window: {error}"))?;

    Ok(())
}

#[derive(Debug, Serialize, Clone)]
struct MarketSnapshot {
    provider: String,
    symbol: String,
    asset_class: String,
    price: Option<f64>,
    change: Option<f64>,
    change_percent: Option<f64>,
    open: Option<f64>,
    high: Option<f64>,
    low: Option<f64>,
    previous_close: Option<f64>,
    volume: Option<f64>,
    currency: Option<String>,
    as_of: Option<String>,
    source_note: String,
}

#[derive(Debug, Serialize)]
struct IndexCategoryCount {
    id: String,
    total: usize,
}

#[derive(Debug, Serialize)]
struct IndexOverviewRow {
    id: String,
    symbol: String,
    name: String,
    region: String,
    currency: Option<String>,
    price: Option<f64>,
    change: Option<f64>,
    change_percent: Option<f64>,
    open: Option<f64>,
    high: Option<f64>,
    low: Option<f64>,
    previous_close: Option<f64>,
    as_of: Option<String>,
    technical_rating: String,
}

#[derive(Debug, Serialize)]
struct IndicesOverviewResponse {
    provider: String,
    category: String,
    updated_at: Option<String>,
    source_note: String,
    categories: Vec<IndexCategoryCount>,
    rows: Vec<IndexOverviewRow>,
}

struct ProviderSymbols {
    finnhub: Option<&'static str>,
    massive: Option<&'static str>,
    twelvedata: Option<&'static str>,
}

struct IndexDefinition {
    id: &'static str,
    code: &'static str,
    name: &'static str,
    region: &'static str,
    currency: &'static str,
    categories: &'static [&'static str],
    symbols: ProviderSymbols,
}

const INDEX_CATEGORY_IDS: &[&str] = &[
    "all",
    "major",
    "us",
    "sp-sectors",
    "currency",
    "americas",
    "europe",
    "asia",
    "pacific",
    "middle-east",
    "africa",
];

const INDEX_DEFINITIONS: &[IndexDefinition] = &[
    IndexDefinition {
        id: "spx",
        code: "SPX",
        name: "S&P 500",
        region: "United States",
        currency: "USD",
        categories: &["all", "major", "us", "americas"],
        symbols: ProviderSymbols {
            finnhub: Some("^GSPC"),
            massive: Some("I:SPX"),
            twelvedata: Some("SPX"),
        },
    },
    IndexDefinition {
        id: "ixic",
        code: "IXIC",
        name: "US Composite Index",
        region: "United States",
        currency: "USD",
        categories: &["all", "major", "us", "americas"],
        symbols: ProviderSymbols {
            finnhub: Some("^IXIC"),
            massive: Some("I:IXIC"),
            twelvedata: Some("IXIC"),
        },
    },
    IndexDefinition {
        id: "dji",
        code: "DJI",
        name: "Dow Jones Industrial Average Index",
        region: "United States",
        currency: "USD",
        categories: &["all", "major", "us", "americas"],
        symbols: ProviderSymbols {
            finnhub: Some("^DJI"),
            massive: Some("I:DJI"),
            twelvedata: Some("DJI"),
        },
    },
    IndexDefinition {
        id: "vix",
        code: "VIX",
        name: "CBOE Volatility Index",
        region: "United States",
        currency: "USD",
        categories: &["all", "major", "us", "americas"],
        symbols: ProviderSymbols {
            finnhub: Some("^VIX"),
            massive: Some("I:VIX"),
            twelvedata: Some("VIX"),
        },
    },
    IndexDefinition {
        id: "tsx",
        code: "TSX",
        name: "S&P/TSX Composite Index",
        region: "Canada",
        currency: "CAD",
        categories: &["all", "major", "americas"],
        symbols: ProviderSymbols {
            finnhub: Some("^GSPTSE"),
            massive: Some("I:TSX"),
            twelvedata: Some("TSX"),
        },
    },
    IndexDefinition {
        id: "ukx",
        code: "UKX",
        name: "UK 100 Index",
        region: "United Kingdom",
        currency: "GBP",
        categories: &["all", "major", "europe"],
        symbols: ProviderSymbols {
            finnhub: Some("^FTSE"),
            massive: Some("I:UKX"),
            twelvedata: Some("UKX"),
        },
    },
    IndexDefinition {
        id: "dax",
        code: "DAX",
        name: "DAX Index",
        region: "Germany",
        currency: "EUR",
        categories: &["all", "major", "europe"],
        symbols: ProviderSymbols {
            finnhub: Some("^GDAXI"),
            massive: Some("I:DAX"),
            twelvedata: Some("DAX"),
        },
    },
    IndexDefinition {
        id: "px1",
        code: "PX1",
        name: "CAC 40 Index",
        region: "France",
        currency: "EUR",
        categories: &["all", "major", "europe"],
        symbols: ProviderSymbols {
            finnhub: Some("^FCHI"),
            massive: Some("I:PX1"),
            twelvedata: Some("PX1"),
        },
    },
    IndexDefinition {
        id: "ftmib",
        code: "FTMIB",
        name: "MILANO ITALIA BORSA INDEX",
        region: "Italy",
        currency: "EUR",
        categories: &["all", "major", "europe"],
        symbols: ProviderSymbols {
            finnhub: Some("FTSEMIB.MI"),
            massive: Some("I:FTMIB"),
            twelvedata: Some("FTMIB"),
        },
    },
    IndexDefinition {
        id: "n225",
        code: "N225",
        name: "Japan 225 Index",
        region: "Japan",
        currency: "JPY",
        categories: &["all", "major", "asia", "pacific"],
        symbols: ProviderSymbols {
            finnhub: Some("^N225"),
            massive: Some("I:N225"),
            twelvedata: Some("N225"),
        },
    },
    IndexDefinition {
        id: "kospi",
        code: "KOSPI",
        name: "KOREA COMPOSITE STOCK PRICE INDEX (KOSPI)",
        region: "South Korea",
        currency: "KRW",
        categories: &["all", "major", "asia"],
        symbols: ProviderSymbols {
            finnhub: Some("^KS11"),
            massive: Some("I:KOSPI"),
            twelvedata: Some("KOSPI"),
        },
    },
    IndexDefinition {
        id: "hsi",
        code: "HSI",
        name: "Hang Seng Index",
        region: "Hong Kong",
        currency: "HKD",
        categories: &["all", "asia"],
        symbols: ProviderSymbols {
            finnhub: Some("^HSI"),
            massive: Some("I:HSI"),
            twelvedata: Some("HSI"),
        },
    },
    IndexDefinition {
        id: "xjo",
        code: "XJO",
        name: "S&P/ASX 200",
        region: "Australia",
        currency: "AUD",
        categories: &["all", "pacific"],
        symbols: ProviderSymbols {
            finnhub: Some("^AXJO"),
            massive: Some("I:XJO"),
            twelvedata: Some("XJO"),
        },
    },
    IndexDefinition {
        id: "nz50",
        code: "NZ50",
        name: "S&P/NZX 50 Index",
        region: "New Zealand",
        currency: "NZD",
        categories: &["all", "pacific"],
        symbols: ProviderSymbols {
            finnhub: None,
            massive: Some("I:NZ50"),
            twelvedata: Some("NZ50"),
        },
    },
    IndexDefinition {
        id: "ta35",
        code: "TA35",
        name: "TA-35 Index",
        region: "Israel",
        currency: "ILS",
        categories: &["all", "middle-east"],
        symbols: ProviderSymbols {
            finnhub: None,
            massive: Some("I:TA35"),
            twelvedata: Some("TA35"),
        },
    },
    IndexDefinition {
        id: "jalsh",
        code: "JALSH",
        name: "FTSE/JSE All Share",
        region: "South Africa",
        currency: "ZAR",
        categories: &["all", "africa"],
        symbols: ProviderSymbols {
            finnhub: None,
            massive: Some("I:JALSH"),
            twelvedata: Some("JALSH"),
        },
    },
    IndexDefinition {
        id: "dxy",
        code: "DXY",
        name: "US Dollar Currency Index",
        region: "Global",
        currency: "USD",
        categories: &["all", "currency", "americas"],
        symbols: ProviderSymbols {
            finnhub: Some("DX-Y.NYB"),
            massive: Some("I:DXY"),
            twelvedata: Some("DXY"),
        },
    },
    IndexDefinition {
        id: "xlb",
        code: "XLB",
        name: "Materials Select Sector",
        region: "United States",
        currency: "USD",
        categories: &["all", "sp-sectors", "us"],
        symbols: ProviderSymbols {
            finnhub: Some("XLB"),
            massive: Some("XLB"),
            twelvedata: Some("XLB"),
        },
    },
    IndexDefinition {
        id: "xle",
        code: "XLE",
        name: "Energy Select Sector",
        region: "United States",
        currency: "USD",
        categories: &["all", "sp-sectors", "us"],
        symbols: ProviderSymbols {
            finnhub: Some("XLE"),
            massive: Some("XLE"),
            twelvedata: Some("XLE"),
        },
    },
    IndexDefinition {
        id: "xlf",
        code: "XLF",
        name: "Financial Select Sector",
        region: "United States",
        currency: "USD",
        categories: &["all", "sp-sectors", "us"],
        symbols: ProviderSymbols {
            finnhub: Some("XLF"),
            massive: Some("XLF"),
            twelvedata: Some("XLF"),
        },
    },
    IndexDefinition {
        id: "xlk",
        code: "XLK",
        name: "Technology Select Sector",
        region: "United States",
        currency: "USD",
        categories: &["all", "sp-sectors", "us"],
        symbols: ProviderSymbols {
            finnhub: Some("XLK"),
            massive: Some("XLK"),
            twelvedata: Some("XLK"),
        },
    },
    IndexDefinition {
        id: "xlv",
        code: "XLV",
        name: "Health Care Select Sector",
        region: "United States",
        currency: "USD",
        categories: &["all", "sp-sectors", "us"],
        symbols: ProviderSymbols {
            finnhub: Some("XLV"),
            massive: Some("XLV"),
            twelvedata: Some("XLV"),
        },
    },
    IndexDefinition {
        id: "xli",
        code: "XLI",
        name: "Industrial Select Sector",
        region: "United States",
        currency: "USD",
        categories: &["all", "sp-sectors", "us"],
        symbols: ProviderSymbols {
            finnhub: Some("XLI"),
            massive: Some("XLI"),
            twelvedata: Some("XLI"),
        },
    },
    IndexDefinition {
        id: "xlp",
        code: "XLP",
        name: "Consumer Staples Select Sector",
        region: "United States",
        currency: "USD",
        categories: &["all", "sp-sectors", "us"],
        symbols: ProviderSymbols {
            finnhub: Some("XLP"),
            massive: Some("XLP"),
            twelvedata: Some("XLP"),
        },
    },
    IndexDefinition {
        id: "xly",
        code: "XLY",
        name: "Consumer Discretionary Select Sector",
        region: "United States",
        currency: "USD",
        categories: &["all", "sp-sectors", "us"],
        symbols: ProviderSymbols {
            finnhub: Some("XLY"),
            massive: Some("XLY"),
            twelvedata: Some("XLY"),
        },
    },
    IndexDefinition {
        id: "xlu",
        code: "XLU",
        name: "Utilities Select Sector",
        region: "United States",
        currency: "USD",
        categories: &["all", "sp-sectors", "us"],
        symbols: ProviderSymbols {
            finnhub: Some("XLU"),
            massive: Some("XLU"),
            twelvedata: Some("XLU"),
        },
    },
    IndexDefinition {
        id: "xlc",
        code: "XLC",
        name: "Communication Services Select Sector",
        region: "United States",
        currency: "USD",
        categories: &["all", "sp-sectors", "us"],
        symbols: ProviderSymbols {
            finnhub: Some("XLC"),
            massive: Some("XLC"),
            twelvedata: Some("XLC"),
        },
    },
    IndexDefinition {
        id: "xlre",
        code: "XLRE",
        name: "Real Estate Select Sector",
        region: "United States",
        currency: "USD",
        categories: &["all", "sp-sectors", "us"],
        symbols: ProviderSymbols {
            finnhub: Some("XLRE"),
            massive: Some("XLRE"),
            twelvedata: Some("XLRE"),
        },
    },
];

#[derive(Debug, Deserialize)]
struct FinnhubQuote {
    c: Option<f64>,
    d: Option<f64>,
    dp: Option<f64>,
    h: Option<f64>,
    l: Option<f64>,
    o: Option<f64>,
    pc: Option<f64>,
    t: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct MassiveAggResponse {
    results: Option<Vec<MassiveAgg>>,
}

#[derive(Debug, Deserialize)]
struct MassiveAgg {
    c: Option<f64>,
    h: Option<f64>,
    l: Option<f64>,
    o: Option<f64>,
    v: Option<f64>,
    t: Option<i64>,
}

#[derive(Debug, Deserialize, Clone)]
struct TwelveQuote {
    symbol: Option<String>,
    currency: Option<String>,
    close: Option<String>,
    open: Option<String>,
    high: Option<String>,
    low: Option<String>,
    previous_close: Option<String>,
    change: Option<String>,
    percent_change: Option<String>,
    volume: Option<String>,
    datetime: Option<String>,
    message: Option<String>,
}

#[tauri::command]
fn get_market_snapshot(
    provider: String,
    symbol: String,
    asset_class: String,
) -> Result<MarketSnapshot, String> {
    load_env();

    let client = build_http_client()?;

    match provider.as_str() {
        "finnhub" => fetch_finnhub(&client, &symbol, &asset_class),
        "massive" => fetch_massive(&client, &symbol, &asset_class),
        "twelvedata" => fetch_twelve_data(&client, &symbol, &asset_class),
        _ => Err(format!("Unsupported market data provider: {provider}")),
    }
}

#[tauri::command]
fn get_indices_overview(
    category: String,
    preferred_provider: Option<String>,
) -> Result<IndicesOverviewResponse, String> {
    load_env();

    let selected_category = normalize_category(&category);
    let provider = resolve_indices_provider(preferred_provider.as_deref())?;
    let client = build_http_client()?;
    let definitions = definitions_for_category(&selected_category);

    let (rows, unavailable_count) = match provider {
        "twelvedata" => fetch_indices_with_twelvedata(&client, &definitions)?,
        "massive" => fetch_indices_with_symbol_loop(&client, &definitions, provider)?,
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

    Ok(IndicesOverviewResponse {
        provider: provider.to_string(),
        category: selected_category,
        updated_at,
        source_note,
        categories: category_counts(),
        rows,
    })
}

fn build_http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(12))
        .build()
        .map_err(|error| format!("Failed to create HTTP client: {error}"))
}

fn load_env() {
    let _ = dotenvy::dotenv();

    if let Ok(current_dir) = env::current_dir() {
        let candidates: [PathBuf; 2] = [current_dir.join(".env"), current_dir.join("..").join(".env")];

        for path in candidates {
            if path.exists() {
                let _ = dotenvy::from_path(path);
            }
        }
    }
}

fn env_key(names: &[&str]) -> Result<String, String> {
    for name in names {
        if let Ok(value) = env::var(name) {
            if !value.trim().is_empty() {
                return Ok(value);
            }
        }
    }

    Err(format!(
        "Missing API key. Expected one of: {}",
        names.join(", ")
    ))
}

fn has_env_key(names: &[&str]) -> bool {
    names.iter()
        .any(|name| env::var(name).map(|value| !value.trim().is_empty()).unwrap_or(false))
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
    if let Some(provider) = preferred.map(str::trim).filter(|provider| !provider.is_empty()) {
        return match provider {
            "twelvedata" if has_env_key(&["TWELVE_DATA_API_KEY", "TWELVEDATA_API_KEY"]) => {
                Ok("twelvedata")
            }
            "massive" if has_env_key(&["MASSIVE_API_KEY", "POLYGON_API_KEY"]) => Ok("massive"),
            "finnhub" if has_env_key(&["FINNHUB_API_KEY", "FINNHUB_TOKEN"]) => Ok("finnhub"),
            "twelvedata" | "massive" | "finnhub" => Err(format!(
                "{} API key is not configured for aggregated indices",
                provider_label(provider)
            )),
            _ => Err(format!("Unsupported market data provider: {provider}")),
        };
    }

    if has_env_key(&["TWELVE_DATA_API_KEY", "TWELVEDATA_API_KEY"]) {
        return Ok("twelvedata");
    }

    if has_env_key(&["MASSIVE_API_KEY", "POLYGON_API_KEY"]) {
        return Ok("massive");
    }

    if has_env_key(&["FINNHUB_API_KEY", "FINNHUB_TOKEN"]) {
        return Ok("finnhub");
    }

    Err(
        "Missing API key. Configure TWELVE_DATA_API_KEY, MASSIVE_API_KEY/POLYGON_API_KEY, or FINNHUB_API_KEY."
            .to_string(),
    )
}

fn normalize_category(category: &str) -> String {
    let normalized = category.trim().to_ascii_lowercase();

    if INDEX_CATEGORY_IDS.contains(&normalized.as_str()) {
        normalized
    } else {
        "all".to_string()
    }
}

fn definitions_for_category(category: &str) -> Vec<&'static IndexDefinition> {
    INDEX_DEFINITIONS
        .iter()
        .filter(|definition| category == "all" || definition.categories.contains(&category))
        .collect()
}

fn category_counts() -> Vec<IndexCategoryCount> {
    INDEX_CATEGORY_IDS
        .iter()
        .map(|category_id| IndexCategoryCount {
            id: (*category_id).to_string(),
            total: definitions_for_category(category_id).len(),
        })
        .collect()
}

fn index_symbol_for_provider(definition: &IndexDefinition, provider: &str) -> Option<&'static str> {
    match provider {
        "finnhub" => definition.symbols.finnhub,
        "massive" => definition.symbols.massive,
        "twelvedata" => definition.symbols.twelvedata,
        _ => None,
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
    let mut rows = Vec::with_capacity(definitions.len());
    let mut unavailable_count = 0;

    for definition in definitions {
        let Some(provider_symbol) = index_symbol_for_provider(definition, provider) else {
            unavailable_count += 1;
            rows.push(index_row_from_snapshot(definition, None));
            continue;
        };

        let snapshot = match provider {
            "finnhub" => fetch_finnhub(client, provider_symbol, "index").ok(),
            "massive" => fetch_massive(client, provider_symbol, "index").ok(),
            "twelvedata" => fetch_twelve_data(client, provider_symbol, "index").ok(),
            _ => None,
        };

        if snapshot.is_none() {
            unavailable_count += 1;
        }

        rows.push(index_row_from_snapshot(definition, snapshot));
    }

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

    let bulk_response = client
        .get(url)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| format!("Twelve Data bulk request failed: {error}"))?;

    let value: Value = bulk_response
        .json()
        .map_err(|error| format!("Twelve Data bulk response parse failed: {error}"))?;

    if let Ok(snapshots) = parse_twelvedata_bulk_quotes(value) {
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
                }

                index_row_from_snapshot(definition, snapshot)
            })
            .collect();

        return Ok((rows, unavailable_count));
    }

    fetch_indices_with_symbol_loop(client, definitions, "twelvedata")
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
                let snapshot = market_snapshot_from_twelve_quote_with_fallback(quote, "index", &symbol_key);
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
    let technical_rating = technical_rating(snapshot.as_ref().and_then(|snapshot| snapshot.change_percent));

    IndexOverviewRow {
        id: definition.id.to_string(),
        symbol: definition.code.to_string(),
        name: definition.name.to_string(),
        region: definition.region.to_string(),
        currency,
        price: snapshot.as_ref().and_then(|snapshot| snapshot.price),
        change: snapshot.as_ref().and_then(|snapshot| snapshot.change),
        change_percent: snapshot.as_ref().and_then(|snapshot| snapshot.change_percent),
        open: snapshot.as_ref().and_then(|snapshot| snapshot.open),
        high: snapshot.as_ref().and_then(|snapshot| snapshot.high),
        low: snapshot.as_ref().and_then(|snapshot| snapshot.low),
        previous_close: snapshot.as_ref().and_then(|snapshot| snapshot.previous_close),
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

    let quote: FinnhubQuote = client
        .get(url)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| format!("Finnhub request failed: {error}"))?
        .json()
        .map_err(|error| format!("Finnhub response parse failed: {error}"))?;

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

    let response: MassiveAggResponse = client
        .get(url)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| format!("Massive request failed: {error}"))?
        .json()
        .map_err(|error| format!("Massive response parse failed: {error}"))?;

    let result = response
        .results
        .and_then(|mut results| results.pop())
        .ok_or_else(|| "Massive returned no aggregate data for this symbol".to_string())?;

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

    let quote: TwelveQuote = client
        .get(url)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| format!("Twelve Data request failed: {error}"))?
        .json()
        .map_err(|error| format!("Twelve Data response parse failed: {error}"))?;

    if let Some(message) = quote.message.clone() {
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
    value.and_then(|value| value.parse::<f64>().ok())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if let (Some(window), Some(icon)) =
                (app.get_webview_window("main"), app.default_window_icon())
            {
                window.set_icon(icon.clone())?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            show_window,
            get_market_snapshot,
            get_indices_overview
        ])
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
