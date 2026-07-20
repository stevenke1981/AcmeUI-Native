# P1-01 Acme Visual System V2

## 設計定位

- Linear：清楚層級、低飽和、精準 selected state。
- Zed：桌面密度、工具列與編輯器效率。
- Arc：柔和浮層、精緻但短促的 transition。
- Apple Settings：設定分組與易讀性。

只參考設計原則，不複製商標、素材或專有畫面。

## Theme V2

新增：

- SurfaceTokens：app/panel/surface/elevated/overlay/sunken。
- TextTokens：primary/secondary/tertiary/disabled/inverse/link。
- BorderTokens：subtle/default/strong/focus。
- SemanticTokens：primary/success/warning/danger/info，各有 default/hover/pressed/soft/on_color。
- TypographyScale：display/title_large/title/body/body_compact/label/caption/code。
- SpacingScale：0/2/4/6/8/12/16/20/24/32/40/48/64。
- RadiusScale：0/3/5/7/10/14/pill。
- ControlSize：xs/sm/md/lg。
- Density：compact/comfortable/spacious。
- Elevation：none/small/medium/large。
- Motion：instant/fast/normal/slow + reduced motion。
- IconSize：12/14/16/20/24。

## 桌面建議字級

- Title Large 20/28 semibold
- Title 16/24 semibold
- Body 13/20 regular
- Body Compact 12/18 regular
- Label 12/16 medium
- Caption 11/16 regular

## 驗收

- Theme validates all tokens。
- Light/Dark/High Contrast 三種模式。
- Compact/Comfortable/Spacious screenshots。
- 無 widget hardcoded visual metric。
