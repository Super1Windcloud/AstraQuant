import { Link } from "@tanstack/react-router"
import { ChevronLeft, ChevronRight } from "lucide-react"
import { type ReactNode, useEffect, useState } from "react"

import { useI18n } from "@/lib/i18n"
import { marketNavigationItems } from "@/lib/market-data"
import { cn } from "@/lib/utils"

interface MarketSidebarProps {
  footer?: ReactNode
}

const sidebarStorageKey = "market-sidebar-collapsed"

const baseItemClass =
  "flex h-9 w-full items-center rounded-md px-3 text-left text-sm transition-colors"

function readCollapsedState() {
  if (typeof window === "undefined") {
    return false
  }

  return window.localStorage.getItem(sidebarStorageKey) === "1"
}

function getCompactLabel(value: string) {
  const label = value.trim()
  return label.slice(0, 1) || "•"
}

export function MarketSidebar({ footer }: MarketSidebarProps) {
  const { t } = useI18n()
  const [isCollapsed, setIsCollapsed] = useState(readCollapsedState)

  useEffect(() => {
    window.localStorage.setItem(sidebarStorageKey, isCollapsed ? "1" : "0")
  }, [isCollapsed])

  return (
    <aside
      className={cn(
        "hidden h-full shrink-0 overflow-hidden border-r border-border/60 bg-background/78 backdrop-blur-xl supports-[backdrop-filter]:bg-background/60 transition-[width] duration-150 ease-out lg:flex lg:flex-col",
        isCollapsed ? "w-[72px]" : "w-[248px]"
      )}
    >
      <div
        className={cn(
          "flex h-12 items-center border-b border-border/60",
          isCollapsed ? "justify-between px-2" : "justify-between px-4"
        )}
      >
        <span className="text-sm font-semibold text-foreground">
          {isCollapsed ? "AQ" : "AstraQuant"}
        </span>
        <button
          type="button"
          className="flex h-8 w-8 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-accent/40 hover:text-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
          onClick={() => setIsCollapsed((value) => !value)}
          aria-label={t(isCollapsed ? "sidebarToggleOpen" : "sidebarToggleClose")}
          title={t(isCollapsed ? "sidebarToggleOpen" : "sidebarToggleClose")}
        >
          {isCollapsed ? <ChevronRight className="size-4" /> : <ChevronLeft className="size-4" />}
        </button>
      </div>

      <div className={cn("min-h-0 flex-1 overflow-y-auto py-4", isCollapsed ? "px-2" : "px-3")}>
        {isCollapsed ? null : (
          <div className="px-2 text-[11px] tracking-wide text-muted-foreground uppercase">
            Markets
          </div>
        )}
        <div className={cn(isCollapsed ? "space-y-2" : "mt-2 space-y-1")}>
          {marketNavigationItems.map((item) => {
            const label = t(item.i18nKey as never)
            const itemClassName = cn(
              baseItemClass,
              isCollapsed ? "justify-center px-0 text-[13px] font-medium" : ""
            )

            return (
              <Link
                key={item.to}
                to={item.to}
                activeOptions={{ exact: true }}
                className={itemClassName}
                activeProps={{
                  className: cn(itemClassName, "bg-accent/50 text-foreground"),
                }}
                inactiveProps={{
                  className: cn(
                    itemClassName,
                    "text-muted-foreground hover:bg-accent/35 hover:text-foreground"
                  ),
                }}
                aria-label={label}
                title={label}
              >
                {isCollapsed ? getCompactLabel(label) : label}
              </Link>
            )
          })}
        </div>
      </div>

      {isCollapsed ? null : (
        <div className="border-t border-border/60 px-4 py-3 text-xs text-muted-foreground">
          {footer}
        </div>
      )}
    </aside>
  )
}
