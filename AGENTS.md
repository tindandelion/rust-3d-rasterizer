# AGENTS.md

Guidance for coding agents working in this repository.

## What this repo is

This is the **`gh-pages` branch** of [`tindandelion/rust-3d-rasterizer`](https://github.com/tindandelion/rust-3d-rasterizer). It hosts a [Jekyll](https://jekyllrb.com/) site (Minima theme) published via GitHub Pages at <https://tindandelion.com/rust-3d-rasterizer/>. The site is a **project diary** for the Rust 3D rasterizer.

The actual Rust source code lives on the **`main` branch** of the same repository — do not look for it here.

## Project documentation 

- Project plans live in `doc/planning` directory of the `main` branch. 
- Project session logs live in `doc/diary` directory of the `main` branch. 

## Tech stack

- **Jekyll** with the **Minima** theme (see `Gemfile`, `_config.yml`).
- **kramdown** (GFM parser) for Markdown.
- **SCSS** for styles (`assets/main.scss`), imports Minima and overrides selectively.
- Plugins: `jekyll-feed`. Pinned to the `github-pages` gem so behavior matches GitHub's build.
- Dev environment: Ruby 3.3 dev container (`.devcontainer/devcontainer.json`).

## Repo layout

```
_config.yml      Site config (title, baseurl, theme, plugins)
_includes/       Partial templates (head.html, header.html) overriding Minima
_layouts/        Page layouts (home.html) overriding Minima
_pages/          Standalone pages (index.md, 404.html); included via `include:` in _config.yml
_posts/          Blog posts — create as YYYY-MM-DD-title.md (does not exist yet)
assets/          SCSS and static assets
.devcontainer/   Ruby dev container
.github/         dependabot.yml
_site/           Build output (gitignored)
```

## Local development

```bash
bundle install
bundle exec jekyll serve --livereload
# http://127.0.0.1:4000/rust-3d-rasterizer/
```

Note the `baseurl: /rust-3d-rasterizer` in `_config.yml` — always use `{{ "..." | relative_url }}` in templates and `[text][ref]` Markdown links rather than hardcoding paths.

## Conventions

- **New blog posts** go in `_posts/` as `YYYY-MM-DD-slug.md` with front matter (`layout: post`, `title:`, optional `date:`).
- **New standalone pages** go in `_pages/` (already wired via the `include:` key in `_config.yml`); set a `permalink:` in front matter.
- **Layout/include overrides** of the Minima theme should be minimal — only override files that genuinely need changes; let Minima provide the rest.
- **Styles**: edit `assets/main.scss`. Keep overrides scoped; don't fork the Minima stylesheet wholesale.
- **External code links** point to the `main` branch of `rust-3d-rasterizer` (see `_pages/index.md` for examples).
- **Bold in diary posts**: use sparingly — only to highlight truly important bits (e.g. one pivotal outcome or diagnosis). Prefer plain prose; use backticks for code and identifiers. Do not bold routine terms, axes, counts, or milestone names for decoration.
- **Italics for new terms**: when introducing a graphics or project term for the first time in a post, set it in italics (e.g. `_directional light_`, `_ambient light_`). After that first mention, use plain text unless the term is also a code identifier (then use backticks).

## Don't

- Don't commit `_site/` (already in `.gitignore`).
- Don't add Jekyll plugins outside the [GitHub Pages allowlist](https://pages.github.com/versions/) — they won't run on the deployed site.
- Don't change `baseurl` or `url` in `_config.yml` without a clear reason; the deployed URL depends on them.
