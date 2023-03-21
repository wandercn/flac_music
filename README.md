# [flac_music](https://github.com/wandercn/flac_music)

[![crates.io](https://img.shields.io/crates/v/flac_music.svg?color=yellow)](https://crates.io/crates/flac_music)
[![GPL3 licensed](https://img.shields.io/github/license/wandercn/flac_music.svg)](./LICENSE)
[![Downloads of Crates.io](https://img.shields.io/crates/d/flac_music.svg)](https://crates.io/crates/flac_music)
[![Lines of code](https://img.shields.io/tokei/lines/github/wandercn/flac_music.svg)](#)
[![Build](https://img.shields.io/github/workflow/status/wandercn/flac_music/Rust.svg)](#)
[![Languages](https://img.shields.io/github/languages/top/wandercn/flac_music.svg)](#)
# flac_music
Music player based on druid GUI


# 特性

支持本地音乐文件 ".flac", ".mp3", ".wav", ".m4a" 格式的播放。

支持多次导入文件夹，添加音乐文件列表。

播放控制支持简单的，暂停，上一首，下一首等。

支持二级子目录扫描导入文件列表

本项目是用rust基于开源项目 druid 和 rodio创建。

本人对rust GUI项目 druid比较感兴趣，目前项目比较粗糙，还将继续优化。

# Todo

- [] 打包ffmpeg依赖库文件到 flac_music.app/Contents/Frameworks
 
- [ ] Pack universal APP,适配aarch64+x86_64

# test run app

`cargo run`

# pack app

### 1. MacOSX

cargo build -r

 ### Created flac_music.app


make app
```
Created 'flac_music.app' in 'target/release/macos'
xattr -c target/release/macos/flac_music.app/Contents/Info.plist
xattr -c target/release/macos/flac_music.app/Contents/Resources/flac_music.icns
```

### Packing disk image flac_music.dmg

make dmg
```
Packing disk image...
................................
created: target/release/macos/flac_music.dmg
Packed 'flac_music.app' in 'target/release/macos'
```

# Install

 linux (x86_64)

Install on Debian/Ubuntu x86_64
```
 dpkg -i ./flac-music_v0.2.2_amd64.deb
```
 linux (arm64)

Install on Debian/Ubuntu arm64
```
 dpkg -i ./flac-music_v0.2.2_arm64.deb
```
# FAQ

1. macOS系统限制，提示”提示文件已损坏”，处理方法。

sudo xattr -d com.apple.quarantine /Applications/xxxx.app，注意：/Applications/xxxx.app 换成你的App路径。指定放行，删除com.apple.quarantine元数据文件，使您可以执行可执行文件。
