import { createFileRoute } from "@tanstack/react-router"

import { MarketOverviewPage } from "@/components/markets/overview-page"

export const Route = createFileRoute("/futures")({
  component: FuturesPage,
})

function FuturesPage() {
  return <MarketOverviewPage kind="futures" />
}
