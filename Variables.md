# RUST_CAD — User‑Environment Variables Master Reference

This is the **canonical list** of every setting that lives (or will live)
in `cad_app::settings::UserEnv`. Every row tracks:

- **Short name** — cryptic 5–7 char identifier (`SpTGSZ` style). No
  underscores, mixed case. See memory file
  `feedback_rust_cad_settings_naming.md` for the convention.
- **Description** — plain English; what the UI label shows.
- **Status** — what state the wiring is in:
  - **● Active** — fully wired in code; the value affects behavior.
  - **◐ Planned** — defined in `UserEnv`, surfaced in the UI, but the
    feature it controls isn't built yet (one-line wire-up when it lands).
  - **○ Stub** — defined for forward-compat / pasting AutoCAD config
    files; we don't plan to implement.
  - **◌ Tentative** — kept in code but uncertain whether we need it.
    Revisit later; either promote to Planned or remove from `UserEnv`
    once we know.
- **Wired at** — for Active entries only, the code location.

> **Process rule**: any new behavior toggle or hardcoded constant the
> user might want to tune goes into `UserEnv` *first* (with a name added
> here), then gets wired. Anything we *don't* put here we'll forget. This
> file is the source of truth.

> **Source-list reconciliation**: AutoCAD SYSVAR lists arrive from many
> sources (Autodesk docs, blogs, forum dumps). When two sources spell the
> same setting differently — usually casing (`RtDsp` vs `RTDsp`) or the
> `Ent → Dob` rename rule (`EntMenu` → `DobMenu`) — we pick **one
> canonical short name** and add a note on the row listing the alternate
> spellings. The persisted file format (`~/.config/rust_cad/user_env.txt`)
> uses the canonical name; the parser should eventually accept aliases on
> read for forward-compat. Today, only canonical names are recognized.

---

## 1 — Display & Visual Feedback

| Short | Description | Status | Wired |
|-------|-------------|--------|-------|
| `AperBx` | Aperture box on/off | ○ Stub | — |
| `BkgPlt` | Background plotting on/off | ○ Stub (no plot subsystem) | — |
| `CrsACol` | Crossing‑selection area colour | ◐ Planned | (used today as hardcoded `Color32(120,230,120)` in app.rs window overlay) |
| `CrsHrS` | Crosshair size (screen %) | **● Active** | `app.rs` — crosshair render uses `env.CrsHrS as f32 / 100.0`; settings slider (default 5) |
| `DrDspM` | Dragging display during MOVE/COPY | ◐ Planned | (Move tool already shows ghost; will respect this flag once wired) |
| `GalVw` | Block gallery view on/off | ○ Stub (no blocks) | — |
| `HltSel` | Highlight selected objects | **● Active** | `app.rs` — `if env.HltSel` gates the selected-dobject highlight colour (default true) |
| `HpQckP` | Hatch quick preview on/off | ○ Stub (no hatch) | — |
| `ImgHlt` | Image frame highlight on/off | ○ Stub (no raster) | — |
| `IntsCol` | Intersection marker colour | ◐ Planned | (currently hardcoded `Color32(255,90,90)` in app.rs intersection render) |
| `IntsDsp` | Intersection marker display | ◐ Planned | (currently always shown when computed) |
| `LnFade` | Line fading in edit mode | ○ Stub | — |
| `LtGlyD` | Light glyph display | ○ Stub (no lights) | — |
| `LyLkFd` | Locked-layer fade percentage | ◐ Planned (needs layers) | — |
| `MTxtFx` | Mtext fixed-width editor on/off | ○ Stub (no text) | — |
| `OleHid` | Hide OLE objects on/off | ○ Stub | — |
| `PcBnd` | Point-cloud bounding-box display | ○ Stub (2D-only) | — |
| `PcClpF` | Point-cloud clip frame display | ○ Stub (2D-only) | — |
| `PrvFlt` | Preview filter for commands | ○ Stub | — |
| `RllTp` | Tooltips on dobject rollover | ◐ Planned | — |
| `RvClCrM` | Revcloud creation mode | ○ Stub (no revcloud) | — |
| `RvClGrp` | Revcloud grip display | ○ Stub | — |
| `SelAr` | Selection area effect | ◐ Planned | (window/crossing rect overlay already shown) |
| `SelPrv` | Preview highlight of selection | **● Active** | `app.rs` — `if env.SelPrv` gates cursor-over hover preview (default true) |
| `SelPrvL` | Selection preview dobject limit | ◐ Planned | — |
| `TrkPth` | Tracking path display mode | ○ Stub | — |
| `TrnDsp` | Object transparency display | ○ Stub (no per-dobject alpha yet) | — |
| `TryIco` | Tray icon display | ○ Stub (no tray) | — |
| `TryTim` | Tray notification timeout | ○ Stub | — |
| `WinACol` | Window-selection area colour | ◐ Planned | (hardcoded `Color32(120,170,255)` in select overlay) |
| `WmfBkg` | WMF background colour | ○ Stub (no WMF) | — |
| `WmfFrg` | WMF foreground colour | ○ Stub | — |
| `WpFrmM` | Frame display of wipeouts | ○ Stub (no wipeouts) | — |
| `LodAnc` | APX (user-toggled draft display) dot-anchor strategy: 0=bbox center, 1=primitive center, 2=first vertex. APX mode is toggled by a status-bar button | **● Active** | `app.rs` — APX draft render anchor (default 0) |

## 2 — Selection & Grips

| Short | Description | Status | Wired |
|-------|-------------|--------|-------|
| `GrClrS` | Selected (hot) grip colour | **● Active** | `app.rs draw_grips` — `env.GrClrS` (default 0xFF6464) |
| `GrClrU` | Unselected grip colour | **● Active** | `app.rs draw_grips` — `env.GrClrU` (default 0x4099FF) |
| `GrpBlk` | Show grips inside blocks | ○ Stub (no blocks) | — |
| `GrpEnb` | Enable/disable grips | **● Active** | `app.rs` — toolbar button, `Command::GripsToggle`, `if self.env.GrpEnb` in render loop |
| `GrpObjL` | Maximum dobjects for grip display | ◐ Planned | — |
| `GrpSz` | Grip size (pixels) | **● Active** | `app.rs draw_grips` — `env.GrpSz` (default 4) |
| `GrpTip` | Grip hover tooltips on/off | ○ Stub | — |
| `HidTxt` | Hide text during move/rotate | ○ Stub (no text) | — |
| `ObjIsoM` | Object isolation mode | ○ Stub | — |
| `OsnNdLg` | Osnap node legacy mode | ○ Stub | — |
| `OsnOpt` | Object snap options | ◐ Planned (overlaps `snap_enabled` bitset) | — |
| `PkAdd` | Selection add mode | ◐ Planned (overlaps `select_remove_mode`) | — |
| `PkAuto` | Implied window selection | ◐ Planned | — |
| `PkDrag` | Selection by dragging | ◐ Planned | — |
| `PkFrst` | Noun/verb selection | ◐ Planned | — |
| `SelCyc` | Selection cycling on/off | ◐ Planned (relates to Tab cycling) | — |
| `SelOfSc` | Select off-screen dobjects | ◐ Planned | — |
| `SubSelM` | Subobject selection mode | ○ Stub (no subobjects) | — |
| `GrpHvR` | Grip hover + grab radius (pixels) — within this distance a grip highlights and a click/drag grabs it | **● Active** | `app.rs` — grip hover highlight + grab tolerance (default 25) |
| `SelDmTm` | Selection-drag activation hold time (ms) — a press becomes a window-drag only after being held this long; a faster press-drag is a click | **● Active** | `app.rs` click/drag classifier — the hold gate (default 250); see `CLICK_DRAG_HANDLER.md` |

## 3 — Object Snaps, Tracking & Precision

| Short | Description | Status | Wired |
|-------|-------------|--------|-------|
| `OsnCrd` | Osnap coordinate override keyboard | ◐ Planned | — |
| `PkBxSz` | Pickbox height (pixels) — click hit-test tolerance | **● Active** | `app.rs` — hit-test tolerance in selection / nearest-entity picks (default 10) |
| `PolAdA` | Polar additional angles | ◐ Planned (needs polar tracking) | — |
| `PolAng` | Polar angle setting | ◐ Planned | — |
| `PolDst` | Polar snap distance | ◐ Planned | — |
| `PolMod` | Polar tracking mode | ◐ Planned | — |
| `SpTGSZ` | Object-snap target height (pixels) | **● Active** | `app.rs` — `world_radius = env.SpTGSZ as f64 / scale` in `find_all_snaps` call |
| `TmpOvr` | Temporary override keys | ◐ Planned | — |
| `GrdEnb` | Background grid display (AutoCAD `GRIDMODE`) — F7 toggles | **● Active** | `app.rs` — grid overlay render gate (default on) |
| `GrdSnp` | Snap cursor to grid intersections (`SNAPMODE`) — F9 toggles; osnap wins over it | **● Active** | `app.rs` — rounds drafting coords to `GrdSpc` (default off) |
| `GrdSpc` | Grid spacing in world units (`GRIDUNIT`) — shared by display grid + snap rounding | **● Active** | `app.rs` — grid draw + snap rounding (default 10.0) |
| `CrdEnb` | **CARD** — cardinal-directions drafting lock (cursor H or V only from the anchor). AutoCAD `ORTHOMODE`. F8 / `card` cmd / status badge. Legacy key `OrtEnb` accepted on load. | **● Active** | `app.rs` — projects cursor onto the nearer axis (default off) |

## 4 — Editing & Behavior

| Short | Description | Status | Wired |
|-------|-------------|--------|-------|
| `AtDlgM` | Attribute entry dialog on INSERT | ◌ Tentative (in code, no attribute system yet — keep & revisit) | — |
| `AtPrmM` | Attribute prompting during INSERT | ◌ Tentative (in code, no attribute system yet — keep & revisit) | — |
| `BActBM` | Block action bar display mode | ○ Stub (no blocks) | — |
| `BlkEdLk` | Lock block editor from editing | ○ Stub | — |
| `BlkEdtr` | Block editor open/close state | ○ Stub | — |
| `BlkMrL` | Block MRU list length | ○ Stub | — |
| `BndTyp` | Xref bind type | ○ Stub (no xrefs) | — |
| `CmDlgM` | Dialog boxes for PLOT, etc. | ○ Stub (no plot) | — |
| `DblClkE` | Double-click editing on/off | ◐ Planned | — |
| `EdgMod` | Edge-mode for trim / extend. ON = treat cutting / boundary edges as their infinite extensions for "imaginary intersection" cuts; OFF = use only intersections on the visible curve. AutoCAD analog: `EDGEMODE` | **● Active** | `app.rs` — trim/extend honour `env.EdgMod` (default on) |
| `FltRad` | Default fillet radius (`FILLETRAD`); set inline `fillet <r>`, persists | **● Active** | `app.rs` fillet — `env.FltRad` (default 0.0) |
| `ChmDs1` | Default chamfer distance on the FIRST line (`CHAMFERA`) | **● Active** | `app.rs` chamfer — `env.ChmDs1` (default 0.0) |
| `ChmDs2` | Default chamfer distance on the SECOND line (`CHAMFERB`) | **● Active** | `app.rs` chamfer — `env.ChmDs2` (default 0.0) |
| `OfsDis` | Default offset distance (`OFFSETDIST`); set inline `offset <d>`, persists, bare `offset` reuses | **● Active** | `app.rs` offset — `env.OfsDis` (default 1.0) |
| `WlThk` | Default wall thickness for the `wall` command (±t/2 about the centerline); set inline `wall <t>` | **● Active** | `app.rs` wall — `env.WlThk` (default 0.20) |
| `TxHt` | Default text height (world units) for the `text` command | **● Active** | `app.rs` text — `env.TxHt` (default 0.25) |
| `WlCnL` | Wall centerline visible — dashed half-alpha overlay on every `Geom::Wall` | **● Active** | `app.rs` wall render — `if env.WlCnL` (default true) |
| `HpMaxA` | Maximum hatch area for preview | ○ Stub (no hatch) | — |
| `HpObjW` | Hatch dobject warning limit | ○ Stub | — |
| `HpSep` | Separate hatch dobjects on/off | ○ Stub | — |
| `InpHMd` | Dynamic input history display mode | ◐ Planned (needs prompt-driven cmd line) | — |
| `MTjigS` | Mtext sample string for jig | ○ Stub | — |
| `PedAcc` | Suppress PEDIT convert prompt | ○ Stub (no polyline) | — |
| `PrsPul` | Presspull behavior mode | ○ Stub (2D-only) | — |
| `RefPtTp` | Reference path type | ○ Stub (no xrefs) | — |
| `SavFid` | Save visual fidelity for annotative | ○ Stub | — |
| `SbyLyr` | SetByLayer mode | ◐ Planned (layer table now exists; needs UI command to bulk-set selection to ByLayer) | — |
| `SrfAsc` | Surface associativity | ○ Stub (no surfaces) | — |
| `TblInd` | Table cell indicator on/off | ○ Stub (no tables) | — |
| `TblTbr` | Table toolbar on/off | ○ Stub | — |
| `TrmMd` | Trim mode shared by Fillet/Chamfer (`TRIMMODE`). `true`=trim originals back to the corner; `false`=keep full-length + add the arc/bevel separately ("No Trim"). Toggle `t`/`nt` at the prompt. (Canonical code name `TrmMd`; `TrmMod` was the original catalog spelling.) | **● Active** | `app.rs` fillet/chamfer — `env.TrmMd` (default true) |
| `XEdit` | Edit in-place on/off | ○ Stub (no ref edit) | — |
| `XFdCtl` | Ref-edit object fading | ○ Stub | — |

## 5 — File & Save Management

| Short | Description | Status | Wired |
|-------|-------------|--------|-------|
| `AudCtl` | Create audit report file | ◐ Planned (needs save/load) | — |
| `AutoPub` | Automatic publish on save/close | ○ Stub (no publish) | — |
| `DgnMpP` | DGN mapping file path | ○ Stub (no DGN) | — |
| `DwgChk` | Check for non-Autodesk DWG files | ○ Stub (no DWG) | — |
| `IsvBak` | Incremental save backup creation | ◐ Planned | — |
| `IsvPrc` | Incremental save percentage | ◐ Planned | — |
| `LogFlM` | Log file on/off | ◐ Planned | — |
| `LogFlP` | Log file path | ◐ Planned | — |
| `OpnPrt` | Open partial DWG file | ○ Stub | — |
| `RcovMd` | Drawing recovery mode | ◐ Planned | — |
| `SavFP` | Automatic save file path | ◐ Planned | — |
| `SavTim` | Automatic save interval (minutes) | ◐ Planned | — |
| `SigWarn` | Digital signature warning | ○ Stub | — |
| `SldChk` | 3D solid validation on/off | ○ Stub (2D-only) | — |
| `TrstPth` | Trusted file paths | ○ Stub | — |

## 6 — External References (Xrefs) & Images

| Short | Description | Status | Wired |
|-------|-------------|--------|-------|
| `XrLdMd` | External-reference demand-loading | ◐ Planned (settings-only) | field exists + persisted + in UI; no xref runtime yet |
| `XrTmpP` | Path for temporary xref copies | ◐ Planned (settings-only) | field exists + persisted + in UI; no xref runtime yet |
| `XrCtl` | Xref log file on/off | ○ Stub | — |
| `XrLyr` | Default layer for xref insertion | ○ Stub | — |
| `XrNtfy` | Xref change notification | ○ Stub | — |
| `XrTyp` | Default xref type | ○ Stub | — |
| `XdwFd` | Xref drawing fade percentage | ○ Stub | — |
| `RastDpi` | Raster image DPI for plotting | ○ Stub | — |
| `RastPrc` | Raster image memory percentage | ○ Stub | — |
| `RastThr` | Raster image memory threshold | ○ Stub | — |
| `OleQlty` | OLE plot quality | ○ Stub | — |
| `OleStrt` | OLE application startup on load | ○ Stub | — |
| `PdfShx` | PDF SHX text handling | ○ Stub | — |
| `PdfShxL` | PDF SHX text layer | ○ Stub | — |

## 7 — User Interface & Workspace

| Short | Description | Status | Wired |
|-------|-------------|--------|-------|
| `DobMenu` | Enterprise CUI menu file (renamed from `EntMenu` to keep the `Ent→Dob` rule consistent — the original AutoCAD source name was `ENTERPRISEMENU`, the `Ent` was for *Enterprise* not *Entity*, but the rename was applied anyway to avoid the visual collision) | ○ Stub (won't implement) | — |
| `LokUI` | Lock toolbars/palettes position | ○ Stub (no movable palettes) | — |
| `MnuBar` | Display the classic menu bar | ◐ Planned | — |
| `MnuCtl` | Menu control (screen menu) | ○ Stub (legacy) | — |
| `NavBar` | Navigation bar display | ○ Stub | — |
| `NavCube` | ViewCube display | ○ Stub (2D-only) | — |
| `PalOpq` | Palette transparency | ○ Stub | — |
| `QpLoc` | Quick-properties location | ○ Stub (no quick-props panel) | — |
| `QpMod` | Quick-properties mode | ○ Stub | — |
| `RibSta` | Ribbon minimized state | ○ Stub (no ribbon) | — |
| `ScrnBx` | Screen menu boxes (legacy) | ○ Stub | — |
| `ShctMn` | Shortcut menu on/off | ◐ Planned | — |
| `StartUp` | Startup dialog mode | ◐ Planned | — |
| `TbCust` | Toolbar customize on/off | ○ Stub | — |
| `TltEnb` | Show toolbar/ribbon tooltips | ◐ Planned (egui has built-in) | — |
| `TltMrg` | Tooltip merge on/off | ○ Stub | — |
| `TltTrn` | Tooltip transparency | ○ Stub | — |
| `TpPalP` | Tool palette path | ○ Stub | — |
| `TxtEd` | Text editor application | ○ Stub | — |

## 8 — Plot & Publish

| Short | Description | Status | Wired |
|-------|-------------|--------|-------|
| `PapUpd` | Paper-size update warning | ○ Stub (no plot) | — |
| `PStPlc` | Plot style policy for new drawings | ○ Stub | — |
| `PubAll` | Publish all sheets | ○ Stub | — |
| `PubHch` | Publish hatch on/off | ○ Stub | — |

## 9 — System & Performance

| Short | Description | Status | Wired |
|-------|-------------|--------|-------|
| `FlDlgM` | Suppress file-navigation dialogs | ◐ Planned (settings-only) | field exists + persisted + in UI; will gate native dialogs when file I/O lands |
| `FntAlt` | Alternate font when font not found | ○ Stub (no text) | — |
| `FntMap` | Font mapping file path | ○ Stub | — |
| `LspAsD` | Load acad.lsp into every drawing | ○ Stub (no LISP) | — |
| `MxActVp` | Maximum active viewports | ○ Stub (no multi-vp) | — |
| `MxSort` | Maximum list sort size | ◐ Planned | — |
| `PrxNot` | Proxy dobject notice | ○ Stub | — |
| `PrxShw` | Proxy dobject display | ○ Stub | — |
| `PrxWeb` | Proxy web search on/off | ○ Stub | — |
| `StdViol` | Standards-violation notification | ○ Stub | — |
| `SysMon` | System-variable monitor on/off | ◐ Planned | — |
| `TreMax` | Tree memory limit | ◐ Planned (relates to spatial-index size cap) | — |
| `TxtFil` | Text fill on/off | ○ Stub (no text) | — |
| `TxtQlt` | Text quality | ○ Stub | — |
| `UntMod` | Unit display mode | ◐ Planned | — |
| `WhipArc` | Arc/circle smoothness | ◐ Planned (affects tessellation in `draw_dobject`) | — |

## 10 — View & Navigation

| Short | Description | Status | Wired |
|-------|-------------|--------|-------|
| `GeoLoc` | Geolocation marker visibility | ○ Stub | — |
| `LayTab` | Model/Layout tab display | ○ Stub (no layouts) | — |
| `RtDsp` | Real-time pan/zoom display (canonical; also seen as `RTDsp` in some source lists — both casings resolve to this row, AutoCAD source `RTDISPLAY`) | ◐ Planned | — |
| `StepSz` | Walk/fly step size | ○ Stub (no walk/fly) | — |
| `StpPrSc` | Walk/fly steps per second | ○ Stub | — |
| `SunPrW` | Sun properties window on/off | ○ Stub (no sun/3D) | — |
| `UcsOrt` | Orthographic UCS toggle | ○ Stub (2D-only) | — |
| `UcsIcn` | UCS origin-marker icon on/off (AutoCAD `UCSICON`) — origin dot + X/Y axis arrows | **● Active** | `app.rs` — UCS icon render (default on) |
| `UcsMod` | UCS icon placement: 0=bottom-left corner always; 1=anchor at world (0,0) when visible (`UCSICON ORigin`) | **● Active** | `app.rs` — icon placement (default 0) |
| `UcsAvP` | Avatar image path (PNG/SVG) drawn on the UCS icon X-axis; empty → placeholder rectangle | **● Active** | `app.rs` — UCS icon avatar; persisted (default empty) |
| `VtDur` | Smooth view transition duration | ◐ Planned | — |
| `VtEnbl` | Smooth view transition on/off | ◐ Planned | — |
| `VtFps` | Smooth view transition speed (FPS) | ◐ Planned | — |
| `VwUpdA` | View update automatic | ◐ Planned | — |
| `ZmFact` | Mouse wheel zoom factor | ◐ Planned (currently `0.0015` in scroll handler) | — |
| `ZmWhl` | Mouse wheel zoom direction | ◐ Planned | — |

## 11 — Miscellaneous / Other

| Short | Description | Status | Wired |
|-------|-------------|--------|-------|
| `Chrma` | Colour-book display mode | ○ Stub (no color books) | — |
| `LyrDlgM` | Layer properties manager mode | ◐ Planned (needs layer panel) | — |
| `LyrFlA` | Layer-filter alert on/off | ◐ Planned | — |
| `LyrNtf` | Layer notification on/off | ◐ Planned | — |
| `MTxtEd` | Multiline text editor application | ○ Stub (no text) | — |
| `PrjNam` | Project file search path | ◐ Planned | — |
| `SsmAuto` | Sheet Set Manager auto open | ○ Stub (won't implement) | — |
| `SsmPol` | Sheet Set Manager poll time | ○ Stub | — |
| `SsmSta` | Sheet Set Manager status | ○ Stub | — |

---

## 12 — RUST_CAD‑specific (not from AutoCAD)

These are settings RUST_CAD needs that don't have an AutoCAD equivalent —
they cover features we built that AutoCAD doesn't have (or has under a
different model): the spatial index, the snap framework, the GPU path.

| Short | Description | Status | Wired |
|-------|-------------|--------|-------|
| `GpuRnd` | Rendering path: 0=CPU, 1=GPU‑auto, 2=GPU‑forced | ◐ Planned (currently in debug window radio buttons) | — |
| `FpsDsp` | FPS overlay visibility | ◐ Planned (currently always on) | — |
| `IdxDsp` | Spatial‑index status overlay | ◐ Planned (currently always on) | — |
| `IdxCel` | Spatial-index target cells per dobject (auto vs override) | ◐ Planned (currently `auto_cell_size(.., 10.0)`) | — |
| `BgCol` | Canvas background colour | ◐ Planned (currently `Color32(18,22,28)`) | — |
| `SnpPri` | Snap priority order (RUST_CAD's 8 kinds, user-customisable) | ◐ Planned (currently hardcoded in `SnapKind::priority()`) | — |
| `SnpAct` | Default `SnapSet::defaults()` (which snaps are on at startup) | ◐ Planned | — |
| `TabCyc` | Tab cycling between snap candidates on/off | ◐ Planned (currently always on) | — |
| `CmdEcho` | Echo commands to history | ◐ Planned (currently always on) | — |
| `CmdHisM` | Command history retention size | ◐ Planned (currently unbounded `Vec<String>`) | — |
| `RubBnd` | Rubber-band style: solid / dashed / animated | ◐ Planned | — |
| `MvDdsp` | Move-tool ghost render style: ghost / outline / off | ◐ Planned (currently always ghost) | — |
| `RsmCmp` | `.rsm` save format: uncompressed / LZ4 / zstd | ◐ Planned (when file I/O lands) | — |
| `RsmBak` | Keep `.rsm.bak` on save | ◐ Planned | — |

---

## 13 — Code‑audit additions (hardcoded values today that should be settings)

Found by grepping `app.rs` and `gpu.rs`. Each is currently a magic number
or hex colour; should become a `UserEnv` field so the user controls it
without rebuilding.

| Short | Description | Current hardcoded value | Where |
|-------|-------------|-------------------------|-------|
| `DefDClr` | Default dobject (unselected) colour | `0xAAC8E6` (rgb 170,200,230) | `app.rs draw_dobject` paths |
| `SelClr` | Selected dobject highlight colour | `0xFFC850` (rgb 255,200,80) | `app.rs draw_dobject` |
| `SnpSrcClr` | Snap‑source entity highlight colour | `0x78F0FF` (rgb 120,240,255) | `app.rs draw_dobject` |
| `SnpClr` | Snap glyph + label colour | `0x50E6F0` (rgb 80,230,240) | `app.rs draw_snap_glyph` |
| `IntClr` | Intersection marker colour | `0xFF5A5A` (rgb 255,90,90) | `app.rs intersection render` |
| `ExtClr` | Imaginary‑extension dashed‑line base colour | `0xFFC85A` (rgb 255,200,90) | `app.rs draw_dashed_line/arc` |
| `PreClr` | Preview / rubber-band colour | `0xFFDC64` (rgb 255,220,100) | `app.rs preview blocks` |
| `ExtSpd` | Extension‑dash drift speed (px/sec) | `60.0` | `app.rs` |
| `ExtFade` | Extension‑dash alpha base (0.0–1.0) | `0.55` | `app.rs` |
| `ExtDshL` | Extension‑dash length (px) | `7.0` | `app.rs` |
| `ExtGapL` | Extension‑dash gap (px) | `4.0` | `app.rs` |
| `WinDshSpd` | Selection‑window dash drift speed | `40.0` | `app.rs select overlay` |
| `SelDshClr` | Selection-basket dashed overlay colour | `0xB4D2E6` (rgb 180,210,230) | `app.rs in_selection branch` |
| `SelDshW`   | Selection-basket dashed overlay stroke width (px) | `1.6` | `app.rs draw_dobject_dashed` |
| `SelDshL`   | Selection-basket dash length (px) | `6.0` | `app.rs in_selection branch` |
| `SelDshG`   | Selection-basket dash gap (px) | `4.0` | `app.rs in_selection branch` |
| `SelPlsMin` | Selection-basket pulse alpha min (0.0–1.0) | `0.15` | `app.rs pulse_alpha` (shared with trim/extend) |
| `SelPlsMax` | Selection-basket pulse alpha max (0.0–1.0) | `0.85` | `app.rs pulse_alpha` (shared with trim/extend) |
| `SelPlsHz`  | Selection-basket pulse frequency (cycles/sec) | `1.4` | `app.rs pulse` (shared with trim/extend) |
| `HitTolPx` | Hit-test tolerance in pixels (overlaps `PkBxSz`) | `10.0` | `app.rs nearest_entity_under` |
| `IntRad` | `∩ click` search radius in pixels | `50.0` | `app.rs intersect_pending_click` |
| `PairLim` | Maximum candidate pair count before ∩ refuses | `5_000_000` | `app.rs PAIR_LIMIT` |
| `TabCycR` | Cursor-move px before Tab cycle resets | `4.0` | `app.rs snap_cycle_anchor check` |
| `ArrCol` / `ArrRow` / `ArrDX` / `ArrDY` | Array dialog defaults | `10 / 10 / 50.0 / 50.0` | `app.rs CadApp Default` |
| `DfltZm` | Default zoom scale at app start | `6.0 px/u` | `app.rs CadApp Default` |
| `DemoOn` | Load demo dobjects on startup | `true` (always) | `app.rs CadApp::default()` |
| `GpuRgWd` | GPU circle ring thickness (multiples of aa) | `1.0 * aa` | `gpu.rs FS shader` |
| `TessCirc` | Circle CPU tessellation factor | `r_px * 0.5, clamp(8..256)` | `app.rs draw_dobject Circle` |
| `TessArc` | Arc CPU tessellation factor | `r_px * 0.5, clamp(8..256)` | `app.rs draw_dobject Arc` |
| `TessEll` | Ellipse tessellation factor | `r_px * 0.7, clamp(16..512)` | `app.rs draw_dobject Ellipse` |
| `TessEArc` | EllipseArc tessellation factor | `r_px * 0.7, clamp(12..512)` | `app.rs draw_dobject EllipseArc` |

---

## Process — how new settings get added

1. **Identify a new behavior toggle or hardcoded value.**
2. **Add a row here** in the right section. Pick a cryptic 5–7 char name
   following the convention; pick the right status badge.
3. **Add the field to `cad_app::settings::UserEnv`** with a doc comment
   that mirrors this table's description.
4. **Default value** matches the current hardcoded value (or AutoCAD's
   default), so behavior doesn't change unexpectedly when the field is
   introduced.
5. **Persist** the field by adding it to `UserEnv::save()` and the match
   arm in `UserEnv::set()`.
6. **Surface in the settings window** with an `env_bool` / `env_u8` /
   `env_color` / `env_text` / `env_u8_choice` widget.
7. **Wire the read site** — replace the hardcoded value with
   `self.env.<Field>`.
8. **Update this file**: status badge moves from `◐ Planned` to `● Active`
   and the "Wired" column gets the location.

The point of this file: every time we discover a new setting candidate,
it gets recorded here even before we have time to wire it. Never trust
human memory across sessions — trust the file.

---

## Currently Active wiring (the short list)

**Status snapshot — 2026-06-24 (commit `36ee804`).** `UserEnv` currently
declares **40 fields**. Of those:

- **28 ● Active** — read by behaviour code, not just the settings widget.
- **3 settings-only** (`XrLdMd`, `XrTmpP`, `FlDlgM`) — persisted + in the UI,
  but the subsystem they gate (xref / file-I/O) isn't built, so they don't yet
  change runtime behaviour. Treated as "awaiting wiring".
- **9 ◐ Planned widgets** (`AtDlgM`, `AtPrmM`, `CmDlgM`, `DrDspM`, `GrpBlk`,
  `MnuBar`, `RllTp`, `TltEnb`, `WpFrmM`) — defined + have a settings widget,
  feature not built.
- The **~150 other rows** above are catalogue-only (`◐`/`○`/`◌`) — named here
  for forward-compat but NOT yet fields in `UserEnv`.

The 28 ● Active fields, by family:

| Family | Fields |
|--------|--------|
| Snap / pick | `SpTGSZ`, `PkBxSz`, `CrsHrS` |
| Selection | `SelDmTm`, `SelPrv`, `HltSel` |
| Grips | `GrpEnb`, `GrpSz`, `GrpHvR`, `GrClrU`, `GrClrS` |
| Editing defaults | `FltRad`, `ChmDs1`, `ChmDs2`, `OfsDis`, `WlThk`, `TxHt`, `TrmMd`, `EdgMod` |
| Walls | `WlCnL` (+ `WlThk` above) |
| Grid / CARD | `GrdEnb`, `GrdSnp`, `GrdSpc`, `CrdEnb` |
| UCS icon | `UcsIcn`, `UcsMod`, `UcsAvP` |
| Draft display | `LodAnc` |

---

## How to regenerate this status (run the same)

This catalogue drifts as features land. **Do not hand-maintain the ●/◐ badges
from memory** — re-derive them from the code. Run these three from the repo
root; the union tells you the truth:

```bash
# 1. The fields that ACTUALLY EXIST in UserEnv right now:
sed -n '/pub struct UserEnv/,/^}/p' cad_app/src/settings.rs \
  | grep -oE 'pub [A-Za-z0-9_]+:' | sed 's/pub //; s/://'

# 2. The fields READ by behaviour code (count per field).
#    A field here with a count > its settings-widget reads is ● Active.
#    Exclude settings.rs (the UI) + the env.save/get/set/txt plumbing:
grep -rhoE 'env\.[A-Z][A-Za-z0-9_]+' \
    cad_app/src/app.rs cad_app/src/app/ cad_app/src/gpu.rs cad_kernel/src/ cad_wall/src/ \
  | grep -vE 'env\.(save|set|get|txt)' | sort | uniq -c | sort -rn

# 3. UserEnv fields that have NO row in this file yet (need adding):
for f in $(sed -n '/pub struct UserEnv/,/^}/p' cad_app/src/settings.rs \
            | grep -oE 'pub [A-Za-z0-9_]+:' | sed 's/pub //; s/://'); do
  grep -q "\`$f\`" Variables.md || echo "MISSING ROW: $f"
done
```

**Classification rule:** a field is **● Active** iff it is read somewhere other
than its own settings widget (check the file:line of each read with
`grep -rn 'env.<Field>' cad_app/src`). A field that only appears in the settings
window binding + `env.save()` is **◐ Planned** (defined + surfaced, not acting).
A name that isn't a `UserEnv` field at all is catalogue-only (`◐`/`○`/`◌`).

When you wire a planned field, follow §"Process — how new settings get added"
above, then flip its badge to ● Active here and fill the Wired column. When you
add a brand-new field, run command 3 to confirm it gets a row.
