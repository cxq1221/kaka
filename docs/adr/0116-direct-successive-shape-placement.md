# ADR 0116: Direct successive shape placement

Status: Accepted

## Context

Creating a rectangle, line, or arrow also selects the new mark. Under ADR 0092,
the next blank-canvas drag first cleared that automatic selection, so drawing a
second shape needed two drags. This made repeated shape placement feel broken
even though the first shape remained correct.

## Decision

- With Rectangle, Line, or Arrow active, a blank-canvas drag creates the next
  mark on that same gesture, including when the previous mark is still selected.
- A blank click without movement only clears selection and creates no mark.
  Existing-object body drags and resize handles retain their direct-editing
  behavior.
- Text, Numbered callout, and Label bubble retain ADR 0092's explicit
  finish-or-deselect gesture before placement. Pen and mosaic retain continuous
  drawing.
- Shape tools remain active after placement, and each completed shape remains
  selected so it can be moved or restyled immediately.

This supersedes ADR 0092 only for the blank-canvas placement behavior of
Rectangle, Line, and Arrow.

## Verification

Start with and without an existing selection, then create two consecutive marks
with each shape tool in one drag per mark. Confirm that a blank click only
deselects, that selected-object drags still move the object, that inline text
finishing still consumes its first gesture, and that undo/redo and save/reopen
preserve both shapes. Exercise the behavior in the capture overlay and the
saved-image editor.
