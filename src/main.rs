use druid::im::vector;
use druid::image::Progress;
use druid::widget::{prelude::*, Button, Container, Label, Padding, Scroll, Slider, Split};
use druid::widget::{CrossAxisAlignment, List};
use druid::widget::{Flex, ProgressBar};
use druid::{
    commands, theme, AppDelegate, Color, Command, DelegateCtx, FileDialogOptions, Handled,
    LocalizedString, MenuDesc, MenuItem, SysMods, Target, WidgetExt,
};
use druid::{im::Vector, AppLauncher, Data, Lens, Widget, WindowDesc};
use ffmpeg_next as ffmpeg;
use rodio::{OutputStreamHandle, Source};
use std::fs;
use std::io::BufReader;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};

fn main() {
    let win = WindowDesc::new(ui_builder)
        .menu(make_menu())
        .title("Flac Music v0.2.0")
        .window_size((1200., 600.))
        .show_titlebar(true);

    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let play_sink = Arc::new(Mutex::new(rodio::Sink::try_new(&handle).unwrap()));
    let init_state = AppState {
        app_status: Arc::new(Mutex::new(Status::Stop)),
        play_lists: Vector::new(),
        current_song: Arc::new(Mutex::new(Song::default())),
        volume: 0.3,
        progress_rate: 0.5,
        play_mode: Modes::Order,
        current_play_list: vector![],
        music_dir: "".to_owned(),
        sink: play_sink,
        stream: Arc::new(handle),
    };
    let app = AppLauncher::with_window(win)
        .use_simple_logger()
        .configure_env(|env, _| {
            env.set(theme::WINDOW_BACKGROUND_COLOR, Color::WHITE);
            env.set(theme::LABEL_COLOR, Color::BLACK);
            env.set(theme::TEXT_SIZE_LARGE, 13.0);
            env.set(theme::BUTTON_LIGHT, Color::WHITE);
            env.set(theme::BUTTON_DARK, Color::WHITE);
            env.set(theme::BACKGROUND_DARK, Color::WHITE);
            env.set(theme::BACKGROUND_LIGHT, Color::WHITE);
        })
        .delegate(MenuDelegate)
        .launch(init_state);
}

struct MenuDelegate;

impl AppDelegate<AppState> for MenuDelegate {
    fn command(
        &mut self,
        ctx: &mut DelegateCtx,
        target: Target,
        cmd: &Command,
        data: &mut AppState,
        env: &Env,
    ) -> Handled {
        if let Some(e) = cmd.get(druid::commands::OPEN_FILE) {
            data.music_dir.clear();
            let path = e.path();
            data.music_dir = path.display().to_string();
            data.current_play_list.extend(load_files(&data.music_dir));
            data.current_play_list
                .sort_by(|left, right| left.album.cmp(&right.album));
            return Handled::Yes;
        }
        Handled::No
    }

    fn event(
        &mut self,
        ctx: &mut DelegateCtx,
        window_id: druid::WindowId,
        event: Event,
        data: &mut AppState,
        env: &Env,
    ) -> Option<Event> {
        Some(event)
    }

    fn window_added(
        &mut self,
        id: druid::WindowId,
        data: &mut AppState,
        env: &Env,
        ctx: &mut DelegateCtx,
    ) {
    }

    fn window_removed(
        &mut self,
        id: druid::WindowId,
        data: &mut AppState,
        env: &Env,
        ctx: &mut DelegateCtx,
    ) {
    }
}

fn load_files(dir: &str) -> Vector<Song> {
    let mut songs = vector![];
    let dir = Path::new(dir);
    // 读取当前目录下的音乐文件。
    let mut files: Vec<String> = fs::read_dir(dir)
        .ok()
        .unwrap()
        .map(|res| res.ok().map(|e| e.path().display().to_string()))
        .into_iter()
        .map(|x| x.unwrap())
        .filter(|x| is_music_file(x))
        .collect();

    // 读取目录下的子目录的音乐文件
    if let Ok(other_dirs) = fs::read_dir(dir) {
        for other in other_dirs {
            if let Ok(d) = other {
                if d.path().is_dir() {
                    fs::read_dir(d.path())
                        .ok()
                        .unwrap()
                        .map(|res| res.ok().map(|e| e.path().display().to_string()))
                        .into_iter()
                        .map(|x| x.unwrap())
                        .filter(|x| is_music_file(x))
                        .for_each(|f| songs.push_back(get_song_meta(&f)))
                }
            }
        }
    }

    files.sort();
    for i in &files {
        let s = get_song_meta(i);
        songs.push_back(s);
    }
    songs
}

fn get_song_meta(f: &str) -> Song {
    let mut song = Song::default();
    ffmpeg::init().unwrap();

    match ffmpeg::format::input(&Path::new(f)) {
        Ok(context) => {
            let mut is_has_title = false;
            for (k, v) in context.metadata().iter() {
                let k_lower = k.to_lowercase();
                match k_lower.as_str() {
                    "title" => {
                        song.title = v.to_string();
                        is_has_title = true
                    }
                    "album" => song.album = v.to_string(),
                    "artist" => song.artist = v.to_string(),
                    "date" => song.date = v.to_string(),
                    _ => {
                        if !is_has_title {
                            song.title = {
                                let split_strs: Vec<&str> = f.split("/").collect();
                                let mut name: String = split_strs.last().unwrap().to_string();
                                let music_exts: Vec<&str> = vec![".flac", ".mp3", ".wav", ".m4a"];
                                for ext in music_exts {
                                    name = name.trim_end_matches(ext).to_owned()
                                }
                                name
                            }
                        }
                    }
                }
            }
            song.duration =
                (context.duration() as f64 / f64::from(ffmpeg::ffi::AV_TIME_BASE)).round();
        }
        Err(error) => println!("error:{}", error),
    }

    song.file = f.to_string();
    song
}

fn is_music_file(f: &str) -> bool {
    let music_exts: Vec<&str> = vec![".flac", ".mp3", ".wav", ".m4a"];
    for x in &music_exts {
        if f.ends_with(x) {
            return true;
        }
    }
    return false;
}
fn make_menu<T: Data>() -> MenuDesc<T> {
    let mut base = MenuDesc::empty();
    #[cfg(target_os = "macos")]
    {
        // base = base.append(druid::platform_menus::mac::menu_bar());
        base = MenuDesc::empty()
            .append(
                MenuDesc::new(LocalizedString::new("flac-music-application-menu")).append(
                    MenuItem::new(LocalizedString::new("Quit Flac Music"), commands::QUIT_APP),
                ),
            )
            .append_separator()
            .append(
                MenuDesc::new(LocalizedString::new("File")).append(
                    MenuItem::new(
                        LocalizedString::new("Import"),
                        commands::SHOW_OPEN_PANEL
                            .with(FileDialogOptions::default().select_directories()),
                    )
                    .hotkey(SysMods::Cmd, "o"),
                ),
            )
            .append_separator()
    }
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        base = MenuDesc::empty()
            .append(
                MenuDesc::new(LocalizedString::new("flac-music-menu-file-menu")).append(
                    MenuItem::new(LocalizedString::new("Quit Flac Music"), commands::QUIT_APP),
                ),
            )
            .append_separator()
            .append(
                MenuDesc::new(LocalizedString::new("File")).append(
                    MenuItem::new(
                        LocalizedString::new("Import"),
                        commands::SHOW_OPEN_PANEL
                            .with(FileDialogOptions::default().select_directories()),
                    )
                    .hotkey(SysMods::Cmd, "o"),
                ),
            )
            .append_separator()
        // base = base.append(druid::platform_menus::win::file::default())
    }
    base
}

fn ui_builder() -> impl Widget<AppState> {
    // 音量大小调节控件
    let volume = Flex::row()
        .with_child(Label::new(LocalizedString::new("Volume")))
        .with_child(
            Slider::new()
                .with_range(0.0, 1.)
                .lens(AppState::volume)
                .on_click(|_ctx, data, _env| {
                    data.sink.lock().unwrap().set_volume(data.volume as f32);
                }),
        )
        .align_right()
        .padding(10.0);

    // 当天歌曲名称显示
    let current_song_title = Label::dynamic(|d: &AppState, _env| {
        let current = d.current_song.lock().unwrap();
        if current.playing {
            format!("{}   -   {}", current.title, current.artist)
        } else {
            "".to_owned()
        }
    })
    .with_text_size(12.0)
    .fix_width(80.);

    // 播放控制按钮
    let play_control = Container::new(
        Flex::row()
            .with_child(
                // 上一首按钮
                Button::new("|<<")
                    .lens(AppState::current_play_list)
                    .on_click(|_ctx, data, _env| {
                        // 通过点击操作主动更新，当前歌曲状态和播放列表标记。
                        if let Some(current) =
                            prev_song(data.play_mode.to_owned(), &mut data.current_play_list)
                        {
                            *data.current_song.lock().unwrap() = current;
                            data.sink.lock().unwrap().set_volume(data.volume as f32);
                            *data.app_status.lock().unwrap() = Status::Prev;
                        }
                    }),
            )
            .with_default_spacer()
            .with_child(
                // 播放按钮
                Button::new(LocalizedString::new("Play"))
                    .lens(AppState::current_play_list)
                    .on_click(|_ctx, data, _env| {
                        *data.app_status.lock().unwrap() = Status::Stop;
                        *data.app_status.lock().unwrap() = Status::Play;
                        if data.sink.lock().unwrap().is_paused() {
                            data.sink.lock().unwrap().play();
                        } else {
                            if data.sink.lock().unwrap().empty() {
                                *data.sink.lock().unwrap() =
                                    rodio::Sink::try_new(&data.stream).unwrap();
                                data.sink.lock().unwrap().set_volume(data.volume as f32);

                                let stream = data.stream.clone();
                                let play_sink = Arc::clone(&data.sink);
                                let mut play_list = data.current_play_list.clone();
                                let current_song = Arc::clone(&mut data.current_song);
                                let app_status = Arc::clone(&mut data.app_status);

                                // 启动单独进程进行进行播放列表内歌曲按顺序播放
                                spawn(move || {
                                    // count 已播放歌曲计数
                                    let mut count = 0;
                                    // 以歌曲数总数为最大数，有限循环播放。play_sink.len最大为1，播放完一首，当len==0时，再加入下一首歌曲。
                                    // 上一首，下一首，切歌操作，以app_status 的状态切换来控制。
                                    while count < play_list.len()
                                        && !app_status.lock().unwrap().same(&Status::Stop)
                                    {
                                        if play_sink.lock().unwrap().empty() {
                                            // 当前播放歌曲为空时，播放第一首歌曲，进入下一次循环。
                                            if current_song.lock().unwrap().title.is_empty() {
                                                let mut cur = play_list.get_mut(0).unwrap();
                                                cur.playing = true;
                                                *current_song.lock().unwrap() = cur.to_owned();
                                                add_paly_song(
                                                    &current_song.lock().unwrap().file,
                                                    &play_sink.lock().unwrap(),
                                                );
                                                continue;
                                            }
                                            let mut status = app_status.lock().unwrap();
                                            match *status {
                                                Status::Play => {
                                                    count += 1;
                                                    if let Some(mut cur) =
                                                        next_song(Modes::Order, &mut play_list)
                                                    {
                                                        cur.playing = true;
                                                        *current_song.lock().expect("lock error") =
                                                            cur;
                                                        add_paly_song(
                                                            &current_song.lock().unwrap().file,
                                                            &play_sink.lock().unwrap(),
                                                        );
                                                    }
                                                }

                                                Status::Stop => break,
                                                Status::Suspend => (),
                                                Status::Next => {
                                                    count += 1;
                                                    if let Some(mut cur) =
                                                        next_song(Modes::Order, &mut play_list)
                                                    {
                                                        cur.playing = true;
                                                        *current_song.lock().expect("lock error") =
                                                            cur;
                                                        add_paly_song(
                                                            &current_song.lock().unwrap().file,
                                                            &play_sink.lock().unwrap(),
                                                        );
                                                    }
                                                    *status = Status::Play;
                                                }
                                                Status::Prev => {
                                                    count -= 1;
                                                    if let Some(mut cur) =
                                                        prev_song(Modes::Order, &mut play_list)
                                                    {
                                                        cur.playing = true;
                                                        *current_song.lock().expect("lock error") =
                                                            cur;
                                                        add_paly_song(
                                                            &current_song.lock().unwrap().file,
                                                            &play_sink.lock().unwrap(),
                                                        );
                                                    }
                                                    *status = Status::Play;
                                                }
                                            }
                                        }
                                        if play_sink.lock().unwrap().len() == 1 {
                                            let status = app_status.lock().unwrap();
                                            match *status {
                                                Status::Play => (),

                                                Status::Stop => break,
                                                Status::Suspend => (),
                                                Status::Next => {
                                                    // rodio::sink stop后就无法重新播放，只能重新初始化rodio::Sink::try_new(&stream)。
                                                    play_sink.lock().unwrap().stop();
                                                    *play_sink.lock().unwrap() =
                                                        rodio::Sink::try_new(&stream).unwrap();
                                                }
                                                Status::Prev => {
                                                    // rodio::sink stop后就无法重新播放，只能重新初始化rodio::Sink::try_new(&stream)。
                                                    play_sink.lock().unwrap().stop();
                                                    *play_sink.lock().unwrap() =
                                                        rodio::Sink::try_new(&stream).unwrap();
                                                }
                                            }
                                        }
                                        sleep(std::time::Duration::from_secs(1));
                                    }
                                });
                            }
                        }
                    }),
            )
            .with_default_spacer()
            .with_child(
                // 暂停按钮
                Button::new(LocalizedString::new("Pause"))
                    .lens(AppState::current_play_list)
                    .on_click(|_ctx, data, _env| {
                        data.sink.lock().unwrap().pause();
                        *data.app_status.lock().unwrap() = Status::Suspend;
                    }),
            )
            .with_default_spacer()
            .with_child(
                // 停止按钮
                Button::new(LocalizedString::new("Stop"))
                    .lens(AppState::current_play_list)
                    .on_click(|_ctx, data, _env| {
                        data.sink.lock().unwrap().stop();
                        *data.app_status.lock().unwrap() = Status::Stop;
                    }),
            )
            .with_default_spacer()
            .with_child(
                // 下一首按钮
                Button::new(">>|")
                    .lens(AppState::current_play_list)
                    .on_click(|_ctx, data, _env| {
                        // 通过点击事件,主动更新当前歌曲状态和播放列表标记。
                        if let Some(current) =
                            next_song(data.play_mode.to_owned(), &mut data.current_play_list)
                        {
                            *data.current_song.lock().unwrap() = current;
                            data.sink.lock().unwrap().set_volume(data.volume as f32);
                            *data.app_status.lock().unwrap() = Status::Next;
                        }
                    }),
            ),
    )
    .align_left();

    // 播放面板
    let play_panel = Flex::column()
        .with_child(
            Flex::row()
                .with_child(play_control)
                .with_spacer(30.0)
                .with_child(current_song_title)
                .with_spacer(150.0)
                .with_child(volume),
        )
        .with_default_spacer()
        .cross_axis_alignment(CrossAxisAlignment::Center);

    let play_list_header = vector![
        LocalizedString::new("Playing"),
        LocalizedString::new("Title"),
        LocalizedString::new("Album"),
        LocalizedString::new("Artist"),
        LocalizedString::new("Duration"),
        LocalizedString::new("Date"),
    ];

    let mut header: Flex<AppState> = Flex::row()
        .with_default_spacer()
        .with_child(Label::new(LocalizedString::new("Playing")))
        .with_spacer(70.0);

    for label in play_list_header.iter().skip(1) {
        header.add_child(Label::new(label.to_owned()));
        header.add_spacer(180.0);
    }

    // 播放列表
    let play_list = Scroll::new(
        Flex::column()
            .with_default_spacer()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .with_flex_child(
                Scroll::new(List::new(make_item).lens(AppState::current_play_list)).vertical(),
                1.0,
            )
            .expand_width(),
    )
    .vertical();

    // 组合完整UI
    Container::new(
        Split::rows(
            play_panel.padding(10.),
            Split::rows(header, play_list).split_point(0.05),
        )
        .split_point(0.1),
    )
    .on_click(|ctx, data, _env| {
        // 同步当前歌曲，到列表同步显示正在播放的箭头(目前要点击窗口才能更新,待优化)
        for v in data.current_play_list.iter_mut() {
            if v.title.same(&data.current_song.lock().unwrap().title) {
                v.playing = true;
            } else {
                v.playing = false;
            }
        }
        ctx.children_changed();
    })
}

#[derive(Data, Lens, Clone)]
struct AppState {
    music_dir: String,
    app_status: Arc<Mutex<Status>>,
    play_lists: Vector<PlayList>,
    current_song: Arc<Mutex<Song>>,
    sink: Arc<Mutex<rodio::Sink>>,
    progress_rate: f64,
    current_play_list: Vector<Song>,
    volume: f64,
    play_mode: Modes,
    stream: Arc<OutputStreamHandle>,
}

#[derive(Clone, Data, PartialEq, Debug)]
enum Status {
    Play,
    Suspend,
    Stop,
    Next,
    Prev,
}

#[derive(Data, Lens, Default, Clone)]
struct PlayList {
    name: String,
    songs: Vector<Song>,
}

#[derive(Data, Lens, Default, Clone)]
struct Current {
    name: String,
    cover_image: String,
}

#[derive(Clone, Data, PartialEq)]
enum Modes {
    Order,
    Random,
    Repet,
}

#[derive(Data, Lens, Default, Clone, Debug)]
struct Song {
    title: String,
    artist: String,
    album: String,
    duration: f64,
    playing: bool,
    date: String,
    file: String,
}

fn make_item() -> impl Widget<Song> {
    Padding::new(
        5.0,
        Flex::row()
            .with_child(
                Label::dynamic(|d: &Song, _| {
                    if d.playing {
                        "|>".to_string()
                    } else {
                        "".to_owned()
                    }
                })
                .fix_width(80.0),
            )
            .with_spacer(50.0)
            .with_child(Label::dynamic(|d: &Song, _| d.title.to_owned()).fix_width(120.0))
            .with_spacer(100.0)
            .with_child(Label::dynamic(|d: &Song, _| d.album.to_owned()).fix_width(120.0))
            .with_spacer(100.0)
            .with_child(Label::dynamic(|d: &Song, _| d.artist.to_owned()).fix_width(120.0))
            .with_spacer(100.0)
            .with_child(Label::dynamic(|d: &Song, _| d.duration.to_string()).fix_width(120.0))
            .with_spacer(100.0)
            .with_child(Label::dynamic(|d: &Song, _| d.date.to_owned()).fix_width(120.0))
            .with_spacer(100.0),
    )
}

// 加入歌曲到音轨sink
fn add_paly_song<'a>(f: &'a str, sink: &'a rodio::Sink) {
    let file = std::fs::File::open(f).unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
    sink.append(source);
}

fn paly_song<'a>(f: &'a str, output: &'a Arc<OutputStreamHandle>) {
    let file = std::fs::File::open(f).unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
    output.play_raw(source.convert_samples()).unwrap();
}

// 获取上一首歌
fn prev_song(play_mode: Modes, play_list: &mut Vector<Song>) -> Option<Song> {
    match play_mode {
        _ => {
            let mut this_index: usize = 0;
            let mut prev_index: usize = 0;
            let max = play_list.len() - 1;
            for (k, v) in play_list.iter_mut().enumerate() {
                if v.playing == true {
                    this_index = k;
                    v.playing = false;
                }
            }
            if this_index == 0 {
                // prev_index = max;
                println!("已经是第一首歌曲!");
                return None;
                // play_list[prev_index].playing = true;
                // return play_list[max].to_owned();
            } else {
                prev_index = this_index - 1;
                play_list[prev_index].playing = true;
                return Some(play_list[prev_index].to_owned());
            }
        }
    }
}

// 获取下一首歌曲
fn next_song(play_mode: Modes, play_list: &mut Vector<Song>) -> Option<Song> {
    match play_mode {
        _ => {
            let mut this_index: usize = 0;
            let mut next_index: usize = 0;
            let max = play_list.len() - 1;
            for (k, v) in play_list.iter_mut().enumerate() {
                if v.playing == true {
                    this_index = k;
                    v.playing = false;
                }
            }
            if this_index == max {
                // next_index = 0;
                println!("已经是最后一首歌曲!");
                // play_list[next_index].playing = true;
                // return play_list[next_index].to_owned();
                return None;
            } else {
                next_index = this_index + 1;
                play_list[next_index].playing = true;
                return Some(play_list[next_index].to_owned());
            }
        }
    }
}
