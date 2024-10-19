#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::{
    keyboard, time,
    widget::{center, text, MouseArea},
    Element, Subscription, Task, Theme,
};
use iced_gif::widget::gif;
use rust_embed::Embed;
use std::time::{Duration, Instant};

const TEXT_COLORS: [iced::Color; 4] = [
    iced::Color::BLACK,                   // Dark
    iced::Color::WHITE,                   // White
    iced::Color::from_rgb(0.8, 0.9, 0.6), // Yellow
    iced::Color::from_rgb(0.2, 0.8, 0.2), // Green
];
const BACKGROUND_COLORS: [iced::Color; 3] = [
    iced::Color::from_rgb(
        0x20 as f32 / 255.0,
        0x22 as f32 / 255.0,
        0x25 as f32 / 255.0,
    ), // Dark
    iced::Color::from_rgb(0.9, 0.9, 0.9),           // Light
    iced::Color::from_rgba(0f32, 0f32, 0f32, 0f32), // Transparent
];

#[derive(Embed)]
#[folder = "assets"]
struct Asset;

fn main() -> iced::Result {
    iced::daemon(AppDaemon::title, AppDaemon::update, AppDaemon::view)
        .subscription(AppDaemon::subscription)
        .theme(AppDaemon::theme)
        .run_with(AppDaemon::new)
}

#[allow(dead_code)]
enum Picture {
    ImageHandle(iced::widget::image::Handle),
    GifFrams(gif::Frames),
}
struct AppDaemon {
    windows: (
        (iced::window::Id, TomatoClock),
        Option<(iced::window::Id, Reminder)>,
    ),
    is_picture_reminder: bool,
    picture_data: Picture,
}

struct Reminder {
    text: String,
    color: iced::Color,
}

impl Default for Reminder {
    fn default() -> Self {
        Self {
            text: ":) Time out!!!!!".to_string(),
            color: iced::Color::from_rgba(0.8, 1.0, 0.0, 0.8),
        }
    }
}

struct TomatoClock {
    duration: Duration,
    state: State,
    mode: Mode,
    pomodoro_duration: Duration,
    theme: CustomTheme,
}

impl Default for TomatoClock {
    fn default() -> Self {
        Self {
            duration: Duration::from_secs(25 * 60),
            state: State::default(),
            mode: Mode::default(),
            pomodoro_duration: Duration::from_secs(25 * 60),
            theme: CustomTheme::default(),
        }
    }
}

struct CustomTheme {
    stop: Theme,
    run: Theme,
    stop_text_color_index: usize,
    stop_background_color_index: usize,
    run_text_color_index: usize,
    run_background_color_index: usize,
}

impl Default for CustomTheme {
    fn default() -> Self {
        Self {
            stop: Theme::custom(
                "stop".to_string(),
                iced::theme::Palette {
                    background: BACKGROUND_COLORS[0],
                    ..Theme::default().palette()
                },
            ),
            run: Theme::custom(
                "run".to_string(),
                iced::theme::Palette {
                    background: BACKGROUND_COLORS[2],
                    ..Theme::default().palette()
                },
            ),
            stop_background_color_index: 0,
            stop_text_color_index: 1,
            run_background_color_index: 2,
            run_text_color_index: 2,
        }
    }
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
    TogglePictureReminder,
}

impl AppDaemon {
    fn new() -> (Self, Task<Message>) {
        let (id, open) = iced::window::open(iced::window::Settings {
            size: iced::Size::new(150f32, 45f32),
            position: iced::window::Position::Centered,
            resizable: false,
            decorations: false,
            transparent: true,
            level: iced::window::Level::AlwaysOnTop,
            icon: Some(iced::window::Icon::from(
                iced::window::icon::from_file_data(include_bytes!("..\\tomato.ico"), None).unwrap(),
            )),
            ..Default::default()
        });
        let picture = Asset::get("reminder.gif").unwrap().data.into_owned();
        let gif_frames = gif::Frames::from_bytes(picture).unwrap();
        (
            Self {
                windows: ((id, TomatoClock::default()), None),
                is_picture_reminder: false,
                picture_data: Picture::GifFrams(gif_frames),
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
                reminder.view(if self.is_picture_reminder {
                    Some(&self.picture_data)
                } else {
                    None
                })
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
                self.windows.1 = Some((id, Reminder::new()));
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
            Message::TogglePictureReminder => {
                self.is_picture_reminder = !self.is_picture_reminder;
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
                    background: BACKGROUND_COLORS[2],
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
            Message::ChangeTextColor => {
                if matches!(self.state, State::Idle) {
                    self.theme.stop_text_color_index =
                        (self.theme.stop_text_color_index + 1) % TEXT_COLORS.len();
                } else {
                    self.theme.run_text_color_index =
                        (self.theme.run_text_color_index + 1) % TEXT_COLORS.len();
                }
            }
            Message::ChangeBackgroundColor => {
                if matches!(self.state, State::Idle) {
                    self.theme.stop_background_color_index =
                        (self.theme.stop_background_color_index + 1) % BACKGROUND_COLORS.len();
                    self.theme.stop = Theme::custom(
                        "stop".to_string(),
                        iced::theme::Palette {
                            background: BACKGROUND_COLORS[self.theme.stop_background_color_index],
                            ..self.theme.stop.palette()
                        },
                    )
                } else {
                    self.theme.run_background_color_index =
                        (self.theme.run_background_color_index + 1) % BACKGROUND_COLORS.len();
                    self.theme.run = Theme::custom(
                        "run".to_string(),
                        iced::theme::Palette {
                            background: BACKGROUND_COLORS[self.theme.run_background_color_index],
                            ..self.theme.run.palette()
                        },
                    )
                }
            }
            Message::Shutdown => return iced::exit(),
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
                keyboard::Key::Character("p") => Some(Message::TogglePictureReminder),
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
            TEXT_COLORS[self.theme.stop_text_color_index]
        } else {
            TEXT_COLORS[self.theme.run_text_color_index]
        })
        .size(40)
        .line_height(iced::widget::text::LineHeight::Absolute(iced::Pixels(
            40f32,
        )));

        MouseArea::new(center(duration))
            .on_press(Message::StartDragging)
            // .on_right_press(Message::TimeOut)
            .into()
    }
    fn theme(&self) -> Theme {
        if let State::Idle = self.state {
            self.theme.stop.clone()
        } else {
            self.theme.run.clone()
        }
    }
}

impl Reminder {
    fn new() -> Self {
        Reminder::default()
    }

    fn view<'a>(&'a self, picture: Option<&'a Picture>) -> Element<Message> {
        match picture {
            Some(Picture::ImageHandle(handle)) => {
                let picture = iced::widget::image(handle);
                MouseArea::new(center(picture.width(400)))
                    .on_press(Message::CloseReminder)
                    .into()
            }
            Some(Picture::GifFrams(frames)) => {
                let picture: gif::Gif<'a> = gif(frames).width(iced::Length::from(400));
                MouseArea::new(center(picture))
                    .on_press(Message::CloseReminder)
                    .into()
            }
            None => MouseArea::new(center(
                text(&self.text).color(self.color).size(180).center(),
            ))
            .on_press(Message::CloseReminder)
            .into(),
        }
    }
}
