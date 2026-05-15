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
- Check relevant source files on `main` and link code references to GitHub.
- Check version tags with `git ls-remote --tags origin '<version>'` when the post references a release.
- Use `git diff` between current version and previous version (versions are tagged x.x.x) to figure out what changes have been made.

## Post Requirements

Create posts in `_posts/` with Jekyll front matter:

```markdown
---
layout: post
title: "Short Human Title"
date: YYYY-MM-DD HH:MM:SS +ZZZZ
authors: Sergey and Cursor
---
```

Do not include the version number in the post title. Keep the title catchy and human-readable; put the version link near the top of the post instead.

Use this top-of-post pattern when a version exists:

```markdown
[Version x.x.x on GitHub][version-x-x-x]{: .no-github-icon}
```

The tag scheme is plain `x.x.x`, not `vx.x.x`.

## Writing Style

- Write as a project diary, not a changelog.
- Treat Sergey and Cursor as pair-programming collaborators.
- Explain what we focused on, what we discussed, and why design/implementation choices were made.
- Keep milestone scope honest; do not overclaim beyond current code.
- Prefer concrete examples from the session logs over generic summaries.
- Mention next steps when they naturally follow from `project-breakdown.md`.
- Use sentence-style capitalization for section/subsection headers inside posts (for example, `## Why the new output uses radial spokes`, not title case). Post titles may still use title case.

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

- Pin render output images to the post’s release tag, not `main`, for example `https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/0.0.3/doc/output/current.webp` for version 0.0.3. The homepage may keep `main` for live project progress.
- The site uses Minima/Jekyll. Run `bundle exec jekyll build` after edits.
- If `authors:` is present, `_layouts/post.html` renders the header as `May 10, 2026 — by Sergey and Cursor`.
- Do not edit `_site/`; it is build output.

## Validation

Before finishing:

- Run `bundle exec jekyll build`.
- Use `ReadLints` on changed Markdown/layout files.
- Mention if a referenced tag does not exist remotely yet.
