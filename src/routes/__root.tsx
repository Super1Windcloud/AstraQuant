import { createRootRoute, Outlet, useLocation } from "@tanstack/react-router"
import { invoke } from "@tauri-apps/api/core"
import { useEffect } from "react"

import { AppIconProvider } from "@/components/app-icon"
import { TailwindIndicator } from "@/components/tailwind-indicator"
import { ThemeProvider } from "@/components/theme-provider"
import { WindowTitlebar } from "@/components/window-titlebar"
import { DashboardLayout } from "@/dashboard/layout"
import { I18nProvider, useI18n } from "@/lib/i18n"
import { MarketProviderStore } from "@/lib/market-provider"
import { cn } from "@/lib/utils"

const marketRoutes = new Set(["/", "/stocks", "/crypto", "/futures", "/etf"])

function AppShell() {
  const { pathname } = useLocation()
  const isMarketRoute = marketRoutes.has(pathname)

  useEffect(() => {
    invoke("show_window").catch((error) => {
      console.error("[AppShell] Failed to invoke show_window", error)
    })
  }, [])

  return (
    <ThemeProvider>
      <AppIconProvider>
        <I18nProvider>
          <MarketProviderStore>
            <div className="flex h-screen w-screen flex-col bg-background">
              <WindowTitlebar />
              <div
                style={{
                  scrollbarWidth: "none",
                }}
                className={cn(
                  "flex-1 bg-background",
                  isMarketRoute
                    ? "overflow-hidden"
                    : "overflow-auto pb-8 scrollbar scrollbar-track-transparent scrollbar-thumb-accent scrollbar-thumb-rounded-md"
                )}
              >
                <Outlet />
              </div>
            </div>
          </MarketProviderStore>
          <TailwindIndicator />
        </I18nProvider>
      </AppIconProvider>
    </ThemeProvider>
  )
}

function NotFound() {
  const { t } = useI18n()

  return (
    <DashboardLayout title={t("pageNotFound")} description={t("pageNotFoundDescription")}>
      <div className="text-muted-foreground">{t("pageNotFoundAction")}</div>
    </DashboardLayout>
  )
}

export const Route = createRootRoute({
  component: AppShell,
  notFoundComponent: NotFound,
})
