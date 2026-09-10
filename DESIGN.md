---
name: figma-mcp-rs
description: Industrial Brutalist system — warm-black ground, paper ink, amber accent, Space Grotesk + JetBrains Mono, zero radius, 1px rules.
colors:
  bg: "#0e0d0b"
  bg-elev: "#15130f"
  bg-offset: "#0a0907"
  ink: "#e8e3d6"
  ink-dim: "#b8b1a0"
  ink-soft: "#8a8378"
  ink-mute: "#807a6e"
  accent: "#d49a4f"
  accent-dim: "#a07a3c"
  rule: "#2a2620"
  rule-strong: "#3d3830"
  positive: "#7fa05f"
  negative: "#b56a4a"
  light-bg: "#f4f0e4"
  light-bg-elev: "#ffffff"
  light-bg-offset: "#eae5d7"
  light-ink: "#1a1814"
  light-ink-dim: "#3d3830"
  light-ink-soft: "#6b6355"
  light-ink-mute: "#75695b"
  light-accent: "#946325"
  light-accent-dim: "#8a5e22"
  light-rule: "#d9d3c2"
  light-rule-strong: "#b8b2a0"
  light-positive: "#4d6b32"
  light-negative: "#8a4528"
typography:
  display:
    fontFamily: "Space Grotesk, system-ui, -apple-system, 'Segoe UI', sans-serif"
    fontSize: "clamp(56px, 8vw, 112px)"
    fontWeight: 500
    lineHeight: 0.95
    letterSpacing: "-0.035em"
  headline:
    fontFamily: "Space Grotesk, system-ui, -apple-system, 'Segoe UI', sans-serif"
    fontSize: "22px"
    fontWeight: 600
    lineHeight: 1.15
    letterSpacing: "-0.02em"
  title:
    fontFamily: "Space Grotesk, system-ui, -apple-system, 'Segoe UI', sans-serif"
    fontSize: "15px"
    fontWeight: 600
    lineHeight: 1.45
    letterSpacing: "-0.015em"
  body:
    fontFamily: "Space Grotesk, system-ui, -apple-system, 'Segoe UI', sans-serif"
    fontSize: "16px"
    fontWeight: 400
    lineHeight: 1.45
  body-mono:
    fontFamily: "JetBrains Mono, ui-monospace, SFMono-Regular, Menlo, Consolas, monospace"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.6
  label:
    fontFamily: "JetBrains Mono, ui-monospace, SFMono-Regular, Menlo, Consolas, monospace"
    fontSize: "10px"
    fontWeight: 600
    lineHeight: 1.4
    letterSpacing: "0.1em"
rounded:
  none: "0px"
  dot: "50%"
spacing:
  s-1: "4px"
  s-2: "8px"
  s-3: "12px"
  s-4: "16px"
  s-5: "20px"
  s-6: "24px"
  s-8: "32px"
  s-10: "40px"
  s-12: "48px"
  s-16: "64px"
  s-20: "80px"
  s-24: "96px"
components:
  button-primary:
    backgroundColor: "{colors.accent}"
    textColor: "{colors.bg}"
    typography: "{typography.label}"
    rounded: "{rounded.none}"
    padding: "0 20px"
    height: "44px"
  button-primary-hover:
    backgroundColor: "transparent"
    textColor: "{colors.accent}"
  input:
    backgroundColor: "{colors.bg}"
    textColor: "{colors.ink}"
    typography: "{typography.body-mono}"
    rounded: "{rounded.none}"
    height: "44px"
    padding: "10px 16px"
  card:
    backgroundColor: "{colors.bg-elev}"
    rounded: "{rounded.none}"
    padding: "24px"
  tag:
    backgroundColor: "transparent"
    textColor: "{colors.ink-mute}"
    typography: "{typography.label}"
    rounded: "{rounded.none}"
    padding: "4px 10px"
  kicker:
    backgroundColor: "transparent"
    textColor: "{colors.accent}"
    typography: "{typography.label}"
    rounded: "{rounded.none}"
    padding: "0 0 0 10px"
---

# Design System: figma-mcp-rs

Exact mirror (authorized, user-owned source) of the **"Industrial Brutalist" v2** system. Values are copied, not adapted; this file is now the system of record for figma-mcp-rs.

## Overview

**Creative North Star: "Industrial Brutalist"**

The system reads as a printed operations ledger set in a warm darkroom: warm-black ground, warm-paper ink, and one amber signal color doing all the pointing. Nothing is rounded, nothing floats — structure is drawn with 1px rules and negative space, exactly like a technical data sheet. Hierarchy comes from type weight, size contrast, and mono micro-labels (uppercase, heavily tracked), never from shadows or color fills.

Density is deliberate: rows over cards, hairline dividers over gaps, labels at 9–11px mono uppercase with 0.1–0.2em tracking against 56–112px display type. The grain overlay (SVG fractal noise, `opacity: 0.035`, `mix-blend-mode: overlay`, fixed) keeps the dark ground from reading as a flat screen.

**Key Characteristics:**
- Radius 0 everywhere; the only circles are the chat loading dots
- 1px rules as the primary structural device (`--rule` internal, `--rule-strong` at edges)
- Amber (`--accent`) is the single signal: links on hover, kickers, primary buttons, focus rings, selection
- All controls and metadata in JetBrains Mono; all reading text in Space Grotesk
- WCAG AA on body text is enforced in the token values themselves (mute tones were deliberately bumped; do not lower them)

## Colors

One warm neutral family plus one amber accent. Dark is the default theme; light is a full peer theme via `[data-theme="light"]`.

### Primary
- **Signal Amber** (#d49a4f, light #946325): the only accent. Kickers, link hovers, focus rings, selection background, primary buttons, active markers. Light-mode value was bumped for AA on paper.

### Neutral
- **Warm Black** (#0e0d0b): page ground.
- **Elevated Soot** (#15130f): raised panels (`--bg-elev`).
- **Offset Pit** (#0a0907): recessed wells, toggles, inputs (`--bg-offset`).
- **Paper Ink** (#e8e3d6): primary text.
- **Dim Paper** (#b8b1a0): secondary body text.
- **Soft Dust** (#8a8378): tertiary text, icon chrome.
- **Mute Dust** (#807a6e): labels, metadata — WCAG AA floor for body text.
- **Rule** (#2a2620) / **Rule Strong** (#3d3830): internal / edge hairlines, also used as 1px grid gaps (gap-collapsing trick: `gap: 1px; background: var(--rule)`).
- **Positive** (#7fa05f) / **Negative** (#b56a4a): status semantics only, never decoration.
- Light theme mirrors all of the above on **Paper** (#f4f0e4 / #ffffff / #eae5d7) with **Ink** (#1a1814, #3d3830, #6b6355, #75695b) and paper rules (#d9d3c2 / #b8b2a0).

### Named Rules
**The One Signal Rule.** Amber carries ≤10% of any surface. If two adjacent elements both pull amber, one is wrong.
**The AA Floor Rule.** `--ink-mute` is the lightest text color permitted on body copy; the values were hand-bumped for WCAG AA — never lower contrast by swapping to a dimmer token "for mood."

## Typography

**Display Font:** Space Grotesk (system-ui stack fallback)
**Body Font:** Space Grotesk (system-ui stack fallback)
**Label/Mono Font:** JetBrains Mono (ui-monospace stack fallback)

**Character:** Space Grotesk gives the display voice a squarish, engineered confidence with tight negative tracking; JetBrains Mono does every job that is "not prose" — labels, controls, data, code. The pairing is the identity: if a control is not mono, it is off-system.

### Hierarchy
- **Display** (500, clamp(56px, 8vw, 112px), 0.95, -0.035em): hero title, once per page.
- **Headline** (600, 22px, 1.15, -0.02em): section titles, card titles, stat numbers.
- **Title** (600, 15px, -0.015em): widget headers.
- **Body** (400, 15–16px, 1.45): reading text in Space Grotesk; `font-variant-numeric: tabular-nums`.
- **Body Mono** (400, 11.5–14px, 1.6): descriptions, sub-copy, banner text.
- **Label** (600, 9–11px mono, 0.1–0.2em, uppercase): kickers, tags, nav, metadata. The 0.2em + border-left-2px-accent form is the **kicker**; 0.1em + border form is the **tag**.

### Named Rules
**The Mono Controls Rule.** Anything the user clicks, types into, or reads as data is JetBrains Mono. Space Grotesk is for sentences and headlines only.

## Layout

Full-bleed horizontal bands stacked vertically, each separated by a 1px rule (`--rule-strong` at band edges, `--rule` inside). No max-width container on marketing pages — bands run edge to edge with 40px side padding (20px under 720px). Grids are built from the 4px spacing scale; multi-cell grids use the 1px-gap-on-rule-background trick instead of visible borders. Density reference: stats rows are 5-up with 18px 24px cells; nav rows are 10–14px vertical rhythm.

Breakpoints (max-width): 1100 (grids collapse to 1–2 col), 900 (masthead stacks), 720, 640, 600, 560, 480.

## Elevation & Depth

**No shadows.** Depth is conveyed tonally: `--bg` → `--bg-elev` (raised) → `--bg-offset` (recessed), plus edge weight (1px rule vs rule-strong). The only overlay is the grain (fixed, above everything, `z-index: 9999`, pointer-events none) and focus rings (2px solid amber, 2px offset). Modality, when needed, is expressed as a 2px solid ink border (see source `#chat-panel`).

### Named Rules
**The Flat Surface Rule.** Never add box-shadow, gradient fills, or blur. A surface that needs to stand out gets a border, a tone step, or amber — in that order.

## Shapes

Radius is 0 on every component — buttons, inputs, cards, tags, banners. The single exception: circular loading dots (4px, 50%). Corners meet at hard right angles; borders butt against each other with `margin-left: -1px` collapse (input + button, stacked cells) rather than rounding or gaps. Underline links use `text-underline-offset: 3px` with 1px thickness.

## Components

Implementation carrier in this repo: **shadcn-solid** (Kobalte + cva + Tailwind v4) in `plugin/src/components/ui/*`, themed via CSS custom properties in `plugin/src/ui/app.css` mapped to these tokens.

### Buttons
- **Shape:** square corners (0px), mono font, 600 weight, uppercase, 0.1em tracking.
- **Primary:** amber bg, ground-color text, 44px height, 20px horizontal padding; hover inverts to transparent bg + amber text (border stays amber); disabled 0.55 opacity.
- **Ghost / icon:** transparent, ink-dim text; hover = ink text. No fills.
- **Focus:** 2px solid amber outline, 2px offset.

### Chips / Tags
- **Style:** transparent bg, 1px rule-strong border, mono 11px uppercase 0.1em, ink-mute text, 4px 10px padding. Star/important variant: amber text + amber border.

### Cards / Containers
- **Corner Style:** 0px. **Background:** `--bg-elev` with 1px `--rule` border. **Shadow:** none. **Internal Padding:** `--s-6` (compact: `--s-4 --s-5`).
- **Data card** (plugin panel): rows of label/value split by 1px `--rule` dividers, 10px 16px row padding, no outer border when embedded in a bordered panel.

### Inputs / Fields
- **Style:** 1px `--rule-strong` border, `--bg` (or `--bg-offset`) fill, mono, 44px height.
- **Focus:** border goes amber + `box-shadow: 0 0 0 2px var(--accent)` (no radius, no glow beyond the 2px ring). Placeholder `--ink-mute`.
- **Adjacent submit** butts with -1px margin, sharing the 1px rule.

### Navigation
- **Style:** mono 11px uppercase 0.1em; hover amber; active = ink text + 1px amber bottom border. Meta items prefixed with `▸ ` glyph in rule-strong.

### Kicker (signature)
- **Style:** mono 10px 600 uppercase 0.2em tracking, amber, `border-left: 2px solid var(--accent); padding-left: 10px`. Opens every section. Variant without border for inline labels.

### Status (plugin-local)
- **State band (hero):** full-width band on `--bg-elev` between 1px `--rule-strong` edges, holding a 10px square status marker (`--positive` connected / `--negative` disconnected) and the state word in Space Grotesk 600 at 24px, -0.02em. The state word is the only display-voice text in the panel.
- **Activity line:** shown only while calls are in flight — 2px `--accent` left border on `--bg-offset`, mono 9px amber, `→ tool_name` left and `×n` in-flight count right (tabular). Never a spinner: the running tool names itself.
- **Data rows:** mono uppercase 0.1em labels against mono values, 1px `--rule` dividers, no card chrome.
- **Input focus:** amber border plus a 2px inset amber outline (the system's focus language; the v4 ring var chain is not used).

## Do's and Don'ts

### Do:
- **Do** copy token values verbatim from this file; they are the mirrored source of truth, AA-tuned.
- **Do** build structure from 1px rules and the 4px spacing scale; butt adjacent controls with -1px margins.
- **Do** use the grain overlay on any full-page dark surface.
- **Do** ship both themes; dark is default, light flips via `[data-theme="light"]` with the paired values above.
- **Do** keep tabular-nums on any numeric readout.

### Don't:
- **Don't** round anything (0 radius) or add shadows, gradients, glows, or backdrop blur.
- **Don't** use amber beyond the One Signal Rule's ~10% budget; never as a large fill except the primary button and badge.
- **Don't** set controls or labels in Space Grotesk — controls are mono, always.
- **Don't** invent additional accent colors or grays; the tokens above are the whole palette.
