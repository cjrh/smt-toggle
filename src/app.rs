use std::sync::mpsc;
use std::sync::Mutex;
use std::time::Duration;

use iced::futures::Stream;
use iced::widget::{button, column, container, row, text, toggler};
use iced::{window, Element, Length, Size, Subscription, Task, Theme};

use crate::smt::{self, SmtStatus};
use crate::tray::TrayEvent;

// Global channel receiver for tray events (set from main)
static TRAY_RECEIVER: Mutex<Option<mpsc::Receiver<TrayEvent>>> = Mutex::new(None);

pub fn set_tray_receiver(receiver: mpsc::Receiver<TrayEvent>) {
    *TRAY_RECEIVER.lock().unwrap() = Some(receiver);
}

#[derive(Debug, Clone)]
pub enum Message {
    SmtToggled(bool),
    SmtStatusUpdated(SmtStatus),
    RefreshStatus,
    SetSmtResult(Result<(), String>),
    WindowClosed(window::Id),
    TrayEvent(TrayEvent),
    GtkTick,
    WindowOpened(window::Id),
}

pub struct App {
    smt_status: SmtStatus,
    is_toggling: bool,
    error_message: Option<String>,
    window_id: Option<window::Id>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let status = smt::read_smt_status().unwrap_or(SmtStatus::Unknown);

        // Open the initial window
        let (id, open_task) = window::open(window::Settings {
            size: Size::new(300.0, 200.0),
            resizable: false,
            decorations: true,
            ..Default::default()
        });

        (
            Self {
                smt_status: status,
                is_toggling: false,
                error_message: None,
                window_id: Some(id),
            },
            open_task.map(Message::WindowOpened),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SmtToggled(enabled) => {
                self.is_toggling = true;
                self.error_message = None;

                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || smt::set_smt_enabled(enabled))
                            .await
                            .map_err(|e| e.to_string())?
                            .map_err(|e| e.to_string())
                    },
                    Message::SetSmtResult,
                )
            }
            Message::SetSmtResult(result) => {
                self.is_toggling = false;
                match result {
                    Ok(()) => {
                        // Refresh status after successful toggle
                        return Task::perform(
                            async {
                                tokio::task::spawn_blocking(|| {
                                    smt::read_smt_status().unwrap_or(SmtStatus::Unknown)
                                })
                                .await
                                .unwrap_or(SmtStatus::Unknown)
                            },
                            Message::SmtStatusUpdated,
                        );
                    }
                    Err(e) => {
                        self.error_message = Some(e);
                    }
                }
                Task::none()
            }
            Message::SmtStatusUpdated(status) => {
                self.smt_status = status;
                Task::none()
            }
            Message::RefreshStatus => Task::perform(
                async {
                    tokio::task::spawn_blocking(|| {
                        smt::read_smt_status().unwrap_or(SmtStatus::Unknown)
                    })
                    .await
                    .unwrap_or(SmtStatus::Unknown)
                },
                Message::SmtStatusUpdated,
            ),
            Message::WindowClosed(id) => {
                // Window was closed, clear the ID
                if self.window_id == Some(id) {
                    self.window_id = None;
                }
                Task::none()
            }
            Message::WindowOpened(id) => {
                self.window_id = Some(id);
                Task::none()
            }
            Message::GtkTick => {
                // Process pending GTK events for the tray icon
                while gtk::events_pending() {
                    gtk::main_iteration_do(false);
                }
                Task::none()
            }
            Message::TrayEvent(tray_event) => match tray_event {
                TrayEvent::ShowWindow => {
                    if let Some(id) = self.window_id {
                        // Window exists, just focus it
                        window::gain_focus(id)
                    } else {
                        // No window, open a new one
                        let (id, open_task) = window::open(window::Settings {
                            size: Size::new(300.0, 200.0),
                            resizable: false,
                            decorations: true,
                            ..Default::default()
                        });
                        self.window_id = Some(id);
                        open_task.map(Message::WindowOpened)
                    }
                }
                TrayEvent::Quit => {
                    std::process::exit(0);
                }
            },
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            // Listen for window close events (window actually closed)
            window::close_events().map(Message::WindowClosed),
            // Poll for GTK events every 100ms to keep the tray icon responsive
            iced::time::every(Duration::from_millis(100)).map(|_| Message::GtkTick),
            // Listen for tray events
            tray_subscription(),
        ])
    }

    pub fn view(&self, _window_id: window::Id) -> Element<'_, Message> {
        let status_text = match self.smt_status {
            SmtStatus::On => "SMT is ON (Hyperthreading enabled)",
            SmtStatus::Off => "SMT is OFF (Hyperthreading disabled)",
            SmtStatus::ForceOff => "SMT is Force OFF (disabled at boot)",
            SmtStatus::NotSupported => "SMT is not supported on this system",
            SmtStatus::Unknown => "SMT status unknown",
        };

        let mut content = column![text(status_text).size(16),].spacing(15);

        if self.smt_status.is_controllable() {
            let toggle = toggler(self.smt_status.is_enabled())
                .label(if self.smt_status.is_enabled() {
                    "Enabled"
                } else {
                    "Disabled"
                })
                .on_toggle(Message::SmtToggled);

            content = content.push(toggle);
        }

        if self.is_toggling {
            content = content.push(text("Applying changes...").size(12));
        }

        if let Some(ref error) = self.error_message {
            content = content.push(text(format!("Error: {}", error)).size(12));
        }

        let refresh_btn = button(text("Refresh")).on_press(Message::RefreshStatus);
        content = content.push(row![refresh_btn].spacing(10));

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }

    pub fn theme(&self, _window_id: window::Id) -> Theme {
        Theme::Dark
    }
}

/// Subscription that polls the tray event channel
fn tray_subscription() -> Subscription<Message> {
    Subscription::run(tray_event_stream)
}

fn tray_event_stream() -> impl Stream<Item = Message> {
    iced::futures::stream::unfold((), |()| async {
        // Poll the tray receiver
        loop {
            tokio::time::sleep(Duration::from_millis(50)).await;

            let event = {
                let receiver = TRAY_RECEIVER.lock().unwrap();
                if let Some(ref rx) = *receiver {
                    rx.try_recv().ok()
                } else {
                    None
                }
            };

            if let Some(evt) = event {
                return Some((Message::TrayEvent(evt), ()));
            }
        }
    })
}
