use std::{
    ops::Range,
    sync::{Arc, Mutex},
};

use gpui::*;

use smallvec::SmallVec;

use crate::theme::Theme;
use gpui::prelude::FluentBuilder;

#[derive(IntoElement)]
pub struct Background {
    children: SmallVec<[AnyElement; 2]>,
}

impl Background {
    pub fn new() -> Self {
        Background {
            children: SmallVec::new(),
        }
    }
}

impl RenderOnce for Background {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        div()
            .bg(theme.background_color)
            .text_color(theme.text_color)
            .size_full()
            .when(self.children.len() > 0, |this| this.children(self.children))
    }
}

impl ParentElement for Background {
    fn extend(&mut self, elements: impl Iterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

#[derive(IntoElement)]
pub struct Layout {
    title_bar: AnyElement,
    body: AnyElement,
}

impl Layout {
    pub fn new() -> Self {
        Self {
            title_bar: TitleBar::new().into_any_element(),
            body: div().into_any_element(),
        }
    }

    pub fn title_bar(mut self, title_bar: impl IntoElement) -> Self {
        self.title_bar = title_bar.into_any_element();
        self
    }

    pub fn body(mut self, body: impl IntoElement) -> Self {
        self.body = body.into_any_element();
        self
    }
}

impl RenderOnce for Layout {
    fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
        div()
            .child(self.title_bar)
            .child(div().p_6().child(self.body))
    }
}

#[derive(IntoElement)]
pub struct TitleBar {
    children: SmallVec<[AnyElement; 2]>,
}

impl TitleBar {
    pub fn new() -> Self {
        TitleBar {
            children: SmallVec::new(),
        }
    }
}

impl RenderOnce for TitleBar {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        div()
            .h_7()
            .flex()
            .items_center()
            .bg(theme.panel_color)
            .border_color(theme.border_color)
            .border_b()
            .when(self.children.len() > 0, |this| this.children(self.children))
    }
}

impl ParentElement for TitleBar {
    fn extend(&mut self, elements: impl Iterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

pub enum ButtonVariant {
    Primary,
    Danger,
}

#[derive(IntoElement)]
pub struct Button {
    base: Div,
    child: AnyElement,
    on_click: Box<dyn Fn(&MouseDownEvent, &mut WindowContext)>,
    variant: ButtonVariant,
}

impl Button {
    #[allow(dead_code)]
    pub fn new(
        child: impl IntoElement,
        on_click: Box<dyn Fn(&MouseDownEvent, &mut WindowContext)>,
    ) -> Self {
        Self {
            base: div(),
            child: child.into_any_element(),
            on_click,
            variant: ButtonVariant::Primary,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn color(&self, theme: &Theme) -> Hsla {
        match self.variant {
            ButtonVariant::Primary => theme.primary_color,
            ButtonVariant::Danger => theme.danger_color,
        }
    }
}

impl RenderOnce for Button {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        let color = self.color(theme);
        let hover_color = hsla(color.h, color.s, (color.l - 0.08).clamp(0., 1.), color.a);

        self.base
            .p_2()
            .rounded_md()
            .hover(|style| style.bg(hover_color))
            .flex()
            .justify_center()
            .items_center()
            .bg(color)
            .on_mouse_down(MouseButton::Left, self.on_click)
            .child(self.child)
    }
}

impl Styled for Button {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

#[derive(IntoElement, Clone)]
pub struct TextInput {
    focus_handle: FocusHandle,
    view: View<TextDisplay>,
    pub model: Model<TextModel>,
}

impl TextInput {
    pub fn new(cx: &mut WindowContext, initial_text: String) -> Self {
        let model = TextModel::init(initial_text.clone(), cx);
        let clone = model.clone();
        let view = cx.new_view(move |cx| {
            let view = TextDisplay {
                model: clone.clone(),
            };
            cx.subscribe(&clone, |_subscriber, _emitter, event, cx| match event {
                TextEvent::Input { text: _ } => {
                    cx.notify();
                }
                _ => {}
            })
            .detach();
            view
        });
        Self {
            focus_handle: cx.focus_handle(),
            view,
            model,
        }
    }
}

pub struct TextModel {
    pub text: String,
    pub selection: Range<usize>,
    pub word_click: (usize, u16),
}

impl TextModel {
    pub fn init(text: String, cx: &mut WindowContext) -> Model<Self> {
        let i = text.len();
        let m = Self {
            text,
            selection: i..i,
            word_click: (0, 0),
        };
        let model = cx.new_model(|_cx| m);
        cx.subscribe(
            &model,
            |subscriber, emitter: &TextEvent, cx| match emitter {
                TextEvent::Input { text: _ } => {
                    subscriber.update(cx, |editor, _cx| {
                        editor.word_click = (0, 0);
                    });
                }
                _ => {}
            },
        )
        .detach();
        model
    }
    pub fn reset(&mut self, cx: &mut ModelContext<Self>) {
        self.text = "".to_string();
        self.selection = 0..0;
        cx.notify();
        cx.emit(TextEvent::Input {
            text: self.text.clone(),
        });
    }
    pub fn word_ranges(&self) -> Vec<Range<usize>> {
        let mut words = Vec::new();
        let mut last_was_boundary = true;
        let mut word_start = 0;
        let s = self.text.clone();

        for (i, c) in s.char_indices() {
            if c.is_alphanumeric() || c == '_' {
                if last_was_boundary {
                    word_start = i;
                }
                last_was_boundary = false;
            } else {
                if !last_was_boundary {
                    words.push(word_start..i);
                }
                last_was_boundary = true;
            }
        }

        // Check if the last characters form a word and push it if so
        if !last_was_boundary {
            words.push(word_start..s.len());
        }

        words
    }
}

pub enum TextEvent {
    Input { text: String },
    Movement(TextMovement),
}
pub enum TextMovement {
    Up,
    Down,
}

impl EventEmitter<TextEvent> for TextModel {}

impl RenderOnce for TextInput {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        cx.focus(&self.focus_handle);

        let theme = cx.global::<Theme>();

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(move |ev, cx| {
                self.model.update(cx, |editor, cx| {
                    let keystroke = &ev.keystroke.key;
                    if ev.keystroke.modifiers.command {
                        match keystroke.as_str() {
                            "a" => {
                                editor.selection = 0..editor.text.len();
                            }
                            "c" => {
                                let selected_text =
                                    editor.text[editor.selection.clone()].to_string();
                                cx.write_to_clipboard(ClipboardItem::new(selected_text));
                            }
                            "v" => {
                                let clipboard = cx.read_from_clipboard();
                                if let Some(clipboard) = clipboard {
                                    let text = clipboard.text();
                                    editor.text.replace_range(editor.selection.clone(), &text);
                                    let i = editor.selection.start + text.len();
                                    editor.selection = i..i;
                                }
                            }
                            "x" => {
                                let selected_text =
                                    editor.text[editor.selection.clone()].to_string();
                                cx.write_to_clipboard(ClipboardItem::new(selected_text));
                                editor.text.replace_range(editor.selection.clone(), "");
                                editor.selection.end = editor.selection.start;
                            }
                            _ => {}
                        }
                    } else if let Some(ime_key) = &ev.keystroke.ime_key {
                        editor.text.replace_range(editor.selection.clone(), ime_key);
                        let i = editor.selection.start + ime_key.len();
                        editor.selection = i..i;
                    } else {
                        match keystroke.as_str() {
                            "up" => {
                                cx.emit(TextEvent::Movement(TextMovement::Up));
                                return;
                            }
                            "down" => {
                                cx.emit(TextEvent::Movement(TextMovement::Down));
                                return;
                            }
                            "left" => {
                                if editor.selection.start > 0 {
                                    let i = if editor.selection.start == editor.selection.end {
                                        editor.selection.start - 1
                                    } else {
                                        editor.selection.start
                                    };
                                    editor.selection = i..i;
                                }
                            }
                            "right" => {
                                if editor.selection.end < editor.text.len() {
                                    let i = if editor.selection.start == editor.selection.end {
                                        editor.selection.end + 1
                                    } else {
                                        editor.selection.end
                                    };
                                    editor.selection = i..i;
                                }
                            }
                            "backspace" => {
                                if editor.selection.start == editor.selection.end
                                    && editor.selection.start > 0
                                {
                                    // necessary for non-ascii characters
                                    let mut start = editor.text[..editor.selection.start].chars();
                                    start.next_back();
                                    let start = start.as_str();
                                    let i = start.len();
                                    editor.text =
                                        start.to_owned() + &editor.text[editor.selection.end..];
                                    editor.selection = i..i;
                                } else {
                                    editor.text.replace_range(editor.selection.clone(), "");
                                    editor.selection.end = editor.selection.start;
                                }
                            }
                            "enter" => {
                                editor.text.insert(editor.selection.start, '\n');
                                let i = editor.selection.start + 1;
                                editor.selection = i..i;
                            }
                            "escape" => {
                                cx.hide();
                            }
                            keystroke_str => {
                                eprintln!("Unhandled keystroke {keystroke_str}")
                            }
                        };
                    }
                    cx.emit(TextEvent::Input {
                        text: editor.text.clone(),
                    });
                });
            })
            .p_4()
            .w_full()
            .border_1()
            .border_color(theme.border_color)
            .text_color(theme.text_color)
            .focus(|style| style.border_color(theme.primary_color))
            .child(self.view)
    }
}

pub struct TextDisplay {
    model: Model<TextModel>,
}

impl Render for TextDisplay {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        let mut text = self.model.read(cx).text.clone();
        let mut selection_style = HighlightStyle::default();

        selection_style.background_color = Some(hsla(0., 0., 0.9, 1.));

        let sel = self.model.read(cx).selection.clone();
        let mut highlights = vec![(sel, selection_style)];

        let mut style = TextStyle::default();
        style.color = theme.text_color;
        if text.len() == 0 {
            text = "Type here...".to_string();
            style.color = theme.text_color;
            highlights = vec![];
        }

        let styled_text = StyledText::new(text + " ").with_highlights(&style, highlights);
        let clone = self.model.clone();
        InteractiveText::new("text", styled_text).on_click(
            self.model.read(cx).word_ranges(),
            move |ev, cx| {
                clone.update(cx, |editor, cx| {
                    let (index, mut count) = editor.word_click;
                    if index == ev {
                        count += 1;
                    } else {
                        count = 1;
                    }
                    match count {
                        2 => {
                            let word_ranges = editor.word_ranges();
                            editor.selection = word_ranges.get(ev).unwrap().clone();
                        }
                        3 => {
                            // Should select the line
                        }
                        4 => {
                            count = 0;
                            editor.selection = 0..editor.text.len();
                        }
                        _ => {}
                    }
                    editor.word_click = (ev, count);
                    cx.notify();
                });
            },
        )
    }
}

#[allow(dead_code)]
enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(IntoElement)]
pub struct Divider {
    orientation: Orientation,
}

impl Divider {
    pub fn horizontal() -> Self {
        Self {
            orientation: Orientation::Horizontal,
        }
    }

    #[allow(dead_code)]
    pub fn vertical() -> Self {
        Self {
            orientation: Orientation::Vertical,
        }
    }
}

impl RenderOnce for Divider {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let base = div()
            .flex()
            .border()
            .border_width(px(0.5))
            .border_color(theme.border_color);
        match self.orientation {
            Orientation::Horizontal => base.h_0(),
            Orientation::Vertical => div().w_0(),
        }
    }
}
