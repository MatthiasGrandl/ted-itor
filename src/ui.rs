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
    pub text_display_view: View<TextDisplay>,
    focus_handle: FocusHandle,
}

impl TextInput {
    pub fn new(cx: &mut WindowContext, initial_text: String) -> Self {
        let i = initial_text.len();
        Self {
            text_display_view: cx.new_view(|_cx| TextDisplay {
                text: initial_text,
                selection: i..i,
                word_click: Arc::new(Mutex::new((0, 0))),
            }),
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn clear(self, cx: &mut WindowContext) {
        self.text_display_view.update(cx, |text_display, cx| {
            text_display.text = String::from("");
            cx.notify();
        })
    }
}

impl RenderOnce for TextInput {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        let text_display_view = self.text_display_view.clone();

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(move |ev, cx| {
                text_display_view.update(cx, |editor, cx| {
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
                            keystroke_str => {
                                eprintln!("Unhandled keystroke {keystroke_str}")
                            }
                        };
                    }

                    cx.notify();
                });
            })
            .p_4()
            .border_l()
            .border_color(transparent_black())
            .focus(|style| style.border_color(theme.border_color))
            .child(self.text_display_view)
    }
}

#[derive(Clone)]
pub struct TextDisplay {
    // TODO: Use Arc<String>? Other places we can reduce clones?
    pub text: String,
    pub selection: Range<usize>,
    pub word_click: Arc<Mutex<(usize, u16)>>,
}

fn split_into_words(s: &str) -> Vec<Range<usize>> {
    let mut words = Vec::new();
    let mut last_was_boundary = true;
    let mut word_start = 0;

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

impl Render for TextDisplay {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let text = self.text.clone();
        let mut selection_style = HighlightStyle::default();
        selection_style.background_color = Some(hsla(0., 0., 0.9, 1.));

        let sel = self.selection.clone();
        let highlights = vec![(sel, selection_style)];
        let word_ranges = split_into_words(text.as_str());
        let word_ranges_clone = word_ranges.clone();
        let styled_text =
            StyledText::new(text + " ").with_highlights(&TextStyle::default(), highlights);
        let view = cx.view().clone();
        let clicked = self.word_click.clone();

        InteractiveText::new("text", styled_text).on_click(word_ranges, move |ev, cx| {
            let mut c = clicked.lock().unwrap();
            if c.0 == ev {
                *c = (ev, c.1 + 1);
            } else {
                *c = (ev, 1);
            }

            match c.1 {
                2 => {
                    cx.update_view(&view, |editor, cx| {
                        editor.selection = word_ranges_clone[ev].clone();
                        cx.notify();
                    });
                }
                3 => {
                    // Should select the line
                }
                4 => {
                    *c = (0, 0);
                    cx.update_view(&view, |editor, cx| {
                        editor.selection = 0..editor.text.len();
                        cx.notify();
                    });
                }
                _ => {}
            }
        })
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
