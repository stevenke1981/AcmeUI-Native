# AcmeUI-Native Design Skill v2

這個 Skill 把原本的 AcmeUI API 參考，升級成可被 Agents 明確執行的：

- 任務模式路由
- Visual Design Director 流程
- App 模板選擇器
- 可複製的 Rust starter views
- 配色與視覺方向圖庫
- Visual QA 90 分 Gate

## 跨技術棧使用

Skill 可用於一般 Rust、GPUI 與 Web UI 專案。使用時保留目標專案原有
component/view model，只轉譯模式路由、模板工作流、Art Direction、
semantic token、WCAG 與 Visual QA。啟用 Skill 不會自動把專案遷移成
AcmeUI-Native，也不代表必須採用 `WidgetNode`。

## 最常用的 Agent 指令

### 建立 Typeless-like 語音輸入 App

```text
使用 acmeui-native-design，Mode=template-first。
選擇 voice-dictation template，不複製 Typeless 品牌與文案。
先建立 .design/template-selection.md、art-direction.md、component-inventory.md，
再把 templates/voice-dictation.rs 整合到新的 apps/<name>。
完成錄音、轉文字、歷史、語氣、設定的狀態規劃，最後執行 Visual QA。
```

### 建立管理 Dashboard

```text
使用 acmeui-native-design 的 dashboard template。
先確認 KPI、篩選、活動列表與圖表的資料來源，再開始實作。
不得使用假 KPI 當作裝飾；Empty/Error/Loading 都要有狀態。
```

### 改善現有 UI

```text
使用 Mode=redesign。
保留既有功能與資料流，先輸出 Art Direction 與 Component Inventory，
再修改版面、字體、semantic theme 與互動狀態。
沒有實際畫面比較不得完成。
```

## 模板不是第三方 Clone

`voice-dictation` 可用來建立 Typeless 類型的產品工作流，但只能參考：

- 語音啟動與狀態
- 即時逐字稿
- 文字清理與語氣
- 歷史紀錄
- 權限、裝置與快捷鍵設定

不得複製第三方商標、產品名稱、專屬文案或像素外觀。

## 資料夾

```text
acmeui-native-design/
├── SKILL.md
├── libraries/
├── assets/
└── templates/
```

模板的 `.rs` 檔是 UI composition starter。建立完整 App 時，仍須將 message 接到 update/event layer，並以現有 runtime app 作參考。
