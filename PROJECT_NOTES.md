# RUST-AutoRASM — Project Notes

Single consolidated place for working notes, design decisions, and pending
discussions from dev sessions. (Formal specs live in their own files:
`ARCHITECTURE.md`, `ROADMAP.md`, `COMMAND_LINE.md`, `Hatch_Pattern_Wishlist.md`,
etc. This file is the running log of decisions + open questions.)

---

## Build & dev environment (Windows)

Dev shifted from Arch Linux → Windows (2026-06-18).

- **Toolchain:** rustup + `stable-x86_64-pc-windows-msvc` (`winget install Rustlang.Rustup`).
  Links against the installed **Visual Studio Build Tools 2026** (MSVC + Windows SDK 10.0.26100).
- **Windows build fix:** `cad_app/Cargo.toml` had `eframe` features `wayland` + `x11`
  (Linux-only). Made them Linux-conditional via
  `[target.'cfg(target_os = "linux")'.dependencies]`, so it builds on Windows
  (native Win32 backend) and the old Arch setup. Re-apply if pulling fresh upstream.
- **Build:** `cargo build --workspace [--release]`. Release profile uses
  `lto=true` + `codegen-units=1` (slow link, ~1 min).

### Auto-rebuild dev loop
- `cargo dev` (alias in `.cargo/config.toml`) or `.\dev.ps1` (`-Release` switch)
  — `cargo-watch` rebuilds + relaunches the app on every save. Keep one
  PowerShell terminal running it; edits auto-reload (the app window blinks).
- It **restarts** the app (in-window state lost each reload). True
  state-preserving hot-reload was declined as too invasive for the ~22k-line `app.rs`.
- **Running-exe lock gotcha:** a running `rust_cad.exe` LOCKS its own file, so
  `cargo build --release` fails with `Access is denied (os error 5)` mid-link.
  Close the app (or kill the process) before rebuilding release. `cargo dev`
  avoids this (kills the old run first). Debug & release are separate files, so
  `cargo run -p cad_app` (debug) works even while an old release exe is open.

---

## ZOOM command (done, 2026-06-18)

AutoCAD-style `zoom`/`z` sub-option flow (`ZoomState` enum + `zoom_*` methods in
`cad_app/src/app.rs`); view driven by `scale` + `world_offset`.
- Wired: **All**(=Extents), **Center**, **Extents**, **Previous** (10-deep
  history), **Scale** (`nX` only), **Window** (+ live amber preview rectangle),
  **Object** (fit selection), **Real-time** (primary drag up/down).
- **Scope decisions (don't re-implement without asking):** All == Extents (no
  drawing-limits concept exists); **Dynamic** intentionally stubbed; Scale is
  `nX`-relative only (no `XP`/paper-space — model-space only — and no absolute scale).

---

## File Open / Save dialog (done, 2026-06-19)

`render_file_dialog` in `cad_app/src/app.rs`.
- **Path bar:** editable, commits on Enter / **Go** (no per-keystroke nav);
  pointing at a file jumps to its folder and preselects it.
- **Drive dropdown:** lists existing drive roots (probes `A:`–`Z:` on Windows).
- **File-type bar:** **DXF (\*.dxf)** | **Native (\*.rsm)**; list filters by type.
  Open defaults to `.rsm`. (NOTE: native extension is **`.rsm`**, not `.rasm`.)
- **Preview pane:** parses the selected `.dxf`/`.rsm` once (cached) and renders a
  fit-to-rect wireframe by temporarily swapping in the preview Document + a fit
  transform; shows `N object(s) · M layer(s)`.
- **Hidden/system filter:** skips Windows hidden (0x2) / system (0x4) entries
  (`$RECYCLE.BIN`, `System Volume Information`) so the list matches Explorer.
- **Layout (2026-06-20):** pinned **top** (path bar) + **bottom** (Type/File/
  buttons) panels via `TopBottomPanel::show_inside`, with the list+preview in a
  filling `CentralPanel`. Window is capped to the screen → always fits, footer
  always visible, **height freely resizable** (earlier `available_height` body
  blew past the screen and clipped the controls).
- **Save (2026-06-20):** `File ▸ Save` writes to the current file (tracked via
  `current_file`, set on open/save); falls back to Save As if none yet.

---

## Selection & editing model (2026-06-20)

- **Pointer-mode selection:** plain click / drag-window = **fresh** selection
  (replace); **Shift** = add; **Alt** = remove. Empty plain click clears.
  Implemented in `click_select(i, shift, alt, fresh)` + `add_window_selection(..,
  shift, alt, fresh)`. KEY FIX: pointer-mode drag applies DIRECTLY (no
  `begin_selection`, which clears) so Shift/Alt+drag add/remove vs the existing
  bunch instead of replacing.
- **Select-session shortcuts** (while a command asks for a selection): `p` =
  previous, `L` = last drafted, `D` = deselect mode. Intercepted in
  `run_command` only when `select_mode != Off`.
- **Del key** erases the selection (focus-independent).
- **LINE is connected:** segments chain (last endpoint → next start); **Esc**
  ends + exits. Re-run `line` for a separate segment.
- **Copy / Paste = Edit menu only** (no Ctrl+C/V keys). Copy → clipboard of
  dobject clones. Paste = placement flow (`PasteState`): pick BASE → DESTINATION
  with a green ghost preview (mirrors COPY), commits clones with fresh handles,
  selected. Esc cancels.

---

## Hatch rules — TO DISCUSS / DECIDE (open)

**Observed (2026-06-19):** selecting a hatch and running Move selects it but it
doesn't move.

**Reason — current model is purely associative.** `cad_kernel`'s `Hatch` stores
ONLY `boundary_handles` (references to other dobjects) and **no geometry of its
own**; the fill is resolved from those boundary entities each frame
(`resolve_hatch_loops`). Consequences:
- `translated()`/`rotated()`/`scaled()` are **no-ops for Hatch**
  (`geom.rs` arms just `h.clone()`), so Move/Copy/Rotate/Scale/Mirror don't move it.
- Deleting/moving a boundary line makes the hatch shrink/vanish or stay put.
- `Hatch::bbox()` is `(0,0)`; hit-test needs the Document to resolve loops.

**Proposed model (user direction, 2026-06-19) — decide later:**
- A hatch should **own a baked, invisible copy of its boundary loops** so it is
  self-sufficient: if the source boundary is erased/removed, the hatch **maintains**
  its stored boundary and stays put.
- Editing the hatch boundary happens only when the user **selects the hatch** and
  changes it (then re-bake the owned loops).
- **Associativity becomes an optional extra layer:** keep `boundary_handles` as a
  link that *re-bakes* the owned loops when a linked source changes — but the owned
  loops are the source of truth. (This is essentially AutoCAD's model.)
- Once a hatch owns geometry, `translated()`/`rotated()`/`scaled()` operate on the
  owned loops → Move/Copy/Rotate "just work", and bbox/hit-test no longer need the
  Document.

**Status:** not implemented. Discuss + decide the mechanism (owned loops format,
when to break vs keep the associative link, migration of existing hatches).

### Move preview (related, pending)
Requested: while moving, show a ghost preview of the selected dobjects following
the cursor (base → destination). COPY and PASTE already have this ghost; MOVE
does not yet. Straightforward; no design fork. Not yet built.
