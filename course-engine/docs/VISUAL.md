# VISUAL — design tokens (franken_ocr product-site DNA)

## Framework lock

Static HTML + hand-written CSS + vanilla JS + WASM. **No** Next/React/Tailwind for learner app.  
Self-host Inter + JetBrains Mono. No CDN required.

## Tokens (implement in `web/assets/css/course.css`)

```css
:root {
  --bg: #060b09;
  --bg-deep: #030706;
  --surface: rgba(255, 255, 255, 0.022);
  --fg: #e8eef2;
  --fg-mid: #cbd5e1;
  --fg-dim: #94a3b8;
  --fg-faint: #748496; /* AA captions */
  --accent: #34d399;   /* correct / CTA */
  --accent-deep: #059669;
  --accent-ink: #04140d;
  --amber: #fbbf24;    /* timer / study-signal caution */
  --red: #f87171;      /* incorrect */
  --violet: #a78bfa;   /* structure tags */
  --line: rgba(255, 255, 255, 0.07);
  --honesty-bg: rgba(251, 191, 36, 0.08);
  --honesty-fg: #fbbf24;
  --sans: "Inter", system-ui, sans-serif;
  --mono: "JetBrains Mono", ui-monospace, monospace;
  --wrap: 1180px;
  --wrap-exam: 720px;
  --header-h: 64px;
  --touch-min: 44px;
  --r: 12px;
  --r-pill: 9999px;
}
```

## UX

Hub: Learn · Drill · Mock. Honesty banner always visible (amber, never green “certified”).  
Mock: 40Q / 60:00 / study bar 27. Results show weak modules + short bank hash.

## a11y

Skip link · focus-visible · reduced-motion · color not sole signal · keyboard-complete mock.
