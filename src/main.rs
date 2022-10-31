use std::io::BufReader;
use std::path::Path;
use std::sync::{self, Arc, Mutex};
use std::thread::{self, sleep, spawn};
use std::{fs, io};

use druid::im::vector;

use druid::widget::{
    prelude::*, Button, Container, Label, LabelText, Padding, Scroll, Slider, Split, TextBox,
};
use druid::widget::{CrossAxisAlignment, List};
use druid::widget::{Flex, ProgressBar};
use druid::{
    commands, theme, AppDelegate, Color, Command, DelegateCtx, FileDialogOptions, Handled, LensExt,
    LocalizedString, MenuDesc, MenuItem, SysMods, Target, WidgetExt,
};
use druid::{im::Vector, AppLauncher, Data, Lens, Widget, WindowDesc};

use ffmpeg::ffi::swscale_license;
use ffmpeg_next as ffmpeg;
use rodio::{OutputStreamHandle, Source};
fn main() {
    let win = WindowDesc::new(ui_builder)
        .menu(make_menu())
        .title("Flac Music")
        .window_size((1200., 600.))
        .show_titlebar(true);

    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = Arc::new(Mutex::new(rodio::Sink::try_new(&handle).unwrap()));
    let init_state = AppState {
        app_status: Status::Stop,
        play_lists: Vector::new(),
        current_song: Arc::new(Mutex::new(Song::default())),
        volume: 0.3,
        progress_rate: 0.5,
        play_mode: Modes::Order,
        current_play_list: vector![],
        search_text: "search".into(),
        music_dir: "".to_owned(),
        sink: sink,
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
            println!("file path: {:?}", path.display());
            data.music_dir = path.display().to_string();
            println!("{}", data.music_dir);
            data.current_play_list = load_files(&data.music_dir);
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
    let mut files: Vec<String> = fs::read_dir(dir)
        .ok()
        .unwrap()
        .map(|res| res.ok().map(|e| e.path().display().to_string()))
        .into_iter()
        .map(|x| x.unwrap())
        .filter(|x| is_music_file(x))
        .collect();
    files.sort();
    for i in &files {
        let s = get_song_meta(i);
        println!("song: {:?}", s);
        songs.push_back(s);
    }
    songs
}

fn get_song_meta(f: &str) -> Song {
    let mut song = Song::default();
    ffmpeg::init().unwrap();

    match ffmpeg::format::input(&Path::new(f)) {
        Ok(context) => {
            for (k, v) in context.metadata().iter() {
                let k_lower = k.to_lowercase();
                match k_lower.as_str() {
                    "title" => song.title = v.to_string(),
                    "album" => song.album = v.to_string(),
                    "artist" => song.artist = v.to_string(),
                    "date" => song.date = v.to_string(),
                    _ => (),
                }
            }
            song.duration =
                (context.duration() as f64 / f64::from(ffmpeg::ffi::AV_TIME_BASE)).round();
        }
        Err(error) => (),
    }

    song.file = f.to_string();
    song
}

fn is_music_file(f: &str) -> bool {
    let music_exts: Vec<&str> = vec![".flac", ".mp3", ".m4a", ".ogg", ".wav", ".ape"];
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
            .append_separator();
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
            .append_separator();
        // base = base.append(druid::platform_menus::win::file::default())
    }
    base
}

fn ui_builder() -> impl Widget<AppState> {
    let vol = Flex::row()
        .with_child(Label::new(LocalizedString::new("Volume")))
        .with_child(
            Slider::new()
                .with_range(0.0, 1.)
                .lens(AppState::volume)
                .on_click(|_ctx, data, _env| {
                    data.sink.lock().unwrap().set_volume(data.volume as f32);
                    println!("音量大小: {}", data.volume);
                }),
        )
        .align_right()
        .padding(10.0);

    let title_label = Label::dynamic(|d: &AppState, _env| {
        let current = d.current_song.lock().unwrap();
        if current.playing {
            format!("{}   -   {}", current.title, current.artist)
        } else {
            "".to_owned()
        }
    })
    .with_text_size(12.0)
    .fix_width(80.)
    .on_click(|ctx, data, env| {
        println!("1111:{}", data.current_song.lock().unwrap().title);
        ctx.children_changed()
    });

    let contrl_tab = Container::new(
        Flex::row()
            .with_child(
                Button::new("|<<")
                    .lens(AppState::current_play_list)
                    .on_click(|_ctx, data, _env| {
                        let current =
                            get_prev_one(data.play_mode.to_owned(), &mut data.current_play_list);
                        *data.current_song.lock().unwrap() = current;
                        // data.sink = Arc::new(rodio::Sink::try_new(&data.stream).unwrap());
                        data.sink.lock().unwrap().set_volume(data.volume as f32);
                        // set_paly_song(&data.current_song.lock().unwrap().file, &mut data.sink)
                    }),
            )
            .with_default_spacer()
            .with_child(
                Button::new(LocalizedString::new("Play"))
                    .lens(AppState::current_play_list)
                    .on_click(|_ctx, data, _env| {
                        // let (tx, rx) = mpsc::channel::<Song>();
                        // if data.current_song.lock().unwrap().title.is_empty() {
                        //     data.current_play_list[0].playing = true;
                        //     *data.current_song.lock().unwrap() = data.current_play_list[0].clone();
                        // }
                        if data.sink.lock().unwrap().is_paused() {
                            data.sink.lock().unwrap().play();
                        } else {
                            if data.sink.lock().unwrap().empty() {
                                println!("sink empty: {}", data.sink.lock().unwrap().len());
                                *data.sink.lock().unwrap() =
                                    rodio::Sink::try_new(&data.stream).unwrap();

                                data.sink.lock().unwrap().set_volume(data.volume as f32);

                                let stream = data.stream.clone();
                                // set_paly_song(&data.current_song.file, &mut data.sink);
                                let temp_sink = Arc::clone(&data.sink);
                                let mut list = data.current_play_list.clone();
                                let m = Arc::clone(&mut data.current_song);
                                let hand = spawn(move || {
                                    let mut count = 1;
                                    while count < list.len() {
                                        if temp_sink.lock().unwrap().empty() {
                                            count += 1;
                                            if let Some(mut cur) = list.pop_front() {
                                                println!("staring...");
                                                cur.playing = true;
                                                *m.lock().expect("lock error") = cur;
                                                set_paly_song(
                                                    &m.lock().unwrap().file,
                                                    &temp_sink.lock().unwrap(),
                                                );

                                                println!("add song: {}", m.lock().unwrap().title);
                                                // temp_sink.lock().unwrap().sleep_until_end();
                                            }
                                        }
                                    }
                                    // let mut is_end = false;

                                    // is_end = true;
                                    // while !temp_sink.lock().unwrap().empty() && is_end {
                                    //     println!("staring...");
                                    //     if temp_sink.lock().unwrap().len() == 1 {
                                    //         *m.lock().expect("lock error") =
                                    //             get_next_one(Modes::Order, &mut list);
                                    //         set_paly_song(
                                    //             &m.lock().unwrap().file,
                                    //             &temp_sink.lock().unwrap(),
                                    //         );
                                    //         println!("add song: {}", m.lock().unwrap().title);
                                    //         temp_sink.lock().unwrap().sleep_until_end();
                                    //     }
                                    //     sleep(std::time::Duration::from_secs(2));
                                    // }
                                    println!("ending");
                                });
                            }
                        }
                    }),
            )
            .with_default_spacer()
            .with_child(
                Button::new(LocalizedString::new("Pause"))
                    .lens(AppState::current_play_list)
                    .on_click(|_ctx, data, _env| {
                        data.sink.lock().unwrap().pause();
                    }),
            )
            .with_default_spacer()
            .with_child(
                Button::new(LocalizedString::new("Stop"))
                    .lens(AppState::current_play_list)
                    .on_click(|_ctx, data, _env| {
                        // data.sink.lock().unwrap().stop();
                        *data.sink.lock().unwrap() = rodio::Sink::try_new(&data.stream).unwrap();
                    }),
            )
            .with_default_spacer()
            .with_child(
                Button::new(">>|")
                    .lens(AppState::current_play_list)
                    .on_click(|_ctx, data, _env| {
                        let current =
                            get_next_one(data.play_mode.to_owned(), &mut data.current_play_list);
                        *data.current_song.lock().unwrap() = current;
                        // data.current_song = Arc::new(Mutex::new(current));
                        // data.sink = Arc::new(rodio::Sink::try_new(&data.stream).unwrap());
                        data.sink.lock().unwrap().set_volume(data.volume as f32);
                        // set_paly_song(&data.current_song.lock().unwrap().file, &mut data.sink)
                    }),
            ),
    )
    .align_left();

    let progress = Slider::new()
        .with_range(0.0, 100.0)
        .fix_width(800.0)
        .lens(AppState::progress_rate);

    let playlab = Flex::column()
        .with_child(
            Flex::row()
                .with_child(contrl_tab)
                .with_spacer(80.0)
                .with_child(title_label)
                .with_spacer(120.0)
                .with_child(vol),
        )
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
    for lab in play_list_header.iter().skip(1) {
        header.add_child(Label::new(lab.to_owned()));
        header.add_spacer(180.0);
    }

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

    Container::new(
        Split::rows(
            playlab,
            Split::rows(header, play_list).split_point(0.05), // .on_click(|_ctx, data, _env| {
                                                              //     for x in data.current_play_list.iter_mut() {
                                                              //         if x.playing {
                                                              //             *data.current_song.lock().unwrap() = Song {
                                                              //                 title: x.title.to_string(),
                                                              //                 album: x.album.to_string(),
                                                              //                 artist: x.artist.to_string(),
                                                              //                 file: x.file.to_string(),
                                                              //                 date: x.date.to_string(),
                                                              //                 duration: x.duration,
                                                              //                 playing: x.playing,
                                                              //             };

                                                              //             // data.sink = Arc::new(rodio::Sink::try_new(&data.stream).unwrap());
                                                              //             data.sink.lock().unwrap().set_volume(data.volume as f32);
                                                              //             // set_paly_song(&data.current_song.lock().unwrap().file, &mut data.sink);
                                                              //         }
                                                              //     }
                                                              // }),
        )
        .split_point(0.1),
    )
    .on_click(|ctx, data, _env| {
        for v in data.current_play_list.iter_mut() {
            if v.title.same(&data.current_song.lock().unwrap().title) {
                v.playing = true;
                println!("x: {} status: {}", v.title, v.playing);
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
    app_status: Status,
    play_lists: Vector<PlayList>,
    current_song: Arc<Mutex<Song>>,
    sink: Arc<Mutex<rodio::Sink>>,
    progress_rate: f64,
    current_play_list: Vector<Song>,
    volume: f64,
    play_mode: Modes,
    search_text: String,
    stream: Arc<OutputStreamHandle>,
}

#[derive(Clone, Data, PartialEq)]
enum Status {
    Play,
    Suspend,
    Stop,
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
            .with_child(
                Label::dynamic(|d: &Song, _| d.title.to_owned())
                    .fix_width(120.0)
                    .on_click(move |ctx, data, _env| {
                        data.playing = true;
                    }),
            )
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

fn set_paly_song<'a>(f: &'a str, sink: &'a rodio::Sink) {
    let file = std::fs::File::open(f).unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
    sink.append(source);
    if sink.empty() {
        println!("is stop");
    }
}

fn paly_song<'a>(f: &'a str, output: &'a Arc<OutputStreamHandle>) {
    let file = std::fs::File::open(f).unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
    output.play_raw(source.convert_samples()).unwrap();
}

fn get_prev_one(play_mode: Modes, play_list: &mut Vector<Song>) -> Song {
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
                prev_index = max;
                println!("已经是第一首歌曲!");
                play_list[prev_index].playing = true;
                return play_list[max].to_owned();
            } else {
                prev_index = this_index - 1;
                play_list[prev_index].playing = true;
                return play_list[prev_index].to_owned();
            }
        }
    }
}

fn get_next_one(play_mode: Modes, play_list: &mut Vector<Song>) -> Song {
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
                next_index = 0;
                println!("已经是最后一首歌曲!");
                play_list[next_index].playing = true;
                return play_list[next_index].to_owned();
            } else {
                next_index = this_index + 1;
                play_list[next_index].playing = true;
                return play_list[next_index].to_owned();
            }
        }
    }
}
