import { Link } from "@tanstack/react-router"
import { ArrowLeft, ChartCandlestick, RefreshCw } from "lucide-react"
import { useEffect, useState } from "react"

import { MarketPriceChart } from "@/components/markets/market-price-chart"
import { MarketSidebar } from "@/components/markets/sidebar"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { type Locale, useI18n } from "@/lib/i18n"
import {
  getMarketChartSeries,
  getMarketItemDetail,
  type MarketChartSeriesResponse,
  type MarketItemDetailResponse,
  type MarketKind,
  type MarketProvider,
  providerLabels,
  resolvePreferredProvider,
} from "@/lib/market-data"
import { useMarketProviderStore } from "@/lib/market-provider"
import { cn } from "@/lib/utils"

interface MarketDetailPageProps {
  itemId: string
  kind: MarketKind
}

export function MarketDetailPage({ itemId, kind }: MarketDetailPageProps) {
  const { t, locale } = useI18n()
  const { aggregateProvider } = useMarketProviderStore()
  const effectiveProvider = resolvePreferredProvider(kind, aggregateProvider)
  const [reloadToken, setReloadToken] = useState(0)
  const [detail, setDetail] = useState<MarketItemDetailResponse | null>(null)
  const [chartSeries, setChartSeries] = useState<MarketChartSeriesResponse | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [isChartLoading, setIsChartLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [chartError, setChartError] = useState<string | null>(null)
  const [isChartDialogOpen, setIsChartDialogOpen] = useState(false)

  useEffect(() => {
    let ignore = false
    void reloadToken

    async function load() {
      setIsLoading(true)
      setIsChartLoading(true)
      setError(null)
      setChartError(null)
      setChartSeries(null)

      const [detailResult, chartResult] = await Promise.allSettled([
        getMarketItemDetail(kind, itemId, effectiveProvider),
        getMarketChartSeries(kind, itemId, effectiveProvider),
      ])

      if (ignore) {
        return
      }

      if (detailResult.status === "fulfilled") {
        setDetail(detailResult.value)
      } else {
        setDetail(null)
        setError(
          detailResult.reason instanceof Error
            ? detailResult.reason.message
            : String(detailResult.reason)
        )
      }

      if (chartResult.status === "fulfilled" && chartResult.value.points.length > 0) {
        setChartSeries(chartResult.value)
      } else if (chartResult.status === "rejected") {
        setChartError(
          chartResult.reason instanceof Error
            ? chartResult.reason.message
            : String(chartResult.reason)
        )
      }

      setIsLoading(false)
      setIsChartLoading(false)
    }

    load()

    return () => {
      ignore = true
    }
  }, [effectiveProvider, itemId, kind, reloadToken])

  const listPath = kind === "indices" ? "/" : `/${kind}`
  const sidebarFooter = formatAggregateProvider(
    aggregateProvider,
    effectiveProvider,
    detail?.provider ?? null,
    t
  )
  const hasChartData = Boolean(chartSeries && chartSeries.points.length > 0)
  const chartSourceLabel = chartSeries
    ? `${getProviderLabel(chartSeries.provider)} · ${chartSeries.source_note}`
    : null

  return (
    <main className="flex min-h-full bg-background text-foreground">
      <MarketSidebar footer={sidebarFooter} />

      <section className="min-w-0 flex-1 bg-background">
        <div className="mx-auto flex w-full max-w-[1600px] flex-col px-5 pt-6 pb-8 sm:px-8">
          <div className="flex items-center justify-between gap-4">
            <Link
              to={listPath as never}
              className="inline-flex items-center gap-2 text-sm text-muted-foreground transition-colors hover:text-foreground"
            >
              <ArrowLeft className="size-4" />
              {t("detailBackToList")}
            </Link>

            <Button
              type="button"
              variant="outline"
              size="icon-sm"
              className="border-border/60 bg-background/68 text-foreground backdrop-blur-md hover:bg-accent/50 hover:text-foreground"
              onClick={() => setReloadToken((value) => value + 1)}
              disabled={isLoading}
              aria-label={t("marketsRefresh")}
            >
              <RefreshCw className={cn("size-4", isLoading && "animate-spin")} />
            </Button>
          </div>

          {error ? (
            <div className="mt-4 border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
              {t("detailLoadFailed")}: {error}
            </div>
          ) : null}

          <div className="mt-6 grid gap-6 xl:grid-cols-[minmax(0,1fr)_320px]">
            <section className="min-w-0">
              <div className="rounded-xl border border-border/60 bg-background/72 px-6 py-6 backdrop-blur-xl supports-[backdrop-filter]:bg-background/58">
                <div className="flex flex-wrap items-center gap-3">
                  <span className="rounded-md bg-accent/45 px-2 py-1 text-[12px] font-medium text-foreground">
                    {detail?.symbol ?? itemId.toUpperCase()}
                  </span>
                  {detail?.technical_rating ? (
                    <span
                      className={cn("text-sm font-medium", getRatingClass(detail.technical_rating))}
                    >
                      {detail.technical_rating}
                    </span>
                  ) : null}
                </div>

                <h1 className="mt-4 text-[26px] font-semibold text-foreground">
                  {detail?.name ?? itemId}
                </h1>

                <div className="mt-3 flex flex-wrap gap-x-6 gap-y-2 text-sm text-muted-foreground">
                  <span>
                    {t("detailRegion")}: {detail?.region ?? "--"}
                  </span>
                  <span>
                    {t("detailProvider")}:{" "}
                    {detail?.provider ? providerLabels[detail.provider] : "--"}
                  </span>
                  <span>
                    {t("detailUpdatedAt")}: {detail?.as_of ?? "--"}
                  </span>
                </div>

                <div className="mt-6 grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
                  <MetricCard
                    label={t("indicesTablePrice")}
                    value={formatMarketValue(detail?.price, locale, 2)}
                    accent={getSignedColorClass(detail?.change)}
                  />
                  <MetricCard
                    label={t("indicesTableChangePct")}
                    value={formatPercent(detail?.change_percent, locale)}
                    accent={getSignedColorClass(detail?.change_percent)}
                  />
                  <MetricCard
                    label={t("indicesTableChange")}
                    value={formatSignedValue(detail?.change, locale, 2)}
                    accent={getSignedColorClass(detail?.change)}
                  />
                  <MetricCard
                    label={t("indicesTableHigh")}
                    value={formatMarketValue(detail?.high, locale, 2)}
                  />
                  <MetricCard
                    label={t("indicesTableLow")}
                    value={formatMarketValue(detail?.low, locale, 2)}
                  />
                  <MetricCard
                    label={t("detailPreviousClose")}
                    value={formatMarketValue(detail?.previous_close, locale, 2)}
                  />
                </div>
              </div>

              <div className="mt-6 rounded-xl border border-border/60 bg-background/72 px-4 py-4 backdrop-blur-xl supports-[backdrop-filter]:bg-background/58">
                <div className="mb-4 flex items-center justify-between gap-4">
                  <div className="text-sm font-medium text-foreground">{t("detailChartTitle")}</div>
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon-sm"
                    className="text-muted-foreground hover:text-foreground"
                    onClick={() => setIsChartDialogOpen(true)}
                    disabled={!hasChartData}
                    aria-label={t("detailChartOpen")}
                    title={t("detailChartOpen")}
                  >
                    <ChartCandlestick className="size-4" />
                  </Button>
                </div>

                {isChartLoading ? (
                  <div className="flex h-[520px] items-center justify-center rounded-lg border border-dashed border-border/60 text-sm text-muted-foreground">
                    {t("detailChartLoading")}
                  </div>
                ) : hasChartData && chartSeries ? (
                  <>
                    <MarketPriceChart
                      data={chartSeries.points}
                      locale={locale}
                      seriesType={chartSeries.series_type}
                    />
                    <div className="mt-3 flex flex-wrap items-center justify-between gap-3 text-xs text-muted-foreground">
                      <span>{chartSourceLabel}</span>
                      <a
                        href="https://www.tradingview.com/"
                        target="_blank"
                        rel="noreferrer"
                        className="transition-colors hover:text-foreground"
                      >
                        {t("detailChartAttribution")}
                      </a>
                    </div>
                  </>
                ) : chartError ? (
                  <div className="flex h-[520px] items-center justify-center rounded-lg border border-dashed border-border/60 px-6 text-center text-sm text-muted-foreground">
                    {t("detailChartLoadFailed")}: {chartError}
                  </div>
                ) : (
                  <div className="flex h-[520px] items-center justify-center rounded-lg border border-dashed border-border/60 text-sm text-muted-foreground">
                    {t("detailChartUnavailable")}
                  </div>
                )}
              </div>
            </section>

            <aside className="space-y-4">
              <div className="rounded-xl border border-border/60 bg-background/72 px-5 py-5 backdrop-blur-xl supports-[backdrop-filter]:bg-background/58">
                <div className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
                  {t("detailSource")}
                </div>
                <div className="mt-3 text-sm leading-7 text-foreground/85">
                  {detail?.source_note ?? "--"}
                </div>
              </div>

              <div className="rounded-xl border border-border/60 bg-background/72 px-5 py-5 backdrop-blur-xl supports-[backdrop-filter]:bg-background/58">
                <div className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
                  {t("aggregateProvider")}
                </div>
                <div className="mt-3 text-sm text-foreground/85">{sidebarFooter}</div>
              </div>
            </aside>
          </div>
        </div>
      </section>

      <Dialog open={isChartDialogOpen} onOpenChange={setIsChartDialogOpen}>
        <DialogContent className="max-w-[min(1200px,calc(100vw-2rem))] gap-0 p-0 sm:max-w-[min(1200px,calc(100vw-2rem))]">
          <DialogHeader className="border-b border-border/60 px-5 py-4">
            <DialogTitle>{detail?.name ?? detail?.symbol ?? itemId.toUpperCase()}</DialogTitle>
            <DialogDescription>
              {t("detailChartTitle")}
              {chartSourceLabel ? ` · ${chartSourceLabel}` : ""}
            </DialogDescription>
          </DialogHeader>

          {chartSeries ? (
            <div className="px-4 pt-4 pb-4">
              <MarketPriceChart
                className="rounded-md border-border/60"
                data={chartSeries.points}
                height={720}
                locale={locale}
                seriesType={chartSeries.series_type}
              />
              <div className="mt-3 flex flex-wrap items-center justify-between gap-3 px-1 text-xs text-muted-foreground">
                <span>{chartSourceLabel}</span>
                <a
                  href="https://www.tradingview.com/"
                  target="_blank"
                  rel="noreferrer"
                  className="transition-colors hover:text-foreground"
                >
                  {t("detailChartAttribution")}
                </a>
              </div>
            </div>
          ) : (
            <div className="px-5 py-10 text-sm text-muted-foreground">
              {t("detailChartUnavailable")}
            </div>
          )}
        </DialogContent>
      </Dialog>
    </main>
  )
}

function MetricCard({ accent, label, value }: { accent?: string; label: string; value: string }) {
  return (
    <div className="rounded-lg border border-border/60 bg-background/55 px-4 py-4">
      <div className="text-xs text-muted-foreground">{label}</div>
      <div className={cn("mt-2 text-lg font-semibold text-foreground", accent)}>{value}</div>
    </div>
  )
}

function formatMarketValue(
  value: number | null | undefined,
  locale: Locale,
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
  locale: Locale,
  maximumFractionDigits = 2
) {
  if (typeof value !== "number") {
    return "--"
  }

  const formatted = formatMarketValue(Math.abs(value), locale, maximumFractionDigits)
  return `${value > 0 ? "+" : value < 0 ? "-" : ""}${formatted}`
}

function formatPercent(value: number | null | undefined, locale: Locale) {
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
    return "text-foreground"
  }

  if (value > 0) {
    return "text-emerald-400"
  }

  if (value < 0) {
    return "text-rose-400"
  }

  return "text-foreground"
}

function getRatingClass(value: string) {
  if (value === "Strong buy" || value === "Buy") {
    return "text-emerald-400"
  }

  if (value === "Strong sell" || value === "Sell") {
    return "text-rose-400"
  }

  return "text-amber-300"
}

function formatAggregateProvider(
  aggregateProvider: "auto" | MarketProvider,
  effectiveProvider: MarketProvider | undefined,
  resolvedProvider: MarketProvider | null,
  t: (key: never) => string
) {
  if (aggregateProvider === "auto") {
    return resolvedProvider
      ? `${t("aggregateProviderAuto" as never)} · ${providerLabels[resolvedProvider]}`
      : t("aggregateProviderAuto" as never)
  }

  if (effectiveProvider && effectiveProvider !== aggregateProvider) {
    return `${providerLabels[aggregateProvider]} -> ${providerLabels[effectiveProvider]}`
  }

  return providerLabels[effectiveProvider ?? aggregateProvider]
}

function getProviderLabel(provider: MarketProvider) {
  return providerLabels[provider] ?? provider
}
