---
name: write-post
description: Draft milestone blog posts for the rust-3d-rasterizer Jekyll diary. Use when writing a new project article, milestone post, release write-up, or diary entry from session logs, planning docs, implementation commits, or version tags.
---

# Write Post

Use this skill when Sergey asks for a new blog post/article for the 3D rasterizer project diary.

## Source Material

Gather context before writing:

- Read `AGENTS.md` for repo conventions.
- Read the homepage `_pages/index.md` for voice and project framing.
- Inspect `main` branch docs with `git show` / `git ls-tree`, especially:
  - `doc/diary/session-*.md`
  - `doc/planning/project-spec.md`
  - `doc/planning/project-breakdown.md`
  - `README.md`
- Read Cursor agent transcripts 
- Check relevant source files on `main` and link code references to GitHub.
- Check version tags with `git ls-remote --tags origin '<version>'` when the post references a release.
- Use `git diff` between current version and previous version (versions are tagged x.x.x) to figure out what changes have been made.

### Cursor agent transcripts

For milestone posts tied to a release tag, find the relevant Cursor chat logs with `.cursor/skills/write-post/scripts/transcripts-for-tags.sh`. 

Run from the repo root (gh-pages):

```bash
.cursor/skills/write-post/scripts/transcripts-for-tags.sh 0.0.6
```

Or pass an explicit range (previous release through current):

```bash
.cursor/skills/write-post/scripts/transcripts-for-tags.sh 0.0.5 0.0.6
```

Behavior:

- **One tag** — lists transcripts from the previous semver tag through that tag (e.g. `0.0.5` → `0.0.6` for `0.0.6`).
- **Two or more tags** — lists transcripts between the earliest and latest tag times.

Then **read every `.jsonl` file** the script prints. Use them for session narrative: what we focused on, design discussions, and implementation choices. Prefer transcript detail over generic summaries when it matches the code and diary docs.

If the script fails, say so and continue from `doc/diary` and `git diff` only.

## Post Requirements

Create posts in `_posts/` with Jekyll front matter:

```markdown
---
layout: post
title: "Short Human Title"
date: YYYY-MM-DD HH:MM:SS +ZZZZ
authors: Sergey and Cursor
# tags: [bugfix]   # optional — see Writing Style
---
```

Do not include the version number in the post title. Keep the title catchy and human-readable; put the version link near the top of the post instead.

Use this top-of-post pattern when a version exists:

```markdown
[Version x.x.x on GitHub][version-x-x-x]{: .no-github-icon}
```

The tag scheme is plain `x.x.x`, not `vx.x.x`.

## Writing Style

### Default: tutorial diary

Write as a **project diary**, not a changelog — but lean **tutorial / educational** unless the post is tagged otherwise (see below). Treat Sergey and Cursor as pair-programming collaborators.

Almost every milestone post introduces **new graphics or pipeline concepts**. The reader should finish the post **familiar with those ideas** — what they mean, why we use them, and how they fit this milestone — not just aware that the code changed. Structure for learning:

1. **Hook** — what changed visually and how it connects to the previous release.
2. **What you will see** — concrete demo/output before theory.
3. **Concepts** — explain models, conventions, or math the milestone depends on (subsections per idea; analogies and diagrams where they help).
4. **Implementation** — brief bridge to code (see below).
5. **What comes next** — natural follow-on from `project-breakdown.md` when applicable.

Prefer patient, accessible prose over expert shorthand. Use session logs and transcripts for *accuracy* and *motivation*, but teach the concepts even if the session was a long debug thread.

Keep milestone scope honest; do not overclaim beyond current code. Prefer concrete examples from the session logs over generic summaries. Use sentence-style capitalization for section/subsection headers inside posts (for example, `## Why the new output uses radial spokes`, not title case). Post titles may still use title case.

### Introducing terms

Follow `AGENTS.md`: set a term in **italics** on its **first** mention in the post (e.g. `_directional light_`, `_Lambertian diffuse_`). Later mentions use plain text unless the word is also a Rust identifier (then backticks). Link the first meaningful mention to Wikipedia, docs, or prior diary posts when helpful.

### Implementation sections

Keep **implementation details short** by default: name the main types/functions, how they map to the concepts above, and one or two design notes worth remembering. Link symbols to GitHub on first meaningful mention.

Expand implementation only when the code is **genuinely tricky** — subtle bugs, sign conventions, naming mismatches that teach something, or refactors that are hard to infer from the diff alone. Do not walk file-by-file through straightforward wiring.

### Exceptions: `bugfix` and `refactor` tags

Optional front matter: `tags: bugfix` or `tags: refactor`.

- **`bugfix`** — investigative narrative is fine (symptom → reproduction → diagnosis → fix). Still explain any convention the reader must not repeat, but skip the full tutorial arc if the post is mainly about a mistake and its patch.
- **`refactor`** — focus on what moved, why, and what stays the same for readers of older posts; concept primers only where the refactor changes mental models.

Untagged milestone posts should follow the tutorial diary shape above (see e.g. `_posts/2026-05-22-the-cube-gets-light.md` for tone and depth on concepts vs. code).

## Math and formulas

The site loads **MathJax** in `_includes/head.html`. Use LaTeX in post bodies:

- **Inline:** `$...$` (e.g. `$t \in [0, 2\pi)$`, `$R_z(t)\, R_y(t)\, R_x(t)$`).
- **Display:** `$$...$$` on their own lines for larger formulas.

**When to use LaTeX:** expressions with **symbolic** notation — variables ($t$, $i$, $\alpha$), rotation matrices, intervals, vectors, $\pi$ / $\tau$, axis labels ($+\mathrm{Y}$), and similar.

**When not to:** values that are **only numbers and units** in prose — write **800×600**, **360** frames, **20 ms**, **50 fps**, **7.2 s**, version numbers, and plain counts in normal Markdown (bold where emphasis helps). Do not wrap those in `$...$`.

## Links And References

Use reference-style Markdown links.

For links between diary posts:

- After creating a new post, update the immediately previous related post to include a natural inline "next step" link to the new post (keep the original narrative voice smooth; do not turn it into a retrospective rewrite).
- Use Jekyll post links in reference definitions (not hardcoded absolute URLs):

```markdown
[post-some-title]: {{site.baseurl}}/{% post_url YYYY-MM-DD-slug %}
```

For code references:

- Link symbols, modules, tests, constants, and APIs to GitHub source on `main` or the relevant version tag.
- Link a given source-code reference only on its first meaningful mention; later mentions can stay as plain backticked text.
- Prefer single-line anchors (`#L<start>`) over line ranges (`#Lx-Ly`) for source links.
- Example:

```markdown
[`FrameBuffer`][source-framebuffer]

[source-framebuffer]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/src/framebuffer.rs#L6
```

For common knowledge:

- Add helpful references for file formats, algorithms, data structures, graphics concepts, and crates.
- Prefer Wikipedia for broad concepts and crates.io/docs.rs for Rust crates.
- Do not turn every technical word into a link; link the first meaningful mention.

## Site Conventions

- Pin render output images to the post’s release tag, not `main`, with an explicit version split:
  - **`< 0.1.0`**: `https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/<VERSION>/doc/output/current.webp` (example: `.../0.0.17/doc/output/current.webp`)
  - **`>= 0.1.0`**: `https://github.com/tindandelion/rust-3d-rasterizer/releases/download/<VERSION>/scene.webp` (example: `.../releases/download/0.1.0/scene.webp`)
- The site uses Minima/Jekyll. Run `bundle exec jekyll build` after edits.
- If `authors:` is present, `_layouts/post.html` renders the header as `May 10, 2026 — by Sergey and Cursor`.
- Do not edit `_site/`; it is build output.

## Validation

Before finishing:

- Run `bundle exec jekyll build`.
- Use `ReadLints` on changed Markdown/layout files.
- Mention if a referenced tag does not exist remotely yet.
