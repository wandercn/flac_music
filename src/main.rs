use std::io::BufReader;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::thread::spawn;
use std::{fs, io};

use druid::im::vector;

use druid::piet::Text;
use druid::text::TextInput;
use druid::widget::{
    prelude::*, Align, Button, Container, Label, LabelText, ListIter, Padding, Scroll, Slider,
    Split, TextBox,
};
use druid::widget::{CrossAxisAlignment, List};
use druid::widget::{Flex, ProgressBar};
use druid::{
    commands, AppDelegate, Color, Command, DelegateCtx, FileDialogOptions, Handled,
    LocalizedString, MenuDesc, MenuItem, SysMods, Target, TextAlignment, WidgetExt,
};
use druid::{im::Vector, AppLauncher, Data, Lens, Widget, WindowDesc};
use ffmpeg_next as ffmpeg;
use rodio::OutputStreamHandle;
fn main() {
    let win = WindowDesc::new(ui_builder)
        .menu(make_menu())
        .title("Flac Music")
        .window_size((1200., 600.))
        .show_titlebar(true);

    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = Rc::new(rodio::Sink::try_new(&handle).unwrap());
    let initState = AppState {
        app_status: Status::Stop,
        play_lists: Vector::new(),
        current_song: Song::default(),
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
        .delegate(MenuDelegate)
        .launch(initState);
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
                MenuDesc::new(LocalizedString::new("文件")).append(
                    MenuItem::new(
                        LocalizedString::new("导入"),
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
                MenuDesc::new(LocalizedString::new("文件")).append(
                    MenuItem::new(
                        LocalizedString::new("导入"),
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
        .with_child(Label::new("Volume"))
        .with_child(
            Slider::new()
                .with_range(0.0, 1.)
                .lens(AppState::volume)
                .on_click(|_ctx, data, _env| {
                    data.sink.set_volume(data.volume as f32);
                    println!("音量大小: {}", data.volume);
                }),
        )
        .align_right()
        .padding(10.0);
    let search_text = TextBox::new()
        .with_text_alignment(TextAlignment::Center)
        .padding(10.0)
        .lens(AppState::search_text);
    let contrl_tab = Container::new(
        Flex::row()
            .with_child(Button::new("|<<"))
            .with_default_spacer()
            .with_child(Button::new("Play").lens(AppState::current_song).on_click(
                |_ctx, data, _env| {
                    if data.sink.is_paused() {
                        data.sink.play();
                    } else {
                        if data.sink.len() == 1 || data.sink.empty() {
                            println!("sink empty: {}", data.sink.len());
                            data.sink = Rc::new(rodio::Sink::try_new(&data.stream).unwrap());
                            data.sink.set_volume(data.volume as f32);
                            set_paly_song(&data.current_song.file, &mut data.sink)
                        } else {
                            data.sink.play();
                            println!("sink : {}", data.sink.len())
                        }
                    }
                    println!("playing: {}", data.current_song.title);
                },
            ))
            .with_default_spacer()
            .with_child(Button::new("Pause").lens(AppState::current_song).on_click(
                |_ctx, data, _env| {
                    data.sink.pause();
                },
            ))
            .with_default_spacer()
            .with_child(Button::new("Stop").lens(AppState::current_song).on_click(
                |_ctx, data, _env| {
                    data.sink.stop();
                },
            ))
            .with_default_spacer()
            .with_child(Button::new(">>|"))
            .with_default_spacer()
            .with_child(vol)
            .with_default_spacer()
            .padding(2.0)
            .center(),
    );

    let progress = Slider::new()
        .with_range(0.0, 100.0)
        .fix_width(800.0)
        .lens(AppState::progress_rate);

    let playlab = Flex::column()
        .with_child(Flex::row().with_child(search_text).with_child(contrl_tab))
        .with_default_spacer();
    let play_list_header = vector!["Playing", "Title", "Album", "Artist", "Date", "duration"];
    let mut header: Flex<AppState> = Flex::row()
        .with_default_spacer()
        .with_child(Label::new(play_list_header[0]))
        .with_spacer(70.0);
    for lab in play_list_header.iter().skip(1) {
        header.add_child(Label::new(*lab));
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
            Split::rows(header, play_list)
                .split_point(0.05)
                .on_click(|_ctx, data, _env| {
                    for x in data.current_play_list.iter_mut() {
                        if x.playing {
                            data.current_song = Song {
                                title: x.title.to_string(),
                                album: x.album.to_string(),
                                artist: x.artist.to_string(),
                                file: x.file.to_string(),
                                date: x.date.to_string(),
                                duration: x.duration,
                                playing: x.playing,
                            };

                            data.sink = Rc::new(rodio::Sink::try_new(&data.stream).unwrap());
                            data.sink.set_volume(data.volume as f32);
                            set_paly_song(&data.current_song.file, &mut data.sink);

                            x.playing = false;
                        }
                    }
                }),
        )
        .split_point(0.1),
    )
}

#[derive(Data, Lens, Clone)]
struct AppState {
    music_dir: String,
    app_status: Status,
    play_lists: Vector<PlayList>,
    current_song: Song,
    sink: Rc<rodio::Sink>,
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
                        "playing".to_owned()
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
                    .on_click(move |_ctx, data, _env| {
                        data.playing = true;
                    }),
            )
            .with_spacer(100.0)
            .with_child(Label::dynamic(|d: &Song, _| d.album.to_owned()).fix_width(120.0))
            .with_spacer(100.0)
            .with_child(Label::dynamic(|d: &Song, _| d.artist.to_owned()).fix_width(120.0))
            .with_spacer(100.0)
            .with_child(Label::dynamic(|d: &Song, _| d.date.to_owned()).fix_width(120.0))
            .with_spacer(100.0)
            .with_child(Label::dynamic(|d: &Song, _| d.duration.to_string()).fix_width(120.0))
            .with_spacer(100.0),
    )
}

fn set_paly_song(f: &str, sink: &mut Rc<rodio::Sink>) {
    let file = std::fs::File::open(f).unwrap();
    sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());
}
