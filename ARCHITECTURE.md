# RUST_CAD — Architecture & Library Layout

Authoritative map of the workspace crates, how they depend on each other, and
**where new code goes**. Read this before adding a feature or moving code.

---

## 1. Layering principle

Three layers, bottom to top:

1. **MODEL** — `cad_kernel` (+ `cad_nurbs`). Pure data + math, UI-free. The
   `Geom` enum, every entity **data type** (Line, Arc, Circle, Ellipse, Wall,
   Dim, Text, Polyline, Spline, Hatch…), the style tables (LayerTable,
   DimStyleTable, WallStyleTable…), the `Document`, and the cross-cutting
   per-variant behavior that has to `match` the enum (transform, snap,
   intersect, bbox, grips, parser).

2. **FEATURE crates** — one per major smart feature: `cad_wall`, `cad_dim`,
   `cad_text`. They **depend on `cad_kernel`** and hold the feature
   *algorithms* that are NOT just data: the wall junction solver + curved-wall
   derive, the dimension render-geometry + number formatting, text layout.
   Pure, UI-free, headless-testable. They consume kernel data types and return
   plain results / render primitives (`Vec<(Vec2,Vec2)>`, etc.).

3. **APP / IO** — `cad_io` (file formats), `cad_app` (the egui GUI: paints the
   primitives the feature crates derive, and hosts dialogs + tool input),
   `cad_cli` (headless command runner).

### Why feature crates depend on the kernel (not the reverse)

The `Geom` enum in `cad_kernel` must *name* `Wall`/`Dim`/`Text`, so those data
types live in the kernel. A "leaf entity crate" would force
`cad_kernel → cad_wall → cad_kernel` — a **cycle**. Putting the feature crates
*above* the kernel (logic only; data in the kernel) is cycle-free. This is the
chosen design (user-approved: "wall depending on cad_kernel is OK").

> Alternative considered & rejected: extract a `cad_math` base crate (Vec2 +
> primitives) and make `cad_dim`/`cad_wall`/`cad_text` leaf crates under it.
> That gives the entities full independence but is a much bigger refactor and
> isn't needed once feature crates may depend on the kernel.

---

## 2. Crates

| Crate | Type | Role | Depends on |
|---|---|---|---|
| `cad_nurbs` | lib | Pure-Rust NURBS / B-spline curve math (leaf). | — |
| `cad_kernel` | lib | Model + math core: `Geom`, all entity data types, styles, `Document`, transform/snap/intersect/bbox/parser. | cad_nurbs |
| `cad_wall` | lib | **Wall feature logic**: junction solver (`solve_faces`), curved-wall derive, future T/X-junctions, openings, rooms, convert-closed→wall. | cad_kernel |
| `cad_dim`  | lib | *(planned)* Dimension feature logic: render-geometry, number formatting. | cad_kernel |
| `cad_text` | lib | *(planned)* Text feature logic: layout, wrapping, alignment. | cad_kernel |
| `cad_io` | lib | File I/O: `dxf`, `rsm` (native). | cad_kernel |
| `cad_snap` | lib | Thin public facade over `cad_kernel::snap` (external API / doctests; not consumed internally). | cad_kernel |
| `cad_cli` | bin | Headless command runner (parse + apply, no GUI). | cad_kernel |
| `cad_app` | bin `rust_cad` | The eframe/egui GUI: paints primitives, dialogs, tool input. | cad_kernel, cad_io, cad_wall, (cad_dim, cad_text) |

### Dependency graph

```
cad_nurbs
   └─ cad_kernel ─┬─ cad_wall ─┐
                  ├─ cad_dim  ─┤
                  ├─ cad_text ─┤
                  ├─ cad_io  ──┤
                  ├─ cad_snap  │   (facade; not consumed internally)
                  ├─ cad_cli   │
                  └────────────┴─ cad_app
```

---

## 3. Where does new code go? (decision rule)

- A new entity **data type**, a `Geom` variant, or per-variant
  transform/snap/intersect/bbox/grip logic → **`cad_kernel`**.
- A **feature algorithm** (a solver, a derive-to-render-primitives function,
  formatting) that is UI-free → the **feature crate** (`cad_wall`/`cad_dim`/
  `cad_text`).
- egui **rendering**, **dialogs**, **tool input handling** → **`cad_app`**.
- A **file format** reader/writer → **`cad_io`**.

Litmus test: *"Could this run in a headless test with no egui?"* If yes and it's
feature-specific, it belongs in the feature crate, not `cad_app`.

---

## 4. Adding a feature — workflow

1. **Data**: add the type + `Geom` variant + match arms in `cad_kernel`.
2. **Logic**: put the algorithm in the feature crate; it takes kernel types and
   returns plain results / render primitives (no egui).
3. **UI**: in `cad_app`, paint the primitives the feature crate returns; add the
   dialog + tool/command flow.
4. **IO**: add serialization in `cad_io` (RSM tag / DXF codes).

---

## 5. Migration status (today → target)

- [x] **`cad_wall`** — scaffolded. `solve_faces` (the junction solver) moved out
  of `cad_app/src/wall.rs` into the `cad_wall` lib; `cad_app` depends on it and
  calls `cad_wall::solve_faces`. Wall *data* (`Wall`, `WallStyle`, `bulge_arc`)
  stays in `cad_kernel`.
- [ ] **`cad_dim`** — move `dim_render_geometry` (+ formatting) out of `cad_app`.
- [ ] **`cad_text`** — move text layout helpers out of `cad_app`.
- [ ] Decide whether to retire or wire in `cad_snap` (currently orphaned).

Each step compiles green on its own: feature crates only *add* a dependency
edge; `cad_kernel` re-exports keep `cad_io`/`cad_app` source mostly unchanged.
