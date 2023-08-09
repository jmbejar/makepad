pub use makepad_widgets;
use makepad_widgets::*;

pub mod change;
pub mod char;
pub mod code_editor;
pub mod extent;
pub mod inlays;
pub mod iter;
pub mod line;
pub mod point;
pub mod range;
pub mod selection;
pub mod settings;
pub mod state;
pub mod str;
pub mod text;
pub mod token;
pub mod widgets;
pub mod wrap;

pub use self::{
    state::{Session,Document},
    change::Change, code_editor::CodeEditor, extent::Extent, line::Line, point::Point,
    range::Range, selection::Selection, settings::Settings, text::Text, token::Token,
};

pub fn live_design(cx: &mut Cx) {
    crate::code_editor::live_design(cx);
}
