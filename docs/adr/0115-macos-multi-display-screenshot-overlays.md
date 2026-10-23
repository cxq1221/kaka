# ADR 0115: macOS multi-display screenshot overlays

Status: Accepted

## Context

The screenshot flow previously froze and covered only the display under the
pointer. After a capture shortcut was invoked, moving the pointer to another
connected display reached the live desktop instead of an interactive overlay.
One combined virtual-desktop image would complicate mixed scale factors,
negative display origins, window hit testing, and coordinate conversion. It
would also delay selection while every display was composited into one large
image.

## Decision

- macOS freezes every connected display at capture startup. The display under
  the pointer is ordered first and remains the deterministic fallback for
  session-wide operations.
- Each frozen display owns one interactive overlay. A new capture closes all
  overlays from the previous capture before creating the next set.
- The active display continues to use the app-lifetime resident `overlay`
  window. Additional displays use `overlay-*` windows for the duration of the
  capture.
- Every display has its own opaque protocol token and frozen PNG. Capture
  startup reserves all tokens before encoding, publishes each image
  independently, and sends `capture-image-ready` only to the owning overlay.
- Capture-started, image-ready, window-bounds, and capture-ended listeners are
  scoped to the current WebView. A global event listener must not receive
  another display's token or geometry.
- Selection, confirmation, OCR, QR recognition, scrolling capture, recording,
  completion placement, and pin placement resolve the display from the overlay
  that initiated the action. The resident display is only a fallback for
  operations that are not initiated by one overlay.
- Escape and cancellation tear down every overlay in the capture session.

## Consequences

The pointer can cross display boundaries without escaping the capture UI, and
each display keeps its native scale and coordinate space. Startup encodes one
PNG per display instead of one total. A display that cannot be captured fails
the whole screenshot start rather than exposing a live desktop through a gap.

Windows and Linux retain their existing single-display flows. This decision
does not merge displays across Spaces, mirroring modes, or uncertain virtual
desktop layouts.

## Verification boundary

Unit-level checks cover protocol token isolation and capture-session state.
Native acceptance requires at least one internal Retina display and one
externally scaled display: both overlays must show the correct frozen image,
accept selection and annotation, confirm the correct display, and close
together on Escape. Automated browser emulation does not establish native
mixed-scale acceptance.
