# ADR 0117: Compact capture toolbar and edge resize affordance

- Status: Accepted
- Date: 2026-10-23

## Context

The screenshot overlay exposed nearly every annotation and capture action in
one wide row. On a typical display the row no longer matched the selection's
visual hierarchy, and its dark treatment competed with the blue capture
selection. The eight visible resize rings were also the only reliable hit
targets: moving just inside or outside an edge left the cursor as a crosshair,
so edge resizing was hard to discover.

Flameshot's selection widget provides useful mainstream evidence for the
interaction: corner zones take priority over edge zones, horizontal edges use a
horizontal resize cursor, vertical edges use a vertical resize cursor, and
diagonals use the matching diagonal cursor. The capture region remains
movable from its center.

## Decision

- Draw the adjustable screenshot selection with a single crisp blue edge and
  compact blue ring handles. Keep the window-hover outline monochrome.
- Put the compact white annotation toolbar below the selection, right-aligned
  to the selection edge when it fits, with a six-point gap. If that side has no
  room, use the existing above/side/inside fallbacks while preserving bounds
  and mode-selector avoidance.
- Keep Select, Rectangle, Arrow, Pen, Mosaic, Text, and Numbered callout in the
  primary row. Keep QR recognition, Undo, Share, Pin, Cancel, and the primary
  completion check directly available. Put Line, Label, Watermark, Redo, OCR,
  Scrolling Screenshot, and selection/tool options in a visible More menu.
- Preserve every existing tool shortcut and action; the More menu is an
  overflow rather than a capability removal. Keep keyboard-visible tool hints,
  toolbar drag behavior, and Escape handling.
- When a selection is committed, an eight-point band around each edge activates
  resizing. Twenty-point corner handles and exact ring hits retain priority.
  Corners show `nwse-resize` or `nesw-resize`; horizontal edges show
  `ew-resize`; vertical edges show `ns-resize`. The center shows a move cursor.
- Commit the final pointer-release position for both edge resizing and moving,
  not only the previous pointer-move frame.
- Once annotation begins, lock the screenshot selection itself. A selected
  area annotation instead uses the same eight-point edge hit band for resizing
  its own bounds, with directional resize cursors, while its selection state
  is shown only by the eight handles rather than a dashed bounding frame.
- Keep Escape, Return, moving from the center, exact-size controls, and all
  eight existing handle directions unchanged.

## Consequences

The screenshot toolbar remains compact without deleting lesser-used actions.
The committed capture selection and selected area annotations are both easier
to resize from ordinary pointer movement. Annotation resizing no longer adds a
dashed rectangle that can be mistaken for part of the exported image. Native
cursor appearance and hit-target behavior require packaged-app acceptance on
macOS in addition to geometry tests.
