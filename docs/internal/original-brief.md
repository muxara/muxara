# Muxara: Build Brief for Claude Code

## Purpose

Muxara is a desktop control plane for developers who run multiple parallel Claude Code sessions and lose context when switching between them.

The problem is not terminal management by itself. The real problem is cognitive overload:

* too many concurrent sessions
* unclear session identity
* unclear current state
* hard to know which session needs attention
* too much time spent reconstructing context from recent terminal output

Muxara should reduce that cognitive burden.

## Core Intent

Build a lightweight desktop application that gives the user a persistent, always-visible overview of active Claude Code sessions, with enough context to quickly orient themselves and jump into the right session at the right time.

Muxara is not a replacement for Claude Code, tmux, or the terminal. It is a thin orchestration and observability layer above them.

Its job is to help the human answer these questions at a glance:

* What sessions currently exist?
* Which session belongs to which ticket or workstream?
* Which sessions are active, blocked, or safe to ignore?
* What is the recent context of each session?
* Which session should I jump into next?

## Product Vision

Muxara should feel like a mission-control dashboard for parallel AI-assisted coding sessions.

The desired user experience is:

* a persistent window that can stay visible on screen
* a compact grid or list of session cards
* each card gives both orientation and recency
* the user can click a card to jump into the underlying session
* the user can create a new session from a visible plus button
* as the number of sessions grows, the dashboard remains usable rather than becoming another source of clutter

The app should optimize for clarity, fast scanning, and low cognitive friction.

## Conceptual Model

Treat each Claude Code session as a first-class work unit.

Each session should have a visible representation in Muxara that helps the user understand:

* identity
* current state
* recent conversational tail
* whether it needs user input or attention

Muxara should externalize session state that is currently hidden inside terminals and conversation history.

## High-Level Functional Expectations

At a high level, Muxara should:

* discover and monitor active Claude Code sessions
* present them as cards inside a persistent desktop window
* allow the user to click a card to focus or switch into that session
* allow the user to create a new session from the UI
* infer useful session metadata from available runtime signals
* update continuously enough to feel live, but without becoming noisy or expensive

Avoid requiring the user to manually maintain state for each session.

## Session Card Vision

Each session should be represented as a single card composed of two distinct visual regions, or two sub-panes within one card.

One side should help with recent context.
The other side should help with orientation.

Examples of useful content include:

* session title or inferred identity
* status such as running, blocked, idle, or waiting on user action
* compact summary or inferred description
* the recent tail of the conversation or recent terminal output
* any signal that helps the user rapidly decide whether this session matters now

Do not treat these examples as rigid requirements. Choose the best representation that serves the core intent: fast re-orientation with minimal mental effort.

## Status and Attention Model

Muxara should help the user distinguish between sessions that:

* need user intervention now
* are actively progressing
* are idle or safe to ignore for the moment

This attention model is central to the product.

The user should be able to scan the dashboard and immediately see which sessions are blocked on permissions, selections, confirmations, or similar interaction gates.

Status should be inferred pragmatically from the available runtime signals rather than depending on manual annotations.

## New Session Flow

Muxara should include a visible plus button or equivalent affordance to create a new session from within the app.

The experience should feel lightweight and immediate.

At a high level, creating a new session should:

* provision a fresh underlying session using the chosen execution model
* start Claude Code in that session, if appropriate
* make the new session appear in the dashboard automatically

Muxara should aim to make session creation feel native to the app, even if the actual execution happens through terminal tooling under the hood.

## Implementation Philosophy

You should choose the implementation plan.

Prioritize a design that is:

* simple enough to ship
* reliable enough for daily use
* extensible enough to evolve
* appropriate for open-source publication

You are free to decide the exact architecture, libraries, UI framework, IPC model, session discovery strategy, and runtime details.

Do not overfit to one narrow implementation too early if a cleaner abstraction is possible.

However, do not over-engineer v1 either.

A strong v1 should focus on delivering the core user value with as little complexity as practical.

## Likely Technical Direction

The application will likely need a small desktop UI plus a local runtime layer that can inspect and switch sessions.

A practical and recommended architecture (as a suggestion, not a constraint) is:

### Desktop Framework

Consider using entity["software","Tauri","Rust-based desktop app framework"] as the primary desktop framework.

Rationale:

* lightweight compared to Electron
* native-feeling performance
* good fit for a persistent always-on-top window
* simple integration between frontend (JS) and backend (Rust or system commands)

Alternative options (if more appropriate):

* Electron (faster iteration, larger footprint)
* Native macOS app (if platform-specific optimisation is desired)

### Frontend (UI Layer)

* React (or similar component-based framework)
* Tailwind CSS or minimal styling system for fast iteration

The UI should implement:

* card-based layout
* two-pane card structure (context + metadata)
* responsive grid that scales with number of sessions

### Backend / System Layer

A thin local backend layer should:

* interface with tmux
* run shell commands
* parse terminal output
* expose structured session data to the UI

Language options:

* Rust (native with Tauri)
* Node.js (simpler for rapid development)
* Python (acceptable if simpler integration is preferred)

### Session Management

Use entity["software","tmux","terminal multiplexer"] (or equivalent) as the execution layer.

Responsibilities:

* create sessions
* list sessions
* capture pane output
* switch between sessions

Muxara should not replace tmux, but build a control layer on top of it.

### Data Flow

At a high level:

* UI requests session list
* backend queries tmux
* backend parses output and derives state
* backend returns structured session objects
* UI renders cards

This loop should refresh periodically (e.g., every 1–2 seconds) to provide a near real-time experience.

### Optional Enhancements (Future)

* LLM-based summarisation of session logs
* richer status detection heuristics
* grouping sessions by workstream or ticket
* persistence of session metadata

These should not be required for v1.

Design this thoughtfully.

## Constraints and Preferences

Keep these principles in mind:

* the app should be useful on a single monitor setup where screen real estate matters
* the dashboard should remain visible without dominating the screen
* the interface should support growing numbers of sessions gracefully
* the product should reduce context switching, not add more process overhead
* the app should not depend on the Claude session itself to manually maintain metadata if it can be inferred externally
* the product should be publishable as an open-source GitHub project under the name Muxara

## Non-Goals

Muxara is not trying to:

* replace the terminal
* replace Claude Code
* become a full ticketing system
* become a complex project management tool
* require heavy manual workflow discipline from the user

It should remain tightly focused on live session visibility, orientation, and switching.

## Product Quality Bar

Aim for something that feels coherent and intentional, not just a technical prototype.

Even in an early version, the app should have:

* a clear mental model
* a clean and readable UI
* sensible defaults
* reliable session refresh behavior
* a credible path for future enhancements

## Suggested Working Style

Start by clarifying the product shape before coding.

Think through:

* the user journey
* the smallest viable product that still feels useful
* what the cards should show
* how status should be inferred
* how new sessions should be created
* how switching back into a session should work
* what makes the app feel calm instead of chaotic

Then choose an implementation plan and execute it.

## Final Instruction

Use your judgment. The important thing is not to mirror these words mechanically.

Build Muxara as a thoughtful, practical, open-source desktop application that helps a developer manage multiple Claude Code sessions with less confusion, less context loss, and faster re-orientation.

