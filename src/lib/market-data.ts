import { invoke } from "@tauri-apps/api/core"

export type MarketProvider = "alpha-vantage" | "finnhub"

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
  "alpha-vantage": "Alpha Vantage",
  finnhub: "Finnhub",
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
    provider: "finnhub",
    category,
    updated_at: rows[0]?.as_of ?? null,
    source_note: "Preview data for browser layout verification",
    categories: previewCategoryCounts,
    rows,
  }
}

export type MarketAsset = "stocks" | "etf" | "crypto" | "futures"

export interface AssetCategoryCount {
  id: string
  total: number
}

export interface AssetOverviewRow {
  id: string
  category_id: string
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

export interface AssetOverviewResponse {
  provider: MarketProvider
  asset: MarketAsset
  updated_at: string | null
  source_note: string
  categories: AssetCategoryCount[]
  rows: AssetOverviewRow[]
}

export interface AssetCategoryMeta {
  id: string
  i18nKey: string
}

export interface MarketAssetConfig {
  asset: MarketAsset
  path: string
  navI18nKey: string
  titleI18nKey: string
  descriptionI18nKey: string
  categories: AssetCategoryMeta[]
}

export const marketNavigationItems = [
  { to: "/" as const, i18nKey: "assetIndex" },
  { to: "/stocks" as const, i18nKey: "assetStock" },
  { to: "/crypto" as const, i18nKey: "assetCrypto" },
  { to: "/futures" as const, i18nKey: "assetFuture" },
  { to: "/etf" as const, i18nKey: "assetEtf" },
]

export const marketAssetConfigs: Record<MarketAsset, MarketAssetConfig> = {
  stocks: {
    asset: "stocks",
    path: "/stocks",
    navI18nKey: "assetStock",
    titleI18nKey: "stocksTitle",
    descriptionI18nKey: "stocksDescription",
    categories: [
      { id: "mega-cap", i18nKey: "stocksSectionMegaCap" },
      { id: "internet", i18nKey: "stocksSectionInternet" },
      { id: "semiconductors", i18nKey: "stocksSectionSemiconductors" },
      { id: "china-adr", i18nKey: "stocksSectionChinaAdr" },
    ],
  },
  etf: {
    asset: "etf",
    path: "/etf",
    navI18nKey: "assetEtf",
    titleI18nKey: "etfTitle",
    descriptionI18nKey: "etfDescription",
    categories: [
      { id: "broad-market", i18nKey: "etfSectionBroadMarket" },
      { id: "sectors", i18nKey: "etfSectionSectors" },
      { id: "fixed-income", i18nKey: "etfSectionFixedIncome" },
      { id: "global-thematic", i18nKey: "etfSectionGlobalThematic" },
    ],
  },
  crypto: {
    asset: "crypto",
    path: "/crypto",
    navI18nKey: "assetCrypto",
    titleI18nKey: "cryptoTitle",
    descriptionI18nKey: "cryptoDescription",
    categories: [
      { id: "majors", i18nKey: "cryptoSectionMajors" },
      { id: "smart-contracts", i18nKey: "cryptoSectionSmartContracts" },
      { id: "payments-meme", i18nKey: "cryptoSectionPaymentsMeme" },
      { id: "infrastructure", i18nKey: "cryptoSectionInfrastructure" },
    ],
  },
  futures: {
    asset: "futures",
    path: "/futures",
    navI18nKey: "assetFuture",
    titleI18nKey: "futuresTitle",
    descriptionI18nKey: "futuresDescription",
    categories: [
      { id: "energy", i18nKey: "futuresSectionEnergy" },
      { id: "metals", i18nKey: "futuresSectionMetals" },
      { id: "grains", i18nKey: "futuresSectionGrains" },
      { id: "softs", i18nKey: "futuresSectionSofts" },
    ],
  },
}

interface AssetPreviewSeed {
  id: string
  category_id: string
  symbol: string
  name: string
  region: string
  currency: string
}

const previewAssetSeeds: Record<MarketAsset, AssetPreviewSeed[]> = {
  stocks: [
    {
      id: "aapl",
      category_id: "mega-cap",
      symbol: "AAPL",
      name: "Apple",
      region: "United States",
      currency: "USD",
    },
    {
      id: "msft",
      category_id: "mega-cap",
      symbol: "MSFT",
      name: "Microsoft",
      region: "United States",
      currency: "USD",
    },
    {
      id: "nvda",
      category_id: "mega-cap",
      symbol: "NVDA",
      name: "NVIDIA",
      region: "United States",
      currency: "USD",
    },
    {
      id: "amzn",
      category_id: "internet",
      symbol: "AMZN",
      name: "Amazon",
      region: "United States",
      currency: "USD",
    },
    {
      id: "googl",
      category_id: "internet",
      symbol: "GOOGL",
      name: "Alphabet Class A",
      region: "United States",
      currency: "USD",
    },
    {
      id: "meta",
      category_id: "internet",
      symbol: "META",
      name: "Meta Platforms",
      region: "United States",
      currency: "USD",
    },
    {
      id: "amd",
      category_id: "semiconductors",
      symbol: "AMD",
      name: "AMD",
      region: "United States",
      currency: "USD",
    },
    {
      id: "tsm",
      category_id: "semiconductors",
      symbol: "TSM",
      name: "Taiwan Semiconductor ADR",
      region: "Taiwan",
      currency: "USD",
    },
    {
      id: "avgo",
      category_id: "semiconductors",
      symbol: "AVGO",
      name: "Broadcom",
      region: "United States",
      currency: "USD",
    },
    {
      id: "baba",
      category_id: "china-adr",
      symbol: "BABA",
      name: "Alibaba ADR",
      region: "China",
      currency: "USD",
    },
    {
      id: "pdd",
      category_id: "china-adr",
      symbol: "PDD",
      name: "PDD Holdings ADR",
      region: "China",
      currency: "USD",
    },
    {
      id: "bidu",
      category_id: "china-adr",
      symbol: "BIDU",
      name: "Baidu ADR",
      region: "China",
      currency: "USD",
    },
  ],
  etf: [
    {
      id: "spy",
      category_id: "broad-market",
      symbol: "SPY",
      name: "SPDR S&P 500 ETF Trust",
      region: "United States",
      currency: "USD",
    },
    {
      id: "qqq",
      category_id: "broad-market",
      symbol: "QQQ",
      name: "Invesco QQQ Trust",
      region: "United States",
      currency: "USD",
    },
    {
      id: "vti",
      category_id: "broad-market",
      symbol: "VTI",
      name: "Vanguard Total Stock Market ETF",
      region: "United States",
      currency: "USD",
    },
    {
      id: "xlk",
      category_id: "sectors",
      symbol: "XLK",
      name: "Technology Select Sector SPDR Fund",
      region: "United States",
      currency: "USD",
    },
    {
      id: "xlf",
      category_id: "sectors",
      symbol: "XLF",
      name: "Financial Select Sector SPDR Fund",
      region: "United States",
      currency: "USD",
    },
    {
      id: "xle",
      category_id: "sectors",
      symbol: "XLE",
      name: "Energy Select Sector SPDR Fund",
      region: "United States",
      currency: "USD",
    },
    {
      id: "tlt",
      category_id: "fixed-income",
      symbol: "TLT",
      name: "iShares 20+ Year Treasury Bond ETF",
      region: "United States",
      currency: "USD",
    },
    {
      id: "lqd",
      category_id: "fixed-income",
      symbol: "LQD",
      name: "iShares iBoxx $ Investment Grade Corporate Bond ETF",
      region: "United States",
      currency: "USD",
    },
    {
      id: "hyg",
      category_id: "fixed-income",
      symbol: "HYG",
      name: "iShares iBoxx $ High Yield Corporate Bond ETF",
      region: "United States",
      currency: "USD",
    },
    {
      id: "eem",
      category_id: "global-thematic",
      symbol: "EEM",
      name: "iShares MSCI Emerging Markets ETF",
      region: "Global",
      currency: "USD",
    },
    {
      id: "smh",
      category_id: "global-thematic",
      symbol: "SMH",
      name: "VanEck Semiconductor ETF",
      region: "United States",
      currency: "USD",
    },
    {
      id: "gld",
      category_id: "global-thematic",
      symbol: "GLD",
      name: "SPDR Gold Shares",
      region: "Global",
      currency: "USD",
    },
  ],
  crypto: [
    {
      id: "btcusd",
      category_id: "majors",
      symbol: "BTC/USD",
      name: "Bitcoin",
      region: "Global",
      currency: "USD",
    },
    {
      id: "ethusd",
      category_id: "majors",
      symbol: "ETH/USD",
      name: "Ethereum",
      region: "Global",
      currency: "USD",
    },
    {
      id: "xrpusd",
      category_id: "majors",
      symbol: "XRP/USD",
      name: "XRP",
      region: "Global",
      currency: "USD",
    },
    {
      id: "solusd",
      category_id: "smart-contracts",
      symbol: "SOL/USD",
      name: "Solana",
      region: "Global",
      currency: "USD",
    },
    {
      id: "bnbusd",
      category_id: "smart-contracts",
      symbol: "BNB/USD",
      name: "BNB",
      region: "Global",
      currency: "USD",
    },
    {
      id: "adausd",
      category_id: "smart-contracts",
      symbol: "ADA/USD",
      name: "Cardano",
      region: "Global",
      currency: "USD",
    },
    {
      id: "dogeusd",
      category_id: "payments-meme",
      symbol: "DOGE/USD",
      name: "Dogecoin",
      region: "Global",
      currency: "USD",
    },
    {
      id: "ltcusd",
      category_id: "payments-meme",
      symbol: "LTC/USD",
      name: "Litecoin",
      region: "Global",
      currency: "USD",
    },
    {
      id: "bchusd",
      category_id: "payments-meme",
      symbol: "BCH/USD",
      name: "Bitcoin Cash",
      region: "Global",
      currency: "USD",
    },
    {
      id: "linkusd",
      category_id: "infrastructure",
      symbol: "LINK/USD",
      name: "Chainlink",
      region: "Global",
      currency: "USD",
    },
    {
      id: "avaxusd",
      category_id: "infrastructure",
      symbol: "AVAX/USD",
      name: "Avalanche",
      region: "Global",
      currency: "USD",
    },
    {
      id: "dotusd",
      category_id: "infrastructure",
      symbol: "DOT/USD",
      name: "Polkadot",
      region: "Global",
      currency: "USD",
    },
  ],
  futures: [
    {
      id: "wti",
      category_id: "energy",
      symbol: "WTI",
      name: "WTI Crude Oil",
      region: "Global",
      currency: "USD",
    },
    {
      id: "brent",
      category_id: "energy",
      symbol: "BRENT",
      name: "Brent Crude Oil",
      region: "Global",
      currency: "USD",
    },
    {
      id: "natgas",
      category_id: "energy",
      symbol: "NATGAS",
      name: "Natural Gas",
      region: "Global",
      currency: "USD",
    },
    {
      id: "copper",
      category_id: "metals",
      symbol: "COPPER",
      name: "Copper",
      region: "Global",
      currency: "USD",
    },
    {
      id: "aluminum",
      category_id: "metals",
      symbol: "ALUMINUM",
      name: "Aluminum",
      region: "Global",
      currency: "USD",
    },
    {
      id: "wheat",
      category_id: "grains",
      symbol: "WHEAT",
      name: "Wheat",
      region: "Global",
      currency: "USD",
    },
    {
      id: "corn",
      category_id: "grains",
      symbol: "CORN",
      name: "Corn",
      region: "Global",
      currency: "USD",
    },
    {
      id: "cotton",
      category_id: "softs",
      symbol: "COTTON",
      name: "Cotton",
      region: "Global",
      currency: "USD",
    },
    {
      id: "coffee",
      category_id: "softs",
      symbol: "COFFEE",
      name: "Coffee",
      region: "Global",
      currency: "USD",
    },
  ],
}

export async function getAssetOverview(asset: MarketAsset, preferredProvider?: MarketProvider) {
  if (!isTauriRuntime()) {
    return buildPreviewAssetOverview(asset)
  }

  return invoke<AssetOverviewResponse>("get_asset_overview", {
    asset,
    preferredProvider,
  })
}

function buildPreviewAssetOverview(asset: MarketAsset): AssetOverviewResponse {
  const seeds = previewAssetSeeds[asset]
  const rows = seeds.map((seed, index) => buildPreviewAssetRow(asset, seed, index))
  const categories = marketAssetConfigs[asset].categories.map((category) => ({
    id: category.id,
    total: rows.filter((row) => row.category_id === category.id).length,
  }))

  return {
    provider: "alpha-vantage",
    asset,
    updated_at: rows[0]?.as_of ?? null,
    source_note: "Preview data for browser layout verification",
    categories,
    rows,
  }
}

function buildPreviewAssetRow(
  asset: MarketAsset,
  seed: AssetPreviewSeed,
  index: number
): AssetOverviewRow {
  const hash = `${asset}:${seed.symbol}`
    .split("")
    .reduce((total, char) => total + char.charCodeAt(0), 0)
  const base = previewBaseValue(asset, hash)
  const changePercent = (((hash + index * 17) % 900) - 450) / 100
  const change = (base * changePercent) / 100
  const previousClose = base - change
  const high = base * (1 + ((hash % 35) + 8) / 1000)
  const low = base * (1 - ((hash % 30) + 7) / 1000)
  const open = previousClose * (1 + (((hash + 11) % 20) - 10) / 1000)

  return {
    id: seed.id,
    category_id: seed.category_id,
    symbol: seed.symbol,
    name: seed.name,
    region: seed.region,
    currency: seed.currency,
    price: roundMetric(base, 2),
    change: roundMetric(change, 2),
    change_percent: roundMetric(changePercent, 2),
    open: roundMetric(open, 2),
    high: asset === "futures" ? null : roundMetric(high, 2),
    low: asset === "futures" ? null : roundMetric(low, 2),
    previous_close: roundMetric(previousClose, 2),
    as_of: `2026-06-07 ${String(9 + (index % 8)).padStart(2, "0")}:${String((index * 7) % 60).padStart(2, "0")}:00`,
    technical_rating:
      changePercent >= 2
        ? "Strong buy"
        : changePercent >= 0.4
          ? "Buy"
          : changePercent <= -2
            ? "Strong sell"
            : changePercent <= -0.4
              ? "Sell"
              : "Neutral",
  }
}

function previewBaseValue(asset: MarketAsset, hash: number) {
  switch (asset) {
    case "stocks":
      return 45 + (hash % 420)
    case "etf":
      return 35 + (hash % 250)
    case "crypto":
      return 0.6 + (hash % 700) * 0.9
    case "futures":
      return 20 + (hash % 130)
  }
}

function roundMetric(value: number, digits: number) {
  return Number(value.toFixed(digits))
}
