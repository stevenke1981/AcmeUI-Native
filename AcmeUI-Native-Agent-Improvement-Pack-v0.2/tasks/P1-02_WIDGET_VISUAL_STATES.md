# P1-02 Widget Visual States

## 共通狀態

Default、Hover、Pressed、Focus Visible、Selected、Disabled、Loading、Invalid。

## Button

- variants：Primary/Secondary/Ghost/Danger。
- sizes：XS/S/M/L。
- leading/trailing icon、loading、full width。
- pressed 必須有獨立 token，不可等同 hover。
- loading 保持原按鈕寬度。

## Card

Plain、Outlined、Elevated、Interactive、Muted。設定頁不應每區都套 Card。

## Input

Label、description、control、validation message。支援 placeholder、clear、readonly、password、invalid。

## Overlay

Tooltip/Popover/Menu/Dialog 使用獨立 OverlayManager：Main/Floating/Modal/Tooltip/Drag/Debug layers。

## 模組重構

```text
acme-widgets/src/
├── foundations/
├── inputs/
├── navigation/
├── overlay/
├── data/
└── prelude.rs
```

保留原有 re-export，避免一次破壞使用者 API。
