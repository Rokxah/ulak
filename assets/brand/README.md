# The Ulak mark

A U whose two arms, the phone and the PC, meet at the bottom, with the
message they carry as an amber spark between them. *Ulak* is Old Turkish for
"messenger".

| File | What it is |
|---|---|
| `icon.svg` | The full-color app icon: night-blue tile, soft glow, sky rim |
| `icon-flat.svg` | The same without the glow and rim, for flat contexts |
| `icon-mono.svg` | The mark alone in white, for one-color uses (notifications, themed icons) |
| `icon-512.png` | The app icon at 512 × 512, as `mark.rs` draws it |

The apps draw the mark themselves from the same geometry:
`desktop/app/src/mark.rs` (the Windows icon, tray and notifications at every
size) and `android/app/src/main/res/drawable/ic_launcher_*.xml` and
`ic_notification.xml`.

Colors: tile `#102A43`, glow `#1E4E79`, rim `#5FB3F9`, phone arm `#F5F9FF`,
PC arm `#CFE3FA`, spark `#FFB547`.

Ulak is a fork of NectarLink and doesn't use its name or mark; see the
[trademark policy](../../TRADEMARKS.md).
