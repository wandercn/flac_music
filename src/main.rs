use druid::im::vector;
use druid::piet::Text;
use druid::text::TextInput;
use druid::widget::{
    prelude::*, Align, Button, Container, Label, LabelText, Scroll, Slider, Split, TextBox,
};
use druid::widget::{CrossAxisAlignment, List};
use druid::widget::{Flex, ProgressBar};
use druid::{im::Vector, AppLauncher, Data, Lens, Widget, WindowDesc};
use druid::{Color, TextAlignment, WidgetExt};

fn main() {
    let win = WindowDesc::new(ui_builder)
        .title("Flac Music")
        .resizable(true)
        .with_min_size((600.0, 600.0));
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
        play_mode: Modes::Order,
        current_play_list: vector![s.clone(), s.clone(), s.clone(), s],
        search_text: "search".into(),
    };
    let app = AppLauncher::with_window(win)
        .use_simple_logger()
        .launch(initState);
}

fn ui_builder() -> impl Widget<AppState> {
    let vol = Flex::row()
        .with_child(Label::new("Volume"))
        .with_child(Slider::new().with_range(1.0, 100.).lens(AppState::volume))
        .align_right();
    let SearchText = TextBox::new()
        .with_text_alignment(TextAlignment::Center)
        .lens(AppState::search_text);
    let ContrlTab = Container::new(
        Flex::row()
            .with_child(Button::new("|<<"))
            .with_child(Button::new("Play"))
            .with_child(Button::new("Pause"))
            .with_child(Button::new("Stop"))
            .with_child(Button::new(">>|"))
            .with_spacer(160.0)
            .with_child(vol)
            .center(),
    );

    let playlab = Flex::row()
        .with_child(SearchText)
        .with_spacer(100.0)
        .with_child(ContrlTab);
    let play_list_header = vector!["Playing", "Album", "Artist", "Date", "duration"];
    let mut header: Flex<AppState> =
        Flex::row().with_child(Label::new(play_list_header[0]).background(Color::GRAY));
    for lab in play_list_header.iter().skip(1) {
        header.add_spacer(50.0);
        header.add_child(Label::new(*lab).background(Color::GRAY));
    }

    let playList = Scroll::new(
        Flex::column()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .with_flex_child(
                Scroll::new(List::new(make_item).lens(AppState::current_play_list)).vertical(),
                1.0,
            )
            .border(Color::GREEN, 2.0)
            .expand_width(),
    );

    Container::new(
        Split::rows(playlab, Split::rows(header, playList).split_point(0.05)).split_point(0.1),
    )
}
#[derive(Data, Lens, Clone)]
struct AppState {
    app_status: Status,
    play_lists: Vector<PlayList>,
    current_song: Current,
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
    progress_rate: f64,
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
}

fn make_item() -> impl Widget<Song> {
    Flex::row()
        .with_child(Label::dynamic(|d: &Song, _| d.title.to_owned()))
        .with_spacer(40.0)
        .with_child(Label::dynamic(|d: &Song, _| d.album.to_owned()))
        .with_spacer(40.0)
        .with_child(Label::dynamic(|d: &Song, _| d.artist.to_owned()))
        .with_spacer(40.0)
        .with_child(Label::dynamic(|d: &Song, _| d.date.to_owned()))
        .with_spacer(40.0)
}
