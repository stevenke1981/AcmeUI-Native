# 視覺參考模板

以下為 AcmeUI Native 自有模板，僅吸收成熟桌面產品的資訊層級原則。

## Template 1 — Precision Workspace

適合 IDE、Editor、管理工具。

- App background：深灰藍或冷白灰。
- Sidebar 比 Main surface 深/暗一級。
- Toolbar 32px、StatusBar 24px。
- 選取項目使用 soft accent background + 2px indicator。
- 多數區塊不用 Card。

## Template 2 — Calm Settings

適合設定與帳號頁。

- Sidebar 220–248px。
- Content 最大寬度 760–880px。
- Section gap 24px。
- Setting row 最小 44px。
- Label 13px、Description 12px。
- Danger Zone 與一般區域明確分開。

## Template 3 — Operations Dashboard

- Header 高度 56–64px。
- KPI 最多四個。
- KPI 使用 Outlined/Muted card，不用重陰影。
- 主要圖表佔最大面積。
- Loading 使用 skeleton，避免整頁 spinner。

## Template 4 — Voice Assistant Desktop

- 錄音狀態是唯一主要視覺焦點。
- Provider/Network 狀態用小 Badge。
- 主按鈕 44–48px，其他按鈕 28–32px。
- 即時 transcript 使用 sunken surface。
- Recording/Processing/Error 用形狀、文字和顏色共同表達。

## 禁止模式

- 每個區塊都套高陰影 Card。
- 大量漸層與霓虹色。
- 所有按鈕都 Primary。
- 16px 以上正文塞入高密度桌面工具。
- Hover 才顯示必要資訊。
