# MoodShell

A living, emotionally reactive terminal companion built with **Rust** (atatui, crossterm, 	okio, serde) and **Python** (Braille dot-matrix multi-frame sprite rendering).

## Features
- **Hybrid Rust + Python Architecture**: 30 FPS TUI, 2D physics, task tracking, and JSON state in Rust; big multi-frame Braille sprites in scripts/companion.py.
- **Emotions & Progression**: Reacts to tasks, deadlines, focus mode, petting, and idle time across moods and levels.
- **Task & Focus Workflow**: Add tasks with @30m or @2h (or --due-mins), complete tasks for XP/streaks, and toggle Focus Mode.

## Controls
-  Add task | d Complete task | p Pet | j Jump |  Focus mode | q Quit
