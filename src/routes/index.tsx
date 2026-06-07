import { createFileRoute } from "@tanstack/react-router"

import { MarketOverviewPage } from "@/components/markets/overview-page"

export const Route = createFileRoute("/")({
  component: IndicesPage,
})

function IndicesPage() {
  return <MarketOverviewPage kind="indices" />
}
