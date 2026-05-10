---
layout: home
title: Welcome
permalink: /
list_title: Project diary
---

This is yet another learning project of mine, again in Rust. This time, I'm building a **3D software rasterizer in Rust**. 

### Our current progress

![Current render output](https://raw.githubusercontent.com/tindandelion/rust-3d-rasterizer/main/doc/output/current.webp)

### Motivations

There are two things I want to learn with this project, running side by side.

The first is **3D rasterization itself**. I started programming in high school, and almost immediately became fascinated by the [demoscene][demoscene]: tiny programs that made very limited PCs draw things that felt nearly impossible at the time. Rotating 3D shapes, lights, shadows, strange graphical effects packed into 64K or even 4K binaries — I had no idea how any of it worked, but I badly wanted to understand it.

Back then, that curiosity had nowhere easy to go. Learning resources were scarce, and I was missing too many basics: mathematics, computer science, and, maybe most importantly, the skill of learning itself. Now, after more than twenty years as a software developer and with the whole internet available as a reference shelf, I can return to that old fascination properly.

In that sense, this project is an homage to my adolescent self: curious, ignorant, and frustrated, but still pulled toward the same questions. So, we'll do some 3D graphics. 

The plan is to build the math and rasterization path on the **CPU** first, then add optional **GPU** rendering with **wgpu** (Metal on Mac) as a stretch goal. Scenes stay small and procedural — working up shapes like a cube, a sphere, and eventually a torus — with lossless **WebP** stills and animations as the main artifacts.

The second motivation, equally important in the modern day and age, is to deliberately practice **AI-assisted ("agentic") coding**. Tools like Cursor and similar agents have changed the day-to-day of writing software, and I want to develop a real, hands-on intuition for working with them — not as a passive autocomplete user, but as someone delegating meaningful work and reviewing the result.

### Working with AI agents

A few things I want to figure out along the way:

* **Use AI as much as possible, but consciously.** Where does it shine? Where does it struggle? Where am I tempted to lean on it instead of actually understanding something?
* **AI as a learning partner**, not just a code generator. Asking it to explain trade-offs, derive the math, propose alternatives, and critique my reasoning — and then verifying what it tells me against a primary source.
* **Keeping the code clean** in spite of AI's tendency to produce *slop*: redundant abstractions, over-engineered helpers, dead code, premature generality, and tests that look plausible but assert nothing useful. I want to find prompting patterns, review habits, and project conventions that push back on this.

We'll be co-authoring the project diary together: me (**Sergey**) as the project owner and reviewer, **Cursor** as the coding partner who helps capture what happened, explain the trade-offs, and keep the notes connected to the code. Over time, I want to trust Cursor more and more with keeping this diary up to date, while still reading critically and steering the voice.

### Project scope

I'll consider the project meaningfully accomplished when I can:

* Render a small set of procedural 3D shapes — at minimum a cube, a sphere, and a torus — with a working camera and basic lighting
* Produce smooth animations of those scenes, captured as lossless WebP
* Run the whole pipeline on the CPU, with the optional `wgpu`-based GPU path as a stretch goal
* Look at the codebase six months from now and still be happy to work in it

We are going to be working in intentionally [small steps][breakdown], so that there's enough time to learn, collaborate, and improve.

### Useful links

* [Project specification][spec] — goals, math, dependencies, coordinate conventions, raster strategy
* [Milestone breakdown][breakdown] — iterative milestones and expected artifacts
* [Source code on GitHub][repo] — the Rust implementation lives on the `main` branch

[spec]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/doc/planning/project-spec.md
[breakdown]: https://github.com/tindandelion/rust-3d-rasterizer/blob/main/doc/planning/project-breakdown.md
[repo]: https://github.com/tindandelion/rust-3d-rasterizer
[bt-blog]: https://www.tindandelion.com/rust-bittorrent-client/
[demoscene]: https://en.wikipedia.org/wiki/Demoscene
