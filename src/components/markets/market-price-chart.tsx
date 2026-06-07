import {
  type AreaData,
  AreaSeries,
  type CandlestickData,
  CandlestickSeries,
  ColorType,
  createChart,
  type Time,
  type UTCTimestamp,
} from "lightweight-charts"
import { useTheme } from "next-themes"
import { useEffect, useRef } from "react"

import { type Locale } from "@/lib/i18n"
import { cn } from "@/lib/utils"

interface MarketChartPoint {
  time: string
  open: number | null
  high: number | null
  low: number | null
  close: number | null
  value: number | null
}

interface MarketPriceChartProps {
  className?: string
  data: MarketChartPoint[]
  height?: number
  locale: Locale
  seriesType: "candlestick" | "area"
}

export function MarketPriceChart({
  className,
  data,
  height = 520,
  locale,
  seriesType,
}: MarketPriceChartProps) {
  const containerRef = useRef<HTMLDivElement | null>(null)
  const { resolvedTheme, theme } = useTheme()
  const themeKey = resolvedTheme ?? theme ?? "system"

  useEffect(() => {
    const container = containerRef.current

    if (!container || data.length === 0) {
      return
    }

    const palette = readChartPalette(themeKey)
    const chart = createChart(container, {
      width: container.clientWidth || 720,
      height: container.clientHeight || height,
      layout: {
        background: {
          type: ColorType.Solid,
          color: palette.background,
        },
        textColor: palette.mutedForeground,
        attributionLogo: true,
      },
      grid: {
        vertLines: { color: withAlpha(palette.border, 0.45) },
        horzLines: { color: withAlpha(palette.border, 0.45) },
      },
      rightPriceScale: {
        borderColor: withAlpha(palette.border, 0.78),
        scaleMargins: { top: 0.08, bottom: 0.08 },
      },
      timeScale: {
        borderColor: withAlpha(palette.border, 0.78),
      },
      crosshair: {
        vertLine: {
          color: withAlpha(palette.foreground, 0.18),
          labelBackgroundColor: palette.accent,
        },
        horzLine: {
          color: withAlpha(palette.foreground, 0.18),
          labelBackgroundColor: palette.accent,
        },
      },
      localization: {
        locale,
      },
      handleScale: {
        mouseWheel: true,
        pinch: true,
      },
      handleScroll: {
        mouseWheel: true,
        pressedMouseMove: true,
        horzTouchDrag: true,
        vertTouchDrag: false,
      },
    })

    if (seriesType === "area") {
      const series = chart.addSeries(AreaSeries, {
        lineColor: palette.areaLine,
        topColor: withAlpha(palette.areaLine, 0.24),
        bottomColor: withAlpha(palette.areaLine, 0.04),
        lineWidth: 2,
        priceLineVisible: true,
      })

      const areaData = data
        .filter((point) => point.value != null)
        .map<AreaData<Time>>((point) => ({
          time: parseChartTime(point.time),
          value: point.value as number,
        }))

      series.setData(areaData)
    } else {
      const series = chart.addSeries(CandlestickSeries, {
        upColor: palette.upColor,
        downColor: palette.downColor,
        borderVisible: false,
        wickUpColor: palette.upColor,
        wickDownColor: palette.downColor,
      })

      const candleData = data
        .filter(
          (point) =>
            point.open != null && point.high != null && point.low != null && point.close != null
        )
        .map<CandlestickData<Time>>((point) => ({
          time: parseChartTime(point.time),
          open: point.open as number,
          high: point.high as number,
          low: point.low as number,
          close: point.close as number,
        }))

      series.setData(candleData)
    }

    chart.timeScale().fitContent()

    const resizeObserver = new ResizeObserver((entries) => {
      const entry = entries[0]

      if (!entry) {
        return
      }

      chart.applyOptions({
        width: Math.floor(entry.contentRect.width),
        height: Math.floor(entry.contentRect.height),
      })
      chart.timeScale().fitContent()
    })

    resizeObserver.observe(container)

    return () => {
      resizeObserver.disconnect()
      chart.remove()
    }
  }, [data, height, locale, seriesType, themeKey])

  return (
    <div
      ref={containerRef}
      className={cn(
        "w-full overflow-hidden rounded-lg border border-border/60 bg-background",
        className
      )}
      style={{ height }}
    />
  )
}

function parseChartTime(value: string): Time {
  if (/^\d+$/.test(value)) {
    return Number(value) as UTCTimestamp
  }

  return value
}

function readChartPalette(themeKey: string) {
  const styles = getComputedStyle(document.documentElement)
  const isLightTheme = themeKey === "light"

  return {
    background: styles.getPropertyValue("--background").trim() || "#111827",
    foreground: styles.getPropertyValue("--foreground").trim() || "#f8fafc",
    mutedForeground: styles.getPropertyValue("--muted-foreground").trim() || "#94a3b8",
    border: styles.getPropertyValue("--border").trim() || "#334155",
    accent: styles.getPropertyValue("--chart-2").trim() || "#0f766e",
    areaLine: styles.getPropertyValue("--chart-2").trim() || "#0f766e",
    upColor: isLightTheme ? "#059669" : "#10b981",
    downColor: isLightTheme ? "#dc2626" : "#f43f5e",
  }
}

function withAlpha(color: string, alpha: number) {
  if (color.startsWith("oklch(") && color.endsWith(")")) {
    const innerColor = color.slice(6, -1)

    if (innerColor.includes("/")) {
      return color
    }

    return `oklch(${innerColor} / ${alpha})`
  }

  if (color.startsWith("rgba(") && color.endsWith(")")) {
    return color
  }

  if (color.startsWith("rgb(") && color.endsWith(")")) {
    return `rgba(${color.slice(4, -1)}, ${alpha})`
  }

  return color
}
