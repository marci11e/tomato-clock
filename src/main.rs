#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::{
    keyboard, time,
    widget::{center, text, MouseArea},
    Element, Subscription, Task, Theme,
};
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

fn main() -> iced::Result {
    iced::application("Tomato Clock", TomatoClock::update, TomatoClock::view)
        .subscription(TomatoClock::subscription)
        .theme(TomatoClock::theme)
        .window(iced::window::Settings {
            size: iced::Size::new(180f32, 60f32),
            position: iced::window::Position::Centered,
            resizable: false,
            decorations: false,
            transparent: true,
            level: iced::window::Level::AlwaysOnTop,
            icon: Some(iced::window::Icon::from(
                iced::window::icon::from_file_data(include_bytes!("..\\tomato.ico"), None).unwrap(),
            )),
            ..Default::default()
        })
        .run_with(TomatoClock::new)
}

struct TomatoClock {
    duration: Duration,
    state: State,
    mode: Mode,
    pomodoro_duration: Duration,
    theme: CustomTheme,
    window_id: Option<iced::window::Id>,
}

impl Default for TomatoClock {
    fn default() -> Self {
        Self {
            duration: Duration::from_secs(25 * 60),
            state: State::default(),
            mode: Mode::default(),
            pomodoro_duration: Duration::from_secs(25 * 60),
            theme: CustomTheme::default(),
            window_id: None,
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
    GetWindowId(iced::window::Id),
    StartDragging,
    ChangeTextColor,
    ChangeBackgroundColor,
}

impl TomatoClock {
    fn new() -> (Self, Task<Message>) {
        (
            Self::default(),
            iced::window::get_oldest().and_then(|id| Task::done(Message::GetWindowId(id))),
        )
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
            Message::GetWindowId(id) => {
                self.window_id = Some(id);
            }
            Message::StartDragging => {
                if let Some(id) = self.window_id {
                    return iced::window::drag(id);
                } else {
                    return iced::window::get_oldest()
                        .and_then(|id| Task::done(Message::GetWindowId(id)));
                }
            }
            Message::Shutdown => return iced::exit(),
        }
        Task::none()
    }
    fn subscription(&self) -> Subscription<Message> {
        let tick = match self.state {
            State::Idle => Subscription::none(),
            State::Ticking { .. } => time::every(Duration::from_millis(1000)).map(Message::Tick), // equal to |_|Message::Tick
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
            TEXT_COLORS[self.theme.stop_text_color_index]
        } else {
            TEXT_COLORS[self.theme.run_text_color_index]
        })
        .size(40);
        MouseArea::new(center(duration))
            .on_press(Message::StartDragging)
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
