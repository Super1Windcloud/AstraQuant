import { createFileRoute } from "@tanstack/react-router"

import { MarketOverviewPage } from "@/components/markets/overview-page"

export const Route = createFileRoute("/crypto")({
  component: CryptoPage,
})

function CryptoPage() {
  return <MarketOverviewPage kind="crypto" />
}
