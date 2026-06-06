import { invoke } from "@tauri-apps/api/core"

export type MarketProvider = "finnhub" | "massive" | "twelvedata"

export const indexCategoryIds = [
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
] as const

export type IndicesCategory = (typeof indexCategoryIds)[number]

export interface IndexCategoryCount {
  id: IndicesCategory
  total: number
}

export interface IndexOverviewRow {
  id: string
  symbol: string
  name: string
  region: string
  currency: string | null
  price: number | null
  change: number | null
  change_percent: number | null
  open: number | null
  high: number | null
  low: number | null
  previous_close: number | null
  as_of: string | null
  technical_rating: string
}

export interface IndicesOverviewResponse {
  provider: MarketProvider
  category: IndicesCategory
  updated_at: string | null
  source_note: string
  categories: IndexCategoryCount[]
  rows: IndexOverviewRow[]
}

export const providerLabels: Record<MarketProvider, string> = {
  finnhub: "Finnhub",
  massive: "Massive",
  twelvedata: "Twelve Data",
}

export const indexCategories: Array<{ id: IndicesCategory; i18nKey: string }> = [
  { id: "all", i18nKey: "indicesFilterAll" },
  { id: "major", i18nKey: "indicesFilterMajor" },
  { id: "us", i18nKey: "indicesFilterUs" },
  { id: "sp-sectors", i18nKey: "indicesFilterSectors" },
  { id: "currency", i18nKey: "indicesFilterCurrency" },
  { id: "americas", i18nKey: "indicesFilterAmericas" },
  { id: "europe", i18nKey: "indicesFilterEurope" },
  { id: "asia", i18nKey: "indicesFilterAsia" },
  { id: "pacific", i18nKey: "indicesFilterPacific" },
  { id: "middle-east", i18nKey: "indicesFilterMiddleEast" },
  { id: "africa", i18nKey: "indicesFilterAfrica" },
]

interface PreviewIndexRow extends IndexOverviewRow {
  categories: IndicesCategory[]
}

const previewRows: PreviewIndexRow[] = [
  {
    id: "spx",
    symbol: "SPX",
    name: "S&P 500",
    region: "United States",
    currency: "USD",
    price: 7383.73,
    change: -200.59,
    change_percent: -2.64,
    open: 7521.2,
    high: 7541.81,
    low: 7368.63,
    previous_close: 7584.32,
    as_of: "2026-06-06 15:59:00",
    technical_rating: "Neutral",
    categories: ["all", "major", "us", "americas"],
  },
  {
    id: "ixic",
    symbol: "IXIC",
    name: "US Composite Index",
    region: "United States",
    currency: "USD",
    price: 25709.43,
    change: -1121.53,
    change_percent: -4.18,
    open: 26432.1,
    high: 26572.25,
    low: 25648.47,
    previous_close: 26830.96,
    as_of: "2026-06-06 15:59:00",
    technical_rating: "Sell",
    categories: ["all", "major", "us", "americas"],
  },
  {
    id: "dji",
    symbol: "DJI",
    name: "Dow Jones Industrial Average Index",
    region: "United States",
    currency: "USD",
    price: 50866.79,
    change: -695.15,
    change_percent: -1.35,
    open: 51410.2,
    high: 51660.4,
    low: 50781.45,
    previous_close: 51561.94,
    as_of: "2026-06-06 15:59:00",
    technical_rating: "Buy",
    categories: ["all", "major", "us", "americas"],
  },
  {
    id: "vix",
    symbol: "VIX",
    name: "CBOE Volatility Index",
    region: "United States",
    currency: "USD",
    price: 21.51,
    change: 6.11,
    change_percent: 39.68,
    open: 15.7,
    high: 21.57,
    low: 15.56,
    previous_close: 15.4,
    as_of: "2026-06-06 15:59:00",
    technical_rating: "Strong buy",
    categories: ["all", "major", "us", "americas"],
  },
  {
    id: "tsx",
    symbol: "TSX",
    name: "S&P/TSX Composite Index",
    region: "Canada",
    currency: "CAD",
    price: 34413.45,
    change: -803.61,
    change_percent: -2.28,
    open: 35040.22,
    high: 35071.82,
    low: 34379.72,
    previous_close: 35217.06,
    as_of: "2026-06-06 16:00:00",
    technical_rating: "Neutral",
    categories: ["all", "major", "americas"],
  },
  {
    id: "ukx",
    symbol: "UKX",
    name: "UK 100 Index",
    region: "United Kingdom",
    currency: "GBP",
    price: 10368.04,
    change: 7.72,
    change_percent: 0.07,
    open: 10335.8,
    high: 10415.74,
    low: 10331.54,
    previous_close: 10360.32,
    as_of: "2026-06-06 16:30:00",
    technical_rating: "Neutral",
    categories: ["all", "major", "europe"],
  },
  {
    id: "dax",
    symbol: "DAX",
    name: "DAX Index",
    region: "Germany",
    currency: "EUR",
    price: 24759.05,
    change: -185.9,
    change_percent: -0.75,
    open: 24990.4,
    high: 25024.05,
    low: 24756.47,
    previous_close: 24944.95,
    as_of: "2026-06-06 16:30:00",
    technical_rating: "Buy",
    categories: ["all", "major", "europe"],
  },
  {
    id: "px1",
    symbol: "PX1",
    name: "CAC 40 Index",
    region: "France",
    currency: "EUR",
    price: 8218.24,
    change: -26.05,
    change_percent: -0.32,
    open: 8241.05,
    high: 8296.01,
    low: 8218.24,
    previous_close: 8244.29,
    as_of: "2026-06-06 16:30:00",
    technical_rating: "Buy",
    categories: ["all", "major", "europe"],
  },
  {
    id: "ftmib",
    symbol: "FTMIB",
    name: "MILANO ITALIA BORSA INDEX",
    region: "Italy",
    currency: "EUR",
    price: 49893.04,
    change: -281.31,
    change_percent: -0.56,
    open: 50062.8,
    high: 50319.55,
    low: 49868.02,
    previous_close: 50174.35,
    as_of: "2026-06-06 16:30:00",
    technical_rating: "Buy",
    categories: ["all", "major", "europe"],
  },
  {
    id: "n225",
    symbol: "N225",
    name: "Japan 225 Index",
    region: "Japan",
    currency: "JPY",
    price: 66587.9,
    change: -882.57,
    change_percent: -1.31,
    open: 67050.15,
    high: 67115.0,
    low: 65862.21,
    previous_close: 67470.47,
    as_of: "2026-06-06 15:00:00",
    technical_rating: "Buy",
    categories: ["all", "major", "asia", "pacific"],
  },
  {
    id: "kospi",
    symbol: "KOSPI",
    name: "KOREA COMPOSITE STOCK PRICE INDEX (KOSPI)",
    region: "South Korea",
    currency: "KRW",
    price: 8160.6,
    change: -478.82,
    change_percent: -5.54,
    open: 8334.5,
    high: 8382.16,
    low: 8038.1,
    previous_close: 8639.42,
    as_of: "2026-06-06 15:30:00",
    technical_rating: "Buy",
    categories: ["all", "major", "asia"],
  },
  {
    id: "hsi",
    symbol: "HSI",
    name: "Hang Seng Index",
    region: "Hong Kong",
    currency: "HKD",
    price: 18442.1,
    change: -316.25,
    change_percent: -1.69,
    open: 18604.32,
    high: 18690.14,
    low: 18398.24,
    previous_close: 18758.35,
    as_of: "2026-06-06 16:10:00",
    technical_rating: "Sell",
    categories: ["all", "asia"],
  },
  {
    id: "xjo",
    symbol: "XJO",
    name: "S&P/ASX 200",
    region: "Australia",
    currency: "AUD",
    price: 7840.15,
    change: 48.16,
    change_percent: 0.62,
    open: 7802.84,
    high: 7851.05,
    low: 7784.33,
    previous_close: 7791.99,
    as_of: "2026-06-06 16:20:00",
    technical_rating: "Buy",
    categories: ["all", "pacific"],
  },
  {
    id: "ta35",
    symbol: "TA35",
    name: "TA-35 Index",
    region: "Israel",
    currency: "ILS",
    price: 2295.48,
    change: 11.75,
    change_percent: 0.51,
    open: 2287.1,
    high: 2301.6,
    low: 2275.22,
    previous_close: 2283.73,
    as_of: "2026-06-06 16:20:00",
    technical_rating: "Buy",
    categories: ["all", "middle-east"],
  },
  {
    id: "jalsh",
    symbol: "JALSH",
    name: "FTSE/JSE All Share",
    region: "South Africa",
    currency: "ZAR",
    price: 84811.36,
    change: -542.74,
    change_percent: -0.64,
    open: 85102.94,
    high: 85330.55,
    low: 84691.12,
    previous_close: 85354.1,
    as_of: "2026-06-06 16:20:00",
    technical_rating: "Neutral",
    categories: ["all", "africa"],
  },
  {
    id: "dxy",
    symbol: "DXY",
    name: "US Dollar Currency Index",
    region: "Global",
    currency: "USD",
    price: 104.28,
    change: 0.42,
    change_percent: 0.4,
    open: 103.88,
    high: 104.34,
    low: 103.76,
    previous_close: 103.86,
    as_of: "2026-06-06 16:20:00",
    technical_rating: "Neutral",
    categories: ["all", "currency", "americas"],
  },
  {
    id: "xlk",
    symbol: "XLK",
    name: "Technology Select Sector",
    region: "United States",
    currency: "USD",
    price: 238.12,
    change: -5.33,
    change_percent: -2.19,
    open: 242.08,
    high: 242.75,
    low: 237.65,
    previous_close: 243.45,
    as_of: "2026-06-06 15:59:00",
    technical_rating: "Sell",
    categories: ["all", "sp-sectors", "us"],
  },
  {
    id: "xlf",
    symbol: "XLF",
    name: "Financial Select Sector",
    region: "United States",
    currency: "USD",
    price: 44.83,
    change: -0.12,
    change_percent: -0.27,
    open: 45.01,
    high: 45.08,
    low: 44.72,
    previous_close: 44.95,
    as_of: "2026-06-06 15:59:00",
    technical_rating: "Neutral",
    categories: ["all", "sp-sectors", "us"],
  },
  {
    id: "xle",
    symbol: "XLE",
    name: "Energy Select Sector",
    region: "United States",
    currency: "USD",
    price: 98.41,
    change: 1.94,
    change_percent: 2.01,
    open: 96.92,
    high: 98.65,
    low: 96.77,
    previous_close: 96.47,
    as_of: "2026-06-06 15:59:00",
    technical_rating: "Strong buy",
    categories: ["all", "sp-sectors", "us"],
  },
]

const previewCategoryCounts = indexCategoryIds.map((id) => ({
  id,
  total: previewRows.filter((row) => row.categories.includes(id)).length,
}))

export async function getIndicesOverview(
  category: IndicesCategory,
  preferredProvider?: MarketProvider
) {
  if (!isTauriRuntime()) {
    return buildPreviewIndicesOverview(category)
  }

  return invoke<IndicesOverviewResponse>("get_indices_overview", {
    category,
    preferredProvider,
  })
}

function isTauriRuntime() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window
}

function buildPreviewIndicesOverview(category: IndicesCategory): IndicesOverviewResponse {
  const rows = previewRows
    .filter((row) => row.categories.includes(category))
    .map(({ categories: _categories, ...row }) => row)

  return {
    provider: "twelvedata",
    category,
    updated_at: rows[0]?.as_of ?? null,
    source_note: "Preview data for browser layout verification",
    categories: previewCategoryCounts,
    rows,
  }
}
