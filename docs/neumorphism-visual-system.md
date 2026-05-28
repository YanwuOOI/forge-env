# Soft Industrial Neumorphism

This document defines the initial visual system for the Tauri-based development environment manager.

## Design Intent

- Direction: professional desktop utility, not consumer toy UI.
- Theme policy: light theme only for v1.
- Visual language: restrained neumorphism with improved contrast and compact density.
- Engineering target: React + Tailwind + CSS Variables inside Tauri.

## Source Notes

- `ui-ux-pro-max` was installed locally at `~/.codex/skills/ui-ux-pro-max`.
- Its search data favored a dark developer-tool aesthetic and recommended accessibility-first "Soft UI Evolution" traits for neumorphism.
- This system keeps the user's requested light theme and professional tone, while adopting the skill's stronger guidance on contrast, visible focus, compact motion, and restrained shadow depth.

## Design Principles

- Use depth to group surfaces, not to decorate everything.
- Keep at most three visual elevation levels on one screen.
- Preserve information density for developer workflows.
- Never rely on shadow alone to express state changes.
- Prefer quiet neutral surfaces with one strong accent color.

## Token Summary

### Colors

| Token | Value | Usage |
|---|---|---|
| `--bg-canvas` | `#E7ECF3` | app background |
| `--bg-panel` | `#EDF2F8` | large surfaces |
| `--bg-elevated` | `#F4F7FB` | cards, dialogs |
| `--bg-inset` | `#E3E9F2` | pressed fields, segmented shells |
| `--text-primary` | `#1F2937` | primary text |
| `--text-secondary` | `#5B6778` | secondary text |
| `--text-muted` | `#7F8A9A` | metadata |
| `--border-soft` | `#D8E0EA` | dividers, subtle outlines |
| `--accent-primary` | `#2F6BFF` | primary action |
| `--accent-hover` | `#255AE0` | primary hover |
| `--accent-soft` | `#DCE7FF` | tinted fills |
| `--success` | `#1F9D68` | success |
| `--warning` | `#D18A1D` | warning |
| `--danger` | `#D14B5A` | destructive |
| `--info` | `#2A8CCF` | informative |
| `--focus-ring` | `rgba(47, 107, 255, 0.38)` | keyboard focus |
| `--scrim` | `rgba(31, 41, 55, 0.18)` | modal backdrop |

### Typography

| Token | Value |
|---|---|
| `--font-ui` | `"Manrope", "SF Pro Text", "Segoe UI", sans-serif` |
| `--font-mono` | `"JetBrains Mono", "SFMono-Regular", monospace` |
| `--text-12` | `0.75rem` |
| `--text-14` | `0.875rem` |
| `--text-16` | `1rem` |
| `--text-20` | `1.25rem` |
| `--text-24` | `1.5rem` |
| `--text-32` | `2rem` |

- Default body text: `14px`.
- Primary workspace title: `20px / 600`.
- Use tabular mono numerals for versions, ports, paths, and job identifiers.

### Spacing and Radius

- Spacing scale: `4 / 8 / 12 / 16 / 24 / 32 / 40 / 48`.
- Radius scale:
  - `--radius-sm: 10px`
  - `--radius-md: 16px`
  - `--radius-lg: 22px`
  - `--radius-pill: 999px`

### Shadows and Motion

| Token | Value |
|---|---|
| `--shadow-raised-sm` | `4px 4px 8px rgba(163, 177, 198, 0.32), -4px -4px 8px rgba(255, 255, 255, 0.85)` |
| `--shadow-raised-md` | `8px 8px 16px rgba(163, 177, 198, 0.34), -8px -8px 16px rgba(255, 255, 255, 0.88)` |
| `--shadow-inset` | `inset 3px 3px 6px rgba(163, 177, 198, 0.30), inset -3px -3px 6px rgba(255, 255, 255, 0.82)` |
| `--motion-hover` | `160ms` |
| `--motion-press` | `120ms` |
| `--motion-panel-enter` | `220ms` |
| `--ease-standard` | `cubic-bezier(0.2, 0.8, 0.2, 1)` |

## Layout Guidance

- App shell uses `--bg-canvas`.
- Sidebar and main workspace are separate raised panels, not flat split panes.
- Top bar should look lighter than panels; do not give it the heaviest shadow.
- Search, host switching, and filter groups should use inset containers to communicate "tool well" behavior.
- Avoid card-on-card-on-card nesting. If a section already sits on a raised panel, inner content should usually be inset or flat.

## Component Rules

### Button

- Primary:
  - background: `--accent-soft`
  - text: `--accent-primary`
  - default: `--shadow-raised-sm`
  - hover: lift slightly and use `--shadow-raised-md`
  - active: switch to `--shadow-inset`
- Secondary:
  - neutral surface, no accent fill
  - same shadow rhythm as primary
- Disabled:
  - reduce contrast and remove lift

### Input and Select

- Base surface: `--bg-inset`
- Shadow: `--shadow-inset`
- Padding: visually generous, never cramped
- Focus: add a visible `2px` focus ring; do not only intensify the existing shadow
- Validation:
  - error state uses `role=alert` nearby
  - color is never the only signal

### Card

- Use for runtime cards, diagnostics, task queue, host summaries.
- Default: `--bg-elevated` + `--shadow-raised-sm`
- Hover: small upward translate and slightly stronger shadow
- Do not stack extra decorative glows or borders

### Sidebar Item

- Container uses inset-selected state instead of a filled pill.
- Active item must change label contrast and icon color, not only shadow.
- Keyboard focus must remain explicit even when selected.

### Segmented Control

- Outer shell: inset
- Selected segment: raised small
- This is the preferred treatment for host mode, runtime scope, and filter toggles.

### Table Row and List Row

- Avoid rigid grid borders.
- Use spacing, subtle bottom separators, and hover lift.
- Tabular data such as version and port columns should use mono figures.

### Modal and Drawer

- Modal shell uses `--shadow-raised-md`.
- Backdrop stays light and neutral; no glassmorphism blur by default.
- Primary action area should stay visually compact and obvious.

## Accessibility and UX Constraints

- Normal text contrast must meet at least `4.5:1`.
- Icon-only buttons must include `aria-label`.
- Keyboard focus must be visible on every interactive component.
- Motion must animate only `transform`, `opacity`, or `box-shadow`.
- Respect `prefers-reduced-motion`.
- Each screen gets one primary CTA at most.

## Tailwind and CSS Variable Rules

- Components consume semantic tokens only; no raw hex values inside component files.
- Use `bg-[var(--token)]`, `text-[var(--token)]`, and shadow utilities mapped from the shared preset.
- Keep semantic naming at the component layer:
  - `surface-canvas`
  - `surface-panel`
  - `surface-elevated`
  - `surface-inset`
  - `text-primary`
  - `text-secondary`
  - `accent-primary`
- Avoid ad hoc shadow values. Use only the three system shadow tokens.

## Screen-Level Application

### Overview

- Overview is the most dashboard-like screen.
- Use one large raised workspace shell, with inset sub-panels for search/filter strips.
- Runtime health cards may use raised small cards with compact badges.

### Languages

- Language version lists should read as tool inventory, not a marketing grid.
- Prefer a dense list with inset-selected filters and raised version cards.

### Projects

- Project detection results should use hierarchy through spacing and mono metadata.
- Missing-runtime warnings should use warning tint plus icon and action, not color alone.

## Anti-Patterns

- Pure white highlights across large surfaces
- Decorative neon glow
- Purple default branding
- Deep drop shadows that feel mobile-game-like
- More than three elevation levels on one screen
- Large empty cards that reduce workspace density
