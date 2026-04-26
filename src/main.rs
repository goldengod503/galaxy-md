use std::path::PathBuf;

use cosmic::app::Settings;
use cosmic::iced::border;
use cosmic::iced_core::text::Highlight;
use cosmic::iced_core::{padding, Background, Color, Length};
use cosmic::iced_runtime::Appearance;
use cosmic::iced_widget::container;
use cosmic::iced_widget::text_editor as ied_text_editor;
use cosmic::theme;
use cosmic::widget::{markdown, scrollable, text_editor};
use cosmic::{executor, prelude::*, Core};

// Tokyo Night palette (Night variant) — mirrors enkia.tokyo-night.
const fn tn(hex: u32) -> Color {
    Color {
        r: ((hex >> 16) & 0xff) as f32 / 255.0,
        g: ((hex >> 8) & 0xff) as f32 / 255.0,
        b: (hex & 0xff) as f32 / 255.0,
        a: 1.0,
    }
}

const TN_BG: Color = tn(0x1a1b26);
const TN_BG_DARK: Color = tn(0x16161e);
const TN_BG_LIGHT: Color = tn(0x24283b);
const TN_FG: Color = tn(0xc0caf5);
const TN_COMMENT: Color = tn(0x565f89);
const TN_BLUE: Color = tn(0x7aa2f7);
const TN_ORANGE: Color = tn(0xff9e64);
const TN_SELECTION: Color = Color {
    r: 0x36 as f32 / 255.0,
    g: 0x4a as f32 / 255.0,
    b: 0x82 as f32 / 255.0,
    a: 0.6,
};

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

// Group markdown items into sections (heading + following body) so the body
// can be wrapped in a lighter container — visually breaks up content under
// each heading.
fn render_sections<'a>(
    items: &'a [markdown::Item],
    settings: markdown::Settings,
    style: markdown::Style,
) -> Element<'a, markdown::Url> {
    use cosmic::widget::Column;

    let mut sections: Vec<Element<'a, markdown::Url>> = Vec::new();
    let mut pending_heading: Option<Element<'a, markdown::Url>> = None;
    let mut pending_body: Vec<Element<'a, markdown::Url>> = Vec::new();

    fn flush<'a>(
        sections: &mut Vec<Element<'a, markdown::Url>>,
        heading: Option<Element<'a, markdown::Url>>,
        body: Vec<Element<'a, markdown::Url>>,
    ) {
        match (heading, body.is_empty()) {
            (Some(h), true) => sections.push(h),
            (Some(h), false) => {
                let body_col = Column::with_children(body).spacing(8);
                let boxed = cosmic::widget::container(body_col)
                    .padding([14, 18])
                    .width(Length::Fill)
                    .style(|_theme| container::Style {
                        background: Some(Background::Color(TN_BG_LIGHT)),
                        text_color: Some(TN_FG),
                        border: border::rounded(8),
                        ..Default::default()
                    });
                sections.push(
                    Column::with_children(vec![h, boxed.into()])
                        .spacing(8)
                        .into(),
                );
            }
            (None, true) => {}
            (None, false) => {
                sections.push(Column::with_children(body).spacing(8).into());
            }
        }
    }

    for (idx, item) in items.iter().enumerate() {
        match item {
            markdown::Item::Heading(_, _) => {
                flush(
                    &mut sections,
                    pending_heading.take(),
                    std::mem::take(&mut pending_body),
                );
                pending_heading = Some(item.view(settings, style, idx));
            }
            _ => {
                pending_body.push(item.view(settings, style, idx));
            }
        }
    }
    flush(&mut sections, pending_heading.take(), pending_body);

    Column::with_children(sections).spacing(16).into()
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

    fn style(&self) -> Option<Appearance> {
        Some(Appearance {
            background_color: TN_BG,
            text_color: TN_FG,
            icon_color: TN_FG,
        })
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
                .size(16)
                .class(theme::iced::TextEditor::Custom(Box::new(
                    |_theme, _status| ied_text_editor::Style {
                        background: Background::Color(TN_BG),
                        border: border::rounded(0),
                        icon: TN_FG,
                        placeholder: TN_COMMENT,
                        value: TN_FG,
                        selection: TN_SELECTION,
                    },
                )));

            cosmic::widget::container(editor)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme| container::Style {
                    background: Some(Background::Color(TN_BG)),
                    text_color: Some(TN_FG),
                    ..Default::default()
                })
                .into()
        } else {
            let style = markdown::Style {
                inline_code_highlight: Highlight {
                    background: Background::Color(TN_BG_DARK),
                    border: border::rounded(4),
                },
                inline_code_padding: padding::left(6).right(6),
                inline_code_color: TN_ORANGE,
                link_color: TN_BLUE,
            };

            let settings = markdown::Settings::with_text_size(16);
            let content = render_sections(&self.items, settings, style)
                .map(Message::LinkClicked);

            let body = cosmic::widget::container(content)
                .padding(24)
                .width(Length::Fill);

            cosmic::widget::container(scrollable(body))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme| container::Style {
                    background: Some(Background::Color(TN_BG)),
                    text_color: Some(TN_FG),
                    ..Default::default()
                })
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

