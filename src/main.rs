use druid::im::vector;
use druid::piet::Text;
use druid::text::TextInput;
use druid::widget::{
    prelude::*, Align, Button, Container, Label, LabelText, Padding, Scroll, Slider, Split, TextBox,
};
use druid::widget::{CrossAxisAlignment, List};
use druid::widget::{Flex, ProgressBar};
use druid::{im::Vector, AppLauncher, Data, Lens, Widget, WindowDesc};
use druid::{
    AppDelegate, Color, Command, DelegateCtx, Handled, MenuDesc, Target, TextAlignment, WidgetExt,
};

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
            let path = e.path();
            println!("file path: {:?}", path.display());
            return Handled::Yes;
        }
        Handled::No
    }
}

fn make_menu<T: Data>() -> MenuDesc<T> {
    let mut base = MenuDesc::empty();
    #[cfg(target_os = "macos")]
    {
        base = druid::platform_menus::mac::menu_bar();
    }
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        base = base.append(druid::platform_menus::win::file::default())
    }
    base
}

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
