import { createFileRoute } from "@tanstack/react-router"

import { MarketOverviewPage } from "@/components/markets/overview-page"

export const Route = createFileRoute("/etf")({
  component: EtfPage,
})

function EtfPage() {
  return <MarketOverviewPage kind="etf" />
}
