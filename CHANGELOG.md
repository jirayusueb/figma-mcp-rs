# Changelog

## [1.0.0](https://github.com/jirayusueb/figma-mcp-rs/compare/v0.1.0...v1.0.0) (2026-09-10)


### ⚠ BREAKING CHANGES

* update_paint_style is removed. Use update_style, which takes the same styleId/name/color/description plus the text, effect, and grid fields.

### Features

* full variable and style read/write, plus hex/rgb/hsl/oklch colors ([0c41a94](https://github.com/jirayusueb/figma-mcp-rs/commit/0c41a94fa85923df827233950c70e4e2e615b6a2))
* **plugin:** apply layout sizing after append, reuse applyAutoLayout ([43b120d](https://github.com/jirayusueb/figma-mcp-rs/commit/43b120dad14b38f38962c5da6703c2a285c1ebe9))
* **plugin:** redesign UI with tailwind, kobalte and cva components ([806cdd0](https://github.com/jirayusueb/figma-mcp-rs/commit/806cdd0982250cc6a36c91dbe2ed840eb6b97ff7))


### Bug Fixes

* **ci:** attach plugin.zip with gh release upload ([005d7ef](https://github.com/jirayusueb/figma-mcp-rs/commit/005d7ef30d2a7259dce5b21a2de6a7f919d676b7))
* **ci:** download only release artifacts, skip dockerbuild build-record ([42176fc](https://github.com/jirayusueb/figma-mcp-rs/commit/42176fcba2222951f5d8c091ea69a942d6de9670))
* **ci:** inline release build into release-please workflow ([d167299](https://github.com/jirayusueb/figma-mcp-rs/commit/d167299cf6075203cd26da876ebff11f5354417a))
* **ci:** release-please job must succeed on dispatch so builds run ([f5d37b9](https://github.com/jirayusueb/figma-mcp-rs/commit/f5d37b903af27e3ff53e42ca715dfcebe5243c69))

## [0.1.0](https://github.com/jirayusueb/figma-mcp-rs/compare/v0.0.1...v0.1.0) (2026-09-10)


### Features

* **ci:** CI and release-please release automation ([dd96556](https://github.com/jirayusueb/figma-mcp-rs/commit/dd96556a31070ac75826daf1b5cc33c9881a24e5))


### Bug Fixes

* **plugin:** typecheck errors; style: rustfmt 1.98 formatting ([38a203f](https://github.com/jirayusueb/figma-mcp-rs/commit/38a203fca7970f7489cbeeaa376dffe9cd495ea2))
