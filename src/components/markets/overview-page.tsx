import { useNavigate } from "@tanstack/react-router"
import { RefreshCw } from "lucide-react"
import { useEffect, useMemo, useState } from "react"

import { MarketSidebar } from "@/components/markets/sidebar"
import { MarketTableSkeletonBody } from "@/components/markets/table-skeleton"
import { Button } from "@/components/ui/button"
import { ScrollArea } from "@/components/ui/scroll-area"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { useI18n } from "@/lib/i18n"
import {
  getAssetOverview,
  getIndicesOverview,
  type AssetCategoryCount,
  type AssetOverviewRow,
  type IndexCategoryCount,
  type IndexOverviewRow,
  type MarketAsset,
  type MarketProvider,
  type MarketViewTab,
  marketAssetConfigs,
  providerLabels,
  resolvePreferredProvider,
} from "@/lib/market-data"
import { useMarketProviderStore } from "@/lib/market-provider"
import { cn } from "@/lib/utils"

const pageSize = 100

type OverviewKind = "indices" | MarketAsset
type OverviewRow = IndexOverviewRow | AssetOverviewRow
type OverviewCategory = IndexCategoryCount | AssetCategoryCount

interface MarketOverviewPageProps {
  kind: OverviewKind
}

export function MarketOverviewPage({ kind }: MarketOverviewPageProps) {
  const { t, locale } = useI18n()
  const { aggregateProvider } = useMarketProviderStore()
  const selectedProvider = aggregateProvider === "auto" ? "finnhub" : (aggregateProvider ?? "finnhub")
  const requestedProvider = aggregateProvider === "auto" ? undefined : aggregateProvider
  const effectiveProvider = resolvePreferredProvider(kind, selectedProvider) ?? selectedProvider
  const navigate = useNavigate()
  const assetConfig = kind === "indices" ? null : marketAssetConfigs[kind]
  const [selectedTab, setSelectedTab] = useState("overview")
  const [activeFilterId, setActiveFilterId] = useState("all")
  const [currentPage, setCurrentPage] = useState(1)
  const [reloadToken, setReloadToken] = useState(0)
  const [rows, setRows] = useState<OverviewRow[]>([])
  const [categories, setCategories] = useState<OverviewCategory[]>([])
  const [tabs, setTabs] = useState<MarketViewTab[]>([])
  const [updatedAt, setUpdatedAt] = useState<string | null>(null)
  const [sourceNote, setSourceNote] = useState("")
  const [resolvedProvider, setResolvedProvider] = useState<MarketProvider | null>(null)
  const [titleKey, setTitleKey] = useState(
    kind === "indices" ? "indicesTitle" : (assetConfig?.titleI18nKey ?? "stocksTitle")
  )
  const [descriptionKey, setDescriptionKey] = useState(
    kind === "indices"
      ? "indicesDescription"
      : (assetConfig?.descriptionI18nKey ?? "stocksDescription")
  )
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let ignore = false
    void reloadToken

    async function load() {
      setIsLoading(true)
      setError(null)

      try {
        if (kind === "indices") {
          const overview = await getIndicesOverview(activeFilterId, requestedProvider)

          if (ignore) {
            return
          }

          setRows(overview.rows)
          setCategories(overview.categories)
          setTabs(overview.tabs)
          setUpdatedAt(overview.updated_at)
          setSourceNote(overview.source_note)
          setResolvedProvider(overview.provider)
          setTitleKey(overview.title_key)
          setDescriptionKey(overview.description_key)
          setActiveFilterId((current) =>
            current === "all" || overview.categories.some((item) => item.id === current)
              ? current
              : "all"
          )
          setSelectedTab((current) =>
            overview.tabs.some((item) => item.id === current)
              ? current
              : (overview.tabs[0]?.id ?? "overview")
          )
        } else {
          const overview = await getAssetOverview(kind, requestedProvider, activeFilterId)

          if (ignore) {
            return
          }

          setRows(overview.rows)
          setCategories(overview.categories)
          setTabs(overview.tabs)
          setUpdatedAt(overview.updated_at)
          setSourceNote(overview.source_note)
          setResolvedProvider(overview.provider)
          setTitleKey(assetConfig?.titleI18nKey ?? "stocksTitle")
          setDescriptionKey(assetConfig?.descriptionI18nKey ?? "stocksDescription")
          setActiveFilterId((current) =>
            current === "all" || overview.categories.some((item) => item.id === current)
              ? current
              : "all"
          )
          setSelectedTab((current) =>
            overview.tabs.some((item) => item.id === current)
              ? current
              : (overview.tabs[0]?.id ?? "overview")
          )
        }
      } catch (requestError) {
        if (ignore) {
          return
        }

        setRows([])
        setCategories([])
        setTabs([])
        setUpdatedAt(null)
        setSourceNote("")
        setResolvedProvider(null)
        setError(requestError instanceof Error ? requestError.message : String(requestError))
      } finally {
        if (!ignore) {
          setIsLoading(false)
        }
      }
    }

    load()

    return () => {
      ignore = true
    }
  }, [
    activeFilterId,
    assetConfig?.descriptionI18nKey,
    assetConfig?.titleI18nKey,
    effectiveProvider,
    kind,
    reloadToken,
  ])

  const filterItems = useMemo(
    () => [
      { id: "all", label: t("filterAll") },
      ...categories.map((item) => ({ id: item.id, label: t(item.label_key as never) })),
    ],
    [categories, t]
  )

  const filteredRows = useMemo(() => sortRowsForTab(rows, selectedTab), [rows, selectedTab])
  const totalPages = Math.max(1, Math.ceil(filteredRows.length / pageSize))

  useEffect(() => {
    if (currentPage > totalPages) {
      setCurrentPage(totalPages)
    }
  }, [currentPage, totalPages])

  useEffect(() => {
    setCurrentPage(1)
  }, [activeFilterId, selectedTab, kind])

  const pagedRows = useMemo(() => {
    const startIndex = (currentPage - 1) * pageSize
    return filteredRows.slice(startIndex, startIndex + pageSize)
  }, [currentPage, filteredRows])

  const isInitialLoading = isLoading && rows.length === 0

  function openDetail(itemId: string) {
    void navigate({
      to: "/market/$kind/$itemId",
      params: {
        kind,
        itemId,
      },
    })
  }

  return (
    <main className="flex h-full min-h-0 overflow-hidden bg-background text-foreground">
      <MarketSidebar
        footer={formatAggregateProvider(
          selectedProvider,
          effectiveProvider,
          resolvedProvider,
          t as (key: string) => string
        )}
      />

      <section className="min-w-0 flex-1 overflow-hidden bg-background">
        <div className="mx-auto flex h-full min-h-0 w-full max-w-[1600px] flex-col px-5 pt-6 pb-6 sm:px-8">
          <div className="max-w-[920px] shrink-0">
            <h1 className="text-[22px] font-semibold tracking-normal text-foreground">
              {t(titleKey as never)}
            </h1>
            <p className="mt-3 text-[15px] leading-8 text-foreground/80">
              {t(descriptionKey as never)}
            </p>
          </div>

          <div className="mt-6 shrink-0 flex flex-wrap gap-2 border-b border-border/60 pb-4">
            {filterItems.map((item) => {
              const isActive = activeFilterId === item.id

              return (
                <button
                  type="button"
                  key={item.id}
                  onClick={() => setActiveFilterId(item.id)}
                  className={cn(
                    "inline-flex h-8 items-center rounded-md border px-3 text-[13px] transition-colors",
                    isActive
                      ? "border-foreground/18 bg-accent text-foreground"
                      : "border-border/70 bg-background text-muted-foreground hover:text-foreground"
                  )}
                >
                  {item.label}
                </button>
              )
            })}
          </div>

          <div className="mt-4 flex shrink-0 items-center justify-between gap-4 border-b border-border/60">
            <Tabs value={selectedTab} onValueChange={setSelectedTab} className="gap-0">
              <TabsList variant="line" className="h-11 gap-5 p-0 text-muted-foreground">
                {tabs.map((tab) => (
                  <TabsTrigger
                    key={tab.id}
                    value={tab.id}
                    className="h-11 rounded-none px-0 text-[15px] data-[state=active]:text-foreground after:bg-primary"
                  >
                    {t(tab.label_key as never)}
                  </TabsTrigger>
                ))}
              </TabsList>
            </Tabs>

            <div className="mb-2 flex items-center gap-3">
              <div className="hidden text-right text-xs text-muted-foreground md:block">
                <div>
                  {formatAggregateProvider(
                    selectedProvider,
                    effectiveProvider,
                    resolvedProvider,
                    t as (key: string) => string
                  )}
                </div>
                <div>{updatedAt ?? sourceNote}</div>
              </div>
              <Button
                type="button"
                variant="outline"
                size="icon-sm"
                className="border-border/60 bg-background text-foreground hover:bg-accent/50 hover:text-foreground"
                onClick={() => setReloadToken((value) => value + 1)}
                disabled={isLoading}
                aria-label={t("marketsRefresh")}
              >
                <RefreshCw className={cn("size-4", isLoading && "animate-spin")} />
              </Button>
            </div>
          </div>

          {error ? (
            <div className="mt-4 shrink-0 border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
              {t("marketsLoadFailed")}: {error}
            </div>
          ) : null}

          <ScrollArea className={cn("min-h-0 flex-1", error ? "mt-4" : "mt-6")}>
            <div className="pb-4 pr-3">
              <section className="overflow-hidden rounded-lg border border-border/70 bg-background">
                <Table className="min-w-[1180px]">
                  <TableHeader>
                    <TableRow className="border-border/70 hover:bg-transparent">
                      <TableHead className="h-auto px-0 py-3 text-xs font-medium text-muted-foreground">
                        <div className="pl-4">
                          <div>{t("indicesTableSymbol")}</div>
                          <div className="mt-1">{filteredRows.length}</div>
                        </div>
                      </TableHead>
                      <TableHead className="px-3 py-3 text-right text-xs font-medium text-muted-foreground">
                        {t("indicesTablePrice")}
                      </TableHead>
                      <TableHead className="px-3 py-3 text-right text-xs font-medium text-muted-foreground">
                        {t("indicesTableChangePct")}
                      </TableHead>
                      <TableHead className="px-3 py-3 text-right text-xs font-medium text-muted-foreground">
                        {t("indicesTableChange")}
                      </TableHead>
                      <TableHead className="px-3 py-3 text-right text-xs font-medium text-muted-foreground">
                        {t("indicesTableHigh")}
                      </TableHead>
                      <TableHead className="px-3 py-3 text-right text-xs font-medium text-muted-foreground">
                        {t("indicesTableLow")}
                      </TableHead>
                      <TableHead className="px-4 py-3 text-right text-xs font-medium text-muted-foreground">
                        {t("indicesTableTechRating")}
                      </TableHead>
                    </TableRow>
                  </TableHeader>
                  {isInitialLoading ? (
                    <MarketTableSkeletonBody rowCount={8} />
                  ) : (
                    <TableBody>
                      {pagedRows.length === 0 ? (
                        <TableRow className="border-border/50 hover:bg-transparent">
                          <TableCell
                            colSpan={7}
                            className="px-4 py-8 text-center text-sm text-muted-foreground"
                          >
                            {t("marketsNoData")}
                          </TableCell>
                        </TableRow>
                      ) : (
                        pagedRows.map((row) => (
                          <TableRow
                            key={row.id}
                            className="cursor-pointer border-border/50 text-[14px] hover:bg-accent/25"
                            onClick={() => openDetail(row.id)}
                          >
                            <TableCell className="px-0 py-0">
                              <div className="grid min-h-[44px] grid-cols-[44px_minmax(0,1fr)] items-center gap-0 pl-4">
                                <AssetBadge symbol={row.symbol} />
                                <div className="min-w-0 py-3">
                                  <div className="flex items-center gap-3">
                                    <span className="rounded bg-accent/50 px-2 py-1 text-[12px] leading-none text-foreground">
                                      {row.symbol}
                                    </span>
                                    <span className="truncate text-foreground">{row.name}</span>
                                  </div>
                                </div>
                              </div>
                            </TableCell>
                            <TableCell className="px-3 py-3 text-right text-foreground">
                              {formatMarketValue(row.price, locale, 2)}
                              {row.currency ? (
                                <span className="ml-1 text-[10px] uppercase text-muted-foreground">
                                  {row.currency}
                                </span>
                              ) : null}
                            </TableCell>
                            <TableCell
                              className={cn(
                                "px-3 py-3 text-right",
                                getSignedColorClass(row.change_percent)
                              )}
                            >
                              {formatPercent(row.change_percent, locale)}
                            </TableCell>
                            <TableCell
                              className={cn(
                                "px-3 py-3 text-right",
                                getSignedColorClass(row.change)
                              )}
                            >
                              {formatSignedValue(row.change, locale, 2)}
                              {row.currency ? (
                                <span className="ml-1 text-[10px] uppercase opacity-80">
                                  {row.currency}
                                </span>
                              ) : null}
                            </TableCell>
                            <TableCell className="px-3 py-3 text-right text-foreground">
                              {formatMarketValue(row.high, locale, 2)}
                            </TableCell>
                            <TableCell className="px-3 py-3 text-right text-foreground">
                              {formatMarketValue(row.low, locale, 2)}
                            </TableCell>
                            <TableCell className="px-4 py-3 text-right">
                              <span
                                className={cn("text-[14px]", getRatingClass(row.technical_rating))}
                              >
                                {row.technical_rating}
                              </span>
                            </TableCell>
                          </TableRow>
                        ))
                      )}
                    </TableBody>
                  )}
                </Table>
              </section>

              <div className="mt-4 grid grid-cols-[1fr_auto_1fr] items-center gap-4 text-xs text-muted-foreground">
                <div className="justify-self-start">
                  {filteredRows.length === 0
                    ? "0-0 / 0"
                    : `${(currentPage - 1) * pageSize + 1}-${Math.min(currentPage * pageSize, filteredRows.length)} / ${filteredRows.length}`}
                </div>
                <div className="min-w-20 text-center">
                  {t("paginationPage")} {currentPage} / {totalPages}
                </div>
                <div className="flex items-center justify-self-end gap-2">
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    disabled={currentPage <= 1}
                    onClick={() => setCurrentPage((value) => Math.max(1, value - 1))}
                  >
                    {t("paginationPrevious")}
                  </Button>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    disabled={currentPage >= totalPages}
                    onClick={() => setCurrentPage((value) => Math.min(totalPages, value + 1))}
                  >
                    {t("paginationNext")}
                  </Button>
                </div>
              </div>
            </div>
          </ScrollArea>
        </div>
      </section>
    </main>
  )
}

function AssetBadge({ symbol }: { symbol: string }) {
  const seed = symbol.charCodeAt(0) + symbol.charCodeAt(symbol.length - 1)
  const palette = [
    "bg-[#c93145]",
    "bg-[#1d9ecb]",
    "bg-[#2182ce]",
    "bg-[#2d9c5a]",
    "bg-[#9a3d7b]",
    "bg-[#4d71c8]",
    "bg-[#ca7b2a]",
    "bg-[#7f49a9]",
  ]
  const color = palette[seed % palette.length]

  return (
    <div className="flex items-center justify-center">
      <div
        className={cn(
          "flex size-6 items-center justify-center rounded-full text-[10px] font-semibold text-white",
          color
        )}
      >
        {symbol.slice(0, 2)}
      </div>
    </div>
  )
}

function sortRowsForTab(rows: OverviewRow[], tab: string) {
  const next = [...rows]

  if (tab === "performance") {
    return next.sort((left, right) => compareMetric(right.change_percent, left.change_percent))
  }

  if (tab === "technicals") {
    return next.sort((left, right) => {
      const ratingDelta =
        technicalWeight(right.technical_rating) - technicalWeight(left.technical_rating)

      if (ratingDelta !== 0) {
        return ratingDelta
      }

      return compareMetric(right.change_percent, left.change_percent)
    })
  }

  return next
}

function formatMarketValue(
  value: number | null | undefined,
  locale: string,
  maximumFractionDigits = 2
) {
  if (typeof value !== "number") {
    return "--"
  }

  const abs = Math.abs(value)
  const digits = abs >= 1000 ? 2 : abs >= 100 ? 2 : abs >= 1 ? 2 : 4

  return new Intl.NumberFormat(locale, {
    minimumFractionDigits: 0,
    maximumFractionDigits: Math.max(maximumFractionDigits, digits),
  }).format(value)
}

function formatSignedValue(
  value: number | null | undefined,
  locale: string,
  maximumFractionDigits = 2
) {
  if (typeof value !== "number") {
    return "--"
  }

  const formatted = formatMarketValue(Math.abs(value), locale, maximumFractionDigits)
  return `${value > 0 ? "+" : value < 0 ? "-" : ""}${formatted}`
}

function formatPercent(value: number | null | undefined, locale: string) {
  if (typeof value !== "number") {
    return "--"
  }

  const formatted = new Intl.NumberFormat(locale, {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(Math.abs(value))

  return `${value > 0 ? "+" : value < 0 ? "-" : ""}${formatted}%`
}

function getSignedColorClass(value: number | null | undefined) {
  if (typeof value !== "number") {
    return "text-muted-foreground"
  }

  if (value > 0) {
    return "text-emerald-600 dark:text-emerald-300"
  }

  if (value < 0) {
    return "text-rose-600 dark:text-rose-300"
  }

  return "text-muted-foreground"
}

function getRatingClass(value: string) {
  if (value === "Strong buy" || value === "Buy") {
    return "text-emerald-600 dark:text-emerald-300"
  }

  if (value === "Strong sell" || value === "Sell") {
    return "text-rose-600 dark:text-rose-300"
  }

  return "text-amber-700 dark:text-amber-300"
}

function compareMetric(left: number | null | undefined, right: number | null | undefined) {
  const leftValue = typeof left === "number" ? left : Number.NEGATIVE_INFINITY
  const rightValue = typeof right === "number" ? right : Number.NEGATIVE_INFINITY

  if (leftValue === rightValue) {
    return 0
  }

  return leftValue > rightValue ? 1 : -1
}

function technicalWeight(value: string) {
  if (value === "Strong buy") {
    return 4
  }

  if (value === "Buy") {
    return 3
  }

  if (value === "Neutral") {
    return 2
  }

  if (value === "Sell") {
    return 1
  }

  return 0
}

function formatAggregateProvider(
  aggregateProvider: string,
  effectiveProvider: string,
  resolvedProvider: MarketProvider | null,
  t: (key: string) => string
) {
  const selectedLabel = providerLabels[aggregateProvider as MarketProvider] ?? aggregateProvider
  const effectiveLabel = providerLabels[effectiveProvider as MarketProvider] ?? effectiveProvider

  if (!resolvedProvider) {
    return `${t("marketsProvider")}: ${selectedLabel}`
  }

  const resolvedLabel = providerLabels[resolvedProvider] ?? resolvedProvider

  if (aggregateProvider === effectiveProvider) {
    return `${t("marketsProvider")}: ${resolvedLabel}`
  }

  return `${t("marketsProvider")}: ${selectedLabel} -> ${effectiveLabel} -> ${resolvedLabel}`
}
