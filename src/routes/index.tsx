import { createFileRoute, Link } from "@tanstack/react-router"
import { ArrowRight, Blocks, Monitor, Package, Palette, TerminalSquare } from "lucide-react"

import { Button } from "@/components/ui/button"

const capabilities = [
  {
    title: "Desktop-ready foundation",
    description: "Start from a Tauri shell that already matches a modern React app structure.",
    icon: Monitor,
  },
  {
    title: "Reusable UI system",
    description: "Build with shadcn/ui components, Radix primitives, and Tailwind CSS v4 tokens.",
    icon: Blocks,
  },
  {
    title: "Routed application setup",
    description: "Add product areas through TanStack Router file routes instead of rebuilding app structure.",
    icon: Package,
  },
  {
    title: "Tooling already wired",
    description: "Biome, Vitest, Cargo, and icon generation are configured so work can start immediately.",
    icon: TerminalSquare,
  },
]

const quickStart = [
  "pnpm install",
  "pnpm dev",
  "pnpm td",
]

export const Route = createFileRoute("/")({
  component: HomePage,
})

function HomePage() {
  return (
    <main className="min-h-full bg-background text-foreground">
      <div className="mx-auto flex w-full max-w-6xl flex-col gap-12 px-6 py-8 lg:px-8">
        <section className="grid gap-8 border-b pb-10 lg:grid-cols-[minmax(0,1.2fr)_minmax(320px,0.8fr)] lg:items-end">
          <div className="space-y-5">
            <div className="inline-flex items-center gap-2 text-sm text-muted-foreground">
              <Palette className="size-4" />
              Tauri UI Template
            </div>
            <div className="space-y-3">
              <h1 className="max-w-3xl text-4xl font-semibold tracking-tight text-balance sm:text-5xl">
                A practical desktop app starter for shipping React interfaces with Tauri.
              </h1>
              <p className="max-w-2xl text-base leading-7 text-muted-foreground sm:text-lg">
                Use this template as a solid baseline for internal tools, product dashboards, or
                cross-platform desktop workflows without spending the first day wiring the stack.
              </p>
            </div>
            <div className="flex flex-wrap gap-3">
              <Button asChild size="lg">
                <Link to="/dashboard">
                  Open dashboard
                  <ArrowRight />
                </Link>
              </Button>
              <Button asChild variant="outline" size="lg">
                <a href="https://tauri.app/" target="_blank" rel="noreferrer">
                  Tauri docs
                </a>
              </Button>
            </div>
          </div>

          <div className="border bg-card">
            <div className="border-b px-5 py-4">
              <h2 className="text-base font-medium">Quick start</h2>
              <p className="mt-1 text-sm text-muted-foreground">
                The default flow keeps frontend and desktop runtime separated and predictable.
              </p>
            </div>
            <div className="space-y-3 px-5 py-5">
              {quickStart.map((command, index) => (
                <div key={command} className="flex items-center gap-4 border p-3">
                  <span className="w-5 text-sm text-muted-foreground">{index + 1}</span>
                  <code className="text-sm font-medium">{command}</code>
                </div>
              ))}
            </div>
          </div>
        </section>

        <section className="grid gap-4 md:grid-cols-2">
          {capabilities.map(({ title, description, icon: Icon }) => (
            <article key={title} className="border bg-card p-5">
              <div className="mb-4 flex items-center gap-3">
                <Icon className="size-4" />
                <h2 className="text-base font-medium">{title}</h2>
              </div>
              <p className="text-sm leading-6 text-muted-foreground">{description}</p>
            </article>
          ))}
        </section>

        <section className="grid gap-6 border-t pt-8 lg:grid-cols-[minmax(0,0.9fr)_minmax(0,1.1fr)]">
          <div className="space-y-3">
            <h2 className="text-2xl font-semibold tracking-tight">What is included</h2>
            <p className="text-sm leading-7 text-muted-foreground">
              The repository already contains a dashboard route, reusable component library,
              Tauri Rust entry points, and utility scripts for formatting, checking, and upgrades.
            </p>
          </div>
          <div className="grid gap-3 sm:grid-cols-2">
            <div className="border p-4">
              <div className="text-sm font-medium">Frontend</div>
              <p className="mt-2 text-sm leading-6 text-muted-foreground">
                React 19, TypeScript, Vite, TanStack Router, and Tailwind CSS v4.
              </p>
            </div>
            <div className="border p-4">
              <div className="text-sm font-medium">UI layer</div>
              <p className="mt-2 text-sm leading-6 text-muted-foreground">
                shadcn/ui components with Radix primitives and theme tokens.
              </p>
            </div>
            <div className="border p-4">
              <div className="text-sm font-medium">Native shell</div>
              <p className="mt-2 text-sm leading-6 text-muted-foreground">
                Rust-based Tauri runtime with desktop capabilities and packaged icons.
              </p>
            </div>
            <div className="border p-4">
              <div className="text-sm font-medium">Developer tools</div>
              <p className="mt-2 text-sm leading-6 text-muted-foreground">
                Biome, Vitest, Husky, and upgrade scripts for frontend and Cargo dependencies.
              </p>
            </div>
          </div>
        </section>
      </div>
    </main>
  )
}
