+++
# EXAMPLE PROJECT: replace with your own work (see CONTENT.md).
example = true
title = "Render Farm"
slug = "render-farm"
year = 2025
client = "Example Studio"
role = "Systems engineer"
categories = ["Systems", "Tooling"]
tools = ["Rust", "Tokio", "PostgreSQL", "Kubernetes"]
featured = true
accent = "#22e6ff"
cover_image = "cover.jpg"
preview_video = "preview.mp4"
summary = "A job scheduler that turned a closet of idle workstations into an overnight render farm, cutting delivery times in half."

[[gallery]]
src = "process-1.webp"
alt = "Architecture sketch of the scheduler"
caption = "Scheduler, workers and the shared cache."

[[gallery]]
src = "outcome.webp"
alt = "Dashboard showing queued and finished jobs"

[[links]]
label = "Write-up"
url = "https://example.com/render-farm"
+++

## Overview

Example copy for a systems project: what was slow, what you built, and
what changed.

## The problem

Renders queued all day and finished after deadlines, while machines sat idle
every night.

## Process

A small Rust service hands out frames to any idle machine, retries failures,
and caches assets so each frame starts in seconds.

## Outcome

Overnight throughput went up 4×, and the team stopped babysitting renders.
