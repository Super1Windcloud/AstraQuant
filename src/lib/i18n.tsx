import { createContext, type ReactNode, useContext, useEffect, useMemo, useState } from "react"

export type Locale = "zh-CN" | "en-US"

const localeStorageKey = "astraquant-locale"

const messages = {
  "zh-CN": {
    appSubtitle: "市场终端",
    appIcon: "应用图标",
    appIconLogo1: "图标 1",
    appIconLogo2: "图标 2",
    language: "语言",
    theme: "主题",
    themeLight: "浅色",
    themeDark: "深色",
    themeDim: "柔暗",
    themeOcean: "深海",
    themeAvocado: "绿调",
    themeSystem: "跟随系统",
    sidebarToggleClose: "折叠侧边栏",
    sidebarToggleOpen: "展开侧边栏",
    searchSymbol: "搜索 symbol",
    refreshQuotes: "刷新行情",
    waitingForData: "等待数据",
    asOf: "截至",
    chartArea: "行情图表区域",
    chartPlaceholder: "聚合接口已就绪，可在这里接入 Lightweight Charts 或 K 线数据。",
    overview: "概览",
    api: "接口",
    notReturned: "未返回",
    pageNotFound: "页面不存在",
    pageNotFoundDescription: "找不到你要访问的页面。",
    pageNotFoundAction: "使用导航返回应用。",
    assetStock: "股票",
    assetIndex: "指数",
    assetEtf: "ETF",
    assetCrypto: "加密货币",
    assetForex: "外汇",
    assetFuture: "期货",
    indicesTitle: "All indices in one place",
    indicesDescription:
      "A market index measures the performance of a basket of assets to help get an idea of market or sector sentiment. Below you can see, browse, and compare major indices, US indices, and more.",
    indicesTabOverview: "Overview",
    indicesTabPerformance: "Performance",
    indicesTabTechnicals: "Technicals",
    indicesFilterAll: "All indices",
    indicesFilterMajor: "Major world indices",
    indicesFilterUs: "US indices",
    indicesFilterSectors: "S&P sectors",
    indicesFilterCurrency: "Currency indices",
    indicesFilterAmericas: "Americas",
    indicesFilterEurope: "Europe",
    indicesFilterAsia: "Asia",
    indicesFilterPacific: "Pacific",
    indicesFilterMiddleEast: "Middle East",
    indicesFilterAfrica: "Africa",
    indicesTableSymbol: "Symbol",
    indicesTablePrice: "Price",
    indicesTableChangePct: "Chg %",
    indicesTableChange: "Chg",
    indicesTableHigh: "High",
    indicesTableLow: "Low",
    indicesTableTechRating: "Tech rating",
    indicesRefresh: "刷新指数",
    indicesLoadFailed: "指数数据加载失败",
    indicesNoData: "当前分类没有返回数据。",
    indicesPreviewSource: "浏览器预览数据",
  },
  "en-US": {
    appSubtitle: "Market Terminal",
    appIcon: "App icon",
    appIconLogo1: "Icon 1",
    appIconLogo2: "Icon 2",
    language: "Language",
    theme: "Theme",
    themeLight: "Light",
    themeDark: "Dark",
    themeDim: "Dim",
    themeOcean: "Ocean",
    themeAvocado: "Avocado",
    themeSystem: "System",
    sidebarToggleClose: "Collapse sidebar",
    sidebarToggleOpen: "Expand sidebar",
    searchSymbol: "Search symbol",
    refreshQuotes: "Refresh quotes",
    waitingForData: "Waiting for data",
    asOf: "as of",
    chartArea: "Chart workspace",
    chartPlaceholder:
      "Data aggregation is ready. Connect Lightweight Charts or candlestick data here.",
    overview: "Overview",
    api: "API",
    notReturned: "Not returned",
    pageNotFound: "Page not found",
    pageNotFoundDescription: "The page you are looking for does not exist.",
    pageNotFoundAction: "Use navigation to return to the app.",
    assetStock: "Stocks",
    assetIndex: "Indices",
    assetEtf: "ETF",
    assetCrypto: "Crypto",
    assetForex: "Forex",
    assetFuture: "Futures",
    indicesTitle: "All indices in one place",
    indicesDescription:
      "A market index measures the performance of a basket of assets to help get an idea of market or sector sentiment. Below you can see, browse, and compare major indices, US indices, and more.",
    indicesTabOverview: "Overview",
    indicesTabPerformance: "Performance",
    indicesTabTechnicals: "Technicals",
    indicesFilterAll: "All indices",
    indicesFilterMajor: "Major world indices",
    indicesFilterUs: "US indices",
    indicesFilterSectors: "S&P sectors",
    indicesFilterCurrency: "Currency indices",
    indicesFilterAmericas: "Americas",
    indicesFilterEurope: "Europe",
    indicesFilterAsia: "Asia",
    indicesFilterPacific: "Pacific",
    indicesFilterMiddleEast: "Middle East",
    indicesFilterAfrica: "Africa",
    indicesTableSymbol: "Symbol",
    indicesTablePrice: "Price",
    indicesTableChangePct: "Chg %",
    indicesTableChange: "Chg",
    indicesTableHigh: "High",
    indicesTableLow: "Low",
    indicesTableTechRating: "Tech rating",
    indicesRefresh: "Refresh indices",
    indicesLoadFailed: "Failed to load index data",
    indicesNoData: "No data returned for this category.",
    indicesPreviewSource: "Browser preview data",
  },
} as const

type MessageKey = keyof (typeof messages)["zh-CN"]

const localeLabels: Record<Locale, string> = {
  "zh-CN": "简体中文",
  "en-US": "English",
}

interface I18nContextValue {
  locale: Locale
  setLocale: (locale: Locale) => void
  t: (key: MessageKey) => string
}

const I18nContext = createContext<I18nContextValue | null>(null)

function readStoredLocale(): Locale {
  if (typeof window === "undefined") {
    return "zh-CN"
  }

  const stored = window.localStorage.getItem(localeStorageKey)
  return stored === "en-US" || stored === "zh-CN" ? stored : "zh-CN"
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(readStoredLocale)

  useEffect(() => {
    document.documentElement.lang = locale
    window.localStorage.setItem(localeStorageKey, locale)
  }, [locale])

  const value = useMemo<I18nContextValue>(
    () => ({
      locale,
      setLocale: setLocaleState,
      t: (key) => messages[locale][key],
    }),
    [locale]
  )

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>
}

export function useI18n() {
  const context = useContext(I18nContext)

  if (!context) {
    throw new Error("useI18n must be used within I18nProvider")
  }

  return context
}

export function getLocaleLabel(locale: Locale) {
  return localeLabels[locale]
}

export const locales: Locale[] = ["zh-CN", "en-US"]
