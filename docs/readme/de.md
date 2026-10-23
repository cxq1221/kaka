<div align="center">
  <img src="../../src-tauri/icons/128x128.png" width="112" alt="Kaka">
  <h1>Kaka</h1>
  <p>Screenshots, Texterkennung und Bildschirmaufnahmen. Ihre Aufnahmen bleiben lokal, sofern Sie sie nicht ausdrücklich teilen.</p>
  <p>Für macOS, Windows und Linux.</p>
  <p>Ein unabhängiger Fork von <a href="https://github.com/yuxino/Kiri">Kiri</a>.</p>
  <p><a href="https://github.com/cxq1221/kaka">GitHub</a> · <a href="https://github.com/cxq1221/kaka/releases/latest">Aktuelle Version herunterladen</a></p>
  <p><a href="../../README.md">简体中文</a> · <a href="en.md">English</a> · <a href="zh-TW.md">繁體中文</a> · <a href="ja.md">日本語</a> · <strong>Deutsch</strong> · <a href="ko.md">한국어</a> · <a href="fr.md">Français</a></p>
  <p>
    <a href="https://github.com/cxq1221/kaka/releases/latest"><img src="https://img.shields.io/github/v/release/cxq1221/kaka?style=flat&amp;logo=github&amp;logoColor=white" alt="Release"></a>
    <a href="https://github.com/cxq1221/kaka/actions/workflows/build.yml?query=branch%3Amain"><img src="https://img.shields.io/github/actions/workflow/status/cxq1221/kaka/build.yml?style=flat&amp;logo=githubactions&amp;logoColor=white&amp;branch=main&amp;event=push&amp;label=CI" alt="CI"></a>
    <a href="../../LICENSE"><img src="https://img.shields.io/github/license/cxq1221/kaka?style=flat&amp;logo=opensourceinitiative&amp;logoColor=white" alt="MIT"></a>
  </p>
</div>

![Kaka](../assets/readme-preview.png)

## Funktionen

- Fenster oder Bereiche aufnehmen, Text, Pfeile und Mosaik hinzufügen und Screenshots als schwebende Referenzen anheften. Nach dem Speichern vergrößern und weiterbearbeiten.
- Nach oben oder unten scrollen und einen langen Screenshot erstellen; bereits erfasste Bereiche werden nicht doppelt hinzugefügt (macOS, Windows, Linux X11).
- Text mit lokalem OCR aus Bildern kopieren und QR-Codes lesen.
- MP4 mit optionalem Systemton und Mikrofon aufnehmen oder als GIF ohne Ton speichern.
- Clips eines Videos kürzen und umordnen, dann eine neue MP4-Datei exportieren.
- Lokale Aufnahmen suchen, mit Tags versehen, favorisieren und exportieren; versehentlich Gelöschtes aus dem Papierkorb wiederherstellen.
- Einen Screenshot aus der Aufnahme-Symbolleiste oder der Abschlusskarte ausdrücklich an einen lokalen oder vertrauenswürdigen LanChat-Raum im LAN senden; er erscheint im Chatverlauf und wird nie automatisch hochgeladen.
- Globale Tastenkürzel direkt in den Einstellungen festlegen, deaktivieren oder zurücksetzen; Aufnahmesteuerungen gelten nur während einer Aufnahme ([Wayland nutzt Desktop-Einstellungen](../linux.md)).

## Erste Schritte

1. Drücken Sie `⇧⌘A` auf macOS oder `Shift+Ctrl+A` auf Windows / Linux X11, um die Aufnahme zu öffnen.
2. Wählen Sie Screenshot, Aufnahme oder OCR, dann ein Fenster oder einen aufgezogenen Bereich.
3. Bestätigen Sie Screenshots mit `Enter`; `Esc` bricht ab. Fertige Screenshots landen in der Zwischenablage und Bibliothek.

[Installation und Hilfe](../usage.de.md) · [Videobearbeitung](../video-editing.de.md) · [Linux](../linux.md) · [Fehler melden](https://github.com/cxq1221/kaka/issues)

## Herkunft und Lizenz

Kaka ist ein unabhängiger Fork von [Kiri](https://github.com/yuxino/Kiri), basierend auf dem MIT-lizenzierten Code. Der ursprüngliche Copyright-Hinweis bleibt in [LICENSE](../../LICENSE) erhalten; Hinweise zur Zuordnung stehen in [NOTICE](../../NOTICE.md).

## Danke

Vielen Dank an alle, die Code schreiben, Probleme melden, Kaka ausprobieren oder teilen.

Besonderer Dank gilt [@kerwin2046](https://github.com/kerwin2046) für die [erste Linux-Unterstützung](https://github.com/yuxino/kiri/pull/20) und [@LLLin000](https://github.com/LLLin000) für die [Korrektur der Screenshot-Ausrichtung bei Windows-Textskalierung](https://github.com/yuxino/kiri/pull/61).

Danke auch an die Communitys von [V2EX](https://www.v2ex.com/), [LINUX DO](https://linux.do/), [Appinn](https://www.appinn.com/), [NodeLoc](https://www.nodeloc.com/) für Rückmeldungen und Weiterempfehlungen.

[Alle Mitwirkenden](https://github.com/cxq1221/kaka/graphs/contributors) · [Mitwirken](../../CONTRIBUTING.md) · [Datenschutz](../privacy/en.md) · [MIT](../../LICENSE) · [NOTICE](../../NOTICE.md) © 2026 cxq1221
