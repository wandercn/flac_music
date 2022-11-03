# flac_music
Music player based on druid GUI


# 特性

支持本地音乐文件 ".flac", ".mp3", ".ogg", ".wav"格式的播放。

支持多次导入文件夹，添加音乐文件列表。

播放控制支持简单的，暂停，上一首，下一首等。

本项目是用rust基于开源项目 druid 和 rodio创建。

本人对rust GUI项目 druid比较感兴趣，目前项目比较粗糙，还将继续优化。


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