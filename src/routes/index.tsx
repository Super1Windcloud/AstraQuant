import { createFileRoute } from "@tanstack/react-router"
import { RefreshCw } from "lucide-react"
import { useEffect, useMemo, useState } from "react"

import { Button } from "@/components/ui/button"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"
import {
  getIndicesOverview,
  indexCategories,
  type IndexOverviewRow,
  type IndicesCategory,
  indexCategoryIds,
  providerLabels,
  type MarketProvider,
} from "@/lib/market-data"
import { useI18n } from "@/lib/i18n"
import { cn } from "@/lib/utils"

export const Route = createFileRoute("/")({
  component: IndicesPage,
})

const defaultCategory: IndicesCategory = "all"
const defaultProvider: MarketProvider | undefined = undefined

function IndicesPage() {
  const { t, locale } = useI18n()
  const [selectedCategory, setSelectedCategory] = useState<IndicesCategory>(defaultCategory)
  const [selectedTab, setSelectedTab] = useState("overview")
  const provider = defaultProvider
  const [reloadToken, setReloadToken] = useState(0)
  const [rows, setRows] = useState<IndexOverviewRow[]>([])
  const [categoryCounts, setCategoryCounts] = useState<Record<IndicesCategory, number>>(
    buildEmptyCategoryCounts
  )
  const [updatedAt, setUpdatedAt] = useState<string | null>(null)
  const [sourceNote, setSourceNote] = useState("")
  const [resolvedProvider, setResolvedProvider] = useState<MarketProvider | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let ignore = false

    async function load() {
      setIsLoading(true)
      setError(null)

      try {
        const overview = await getIndicesOverview(selectedCategory, provider)

        if (ignore) {
          return
        }

        setRows(overview.rows)
        setUpdatedAt(overview.updated_at)
        setSourceNote(overview.source_note)
        setResolvedProvider(overview.provider)
        setCategoryCounts(() => {
          const next = buildEmptyCategoryCounts()

          for (const item of overview.categories) {
            next[item.id] = item.total
          }

          return next
        })
      } catch (requestError) {
        if (ignore) {
          return
        }

        setRows([])
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
  }, [provider, reloadToken, selectedCategory])

  const displayRows = useMemo(() => {
    const next = [...rows]

    if (selectedTab === "performance") {
      return next.sort((left, right) => compareMetric(right.change_percent, left.change_percent))
    }

    if (selectedTab === "technicals") {
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
  }, [rows, selectedTab])

  const countLabel = useMemo(() => {
    const total = categoryCounts[selectedCategory]
    return `${t("indicesTableSymbol")} ${total}`
  }, [categoryCounts, selectedCategory, t])

  return (
    <main className="min-h-full bg-[#1b1d1f] text-[#f2f2f2]">
      <section className="mx-auto flex w-full max-w-[1600px] flex-col px-5 pt-6 pb-8 sm:px-8">
        <div className="flex flex-wrap gap-2">
          {indexCategories.map((category) => {
            const isActive = selectedCategory === category.id

            return (
              <button
                type="button"
                key={category.id}
                onClick={() => setSelectedCategory(category.id)}
                className={cn(
                  "inline-flex h-8 items-center rounded-full border px-3 text-[13px] transition-colors",
                  isActive
                    ? "border-[#6b5a42] bg-[#2a2d31] text-[#f5e5cf]"
                    : "border-[#34373b] bg-[#202326] text-[#d5d7da] hover:bg-[#26292d]"
                )}
              >
                {t(category.i18nKey as never)}
              </button>
            )
          })}
        </div>

        <div className="mt-7 max-w-[760px]">
          <h1 className="text-[22px] font-semibold tracking-normal text-[#f1e2d0]">
            {t("indicesTitle")}
          </h1>
          <p className="mt-3 text-[15px] leading-8 text-[#ddd3c5]">{t("indicesDescription")}</p>
        </div>

        <div className="mt-8 flex items-center justify-between gap-4 border-b border-[#31353a]">
          <Tabs value={selectedTab} onValueChange={setSelectedTab} className="gap-0">
            <TabsList variant="line" className="h-11 gap-5 p-0 text-[#aab0b6]">
              <TabsTrigger
                value="overview"
                className="h-11 rounded-none px-0 text-[15px] data-[state=active]:text-[#f3f4f6] after:bg-[#d7ba95]"
              >
                {t("indicesTabOverview")}
              </TabsTrigger>
              <TabsTrigger
                value="performance"
                className="h-11 rounded-none px-0 text-[15px] data-[state=active]:text-[#f3f4f6] after:bg-[#d7ba95]"
              >
                {t("indicesTabPerformance")}
              </TabsTrigger>
              <TabsTrigger
                value="technicals"
                className="h-11 rounded-none px-0 text-[15px] data-[state=active]:text-[#f3f4f6] after:bg-[#d7ba95]"
              >
                {t("indicesTabTechnicals")}
              </TabsTrigger>
            </TabsList>
          </Tabs>

          <div className="mb-2 flex items-center gap-3">
            <div className="hidden text-right text-xs text-[#8f959c] md:block">
              <div>{resolvedProvider ? providerLabels[resolvedProvider] : "--"}</div>
              <div>{updatedAt ?? sourceNote}</div>
            </div>
            <Button
              type="button"
              variant="outline"
              size="icon-sm"
              className="border-[#3a3e44] bg-[#202326] text-[#dfe3e6] hover:bg-[#292d31] hover:text-[#f5f5f5]"
              onClick={() => setReloadToken((value) => value + 1)}
              disabled={isLoading}
              aria-label={t("indicesRefresh")}
            >
              <RefreshCw className={cn("size-4", isLoading && "animate-spin")} />
            </Button>
          </div>
        </div>

        {error ? (
          <div className="mt-4 border border-[#5f2d2d] bg-[#2b1d1d] px-4 py-3 text-sm text-[#ffb4b4]">
            {t("indicesLoadFailed")}: {error}
          </div>
        ) : null}

        <section className="mt-1 overflow-hidden">
          <Table className="min-w-[1180px]">
            <TableHeader>
              <TableRow className="border-[#31353a] hover:bg-transparent">
                <TableHead className="h-auto px-0 py-3 text-xs font-medium text-[#8d949b]">
                  <div className="pl-4">
                    <div>{t("indicesTableSymbol")}</div>
                    <div className="mt-1">{categoryCounts[selectedCategory]}</div>
                  </div>
                </TableHead>
                <TableHead className="px-3 py-3 text-right text-xs font-medium text-[#8d949b]">
                  {t("indicesTablePrice")}
                </TableHead>
                <TableHead className="px-3 py-3 text-right text-xs font-medium text-[#8d949b]">
                  {t("indicesTableChangePct")}
                </TableHead>
                <TableHead className="px-3 py-3 text-right text-xs font-medium text-[#8d949b]">
                  {t("indicesTableChange")}
                </TableHead>
                <TableHead className="px-3 py-3 text-right text-xs font-medium text-[#8d949b]">
                  {t("indicesTableHigh")}
                </TableHead>
                <TableHead className="px-3 py-3 text-right text-xs font-medium text-[#8d949b]">
                  {t("indicesTableLow")}
                </TableHead>
                <TableHead className="px-4 py-3 text-right text-xs font-medium text-[#8d949b]">
                  {t("indicesTableTechRating")}
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {displayRows.length === 0 ? (
                <TableRow className="border-[#2f3338] hover:bg-transparent">
                  <TableCell colSpan={7} className="px-4 py-8 text-center text-sm text-[#98a0a8]">
                    {isLoading ? t("waitingForData") : t("indicesNoData")}
                  </TableCell>
                </TableRow>
              ) : (
                displayRows.map((row) => (
                  <TableRow
                    key={row.id}
                    className="border-[#2f3338] text-[14px] hover:bg-[#202428]/80"
                  >
                    <TableCell className="px-0 py-0">
                      <div className="grid min-h-[44px] grid-cols-[44px_minmax(0,1fr)] items-center gap-0 pl-4">
                        <IndexBadge symbol={row.symbol} />
                        <div className="min-w-0 py-3">
                          <div className="flex items-center gap-3">
                            <span className="rounded bg-[#25292d] px-2 py-1 text-[12px] leading-none text-[#f0f3f5]">
                              {row.symbol}
                            </span>
                            <span className="truncate text-[#f2f4f6]">{row.name}</span>
                          </div>
                        </div>
                      </div>
                    </TableCell>
                    <TableCell className="px-3 py-3 text-right text-[#eff3f6]">
                      {formatMarketValue(row.price, locale, 2)}
                      {row.currency ? (
                        <span className="ml-1 text-[10px] uppercase text-[#a0a7ae]">
                          {row.currency}
                        </span>
                      ) : null}
                    </TableCell>
                    <TableCell
                      className={cn("px-3 py-3 text-right", getSignedColorClass(row.change_percent))}
                    >
                      {formatPercent(row.change_percent, locale)}
                    </TableCell>
                    <TableCell
                      className={cn("px-3 py-3 text-right", getSignedColorClass(row.change))}
                    >
                      {formatSignedValue(row.change, locale, 2)}
                      {row.currency ? (
                        <span className="ml-1 text-[10px] uppercase opacity-80">{row.currency}</span>
                      ) : null}
                    </TableCell>
                    <TableCell className="px-3 py-3 text-right text-[#edf1f4]">
                      {formatMarketValue(row.high, locale, 2)}
                    </TableCell>
                    <TableCell className="px-3 py-3 text-right text-[#edf1f4]">
                      {formatMarketValue(row.low, locale, 2)}
                    </TableCell>
                    <TableCell className="px-4 py-3 text-right">
                      <span className={cn("text-[14px]", getRatingClass(row.technical_rating))}>
                        {row.technical_rating}
                      </span>
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </section>

        <div className="mt-4 flex items-center justify-between text-xs text-[#818891]">
          <div>{countLabel}</div>
          <div className="text-right">
            <div>{sourceNote || t("indicesPreviewSource")}</div>
            {updatedAt ? <div className="mt-1">{updatedAt}</div> : null}
          </div>
        </div>
      </section>
    </main>
  )
}

function IndexBadge({ symbol }: { symbol: string }) {
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

function buildEmptyCategoryCounts(): Record<IndicesCategory, number> {
  return indexCategoryIds.reduce(
    (result, id) => {
      result[id] = 0
      return result
    },
    {} as Record<IndicesCategory, number>
  )
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
  const formatted = new Intl.NumberFormat(locale, {
    minimumFractionDigits: 0,
    maximumFractionDigits: Math.max(maximumFractionDigits, digits),
  }).format(value)

  return formatted
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
    return "text-[#b1b7be]"
  }

  if (value > 0) {
    return "text-[#5ee7c2]"
  }

  if (value < 0) {
    return "text-[#ff6f79]"
  }

  return "text-[#b1b7be]"
}

function getRatingClass(value: string) {
  if (value === "Strong buy" || value === "Buy") {
    return "text-[#63e0bf]"
  }

  if (value === "Strong sell" || value === "Sell") {
    return "text-[#ff7078]"
  }

  return "text-[#b8a88f]"
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
