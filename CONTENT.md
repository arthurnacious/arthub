# Adding content

Everything the site shows comes from `content/` (text) and `assets/media/` (images and video). It's compiled into the site, so after a change, either `just dev` is already running (it rebuilds on save) or you run `just build`.

If something is wrong (a typo in a field name, a missing image, a bad colour), **the build stops and lists every problem** with its file and line. Nothing broken reaches the live site.

## Add a project

1. **Pick a slug.** Lowercase, digits and dashes: `night-market`. It becomes the URL `/work/night-market`.
2. **Add the media** to `assets/media/work/night-market/`:
   - `cover.jpg` (required)
   - optional `preview.mp4`, the hover loop
   - any gallery images or videos
3. **Write `content/projects/night-market.md`.** Copy an example and edit it. The file name must match the slug.

```toml
+++
title = "Night Market"
slug = "night-market"
year = 2026
client = "Client name"            # or "Personal project"
role = "Lead developer"
categories = ["Web", "3D"]        # drives the filters on /work
tools = ["Rust", "Leptos"]
featured = true                   # home page + an arcade cabinet in the game room
accent = "#ff2e88"                # optional: cabinet glow colour
cover_image = "cover.jpg"         # file name inside assets/media/work/<slug>/
cover_alt = "What the cover shows"  # optional, defaults to the title
preview_video = "preview.mp4"     # optional: plays on hover and as the case-study hero
summary = "One or two sentences. Shown on cards and as the page description."
order = 0                         # optional: sorts projects within the same year (lower first)

[[gallery]]                       # repeat for each image or video, in order
src = "process-1.webp"
alt = "Describe the image for screen readers"
caption = "Optional caption"

[[links]]
label = "Live site"
url = "https://example.com"

[[credits]]
role = "Design"
name = "Someone Great"
+++

## Overview
Markdown from here on is the case study body.

## The problem
...

## Process
...

## Outcome
Tables, lists, quotes and links all work.
```

That's it. The project appears on `/work`, gets its own page, and, if `featured = true`, shows up on the home page and as an arcade cabinet in the game room. The game room fits up to six cabinets; extra featured projects are listed on the site but don't get a cabinet. You don't need to rebuild the game room itself, because the site hands it the featured list when it starts.

Projects are listed newest first by `year`.

## Images and video

| Use | Size | Format | Notes |
| --- | --- | --- | --- |
| **Cover** | 1600 × 1000 (16:10) | JPG or WebP, quality ~80, under 300 KB | Also the arcade-cabinet screen, so no AVIF here. Keep key content away from the edges; cards crop slightly. |
| **Gallery image** | 1600 px on the long side | WebP (or AVIF/JPG), under 400 KB | Portrait images take two rows in the gallery. |
| **Preview / gallery video** | 960 × 600, 4–8 s loop | MP4 (H.264), no audio, under 2 MB | Encode with `-movflags +faststart`. Muted, looped, loads only when hovered. |
| **Portrait** | 1200 × 1500 (4:5) | JPG or WebP | `assets/media/about/` |
| **CV** | — | PDF | `assets/media/`, then set `cv` in `experience.toml` |

The build reads each image's real dimensions, so pages reserve the right space and don't jump while images load. You don't need to enter sizes anywhere.

Handy conversions:

```sh
magick input.png -resize 1600x1000^ -gravity center -extent 1600x1000 -quality 80 cover.jpg
magick input.png -resize 1600x1600\> -quality 80 process-1.webp
ffmpeg -i clip.mov -vf scale=960:-2 -t 6 -an -c:v libx264 -crf 28 -pix_fmt yuv420p -movflags +faststart preview.mp4
```

## The other pages

- **`content/about.md`**: name, role, intro line, portrait, stats strip, "Currently" list (front matter) and your bio (Markdown body).
- **`content/experience.toml`**: one `[[role]]` per job, freelance period, degree or milestone, newest first. `kind` is `job`, `freelance`, `education` or `milestone`. Leave `end = ""` for "Now". Add `cv = "cv.pdf"` to show a download button.
- **`content/skills.toml`**: `[[group]]`s of skills, each with a `level` of `daily`, `confident` or `learning`, shown as chalk tallies rather than percentages.

## Replacing the examples

Everything that ships with the repo is placeholder:
- **Projects:** the four projects (`example = true`, shown with an "Example" badge) and their media folders.
- **Profile pages:** entries marked `placeholder = true` in about, experience and skills (shown with a "Placeholder" badge).

To clear them out:
1. Delete the example `.md` files and their `assets/media/work/<slug>/` folders. Keep at least one project, or the build will say so.
2. Remove the `placeholder` / `example` lines as you replace each entry.

`scripts/make-example-media.sh` regenerates the example images if you ever need them back.
