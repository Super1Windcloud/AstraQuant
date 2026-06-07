#!/usr/bin/env node

const { createRequire } = require("node:module");

const requireFromRoot = createRequire(process.cwd() + "/");
const TradingView = requireFromRoot("@mathieuc/tradingview");

const [, , symbolArg, timeframeArg] = process.argv;

if (!symbolArg || symbolArg === "--help") {
  process.stdout.write("usage: node scripts/tradingview-chart.cjs <EXCHANGE:SYMBOL> [TIMEFRAME]\n");
  process.exit(symbolArg ? 0 : 1);
}

const symbol = symbolArg.trim();
const timeframe = (timeframeArg || "D").trim();
const timeoutMs = Number(process.env.TRADINGVIEW_BRIDGE_TIMEOUT_MS || 15000);

async function main() {
  const clientOptions = {};
  const session = process.env.TRADINGVIEW_SESSIONID?.trim();
  const signature = process.env.TRADINGVIEW_SIGNATURE?.trim();

  if (session && signature) {
    clientOptions.token = session;
    clientOptions.signature = signature;
  }

  const client = new TradingView.Client(clientOptions);
  const chart = new client.Session.Chart();
  let taRating = null;

  const timeout = setTimeout(() => {
    shutdown(
      1,
      new Error(`Timed out after ${timeoutMs}ms while loading TradingView chart for ${symbol}`),
      client
    );
  }, timeoutMs);

  chart.onError((...error) => {
    shutdown(1, new Error(error.filter(Boolean).join(" ") || `TradingView chart error for ${symbol}`), client, timeout);
  });

  chart.onUpdate(() => {
    if (!chart.periods[0]) {
      return;
    }

    const periods = [...chart.periods].reverse();
    const points = periods.map((period) => ({
      time: new Date(period.time * 1000).toISOString().slice(0, 10),
      open: toNumber(period.open),
      high: toNumber(period.max),
      low: toNumber(period.min),
      close: toNumber(period.close),
      value: null,
    }));

    const payload = {
      provider: "tradingview",
      kind: "",
      id: "",
      symbol: "",
      interval: timeframe,
      series_type: "candlestick",
      currency: chart.infos.currency_code || chart.infos.currency_id || null,
      technical_rating: taRating,
      source_note: `TradingView chart session (${symbol})`,
      points,
    };

    shutdown(0, payload, client, timeout);
  });

  void TradingView.getTA(symbol)
    .then((ta) => {
      taRating = normalizeTaRating(ta);
    })
    .catch(() => {
      taRating = null;
    });

  chart.setMarket(symbol, {
    timeframe,
    range: 240,
  });
}

function toNumber(value) {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function normalizeTaRating(ta) {
  const daily = ta && typeof ta === "object" ? ta["1D"] : null;
  const score = daily && typeof daily.All === "number" ? daily.All : null;

  if (score === null) {
    return null;
  }

  if (score >= 0.5) {
    return "Strong buy";
  }

  if (score > 0) {
    return "Buy";
  }

  if (score <= -0.5) {
    return "Strong sell";
  }

  if (score < 0) {
    return "Sell";
  }

  return "Neutral";
}

function shutdown(code, payload, client, timeout) {
  if (timeout) {
    clearTimeout(timeout);
  }

  try {
    client.end();
  } catch {}

  if (code === 0) {
    process.stdout.write(`${JSON.stringify(payload)}\n`);
    process.exit(0);
  }

  const message = payload instanceof Error ? payload.message : String(payload);
  process.stderr.write(`${message}\n`);
  process.exit(code);
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(1);
});
