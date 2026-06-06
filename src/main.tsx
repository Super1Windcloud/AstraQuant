import React from "react"
import ReactDOM from "react-dom/client"
import { attachConsole } from "@tauri-apps/plugin-log"

import App from "./App"

import "./styles/globals.css"

if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
  void attachConsole().catch((error) => {
    console.error("[main] Failed to attach Tauri log console", error)
  })
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
)
