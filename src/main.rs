use std::path::Path;
use std::{fs, io};

use druid::im::vector;
use druid::piet::Text;
use druid::text::TextInput;
use druid::widget::{
    prelude::*, Align, Button, Container, Label, LabelText, Padding, Scroll, Slider, Split, TextBox,
};
use druid::widget::{CrossAxisAlignment, List};
use druid::widget::{Flex, ProgressBar};
use druid::{
    commands, AppDelegate, Color, Command, DelegateCtx, FileDialogOptions, Handled,
    LocalizedString, MenuDesc, MenuItem, SysMods, Target, TextAlignment, WidgetExt,
};
use druid::{im::Vector, AppLauncher, Data, Lens, Widget, WindowDesc};
use gostd::path;

fn main() {
    let win = WindowDesc::new(ui_builder)
        .menu(make_menu())
        .title("Flac Music")
        .window_size((1200., 600.))
        .show_titlebar(true);
    let s = Song {
        title: "以父之名".to_owned(),
        album: "叶惠美".to_owned(),
        artist: "周杰伦".to_owned(),
        date: "2003".to_owned(),
        ..Default::default()
    };
    let initState = AppState {
        app_status: Status::Stop,
        play_lists: Vector::new(),
        current_song: Current::default(),
        volume: 30.,
        progress_rate: 0.5,
        play_mode: Modes::Order,
        current_play_list: vector![
            s.clone(),
            s.clone(),
            s.clone(),
            s.clone(),
            s.clone(),
            s.clone(),
            s.clone(),
            s.clone(),
            s.clone(),
            s.clone(),
            s.clone(),
            s.clone(),
            s.clone(),
            s.clone(),
            s.clone(),
            s.clone(),
            s.clone(),
            s.clone(),
            s
        ],
        search_text: "search".into(),
        music_dir: "".to_owned(),
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
            // let paths = path
            //     .display()
            //     .to_string()
            //     .as_str()
            //     .split("/")
            //     .map(|x| x.to_string())
            //     .collect::<Vec<String>>();
            // let length = paths.len();
            // for v in paths[0_usize..(length - 1_usize)].iter() {
            //     data.music_dir.push_str(v);
            //     data.music_dir.push_str("/");
            // }
            data.music_dir = path.display().to_string();
            println!("{}", data.music_dir);
            load_files(&data.music_dir);
            return Handled::Yes;
        }
        Handled::No
    }
}
fn load_files(dir: &str) -> Result<Vec<String>, io::Error> {
    let dir = Path::new(dir);
    let mut files:Vec<String> = fs::read_dir(dir).ok().unwrap()
        .map(|res| res.ok().map(|e| e.path().display().to_string())).into_iter().map(|x|x.unwrap()).collect();
    files.sort();
    for i in &files{
    println!("list: {}",i.as_str());
    }
    Ok(files)
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

// fn make_menu<T: Data>() -> MenuDesc<T> {
//     let base = MenuDesc::new(LocalizedString::new(""))
//         .append(
//             MenuItem::new(
//                 LocalizedString::new("common-menu-file-open"),
//                 commands::SHOW_OPEN_PANEL.with(FileDialogOptions::default()),
//             )
//             .hotkey(SysMods::Cmd, "o"),
//         )
//         .append(MenuItem::new(
//             LocalizedString::new("macos-menu-about-app"),
//             commands::SHOW_ABOUT,
//         ));
//     base
//     // MenuDesc::new(LocalizedString::new("macos-menu-application-menu"))
//     //     .append(MenuItem::new(
//     //         LocalizedString::new("macos-menu-about-app"),
//     //         commands::SHOW_ABOUT,
//     //     ))
//     //     .append(
//     //         MenuItem::new(
//     //             LocalizedString::new("macos-menu-quit-app"),
//     //             commands::QUIT_APP,
//     //         )
//     //         .hotkey(SysMods::Cmd, "q"),
//     //     )
//     //     .append()
// }

fn ui_builder() -> impl Widget<AppState> {
    let vol = Flex::row()
        .with_child(Label::new("Volume"))
        .with_child(Slider::new().with_range(1.0, 100.).lens(AppState::volume))
        .align_right()
        .padding(10.0);
    let SearchText = TextBox::new()
        .with_text_alignment(TextAlignment::Center)
        .padding(10.0)
        .lens(AppState::search_text);
    let ContrlTab = Container::new(
        Flex::row()
            .with_child(Button::new("|<<"))
            .with_default_spacer()
            .with_child(Button::new("Play"))
            .with_default_spacer()
            .with_child(Button::new("Pause"))
            .with_default_spacer()
            .with_child(Button::new("Stop"))
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
        .with_child(Flex::row().with_child(SearchText).with_child(ContrlTab))
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

    let playList = Scroll::new(
        Flex::column()
            // .with_child(header)
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
        Split::rows(playlab, Split::rows(header, playList).split_point(0.05)).split_point(0.1),
    )
}
#[derive(Data, Lens, Clone)]
struct AppState {
    music_dir: String,
    app_status: Status,
    play_lists: Vector<PlayList>,
    current_song: Current,
    progress_rate: f64,
    current_play_list: Vector<Song>,
    volume: f64,
    play_mode: Modes,
    search_text: String,
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

#[derive(Data, Lens, Default, Clone)]
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
        .with_child(Label::dynamic(|d: &Song, _| d.title.to_owned()).fix_width(120.0))
        .with_spacer(100.0)
        .with_child(Label::dynamic(|d: &Song, _| d.album.to_owned()).fix_width(120.0))
        .with_spacer(100.0)
        .with_child(Label::dynamic(|d: &Song, _| d.artist.to_owned()).fix_width(120.0))
        .with_spacer(100.0)
        .with_child(Label::dynamic(|d: &Song, _| d.date.to_owned()).fix_width(120.0))
        .with_spacer(100.0)
        .with_child(Label::dynamic(|d: &Song, _| d.duration.to_string()).fix_width(120.0))
        .with_spacer(100.0)
}
