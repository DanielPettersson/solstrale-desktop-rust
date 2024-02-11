use std::sync::Arc;

use crate::model::DocumentationStructure;
use eframe::egui;
use eframe::egui::text::{LayoutJob, LayoutSection};
use eframe::egui::{Context, Galley, Id, TextBuffer, TextEdit, TextFormat, Ui, Vec2};
use egui::util::cache::{ComputerMut, FrameCache};
use once_cell::sync::Lazy;
use regex::Regex;

pub static YAML_EDITOR_ID: Lazy<Id> = Lazy::new(|| Id::from("yaml_editor"));
static INDENTATION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("^[\\s-]*").unwrap());
static TEMPLATE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("\\{%.*%}").unwrap());
static YAML_KEY_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("^([\\s-]*)([\\w_]+):").unwrap());
static AUTOCOMPLETE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("^[\\s-]*([\\w_]+)$").unwrap());

pub fn yaml_editor<'a, L>(
    text: &'a mut dyn TextBuffer,
    layouter: &'a mut L,
    min_size: Vec2,
) -> TextEdit<'a>
where
    L: Fn(&Ui, &str, f32) -> Arc<Galley>,
{
    TextEdit::multiline(text)
        .id(*YAML_EDITOR_ID)
        .code_editor()
        .min_size(min_size)
        .layouter(layouter)
}

fn cursor_char_offset(ctx: &Context) -> Option<usize> {
    TextEdit::load_state(ctx, *YAML_EDITOR_ID)
        .and_then(|state| state.cursor.char_range().map(|range| range.primary.index))
}

pub fn get_yaml_path(yaml: &dyn TextBuffer, ctx: &Context) -> Vec<String> {
    match cursor_char_offset(ctx) {
        None => vec![],
        Some(idx) => {
            let mut max_indentation: usize = usize::MAX;
            let mut ret = Vec::new();

            for line in yaml.char_range(0..idx).lines().rev() {
                if TEMPLATE_REGEX.captures(line).is_some() {
                    continue;
                }

                if let Some(cap) = YAML_KEY_REGEX.captures(line) {
                    let indentation = cap.get(1).unwrap().len();
                    if indentation < max_indentation {
                        ret.push(cap.get(2).unwrap().as_str().to_owned());
                    }
                }

                if let Some(m) = INDENTATION_REGEX.find(line) {
                    let indentation = m.as_str().len();
                    if indentation < max_indentation {
                        max_indentation = indentation;
                    }
                }
            }
            ret.reverse();
            ret
        }
    }
}

pub fn autocomplete(text: &mut dyn TextBuffer, doc: &DocumentationStructure, ctx: &Context) {
    if doc.fields.is_empty() {
        return;
    }

    if let Some(mut state) = TextEdit::load_state(ctx, *YAML_EDITOR_ID) {
        if let Some(range) = state.cursor.char_range() {
            let idx = range.primary.index;

            if let Some(last_line) = text.char_range(0..idx).lines().last() {
                if let Some(cap) = AUTOCOMPLETE_REGEX.captures(last_line) {
                    let autocomplete_key = cap.get(1).unwrap().as_str().to_owned();

                    if let Some(autocomplete_val) = doc
                        .fields
                        .iter()
                        .find(|f| f.0.starts_with(&autocomplete_key))
                        .map(|f| f.0[autocomplete_key.len()..].to_owned())
                    {
                        let ins = format!("{}: ", autocomplete_val);
                        text.insert_text(&ins, idx);

                        let cursor = egui::text::CCursor::new(idx + ins.len());
                        state
                            .cursor
                            .set_char_range(Some(egui::text::CCursorRange::one(cursor)));
                        state.store(ctx, *YAML_EDITOR_ID);
                    }
                }
            }
        }
    }
}

pub fn indent_new_line(text: &mut dyn TextBuffer, ctx: &Context) {
    if let Some(mut state) = TextEdit::load_state(ctx, *YAML_EDITOR_ID) {
        if let Some(range) = state.cursor.char_range() {
            let idx = range.primary.index;

            if let Some(last_line) = text.char_range(0..idx).lines().last() {
                let num_spaces_at_start = INDENTATION_REGEX
                    .find(last_line)
                    .map(|m| m.len())
                    .unwrap_or(0);
                let ends_with_colon = last_line.trim().chars().last().unwrap_or(' ') == ':';
                let space_indent = num_spaces_at_start + if ends_with_colon { 2 } else { 0 };

                text.insert_text(&" ".repeat(space_indent), idx);

                let cursor = egui::text::CCursor::new(idx + space_indent);
                state
                    .cursor
                    .set_char_range(Some(egui::text::CCursorRange::one(cursor)));
                state.store(ctx, *YAML_EDITOR_ID);
            }
        }
    }
}

pub fn create_layouter() -> fn(&Ui, &str, f32) -> Arc<Galley> {
    |ui: &Ui, string: &str, _wrap_width: f32| {
        let layout_job = highlight(ui.ctx(), string);
        ui.fonts(|f| f.layout_job(layout_job))
    }
}

fn highlight(ctx: &Context, code: &str) -> LayoutJob {
    impl ComputerMut<&str, LayoutJob> for Highlighter {
        fn compute(&mut self, code: &str) -> LayoutJob {
            self.highlight(code)
        }
    }

    type HighlightCache = FrameCache<LayoutJob, Highlighter>;
    ctx.memory_mut(|mem| mem.caches.cache::<HighlightCache>().get(code))
}

struct Highlighter {
    ps: syntect::parsing::SyntaxSet,
    ts: syntect::highlighting::ThemeSet,
}

impl Default for Highlighter {
    fn default() -> Self {
        Self {
            ps: syntect::parsing::SyntaxSet::load_defaults_newlines(),
            ts: syntect::highlighting::ThemeSet::load_defaults(),
        }
    }
}

impl Highlighter {
    fn highlight(&self, code: &str) -> LayoutJob {
        self.highlight_impl(code).unwrap_or_else(|| {
            LayoutJob::simple(
                code.into(),
                egui::FontId::monospace(12.0),
                egui::Color32::LIGHT_GRAY,
                f32::INFINITY,
            )
        })
    }

    fn highlight_impl(&self, text: &str) -> Option<LayoutJob> {
        use syntect::easy::HighlightLines;
        use syntect::highlighting::FontStyle;
        use syntect::util::LinesWithEndings;

        let syntax = self
            .ps
            .find_syntax_by_name("yaml")
            .or_else(|| self.ps.find_syntax_by_extension("yaml"))?;

        let mut h = HighlightLines::new(syntax, &self.ts.themes["base16-mocha.dark"]);

        let mut job = LayoutJob {
            text: text.into(),
            ..Default::default()
        };

        for line in LinesWithEndings::from(text) {
            for (style, range) in h.highlight_line(line, &self.ps).ok()? {
                let fg = style.foreground;
                let text_color = egui::Color32::from_rgb(fg.r, fg.g, fg.b);
                let italics = style.font_style.contains(FontStyle::ITALIC);
                let underline = style.font_style.contains(FontStyle::ITALIC);
                let underline = if underline {
                    egui::Stroke::new(1.0, text_color)
                } else {
                    egui::Stroke::NONE
                };
                job.sections.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: as_byte_range(text, range),
                    format: TextFormat {
                        font_id: egui::FontId::monospace(12.0),
                        color: text_color,
                        italics,
                        underline,
                        ..Default::default()
                    },
                });
            }
        }

        Some(job)
    }
}

fn as_byte_range(whole: &str, range: &str) -> std::ops::Range<usize> {
    let whole_start = whole.as_ptr() as usize;
    let range_start = range.as_ptr() as usize;
    assert!(whole_start <= range_start);
    assert!(range_start + range.len() <= whole_start + whole.len());
    let offset = range_start - whole_start;
    offset..(offset + range.len())
}
