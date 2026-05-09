---
name: diary-entry
description: 'Create a diary entry with the contents of the current session '
disable-model-invocation: true
---

# Diary Entry

## Overview

Create a diary entry with the contents of the current session.

## When to Use

Use this skill when:

- The user calls the skill as a slash command.

## Instructions

- Export the current session into a file under "doc/diary" directory in Markdown format.
- Preserve as much information as possible.
- The file name should be "session-<timestamp>", where <timestamp> is a human-readable date and time, e.g. "2026-05-09T11".
