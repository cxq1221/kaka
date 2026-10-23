<div align="center">
  <img src="src-tauri/icons/128x128.png" width="112" alt="Kaka">
  <h1>Kaka</h1>
  <p>截图、文字识别和录屏，素材默认保存在本机。</p>
  <p>支持 macOS、Windows 和 Linux。</p>
  <p>基于 <a href="https://github.com/yuxino/Kiri">Kiri</a> 的独立分支。</p>
  <p><a href="https://github.com/cxq1221/kaka">GitHub</a> · <a href="https://github.com/cxq1221/kaka/releases/latest">下载最新版本</a></p>
  <p><strong>简体中文</strong> · <a href="docs/readme/en.md">English</a> · <a href="docs/readme/zh-TW.md">繁體中文</a> · <a href="docs/readme/ja.md">日本語</a> · <a href="docs/readme/de.md">Deutsch</a> · <a href="docs/readme/ko.md">한국어</a> · <a href="docs/readme/fr.md">Français</a></p>
  <p>
    <a href="https://github.com/cxq1221/kaka/releases/latest"><img src="https://img.shields.io/github/v/release/cxq1221/kaka?style=flat&amp;logo=github&amp;logoColor=white" alt="Release"></a>
    <a href="https://github.com/cxq1221/kaka/actions/workflows/build.yml?query=branch%3Amain"><img src="https://img.shields.io/github/actions/workflow/status/cxq1221/kaka/build.yml?style=flat&amp;logo=githubactions&amp;logoColor=white&amp;branch=main&amp;event=push&amp;label=CI" alt="CI"></a>
    <a href="LICENSE"><img src="https://img.shields.io/github/license/cxq1221/kaka?style=flat&amp;logo=opensourceinitiative&amp;logoColor=white" alt="MIT"></a>
  </p>
</div>

![Kaka](docs/assets/readme-preview.png)

## 功能

- 截取窗口或区域，添加文字、箭头和马赛克，也能把截图置顶作参考。保存后可放大查看并继续编辑。
- 上下自由滚动并拼成长截图，返回已截内容不会重复拼接（macOS、Windows、Linux X11）。
- 用本地 OCR 提取图片里的文字，识别二维码。
- 录制 MP4，可选系统声音和麦克风；也可保存为无声 GIF。
- 裁切、重排同一视频的片段，导出新的 MP4。
- 搜索、打标签、收藏和导出本地素材，误删可从回收站恢复。
- 可在截图工具栏或完成卡片中显式分享到本机或可信局域网内的同频，生成聊天记录；不会自动上传。
- 在设置中直接录入、停用或恢复全局快捷键；录屏控制按键仅在录制时生效（[Wayland 由桌面管理](docs/linux.md)）。

## 开始使用

1. 按 `⇧⌘A`（macOS）或 `Shift+Ctrl+A`（Windows / Linux X11）打开捕获面板。
2. 选择截图、录屏或 OCR，点击窗口或拖出一个区域。
3. 截图按 `Enter` 完成，`Esc` 取消；完成后会复制到剪贴板并保存在素材库。

[使用与常见问题](docs/usage.zh-CN.md) · [视频剪辑](docs/video-editing.zh-CN.md) · [Linux](docs/linux.md) · [反馈问题](https://github.com/cxq1221/kaka/issues)

## 来源与许可

Kaka 是 [Kiri](https://github.com/yuxino/Kiri) 的独立分支，基于其 MIT 许可代码继续开发。原始版权声明完整保留在 [LICENSE](LICENSE) 中，修改与归属说明见 [NOTICE](NOTICE.md)。

## 感谢

感谢每一位写代码、提问题、试用和分享的朋友 (๑•̀ㅂ•́)و✧

特别感谢 [@kerwin2046](https://github.com/kerwin2046) 提供 [Linux 初始支持](https://github.com/yuxino/kiri/pull/20)，以及 [@LLLin000](https://github.com/LLLin000) 修复 [Windows 文字缩放下的截图对齐](https://github.com/yuxino/kiri/pull/61)。

也感谢 [V2EX](https://www.v2ex.com/)、[LINUX DO](https://linux.do/)、[Appinn](https://www.appinn.com/)、[NodeLoc](https://www.nodeloc.com/) 社区朋友的反馈与分享。

[所有贡献者](https://github.com/cxq1221/kaka/graphs/contributors) · [贡献指南](CONTRIBUTING.md) · [隐私说明](docs/privacy/zh-CN.md) · [MIT](LICENSE) · [NOTICE](NOTICE.md) © 2026 cxq1221
