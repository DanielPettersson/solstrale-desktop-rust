use std::sync::Arc;
use eframe::egui;
use eframe::egui::text::{LayoutJob, LayoutSection};
use eframe::egui::{Galley, TextBuffer, TextEdit, TextFormat, Ui, Vec2};
use egui::util::cache::{ComputerMut, FrameCache};

pub fn yaml_editor<'a>(text: &'a mut dyn TextBuffer, layouter: &'a mut fn(&Ui, &str, f32) -> Arc<Galley>, min_size: Vec2) -> TextEdit<'a> {
    TextEdit::multiline(text)
        .code_editor()
        .desired_width(f32::INFINITY)
        .min_size(min_size)
        .layouter(layouter)
}

pub fn create_layouter() -> fn(&Ui, &str, f32) -> Arc<Galley> {
    |ui: &Ui, string: &str, _wrap_width: f32| {
        let layout_job = highlight(ui.ctx(), string);
        ui.fonts(|f| f.layout_job(layout_job))
    }
}

fn highlight(ctx: &egui::Context, code: &str) -> LayoutJob {
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
            // Fallback:
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
