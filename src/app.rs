use iced::widget::{button, column, container, row, text, toggler};
use iced::{window, Element, Length, Size, Task, Theme};

use crate::smt::{self, SmtStatus};

#[derive(Debug, Clone)]
pub enum Message {
    SmtToggled(bool),
    SmtStatusUpdated(SmtStatus),
    RefreshStatus,
    SetSmtResult(Result<(), String>),
}

pub struct App {
    smt_status: SmtStatus,
    is_toggling: bool,
    error_message: Option<String>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let status = smt::read_smt_status().unwrap_or(SmtStatus::Unknown);
        (
            Self {
                smt_status: status,
                is_toggling: false,
                error_message: None,
            },
            Task::none(),
        )
    }

    pub fn title(&self) -> String {
        String::from("SMT Toggle")
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
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
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

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }

    pub fn window_settings() -> window::Settings {
        window::Settings {
            size: Size::new(300.0, 200.0),
            resizable: false,
            decorations: true,
            ..Default::default()
        }
    }
}
