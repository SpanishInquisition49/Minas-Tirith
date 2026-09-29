# Minas Tirith — Logo Guidelines (compact)

## 1. The logo

- **Idea — "The Ward":** a bookmarked reference crowned with the city's own guarded skyline. It literalizes what the
  app does (saving/archiving a reference) while tying the shape to the name — a citadel silhouette, without borrowing
  any film or Tolkien-trademark imagery.
- **Versions:** horizontal lockup · stacked lockup · symbol-only · small-size symbol cut (for ≤24 px contexts)
- **Files:** all masters are SVG. `favicon/` holds the raster/ICO/manifest set for web use.

## 2. Files

| File | Use |
| --- | --- |
| `minas-tirith-mark.svg` / `-white.svg` | Symbol only, ≥32 px (app about screens, social avatars, print) |
| `minas-tirith-mark-small.svg` / `-small-white.svg` | Symbol only, ≤24 px (favicon, terminal tab, tiny UI) — bolder single notch, simplified for legibility |
| `minas-tirith-lockup-horizontal.svg` / `-white.svg` | Symbol + wordmark, side by side — primary lockup for headers, README |
| `minas-tirith-lockup-stacked.svg` / `-white.svg` | Symbol above wordmark — square-ish spaces, splash screens |
| `minas-tirith-app-icon.svg` / `.png` | Symbol on a dark rounded-square tile — desktop app icon, dock/launcher |
| `favicon/*` | `favicon.ico`, PNG icon set, `site.webmanifest`, `head-snippet.html` — drop-in for a docs site or web UI |

The wordmark is a custom geometric monospace built from straight strokes (no system font, no licensing to track).
It ships as a stroke path (`stroke-width` on a `<path>`), not filled outlines — fine for screens and most print, but
convert to outlines first for workflows that require pure fills (vinyl cutting, embroidery, some print pipelines).

## 3. Clear space

Keep a clear zone of **1× the symbol's crenellation notch width** around the mark on all sides (roughly 15% of the
symbol's own height). Never crowd it against other UI chrome or type. The zone scales with the logo — never a fixed
pixel distance.

## 4. Minimum size

| Version | Screen |
| --- | --- |
| Horizontal lockup | 120 px wide |
| Stacked lockup | 72 px wide |
| Symbol (`-mark`) | 32 px |
| Symbol (`-mark-small`) | 16 px (favicon / terminal tab) |

Below 32 px, always swap to the `-small` symbol cut — the full crown (three merlons) blurs into a blob at tiny sizes;
the small cut uses one wider notch that stays legible.

## 5. Colour

| Name | HEX | Use |
| --- | --- | --- |
| Ink (primary) | `#17171A` | Mark and wordmark on light backgrounds |
| Parchment | `#F4EFE6` | Mark and wordmark reversed on dark/terminal backgrounds; tile foreground |

The mark is designed to work in a single colour first. There is no secondary/accent brand colour yet — if one is
added later (e.g. for docs-site links or a highlight), keep it out of the mark itself.

**Approved pairs:** ink mark on white/light · parchment (white) mark on ink or on a terminal's dark background.
Don't place the mark on a busy background or photo without a solid tile behind it (see `minas-tirith-app-icon.svg`
for the tile pattern).

## 6. Typography

Wordmark: custom-built geometric monospace (not a licensed font — drawn as straight-line paths). If you ever need to
*set* "Minas Tirith" in running text/UI rather than reproduce the logo, use the project's regular UI font; don't
retype the logo's lettering as live text.

## 7. Don'ts

Don't stretch or squash the mark · don't recolour outside the two-colour palette above · don't add gradients,
drop shadows or outlines · don't rotate it · don't separate the crown from the bookmark shape or redraw it with a
different crenellation count · don't recreate the wordmark by typing "Minas Tirith" in a system font.

## 8. Background

Replaces the earlier placeholder artwork (`images/minas.png`, LOTR fan-art referencing Tolkien's Ring-script and
film styling). This mark is original artwork built for the project: a guarded citadel reduced to a single bookmark
silhouette, sized to survive a 16 px favicon and a terminal tab icon, which the old artwork could not do.
