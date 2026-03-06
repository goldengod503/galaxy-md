use std::path::PathBuf;

use cosmic::app::Settings;
use cosmic::iced::border;
use cosmic::iced_core::text::Highlight;
use cosmic::iced_core::{color, padding, Color, Length};
use cosmic::widget::{markdown, scrollable, text_editor};
use cosmic::{executor, prelude::*, Core};

struct App {
    core: Core,
    items: Vec<markdown::Item>,
    editor_content: text_editor::Content,
    source: String,
    path: PathBuf,
    selectable_mode: bool,
}

#[derive(Clone, Debug)]
enum Message {
    LinkClicked(markdown::Url),
    FileChanged,
    EditorAction(text_editor::Action),
    ToggleSelectable,
}

fn markdown_to_plain_text(source: &str) -> String {
    use pulldown_cmark::{Event, Parser, Tag, TagEnd};

    let parser = Parser::new(source);
    let mut output = String::new();
    let mut list_index: Option<u64> = None;

    for event in parser {
        match event {
            Event::Text(text) | Event::Code(text) => {
                output.push_str(&text);
            }
            Event::SoftBreak => output.push(' '),
            Event::HardBreak => output.push('\n'),
            Event::Start(Tag::Heading { .. }) => {
                if !output.is_empty() && !output.ends_with('\n') {
                    output.push('\n');
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                output.push('\n');
            }
            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) => {
                output.push_str("\n\n");
            }
            Event::Start(Tag::CodeBlock(_)) => {}
            Event::End(TagEnd::CodeBlock) => {
                if !output.ends_with('\n') {
                    output.push('\n');
                }
                output.push('\n');
            }
            Event::Start(Tag::List(ordered)) => {
                list_index = ordered;
            }
            Event::End(TagEnd::List(_)) => {
                list_index = None;
                if !output.ends_with('\n') {
                    output.push('\n');
                }
            }
            Event::Start(Tag::Item) => {
                if let Some(idx) = &mut list_index {
                    output.push_str(&format!("  {idx}. "));
                    *idx += 1;
                } else {
                    output.push_str("  • ");
                }
            }
            Event::End(TagEnd::Item) => {
                if !output.ends_with('\n') {
                    output.push('\n');
                }
            }
            _ => {}
        }
    }

    output.trim_end().to_string()
}

impl cosmic::Application for App {
    type Executor = executor::Default;
    type Flags = (String, Vec<markdown::Item>, String, PathBuf);
    type Message = Message;

    const APP_ID: &'static str = "com.galaxy.md-viewer";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(mut core: Core, flags: Self::Flags) -> (Self, cosmic::app::Task<Self::Message>) {
        let (title, items, source, path) = flags;
        core.set_header_title(title);
        let plain_text = markdown_to_plain_text(&source);
        let editor_content = text_editor::Content::with_text(&plain_text);
        let app = App {
            core,
            items,
            editor_content,
            source,
            path,
            selectable_mode: false,
        };
        (app, cosmic::app::Task::none())
    }

    fn header_end(&self) -> Vec<Element<'_, Self::Message>> {
        let label = if self.selectable_mode {
            "Formatted"
        } else {
            "Select Text"
        };
        vec![
            cosmic::widget::button::text(label)
                .on_press(Message::ToggleSelectable)
                .into(),
        ]
    }

    fn update(&mut self, message: Self::Message) -> cosmic::app::Task<Self::Message> {
        match message {
            Message::LinkClicked(url) => {
                let _ = open::that_in_background(url.to_string());
            }
            Message::FileChanged => {
                if let Ok(source) = std::fs::read_to_string(&self.path) {
                    self.items = markdown::parse(&source).collect();
                    let plain_text = markdown_to_plain_text(&source);
                    self.editor_content = text_editor::Content::with_text(&plain_text);
                    self.source = source;
                }
            }
            Message::EditorAction(action) => {
                if !action.is_edit() {
                    self.editor_content.perform(action);
                }
            }
            Message::ToggleSelectable => {
                self.selectable_mode = !self.selectable_mode;
                if self.selectable_mode {
                    let plain_text = markdown_to_plain_text(&self.source);
                    self.editor_content = text_editor::Content::with_text(&plain_text);
                }
            }
        }
        cosmic::app::Task::none()
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        let path = self.path.clone();
        cosmic::iced::Subscription::run_with_id(
            "file-watcher",
            cosmic::iced_futures::stream::channel(1, move |mut sender| {
                let path = path.clone();
                async move {
                    use cosmic::iced_futures::futures::SinkExt;
                    use notify::Watcher;

                    let (tx, mut rx) = cosmic::iced::futures::channel::mpsc::channel(1);
                    let mut watcher = notify::recommended_watcher(
                        move |event: Result<notify::Event, notify::Error>| {
                            if let Ok(event) = event {
                                if matches!(
                                    event.kind,
                                    notify::EventKind::Modify(_) | notify::EventKind::Create(_)
                                ) {
                                    let _ = tx.clone().try_send(());
                                }
                            }
                        },
                    )
                    .expect("failed to create file watcher");

                    watcher
                        .watch(&path, notify::RecursiveMode::NonRecursive)
                        .expect("failed to watch file");

                    loop {
                        use cosmic::iced::futures::StreamExt;
                        if rx.next().await.is_some() {
                            let _ = sender.send(Message::FileChanged).await;
                        }
                    }
                }
            }),
        )
    }

    fn view(&self) -> Element<'_, Self::Message> {
        if self.selectable_mode {
            let editor = cosmic::widget::TextEditor::new(&self.editor_content)
                .on_action(Message::EditorAction)
                .padding(24)
                .size(16);

            cosmic::widget::container(editor)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            eprintln!("Usage: galaxy-md <file.md>");
            std::process::exit(1);
        });

    let path = std::fs::canonicalize(&path).unwrap_or_else(|e| {
        eprintln!("Failed to resolve {}: {e}", path.display());
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

    cosmic::app::run::<App>(settings, (title, items, source, path))?;

    Ok(())
}
