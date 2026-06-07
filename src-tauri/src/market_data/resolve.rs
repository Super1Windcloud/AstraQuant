use super::asset_catalog::supported_assets;
use super::catalog::INDEX_CATEGORY_IDS;
use super::models::{MarketTableColumn, MarketViewTab};
use log::{error, info, warn};
use std::env;

const MARKET_LOG_TARGET: &str = "astraquant::market_data";

pub(crate) fn resolve_indices_provider(preferred: Option<&str>) -> Result<&'static str, String> {
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
            "tradingview" => {
                info!(
                    target: MARKET_LOG_TARGET,
                    "resolved indices provider requested={} selected=tradingview",
                    provider
                );
                Ok("tradingview")
            }
            "finnhub" => {
                let message =
                    "Finnhub API key is not configured for aggregated indices".to_string();
                warn!(target: MARKET_LOG_TARGET, "{message}");
                Err(message)
            }
            "alpha-vantage" => {
                let message = "Indices aggregated overview currently requires FINNHUB_API_KEY or FINNHUB_TOKEN.".to_string();
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

pub(crate) fn resolve_asset_provider(
    asset: &str,
    preferred: Option<&str>,
) -> Result<&'static str, String> {
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
            "tradingview" => Ok("tradingview"),
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

    if matches!(asset, "crypto" | "futures") {
        info!(
            target: MARKET_LOG_TARGET,
            "resolved asset provider automatically selected=tradingview asset={asset}"
        );
        return Ok("tradingview");
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

pub(crate) fn asset_supports_provider(asset: &str, provider: &str) -> bool {
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

pub(crate) fn provider_label(provider: &str) -> &'static str {
    match provider {
        "alpha-vantage" => "Alpha Vantage",
        "finnhub" => "Finnhub",
        "tradingview" => "TradingView",
        _ => "Unknown provider",
    }
}

pub(crate) fn default_market_tabs() -> Vec<MarketViewTab> {
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

pub(crate) fn default_index_columns() -> Vec<MarketTableColumn> {
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

pub(crate) fn normalize_asset(asset: &str) -> Result<String, String> {
    let normalized = asset.trim().to_ascii_lowercase();

    if supported_assets().contains(&normalized.as_str()) {
        Ok(normalized)
    } else {
        Err(format!("Unsupported market overview asset: {asset}"))
    }
}

pub(crate) fn normalize_category(category: &str) -> String {
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

fn has_env_key(names: &[&str]) -> bool {
    names.iter().any(|name| {
        env::var(name)
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
    })
}
