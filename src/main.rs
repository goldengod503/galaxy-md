use std::path::PathBuf;

use cosmic::app::Settings;
use cosmic::iced::border;
use cosmic::iced_core::text::Highlight;
use cosmic::iced_core::{color, padding, Color, Length};
use cosmic::widget::{markdown, scrollable};
use cosmic::{executor, prelude::*, Core};

struct App {
    core: Core,
    items: Vec<markdown::Item>,
}

#[derive(Clone, Debug)]
enum Message {
    LinkClicked(markdown::Url),
}

impl cosmic::Application for App {
    type Executor = executor::Default;
    type Flags = (String, Vec<markdown::Item>);
    type Message = Message;

    const APP_ID: &'static str = "com.cosmic.md-viewer";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(mut core: Core, flags: Self::Flags) -> (Self, cosmic::app::Task<Self::Message>) {
        let (title, items) = flags;
        core.set_header_title(title);
        let app = App { core, items };
        (app, cosmic::app::Task::none())
    }

    fn update(&mut self, message: Self::Message) -> cosmic::app::Task<Self::Message> {
        match message {
            Message::LinkClicked(url) => {
                let _ = open::that_in_background(url.to_string());
            }
        }
        cosmic::app::Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let cosmic_theme = cosmic::theme::active();
        let cosmic_inner = cosmic_theme.cosmic();
        let accent = Color::from(cosmic_inner.accent.base);

        let style = markdown::Style {
            inline_code_highlight: Highlight {
                background: color!(0x2a2a2a).into(),
                border: border::rounded(4),
            },
            inline_code_padding: padding::left(4).right(4),
            inline_code_color: Color::WHITE,
            link_color: accent,
        };

        let content = markdown::view(
            &self.items,
            markdown::Settings::with_text_size(16),
            style,
        )
        .map(Message::LinkClicked);

        scrollable(
            cosmic::widget::container(content)
                .padding(24)
                .width(Length::Fill),
        )
        .into()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            eprintln!("Usage: cosmic-md <file.md>");
            std::process::exit(1);
        });

    let source = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("Failed to read {}: {e}", path.display());
        std::process::exit(1);
    });

    let title = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Markdown Viewer".into());

    let items: Vec<markdown::Item> = markdown::parse(&source).collect();

    let settings = Settings::default()
        .size(cosmic::iced::Size::new(900.0, 700.0));

    cosmic::app::run::<App>(settings, (title, items))?;

    Ok(())
}
