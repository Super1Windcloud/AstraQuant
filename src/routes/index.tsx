import { createFileRoute } from "@tanstack/react-router"
import {
  BarChart3,
  ChevronDown,
  ChevronsLeft,
  ChevronsRight,
  RefreshCw,
  Search,
} from "lucide-react"
import { useEffect, useMemo, useState } from "react"

import { Button } from "@/components/ui/button"
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible"
import { Input } from "@/components/ui/input"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  type AssetClass,
  getMarketSnapshot,
  type MarketProvider,
  type MarketSnapshot,
  type MarketSymbol,
  marketGroups,
  providerLabels,
} from "@/lib/market-data"
import { useI18n } from "@/lib/i18n"
import { cn } from "@/lib/utils"

export const Route = createFileRoute("/")({
  component: MarketPage,
})

const providers: MarketProvider[] = ["finnhub", "massive", "twelvedata"]

function MarketPage() {
  const { t } = useI18n()
  const [provider, setProvider] = useState<MarketProvider>("finnhub")
  const [selectedSymbol, setSelectedSymbol] = useState<MarketSymbol>(marketGroups[0].symbols[0])
  const [snapshot, setSnapshot] = useState<MarketSnapshot | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [query, setQuery] = useState("")
  const [isSidebarOpen, setIsSidebarOpen] = useState(true)

  const filteredGroups = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase()

    if (!normalizedQuery) {
      return marketGroups
    }

    return marketGroups
      .map((group) => ({
        ...group,
        symbols: group.symbols.filter((item) =>
          `${item.symbol} ${item.name} ${item.exchange ?? ""}`
            .toLowerCase()
            .includes(normalizedQuery)
        ),
      }))
      .filter((group) => group.symbols.length > 0)
  }, [query])

  useEffect(() => {
    let ignore = false

    async function loadSnapshot() {
      setIsLoading(true)
      setError(null)

      try {
        const result = await getMarketSnapshot(provider, selectedSymbol)

        if (!ignore) {
          setSnapshot(result)
        }
      } catch (requestError) {
        if (!ignore) {
          setSnapshot(null)
          setError(requestError instanceof Error ? requestError.message : String(requestError))
        }
      } finally {
        if (!ignore) {
          setIsLoading(false)
        }
      }
    }

    loadSnapshot()

    return () => {
      ignore = true
    }
  }, [provider, selectedSymbol])

  return (
    <main className="flex min-h-full bg-background text-foreground">
      <aside
        className={cn(
          "flex shrink-0 flex-col border-r bg-sidebar transition-[width]",
          isSidebarOpen ? "w-[260px]" : "w-[48px]"
        )}
      >
        <div className="flex h-12 items-center justify-between border-b px-3">
          {isSidebarOpen ? (
            <div className="flex min-w-0 items-center gap-2">
              <BarChart3 className="size-4 shrink-0" />
              <span className="truncate text-sm font-medium">AstraQuant</span>
            </div>
          ) : null}
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            className="ml-auto"
            onClick={() => setIsSidebarOpen((value) => !value)}
            aria-label={isSidebarOpen ? t("sidebarToggleClose") : t("sidebarToggleOpen")}
          >
            {isSidebarOpen ? <ChevronsLeft /> : <ChevronsRight />}
          </Button>
        </div>

        {isSidebarOpen ? (
          <div className="flex min-h-0 flex-1 flex-col">
            <div className="border-b p-3">
              <div className="relative">
                <Search className="-translate-y-1/2 absolute top-1/2 left-2.5 size-4 text-muted-foreground" />
                <Input
                  value={query}
                  onChange={(event) => setQuery(event.target.value)}
                  className="pl-8"
                  placeholder={t("searchSymbol")}
                />
              </div>
            </div>
            <div className="min-h-0 flex-1 overflow-auto py-2">
              {filteredGroups.map((group) => (
                <Collapsible key={group.id} defaultOpen>
                  <CollapsibleTrigger className="flex h-8 w-full items-center justify-between px-3 text-sm font-medium hover:bg-sidebar-accent">
                    <span>{getAssetClassLabel(group.id, t)}</span>
                    <ChevronDown className="size-4 text-muted-foreground" />
                  </CollapsibleTrigger>
                  <CollapsibleContent>
                    <div className="pb-2">
                      {group.symbols.map((item) => (
                        <button
                          type="button"
                          key={`${group.id}-${item.symbol}`}
                          onClick={() => setSelectedSymbol(item)}
                          className={cn(
                            "grid w-full grid-cols-[minmax(0,1fr)_auto] gap-2 px-3 py-2 text-left text-sm hover:bg-sidebar-accent",
                            selectedSymbol.symbol === item.symbol &&
                              selectedSymbol.assetClass === item.assetClass &&
                              "bg-sidebar-accent"
                          )}
                        >
                          <span className="min-w-0">
                            <span className="block truncate font-medium">{item.symbol}</span>
                            <span className="block truncate text-xs text-muted-foreground">
                              {item.name}
                            </span>
                          </span>
                          <span className="self-center text-xs text-muted-foreground">
                            {item.exchange}
                          </span>
                        </button>
                      ))}
                    </div>
                  </CollapsibleContent>
                </Collapsible>
              ))}
            </div>
          </div>
        ) : null}
      </aside>

      <section className="flex min-w-0 flex-1 flex-col">
        <header className="flex h-12 items-center justify-between border-b px-4">
          <div className="min-w-0">
            <h1 className="truncate text-base font-semibold">{selectedSymbol.symbol}</h1>
            <div className="truncate text-xs text-muted-foreground">
              {selectedSymbol.name} · {selectedSymbol.exchange ?? selectedSymbol.assetClass}
            </div>
          </div>
          <div className="flex items-center gap-2">
            <Select
              value={provider}
              onValueChange={(value) => setProvider(value as MarketProvider)}
            >
              <SelectTrigger className="w-[150px]">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {providers.map((item) => (
                  <SelectItem key={item} value={item}>
                    {providerLabels[item]}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button
              type="button"
              variant="outline"
              size="icon-sm"
              disabled={isLoading}
              onClick={() => setSelectedSymbol({ ...selectedSymbol })}
              aria-label={t("refreshQuotes")}
            >
              <RefreshCw className={cn(isLoading && "animate-spin")} />
            </Button>
          </div>
        </header>

        <div className="grid min-h-0 flex-1 grid-rows-[auto_minmax(0,1fr)]">
          <div className="border-b px-5 py-4">
            <div className="flex flex-wrap items-end justify-between gap-4">
              <div>
                <div className="text-3xl font-semibold tracking-tight">
                  {formatPrice(snapshot?.price)}
                  {snapshot?.currency ? (
                    <span className="ml-2 text-sm font-normal text-muted-foreground">
                      {snapshot.currency}
                    </span>
                  ) : null}
                </div>
                <div className={cn("mt-1 text-sm", getChangeClass(snapshot?.change))}>
                  {formatChange(snapshot?.change)} ({formatPercent(snapshot?.change_percent)})
                </div>
              </div>
              <div className="text-right text-xs text-muted-foreground">
                <div>{providerLabels[provider]}</div>
                <div>{snapshot?.as_of ? `${t("asOf")} ${snapshot.as_of}` : t("waitingForData")}</div>
              </div>
            </div>
          </div>

          <div className="grid min-h-0 grid-cols-1 lg:grid-cols-[minmax(0,1fr)_320px]">
            <div className="min-h-[360px] border-b p-5 lg:border-r lg:border-b-0">
              <div className="flex h-full min-h-[320px] items-center justify-center border bg-card">
                <div className="text-center">
                  <BarChart3 className="mx-auto mb-3 size-8 text-muted-foreground" />
                  <div className="text-sm font-medium">{t("chartArea")}</div>
                  <div className="mt-1 max-w-sm text-sm text-muted-foreground">
                    {t("chartPlaceholder")}
                  </div>
                </div>
              </div>
            </div>

            <aside className="min-h-0 overflow-auto p-5">
              <div className="space-y-5">
                {error ? (
                  <div className="border border-destructive/40 bg-destructive/5 p-3 text-sm text-destructive">
                    {error}
                  </div>
                ) : null}

                <section>
                  <h2 className="text-sm font-medium">{t("overview")}</h2>
                  <dl className="mt-3 grid grid-cols-2 gap-px overflow-hidden border bg-border">
                    <QuoteStat label="Open" value={formatPrice(snapshot?.open)} />
                    <QuoteStat label="High" value={formatPrice(snapshot?.high)} />
                    <QuoteStat label="Low" value={formatPrice(snapshot?.low)} />
                    <QuoteStat label="Prev close" value={formatPrice(snapshot?.previous_close)} />
                    <QuoteStat label="Volume" value={formatCompact(snapshot?.volume)} />
                    <QuoteStat label="Asset" value={getAssetClassLabel(selectedSymbol.assetClass, t)} />
                  </dl>
                </section>

                <section>
                  <h2 className="text-sm font-medium">{t("api")}</h2>
                  <div className="mt-3 space-y-2 text-sm">
                    <InfoRow label="Provider" value={providerLabels[provider]} />
                    <InfoRow label="Symbol" value={selectedSymbol.symbol} />
                    <InfoRow label="Source" value={snapshot?.source_note ?? t("notReturned")} />
                  </div>
                </section>
              </div>
            </aside>
          </div>
        </div>
      </section>
    </main>
  )
}

function getAssetClassLabel(assetClass: AssetClass, t: ReturnType<typeof useI18n>["t"]) {
  const labels: Record<AssetClass, ReturnType<typeof t>> = {
    stock: t("assetStock"),
    index: t("assetIndex"),
    etf: t("assetEtf"),
    crypto: t("assetCrypto"),
    forex: t("assetForex"),
    future: t("assetFuture"),
  }

  return labels[assetClass]
}

function QuoteStat({ label, value }: { label: string; value: string }) {
  return (
    <div className="bg-background p-3">
      <dt className="text-xs text-muted-foreground">{label}</dt>
      <dd className="mt-1 truncate text-sm font-medium">{value}</dd>
    </div>
  )
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between gap-4 border-b pb-2">
      <span className="text-muted-foreground">{label}</span>
      <span className="min-w-0 truncate text-right font-medium">{value}</span>
    </div>
  )
}

function formatPrice(value: number | null | undefined) {
  if (typeof value !== "number") {
    return "--"
  }

  return new Intl.NumberFormat("en-US", {
    maximumFractionDigits: value >= 100 ? 2 : 5,
  }).format(value)
}

function formatChange(value: number | null | undefined) {
  if (typeof value !== "number") {
    return "--"
  }

  return `${value > 0 ? "+" : ""}${formatPrice(value)}`
}

function formatPercent(value: number | null | undefined) {
  if (typeof value !== "number") {
    return "--"
  }

  return `${value > 0 ? "+" : ""}${value.toFixed(2)}%`
}

function formatCompact(value: number | null | undefined) {
  if (typeof value !== "number") {
    return "--"
  }

  return new Intl.NumberFormat("en-US", {
    notation: "compact",
    maximumFractionDigits: 2,
  }).format(value)
}

function getChangeClass(value: number | null | undefined) {
  if (typeof value !== "number") {
    return "text-muted-foreground"
  }

  if (value > 0) {
    return "text-emerald-600 dark:text-emerald-400"
  }

  if (value < 0) {
    return "text-red-600 dark:text-red-400"
  }

  return "text-muted-foreground"
}
