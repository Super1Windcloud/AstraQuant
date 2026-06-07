use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone)]
pub(crate) struct MarketSnapshot {
    pub(crate) provider: String,
    pub(crate) symbol: String,
    pub(crate) asset_class: String,
    pub(crate) price: Option<f64>,
    pub(crate) change: Option<f64>,
    pub(crate) change_percent: Option<f64>,
    pub(crate) open: Option<f64>,
    pub(crate) high: Option<f64>,
    pub(crate) low: Option<f64>,
    pub(crate) previous_close: Option<f64>,
    pub(crate) volume: Option<f64>,
    pub(crate) currency: Option<String>,
    pub(crate) as_of: Option<String>,
    pub(crate) technical_rating: Option<String>,
    pub(crate) source_note: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct IndexCategoryCount {
    pub(crate) id: String,
    pub(crate) label_key: String,
    pub(crate) total: usize,
}

#[derive(Debug, Serialize)]
pub(crate) struct AssetCategoryCount {
    pub(crate) id: String,
    pub(crate) label_key: String,
    pub(crate) total: usize,
}

#[derive(Debug, Serialize)]
pub(crate) struct MarketViewTab {
    pub(crate) id: String,
    pub(crate) label_key: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct MarketTableColumn {
    pub(crate) id: String,
    pub(crate) label_key: String,
    pub(crate) align: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct IndexOverviewRow {
    pub(crate) id: String,
    pub(crate) symbol: String,
    pub(crate) name: String,
    pub(crate) region: String,
    pub(crate) currency: Option<String>,
    pub(crate) price: Option<f64>,
    pub(crate) change: Option<f64>,
    pub(crate) change_percent: Option<f64>,
    pub(crate) open: Option<f64>,
    pub(crate) high: Option<f64>,
    pub(crate) low: Option<f64>,
    pub(crate) previous_close: Option<f64>,
    pub(crate) as_of: Option<String>,
    pub(crate) technical_rating: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct IndicesOverviewResponse {
    pub(crate) provider: String,
    pub(crate) category: String,
    pub(crate) title_key: String,
    pub(crate) description_key: String,
    pub(crate) updated_at: Option<String>,
    pub(crate) source_note: String,
    pub(crate) categories: Vec<IndexCategoryCount>,
    pub(crate) tabs: Vec<MarketViewTab>,
    pub(crate) columns: Vec<MarketTableColumn>,
    pub(crate) rows: Vec<IndexOverviewRow>,
}

#[derive(Debug, Serialize)]
pub(crate) struct AssetOverviewRow {
    pub(crate) id: String,
    pub(crate) category_id: String,
    pub(crate) symbol: String,
    pub(crate) name: String,
    pub(crate) region: String,
    pub(crate) currency: Option<String>,
    pub(crate) price: Option<f64>,
    pub(crate) change: Option<f64>,
    pub(crate) change_percent: Option<f64>,
    pub(crate) open: Option<f64>,
    pub(crate) high: Option<f64>,
    pub(crate) low: Option<f64>,
    pub(crate) previous_close: Option<f64>,
    pub(crate) as_of: Option<String>,
    pub(crate) technical_rating: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct AssetOverviewResponse {
    pub(crate) provider: String,
    pub(crate) asset: String,
    pub(crate) updated_at: Option<String>,
    pub(crate) source_note: String,
    pub(crate) categories: Vec<AssetCategoryCount>,
    pub(crate) tabs: Vec<MarketViewTab>,
    pub(crate) rows: Vec<AssetOverviewRow>,
}

#[derive(Debug, Serialize)]
pub(crate) struct MarketItemDetailResponse {
    pub(crate) provider: String,
    pub(crate) kind: String,
    pub(crate) id: String,
    pub(crate) symbol: String,
    pub(crate) name: String,
    pub(crate) region: String,
    pub(crate) currency: Option<String>,
    pub(crate) price: Option<f64>,
    pub(crate) change: Option<f64>,
    pub(crate) change_percent: Option<f64>,
    pub(crate) open: Option<f64>,
    pub(crate) high: Option<f64>,
    pub(crate) low: Option<f64>,
    pub(crate) previous_close: Option<f64>,
    pub(crate) as_of: Option<String>,
    pub(crate) source_note: String,
    pub(crate) technical_rating: String,
    pub(crate) tradingview_symbol: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct MarketChartPoint {
    pub(crate) time: String,
    pub(crate) open: Option<f64>,
    pub(crate) high: Option<f64>,
    pub(crate) low: Option<f64>,
    pub(crate) close: Option<f64>,
    pub(crate) value: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct MarketChartSeriesResponse {
    pub(crate) provider: String,
    pub(crate) kind: String,
    pub(crate) id: String,
    pub(crate) symbol: String,
    pub(crate) interval: String,
    pub(crate) series_type: String,
    pub(crate) source_note: String,
    pub(crate) currency: Option<String>,
    pub(crate) technical_rating: Option<String>,
    pub(crate) points: Vec<MarketChartPoint>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FinnhubQuote {
    pub(crate) c: Option<f64>,
    pub(crate) d: Option<f64>,
    pub(crate) dp: Option<f64>,
    pub(crate) h: Option<f64>,
    pub(crate) l: Option<f64>,
    pub(crate) o: Option<f64>,
    pub(crate) pc: Option<f64>,
    pub(crate) t: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FinnhubCandleResponse {
    #[serde(default)]
    pub(crate) c: Vec<Option<f64>>,
    #[serde(default)]
    pub(crate) h: Vec<Option<f64>>,
    #[serde(default)]
    pub(crate) l: Vec<Option<f64>>,
    #[serde(default)]
    pub(crate) o: Vec<Option<f64>>,
    #[serde(default)]
    pub(crate) t: Vec<i64>,
    pub(crate) s: String,
}
