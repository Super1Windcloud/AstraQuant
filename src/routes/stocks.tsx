import { createFileRoute } from "@tanstack/react-router"

import { MarketOverviewPage } from "@/components/markets/overview-page"

export const Route = createFileRoute("/stocks")({
  component: StocksPage,
})

function StocksPage() {
  return <MarketOverviewPage kind="stocks" />
}
