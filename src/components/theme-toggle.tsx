import { Monitor, Moon, Palette, Sprout, Sun, Waves } from "lucide-react"
import { useTheme } from "next-themes"

import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { useI18n } from "@/lib/i18n"

const themeOptions = [
  { value: "light", labelKey: "themeLight", icon: Sun },
  { value: "dark", labelKey: "themeDark", icon: Moon },
  { value: "dim", labelKey: "themeDim", icon: Palette },
  { value: "ocean", labelKey: "themeOcean", icon: Waves },
  { value: "avocado", labelKey: "themeAvocado", icon: Sprout },
  { value: "system", labelKey: "themeSystem", icon: Monitor },
] as const

export function ThemeToggle() {
  const { setTheme, theme = "system" } = useTheme()
  const { t } = useI18n()

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button type="button" variant="ghost" size="icon-sm" aria-label={t("theme")}>
          <Palette className="size-4" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuRadioGroup value={theme}>
          {themeOptions.map(({ value, labelKey, icon: Icon }) => (
            <DropdownMenuRadioItem key={value} value={value} onClick={() => setTheme(value)}>
              <Icon className="mr-2 size-4" />
              {t(labelKey)}
            </DropdownMenuRadioItem>
          ))}
        </DropdownMenuRadioGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
