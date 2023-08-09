use {
    crate::{inlays::InlineInlay, widgets::InlineWidget, wrap::WrapData, Token},
    std::slice::Iter,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line<'a> {
    pub y: Option<f64>,
    pub column_count: Option<usize>,
    pub fold_column: usize,
    pub scale: f64,
    pub text: &'a str,
    pub tokens: &'a [Token],
    pub inline_inlays: &'a [(usize, InlineInlay)],
    pub wrap_data: Option<&'a WrapData>,
}

impl<'a> Line<'a> {
    pub fn y(&self) -> f64 {
        self.y.unwrap()
    }

    pub fn row_count(&self) -> usize {
        self.wrap_data.unwrap().wraps.len() + 1
    }

    pub fn column_count(&self) -> usize {
        self.column_count.unwrap()
    }

    pub fn height(&self) -> f64 {
        self.row_count() as f64 * self.scale
    }

    pub fn width(&self) -> f64 {
        self.column_to_x(self.column_count())
    }

    pub fn column_to_x(&self, column: usize) -> f64 {
        let column_count_before_fold = column.min(self.fold_column);
        let column_count_after_fold = column - column_count_before_fold;
        column_count_before_fold as f64 + column_count_after_fold as f64 * self.scale
    }

    pub fn fold_column(&self) -> usize {
        self.fold_column
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn wrap_indent_column_count(self) -> usize {
        self.wrap_data.unwrap().indent_column_count
    }

    pub fn text(&self) -> &str {
        self.text
    }

    pub fn tokens(&self) -> &[Token] {
        self.tokens
    }

    pub fn inlines(&self) -> Inlines<'a> {
        Inlines {
            text: self.text,
            inline_inlays: self.inline_inlays.iter(),
            position: 0,
        }
    }

    pub fn wrappeds(&self) -> Wrappeds<'a> {
        let mut inlines = self.inlines();
        Wrappeds {
            inline: inlines.next(),
            inlines,
            wraps: self.wrap_data.unwrap().wraps.iter(),
            position: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Inlines<'a> {
    pub(super) text: &'a str,
    pub(super) inline_inlays: Iter<'a, (usize, InlineInlay)>,
    pub(super) position: usize,
}

impl<'a> Iterator for Inlines<'a> {
    type Item = Inline<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self
            .inline_inlays
            .as_slice()
            .first()
            .map_or(false, |&(position, _)| position == self.position)
        {
            let (_, inline_inlay) = self.inline_inlays.next().unwrap();
            return Some(match *inline_inlay {
                InlineInlay::Text(ref text) => Inline::Text {
                    is_inlay: true,
                    text,
                },
                InlineInlay::Widget(widget) => Inline::Widget(widget),
            });
        }
        if self.text.is_empty() {
            return None;
        }
        let mut mid = self.text.len();
        if let Some(&(byte, _)) = self.inline_inlays.as_slice().first() {
            mid = mid.min(byte - self.position);
        }
        let (text_0, text_1) = self.text.split_at(mid);
        self.text = text_1;
        self.position += text_0.len();
        Some(Inline::Text {
            is_inlay: false,
            text: text_0,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Inline<'a> {
    Text { is_inlay: bool, text: &'a str },
    Widget(InlineWidget),
}

#[derive(Clone, Debug)]
pub struct Wrappeds<'a> {
    pub(super) inline: Option<Inline<'a>>,
    pub(super) inlines: Inlines<'a>,
    pub(super) wraps: Iter<'a, usize>,
    pub(super) position: usize,
}

impl<'a> Iterator for Wrappeds<'a> {
    type Item = Wrapped<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self
            .wraps
            .as_slice()
            .first()
            .map_or(false, |&position| position == self.position)
        {
            self.wraps.next();
            return Some(Wrapped::Wrap);
        }
        Some(match self.inline.take()? {
            Inline::Text { is_inlay, text } => {
                let mut mid = text.len();
                if let Some(&position) = self.wraps.as_slice().first() {
                    mid = mid.min(position - self.position);
                }
                let text = if mid < text.len() {
                    let (text_0, text_1) = text.split_at(mid);
                    self.inline = Some(Inline::Text {
                        is_inlay,
                        text: text_1,
                    });
                    text_0
                } else {
                    self.inline = self.inlines.next();
                    text
                };
                self.position += text.len();
                Wrapped::Text { is_inlay, text }
            }
            Inline::Widget(widget) => {
                self.position += 1;
                Wrapped::Widget(widget)
            }
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Wrapped<'a> {
    Text { is_inlay: bool, text: &'a str },
    Widget(InlineWidget),
    Wrap,
}
