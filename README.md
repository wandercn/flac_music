# flac_music
Music player based on druid GUI

# package app

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