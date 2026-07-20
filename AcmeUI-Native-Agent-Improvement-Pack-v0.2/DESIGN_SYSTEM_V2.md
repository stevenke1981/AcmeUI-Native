# AcmeUI Native Design System V2

## 視覺原則

1. 低飽和，不使用純黑或純白作大面積背景。
2. 優先用 whitespace、surface hierarchy、typography 表達層級，減少框線。
3. 預設 Comfortable，資料工具可 Compact。
4. 一個區塊只保留一個 Primary Action。
5. 對齊與基準線優先於裝飾。
6. 動畫只用於狀態轉換、overlay、expand/collapse。

## Surface 層級

```text
app_background
panel_background
surface
sunken_surface
elevated_surface
overlay_surface
```

## Card 使用規則

只在 KPI、可互動摘要、獨立狀態與浮層使用 Card。Settings 優先 Section + Separator。

## 陰影

- Small：dropdown/tooltips
- Medium：popover/menu
- Large：dialog
- Card 預設 none 或 small

wgpu 第一版可先以多層低 alpha quad 模擬，後續再做 blur shadow atlas；不可為追求陰影先阻塞核心架構。

## Motion

- Fast 80ms
- Normal 140ms
- Slow 220ms
- easing：ease-out / standard
- Reduced motion：位移轉為短 fade 或 instant

## Icon

12/14/16/20/24px。IconButton 必須有 tooltip、accessible label、最小 28×28 hit target。
