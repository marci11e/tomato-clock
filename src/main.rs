#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::{
    keyboard, time,
    widget::{center, text, MouseArea},
    Element, Subscription, Task, Theme,
};
use iced_gif::widget::gif;
use serde::{Deserialize, Serialize};
use std::{
    time::{Duration, Instant},
    vec::Vec,
};
use toml;

const CONFIG_PATH: &str = "tomato.toml";

fn main() -> iced::Result {
    iced::daemon(AppDaemon::title, AppDaemon::update, AppDaemon::view)
        .subscription(AppDaemon::subscription)
        .theme(AppDaemon::theme)
        .run_with(AppDaemon::new)
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl From<Color> for iced::Color {
    fn from(c: Color) -> Self {
        iced::Color::from_rgba(c.r, c.g, c.b, c.a)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct ReminderConfig {
    text: Option<String>,
    color: Option<Color>,
    font_size: Option<u16>,
    image_path: Option<String>,
    width: Option<u16>,
    height: Option<u16>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct TomatoConfig {
    position: Option<[f32; 2]>,
    stop_text_color_index: usize,
    run_text_color_index: usize,
    stop_background_color_index: usize,
    run_background_color_index: usize,
    text_colors: Vec<Color>,
    background_colors: Vec<Color>,
    reminder: ReminderConfig,
}

impl Default for TomatoConfig {
    fn default() -> Self {
        Self {
            position: None,
            stop_text_color_index: 0,
            run_text_color_index: 0,
            stop_background_color_index: 0,
            run_background_color_index: 0,
            text_colors: vec![Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }],
            background_colors: vec![Color {
                r: 0.9,
                g: 0.9,
                b: 0.9,
                a: 1.0,
            }],
            reminder: ReminderConfig {
                text: None,
                color: None,
                font_size: None,
                image_path: None,
                width: None,
                height: None,
            },
        }
    }
}

struct AppDaemon {
    windows: (
        (iced::window::Id, TomatoClock),
        Option<(iced::window::Id, Reminder)>,
    ),
    picture_data: Option<Picture>,
    exist_entity: bool,
    tomato_config: TomatoConfig,
}

enum Picture {
    ImageHandle(iced::widget::image::Handle),
    GifFrams(gif::Frames),
}

struct Reminder {
    text: String,
    color: iced::Color,
    font_size: u16,
    width: Option<u16>,
    height: Option<u16>,
}

impl Default for Reminder {
    fn default() -> Self {
        Self {
            text: ":) Time out!!!!!".to_string(),
            color: iced::Color::from_rgba(0.8, 1.0, 0.0, 0.8),
            font_size: 180,
            width: None,
            height: None,
        }
    }
}

struct TomatoClock {
    duration: Duration,
    state: State,
    mode: Mode,
    pomodoro_duration: Duration,
    run_background_color: iced::Color,
    stop_background_color: iced::Color,
    run_text_color: iced::Color,
    stop_text_color: iced::Color,
}

#[derive(Default)]
enum Mode {
    #[default]
    Pomodoro,
    Stopwatch,
}

#[derive(Default)]
enum State {
    #[default]
    Idle,
    Ticking {
        last_tick: Instant,
    },
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Toggle,
    ToggleMode,
    Reset,
    Tick(Instant),
    IncreasePomodoroDuration,
    DecreasePomodoroDuration,
    Shutdown,
    StartDragging,
    ChangeTextColor,
    ChangeBackgroundColor,
    TimeOut,
    CloseReminder,
    EarlyTermination,
}

impl AppDaemon {
    fn new() -> (Self, Task<Message>) {
        let (mut tomato_config, exist_entity) =
            if let Ok(toml_str) = std::fs::read_to_string(CONFIG_PATH) {
                (
                    toml::from_str(&toml_str).expect("Failed to parse config file"),
                    true,
                )
            } else {
                (TomatoConfig::default(), false)
            };
        // println!("Tomato config: {:#?}", tomato_config);
        let picture_data = if let Some(path) = &tomato_config.reminder.image_path {
            if matches!(
                std::path::Path::new(path)
                    .extension()
                    .map(|ext| ext.to_str()),
                Some(Some("gif"))
            ) {
                Some(Picture::GifFrams(
                    gif::Frames::from_bytes(
                        std::fs::read(path).expect("Failed to read image file"),
                    )
                    .expect("Failed to decode gif file"),
                ))
            } else {
                Some(Picture::ImageHandle(path.into()))
            }
        } else {
            None
        };
        let (id, open) = iced::window::open(iced::window::Settings {
            size: iced::Size::new(150f32, 45f32),
            position: if let Some(position) = tomato_config.position {
                iced::window::Position::Specific(iced::Point::new(position[0], position[1]))
            } else {
                iced::window::Position::Centered
            },
            resizable: false,
            decorations: false,
            transparent: true,
            level: iced::window::Level::AlwaysOnTop,
            icon: Some(iced::window::Icon::from(
                iced::window::icon::from_file_data(include_bytes!("..\\tomato.ico"), None).unwrap(),
            )),
            ..Default::default()
        });
        tomato_config.run_background_color_index =
            if tomato_config.run_background_color_index < tomato_config.background_colors.len() {
                tomato_config.run_background_color_index
            } else {
                tomato_config.background_colors.len() - 1
            };
        tomato_config.stop_background_color_index =
            if tomato_config.stop_background_color_index < tomato_config.background_colors.len() {
                tomato_config.stop_background_color_index
            } else {
                tomato_config.background_colors.len() - 1
            };
        tomato_config.run_text_color_index =
            if tomato_config.run_text_color_index < tomato_config.text_colors.len() {
                tomato_config.run_text_color_index
            } else {
                tomato_config.text_colors.len() - 1
            };
        tomato_config.stop_text_color_index =
            if tomato_config.stop_text_color_index < tomato_config.text_colors.len() {
                tomato_config.stop_text_color_index
            } else {
                tomato_config.text_colors.len() - 1
            };

        (
            Self {
                windows: (
                    (
                        id,
                        TomatoClock::new(
                            tomato_config.background_colors
                                [tomato_config.run_background_color_index]
                                .into(),
                            tomato_config.background_colors
                                [tomato_config.stop_background_color_index]
                                .into(),
                            tomato_config.text_colors[tomato_config.run_text_color_index].into(),
                            tomato_config.text_colors[tomato_config.stop_text_color_index].into(),
                        ),
                    ),
                    None,
                ),
                picture_data,
                exist_entity,
                tomato_config,
            },
            open.then(|_| Task::none()),
        )
    }
    fn title(&self, window: iced::window::Id) -> String {
        if self.windows.0 .0 == window {
            "Tomato Clock".to_string()
        } else {
            "Time out".to_string()
        }
    }

    fn view(&self, window: iced::window::Id) -> Element<Message> {
        if self.windows.0 .0 == window {
            self.windows.0 .1.view()
        } else {
            if let Some((_, reminder)) = &self.windows.1 {
                reminder.view(self.picture_data.as_ref())
            } else {
                iced::widget::horizontal_space().into()
            }
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TimeOut => {
                let (id, open) = iced::window::open(iced::window::Settings {
                    position: iced::window::Position::Centered,
                    resizable: false,
                    decorations: false,
                    transparent: true,
                    level: iced::window::Level::AlwaysOnTop,
                    ..Default::default()
                });
                let ReminderConfig {
                    text,
                    color,
                    font_size,
                    width,
                    height,
                    ..
                } = &self.tomato_config.reminder;
                let reminder = Reminder::new(text, color, font_size, width, height);
                self.windows.1 = Some((id, reminder));
                return open.then(|id| iced::window::maximize(id, true));
            }
            Message::StartDragging => {
                return iced::window::drag(self.windows.0 .0);
            }
            Message::CloseReminder => {
                if let Some((id, _)) = self.windows.1 {
                    self.windows.1 = None;
                    return iced::window::close(id);
                }
            }
            Message::ChangeTextColor => {
                if let State::Idle = self.windows.0 .1.state {
                    self.tomato_config.stop_text_color_index =
                        (self.tomato_config.stop_text_color_index + 1)
                            % self.tomato_config.text_colors.len();
                    self.windows.0 .1.stop_text_color = self.tomato_config.text_colors
                        [self.tomato_config.stop_text_color_index]
                        .into();
                } else {
                    self.tomato_config.run_text_color_index =
                        (self.tomato_config.run_text_color_index + 1)
                            % self.tomato_config.text_colors.len();
                    self.windows.0 .1.run_text_color = self.tomato_config.text_colors
                        [self.tomato_config.run_text_color_index]
                        .into();
                }
            }
            Message::ChangeBackgroundColor => {
                if let State::Idle = self.windows.0 .1.state {
                    self.tomato_config.stop_background_color_index =
                        (self.tomato_config.stop_background_color_index + 1)
                            % self.tomato_config.background_colors.len();
                    self.windows.0 .1.stop_background_color = self.tomato_config.background_colors
                        [self.tomato_config.stop_background_color_index]
                        .into();
                } else {
                    self.tomato_config.run_background_color_index =
                        (self.tomato_config.run_background_color_index + 1)
                            % self.tomato_config.background_colors.len();
                    self.windows.0 .1.run_background_color = self.tomato_config.background_colors
                        [self.tomato_config.run_background_color_index]
                        .into();
                }
            }
            Message::Shutdown => {
                if self.exist_entity {
                    let mut tomato_config = self.tomato_config.clone();
                    return iced::window::get_position(self.windows.0 .0).then(move |pos| {
                        if let Some(iced::Point { x, y }) = pos {
                            tomato_config.position = Some([x, y]);
                        }
                        std::fs::write(CONFIG_PATH, toml::to_string(&tomato_config).unwrap())
                            .expect("Failed to write config file");
                        iced::exit()
                    });
                };
                return iced::exit();
            }
            _ => return self.windows.0 .1.update(message),
        }
        Task::none()
    }

    fn theme(&self, window: iced::window::Id) -> Theme {
        if self.windows.0 .0 == window {
            self.windows.0 .1.theme()
        } else {
            Theme::custom(
                "reminder".to_string(),
                iced::theme::Palette {
                    background: iced::Color::TRANSPARENT,
                    ..Theme::default().palette()
                },
            )
        }
    }
    fn subscription(&self) -> Subscription<Message> {
        self.windows.0 .1.subscription()
    }
}

impl TomatoClock {
    fn new(
        run_background_color: iced::Color,
        stop_background_color: iced::Color,
        run_text_color: iced::Color,
        stop_text_color: iced::Color,
    ) -> Self {
        Self {
            duration: Duration::from_secs(25 * 60),
            state: State::default(),
            mode: Mode::default(),
            pomodoro_duration: Duration::from_secs(25 * 60),
            run_background_color,
            run_text_color,
            stop_background_color,
            stop_text_color,
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Toggle => match self.state {
                State::Idle => {
                    self.state = State::Ticking {
                        last_tick: Instant::now(),
                    };
                }
                State::Ticking { .. } => {
                    self.state = State::Idle;
                }
            },
            Message::ToggleMode => {
                self.state = State::Idle;
                match self.mode {
                    Mode::Pomodoro => {
                        self.mode = Mode::Stopwatch;
                        self.duration = Duration::ZERO;
                    }
                    Mode::Stopwatch => {
                        self.mode = Mode::Pomodoro;
                        self.duration = self.pomodoro_duration;
                    }
                };
            }
            Message::Tick(now) => {
                if let State::Ticking { last_tick } = &mut self.state {
                    if let Mode::Pomodoro = &self.mode {
                        if self.duration > Duration::ZERO + Duration::from_secs(1) {
                            self.duration -= now - *last_tick;
                            *last_tick = now;
                        } else {
                            self.duration = self.pomodoro_duration;
                            self.state = State::Idle;
                            return Task::done(Message::TimeOut);
                        }
                    } else {
                        self.duration += now - *last_tick;
                        *last_tick = now;
                    }
                };
            }
            Message::EarlyTermination => {
                if let Mode::Pomodoro = &self.mode {
                    self.duration = self.pomodoro_duration;
                    self.state = State::Idle;
                    return Task::done(Message::TimeOut);
                }
            }
            Message::Reset => {
                match self.mode {
                    Mode::Pomodoro => self.duration = self.pomodoro_duration,
                    Mode::Stopwatch => self.duration = Duration::ZERO,
                }
                self.state = State::Idle;
            }
            Message::IncreasePomodoroDuration => {
                if matches!(self.state, State::Idle)
                    && matches!(self.mode, Mode::Pomodoro)
                    && self.pomodoro_duration < Duration::from_secs(60 * 60)
                {
                    self.pomodoro_duration += Duration::from_secs(5 * 60);
                    self.duration = self.pomodoro_duration;
                };
            }
            Message::DecreasePomodoroDuration => {
                if matches!(self.state, State::Idle)
                    && matches!(self.mode, Mode::Pomodoro)
                    && self.pomodoro_duration > Duration::from_secs(5 * 60)
                {
                    self.pomodoro_duration -= Duration::from_secs(5 * 60);
                    self.duration = self.pomodoro_duration;
                }
            }
            _ => {}
        }
        Task::none()
    }
    fn subscription(&self) -> Subscription<Message> {
        let tick = match self.state {
            State::Idle => Subscription::none(),
            State::Ticking { .. } => time::every(Duration::from_millis(1000)).map(Message::Tick), // equal to |instant| Message::Tick(instant),
        };
        fn handle_hotkey(key: keyboard::Key, _modifiers: keyboard::Modifiers) -> Option<Message> {
            match key.as_ref() {
                keyboard::Key::Named(keyboard::key::Named::Space) => Some(Message::Toggle),
                keyboard::Key::Named(keyboard::key::Named::Escape) => Some(Message::Shutdown),
                keyboard::Key::Character("r") => Some(Message::Reset),
                keyboard::Key::Character("m") => Some(Message::ToggleMode),
                keyboard::Key::Character("[") => Some(Message::DecreasePomodoroDuration),
                keyboard::Key::Character("]") => Some(Message::IncreasePomodoroDuration),
                keyboard::Key::Character("t") => Some(Message::ChangeTextColor),
                keyboard::Key::Character("b") => Some(Message::ChangeBackgroundColor),
                _ => None,
            }
        }
        Subscription::batch(vec![tick, keyboard::on_key_press(handle_hotkey)])
    }
    fn view(&self) -> Element<Message> {
        const MINUTE: u64 = 60;
        const HOUR: u64 = MINUTE * 60;

        let seconds = self.duration.as_secs();
        let duration = text!(
            "{:0>2}:{:0>2}:{:0>2}",
            seconds / HOUR,
            (seconds % HOUR) / MINUTE,
            seconds % MINUTE,
        )
        .color(if matches!(self.state, State::Idle) {
            self.stop_text_color
        } else {
            self.run_text_color
        })
        .size(40)
        .line_height(iced::widget::text::LineHeight::Absolute(iced::Pixels(
            40f32,
        )));

        MouseArea::new(center(duration))
            .on_press(Message::StartDragging)
            .on_right_press(Message::EarlyTermination)
            .into()
    }
    fn theme(&self) -> Theme {
        if let State::Idle = self.state {
            Theme::custom(
                "stop".to_string(),
                iced::theme::Palette {
                    background: self.stop_background_color.into(),
                    ..Theme::default().palette()
                },
            )
        } else {
            Theme::custom(
                "run".to_string(),
                iced::theme::Palette {
                    background: self.run_background_color.into(),
                    ..Theme::default().palette()
                },
            )
        }
    }
}

impl Reminder {
    fn new(
        text: &Option<String>,
        color: &Option<Color>,
        font_size: &Option<u16>,
        width: &Option<u16>,
        height: &Option<u16>,
    ) -> Self {
        let mut reminder = Reminder::default();
        if let Some(text) = text {
            reminder.text = text.clone();
        }
        if let Some(color) = color {
            reminder.color = color.clone().into();
        }
        if let Some(font_size) = font_size {
            reminder.font_size = font_size.clone();
        }
        reminder.width = *width;
        reminder.height = *height;
        reminder
    }

    fn view<'a>(&'a self, picture: Option<&'a Picture>) -> Element<Message> {
        match picture {
            Some(Picture::ImageHandle(handle)) => {
                let mut picture = iced::widget::image(handle);
                if let Some(width) = self.width {
                    picture = picture.width(width)
                }
                if let Some(height) = self.height {
                    picture = picture.height(height)
                }
                MouseArea::new(center(picture))
                    .on_press(Message::CloseReminder)
                    .into()
            }
            Some(Picture::GifFrams(frames)) => {
                let mut picture: gif::Gif<'a> = gif(frames);
                if let Some(width) = self.width {
                    picture = picture.width(iced::Length::from(width))
                }
                if let Some(height) = self.height {
                    picture = picture.height(iced::Length::from(height))
                }
                MouseArea::new(center(picture))
                    .on_press(Message::CloseReminder)
                    .into()
            }
            None => {
                let mut _text = text(&self.text)
                    .color(self.color)
                    .size(self.font_size)
                    .center();
                if let Some(width) = self.width {
                    _text = _text.width(width)
                }
                if let Some(height) = self.height {
                    _text = _text.height(height)
                }
                MouseArea::new(center(_text))
                    .on_press(Message::CloseReminder)
                    .into()
            }
        }
    }
}
