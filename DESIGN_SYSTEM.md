# AcmeUI-Native Design System V2

> Synthesized from shadcn/ui, Material UI, and Ant Design — optimized for native desktop
> Date: 2026-07-20

---

## Core Philosophy

- **Semantic-first**: All colors, spacing, and typography via design tokens — no hardcoded values
- **Foreground/background pairs**: Every surface token has a matching text token
- **Seed token derivation**: Primary color drives entire palette (inspired by Ant Design)
- **Desktop-optimized**: Denser than web defaults but more breathing room than classic Win32
- **Accessibility**: WCAG AA minimum on all text/background pairs; visible focus rings

---

## 1. Color System

### Seed Colors

| Role | Light | Dark | Usage |
|------|-------|------|-------|
| `background` | `#FAFBFC` | `#0E1217` | Page/Window background |
| `foreground` | `#0F1419` | `#F0F2F5` | Primary text on background |
| `primary` | `#2563EB` | `#5B8DEF` | Primary actions, links |
| `primary-foreground` | `#FFFFFF` | `#0F1419` | Text on primary |
| `secondary` | `#F0F2F5` | `#1C2128` | Secondary surfaces |
| `secondary-foreground` | `#1F2937` | `#E2E5EA` | Text on secondary |
| `accent` | `#E8F0FE` | `#192843` | Soft highlight (sidebar, selected row) |
| `accent-foreground` | `#1D4ED8` | `#8BB4F8` | Text on accent |
| `muted` | `#F4F5F7` | `#161B22` | Subtle hover, disabled bg |
| `muted-foreground` | `#6B7280` | `#8B929A` | Secondary text, placeholder |
| `border` | `#E2E5EA` | `#2D333B` | Default border |
| `input` | `#E2E5EA` | `#2D333B` | Input border |
| `ring` | `#2563EB` | `#5B8DEF` | Focus ring |

### Semantic Status Colors

| Role | Light | Dark | Usage |
|------|-------|------|-------|
| `success` | `#16A34A` | `#4ADE80` | Success state |
| `success-soft` | `#F0FDF4` | `#0D2818` | Success background |
| `warning` | `#D97706` | `#FBBF24` | Warning state |
| `warning-soft` | `#FFFBEB` | `#2D1F04` | Warning background |
| `danger` | `#DC2626` | `#F87171` | Destructive actions, errors |
| `danger-soft` | `#FEF2F2` | `#2D0A0A` | Error background |
| `info` | `#2563EB` | `#60A5FA` | Info state |
| `info-soft` | `#EFF6FF` | `#0D1B3E` | Info background |

### Elevation / Surface Ladder

| Level | Light Surface | Light Shadow | Dark Surface | Dark Shadow | Usage |
|-------|-------------|--------------|-------------|--------------|-------|
| 0 | `background` | none | `background` | none | Page |
| 1 | `#FFFFFF` | `0 1px 2px rgba(0,0,0,0.05)` | `#151B23` | `0 1px 2px rgba(0,0,0,0.3)` | Card, Panel |
| 2 | `#FFFFFF` | `0 4px 12px rgba(0,0,0,0.08)` | `#1C2433` | `0 4px 12px rgba(0,0,0,0.4)` | Popover, Menu |
| 3 | `#FFFFFF` | `0 8px 24px rgba(0,0,0,0.12)` | `#223044` | `0 8px 24px rgba(0,0,0,0.5)` | Dialog, Modal |
| 4 | `#FFFFFF` | `0 16px 48px rgba(0,0,0,0.16)` | `#2A3A50` | `0 16px 48px rgba(0,0,0,0.6)` | Tooltip, Notification |

---

## 2. Typography Scale

Inspired by shadcn/ui's clean hierarchy, optimized for desktop reading (slightly smaller body than web).

| Token | Size | Weight | Line Height | Usage |
|-------|------|--------|-------------|-------|
| `h1` | 28px | 700 | 1.3 | Page title |
| `h2` | 22px | 600 | 1.35 | Section title |
| `h3` | 18px | 600 | 1.4 | Card title |
| `h4` | 16px | 600 | 1.4 | Subsection |
| `body` | 14px | 400 | 1.5 | Default body text |
| `body-sm` | 13px | 400 | 1.5 | Compact body |
| `label` | 13px | 500 | 1.4 | Form label |
| `caption` | 12px | 400 | 1.4 | Helper text, badges |
| `small` | 11px | 400 | 1.3 | Legal, timestamps |
| `code` | 13px | 400 | 1.5 | Monospace code |

### Font Family

```
--font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI Variable", "Segoe UI", Roboto, sans-serif
--font-mono: "Cascadia Code", "JetBrains Mono", "Fira Code", Consolas, monospace
```

---

## 3. Spacing System

4px base grid (wider than Ant Design's 4px but compressed vs MUI's 8px for desktop density):

| Token | Pixels | Usage |
|-------|--------|-------|
| `0.5` | 2px | Icon gap, tight packing |
| `1` | 4px | Inner padding (badge, tag) |
| `1.5` | 6px | Dense button icon gap |
| `2` | 8px | Component gap, form spacing |
| `3` | 12px | Card padding, section gap |
| `4` | 16px | Panel padding, large gap |
| `5` | 20px | Section margin |
| `6` | 24px | Page margin, card group gap |
| `8` | 32px | Major section separation |
| `10` | 40px | Page padding |

---

## 4. Border Radius

| Token | Size | Usage |
|-------|------|-------|
| `none` | 0 | Sharp elements |
| `sm` | 4px | Inputs, cards in data view |
| `md` | 6px | Buttons, default components |
| `lg` | 8px | Cards, dialogs |
| `xl` | 12px | Modals, large surfaces |
| `full` | 9999px | Badges, pills, avatars |

---

## 5. Shadow / Elevation

Multi-layer shadows (inspired by MUI's 25-level shadow system, simplified for desktop):

| Token | Light | Dark | Usage |
|-------|-------|------|-------|
| `sm` | `0 1px 2px rgba(0,0,0,0.04)` | `0 1px 2px rgba(0,0,0,0.3)` | Subtle card |
| `md` | `0 4px 12px rgba(0,0,0,0.06)` | `0 4px 12px rgba(0,0,0,0.35)` | Popover, Menu |
| `lg` | `0 8px 24px rgba(0,0,0,0.08)` | `0 8px 24px rgba(0,0,0,0.45)` | Dialog, Modal |
| `xl` | `0 16px 48px rgba(0,0,0,0.10)` | `0 16px 48px rgba(0,0,0,0.55)` | Notification, Tooltip |

---

## 6. Component Sizing

Standard control heights (inspired by MUI density + shadcn compactness):

| Size | Height | Icon Size | Font | Usage |
|------|--------|-----------|------|-------|
| `xs` | 22px | 12px | 11px | Badge, tag, small meta |
| `sm` | 28px | 14px | 13px | Compact toolbar |
| `md` | 34px | 16px | 14px | Default control |
| `lg` | 40px | 18px | 15px | Primary action |
| `xl` | 48px | 20px | 16px | Hero, mobile |

---

## 7. Component API Patterns

### Builder Pattern (Rust)

```rust
// Every component follows this structure:
Button::new("save")
    .label("儲存設定")
    .variant(ButtonVariant::Primary)
    .size(ControlSize::Md)
    .disabled(false)
    .on_click(AppMsg::Save)

// Semantic shorthand:
Button::new("save").label("儲存").primary()  // primary variant
Button::new("cancel").label("取消").ghost()   // ghost variant
```

### States

Every interactive component supports:

| State | Visual |
|-------|--------|
| Default | Normal appearance |
| Hover | Background lighten/darken 5-8% |
| Active/Pressed | Background darken 10% |
| FocusVisible | 2px ring (offset 2px) |
| Disabled | Opacity 0.4, no interaction |
| Loading | Spinner replaces / overlays content |
| Selected / Checked | Primary color indicator |

### Tone Mapping

| Tone | Background | Foreground | Border |
|------|-----------|------------|--------|
| Neutral | `muted` | `foreground` | `border` |
| Primary | `primary` | `primary-foreground` | `primary` |
| Success | `success` | `white` | `success` |
| Warning | `warning` | `white` | `warning` |
| Danger | `danger` | `white` | `danger` |

Soft variant (for badges, alerts):

| Tone | Background | Foreground |
|------|-----------|------------|
| Neutral-soft | `muted` | `foreground` |
| Primary-soft | `accent` | `accent-foreground` |
| Success-soft | `success-soft` | `success` |
| Warning-soft | `warning-soft` | `warning` |
| Danger-soft | `danger-soft` | `danger` |

---

## 8. Animation Tokens

| Token | Duration | Easing | Usage |
|-------|----------|--------|-------|
| `fast` | 100ms | ease-out | Hover, press |
| `normal` | 200ms | ease-out | Transition, toggle |
| `slow` | 300ms | ease-in-out | Enter/leave |
| `slide` | 250ms | ease-out | Panel slide |
| `spring` | 400ms | spring(0.3, 0.8, 0.2, 1.0) | Bouncy UI |

---

## 9. Component Checklist

| Component | Priority | shadcn | MUI | Ant Design | Our API |
|-----------|----------|--------|-----|------------|---------|
| Button | P0 | ✅ | ✅ | ✅ | `Button::new(id).label().variant().size()` |
| Card | P0 | ✅ | ✅ | ✅ | `card().variant().child()` |
| Badge | P0 | ✅ | ✅ | ✅ | `badge("text").tone().size()` |
| Input | P0 | ✅ | ✅ | ✅ | Built on `TextInput` |
| Select | P0 | ✅ | ✅ | ✅ | `select().option()` |
| Checkbox | P0 | ✅ | ✅ | ✅ | `checkbox().checked()` |
| Radio | P0 | ✅ | ✅ | ✅ | `radio_group().option()` |
| Switch | P0 | ✅ | ✅ | ✅ | `switch().checked()` |
| Slider | P0 | ✅ | ✅ | ✅ | `slider().min().max()` |
| Tabs | P0 | ✅ | ✅ | ✅ | `tabs().tab().variant()` |
| Separator | P0 | ✅ | ✅ | ✅ | `separator().orientation()` |
| Tooltip | P0 | ✅ | ✅ | ✅ | `tooltip().content()` |
| Popover | P0 | ✅ | ✅ | ✅ | `popover().content().placement()` |
| Dialog | P0 | ✅ | ✅ | ✅ | `dialog().title().content()` |
| Menu | P0 | ✅ | ✅ | ✅ | `menu().item().separator()` |
| Progress | P0 | ✅ | ✅ | ✅ | `progress().value().tone()` |
| Skeleton | P0 | ✅ | ✅ | ✅ | `skeleton().width().height()` |
| Spinner | P0 | ✅ | ✅ | ✅ | `spinner().size().tone()` |
| Alert | P1 | ✅ | ✅ | ✅ | `alert().tone().title().description()` |
| Tag | P1 | ✅ | ✅ | ✅ | `tag().text().removable()` |
| Avatar | P1 | ✅ | ✅ | ✅ | `avatar().fallback()` |
| Pagination | P1 | ✅ | ✅ | ✅ | `pagination().total().page()` |
| Breadcrumb | P1 | ✅ | ✅ | ✅ | `breadcrumb().item()` |
| Toast | P1 | ✅ | ✅ | ✅ | Built on Notification |
| Collapsible | P1 | ✅ | ✅ | — | `collapsible().header().content()` |
| Accordion | P1 | ✅ | ✅ | ✅ | `accordion().item().title()` |
| Drawer | P1 | ✅ | ✅ | ✅ | `drawer().title().placement()` |
| Table | P1 | ✅ | ✅ | ✅ | `table().column().row()` |
| Tree | P2 | ✅ | ✅ | ✅ | `tree().node().expanded()` |
| DataGrid | P2 | — | ✅ | ✅ | `datagrid().column().row()` |
| VirtualList | P2 | — | ✅ | ✅ | `virtual_list().children()` |
| DatePicker | P2 | ✅ | ✅ | ✅ | `date_picker().value().on_select()` |
| ColorPicker | P2 | ✅ | ✅ | ✅ | `color_picker().value().on_change()` |
| Dropdown | P1 | ✅ | ✅ | ✅ | Built on Menu + Popover |
| CommandPalette | P2 | ✅ | — | — | Built |
| Calendar | P2 | ✅ | ✅ | — | `calendar(year, month).selected_day()` |
| AspectRatio | P2 | ✅ | ✅ | — | `aspect_ratio(16.0/9.0).width().child()` |
| Flex | P1 | ✅ | ✅ | ✅ | `h_flex().gap().child()` / `v_flex()` |
| ButtonGroup | P1 | ✅ | ✅ | ✅ | `button_group("id").item("A", msg).selected(0)` |
| TagInput | P1 | ✅ | ✅ | ✅ | `tag_input("id").tags(vec!["a","b"]).on_add(msg)` |
| Banner | P1 | ✅ | ✅ | ✅ | `banner("message").tone(Tone::Warning).dismissible()` |
| CommandBar | P1 | ✅ | — | — | `command_bar("id").search_placeholder("Find...").action("icon","label",msg)` |
| Mentions | P2 | — | — | ✅ | `mentions("id").value("@user").option(opt)` |

---

## 10. Implementation Order

### Phase 1 — Theme V2 (now)
- Rewrite `acme-theme` with full token set
- Add soft color tokens (success-soft, warning-soft, danger-soft)
- Add elevation surface ladder
- Add shadow tokens
- Add full typography scale
- Add animation tokens
- Add `ControlSize` with proper px mapping

### Phase 2 — Core Components (now)
- Rewrite `acme-ui` components with V2 design tokens
- Implement consistent state handling (hover/focus/pressed/disabled/loading/selected)
- Add soft variants to Badge, Alert
- Add focus ring support
- Use consistent builder API

### Phase 3 — Gallery (now)
- Redesign gallery with V2 theme
- Add live theme switching
- Show design token reference
- Beautiful component showcases

### Phase 4 — Polish
- Focus ring rendering
- Shadow/elevation rendering in wgpu
- Animation system integration
- Transition effects on hover
