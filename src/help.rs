use eframe::egui::{TextBuffer, Ui};
use regex::Regex;

pub fn show(ui: &mut Ui, yaml_cursor_idx: Option<usize>, yaml: &dyn TextBuffer) {
    if let Some(idx) = yaml_cursor_idx {
        if let Some(identifier) = get_help_identifier(idx, yaml) {
            ui.label(identifier);
        }
    }
}

fn get_help_identifier(idx: usize, yaml: &dyn TextBuffer) -> Option<String> {
    let indentation_regex = Regex::new("^[\\s-]*").unwrap();
    let regex = Regex::new("^([\\s-]*)([\\w_]+):").unwrap();
    let mut max_indentation: usize = usize::MAX;
    let mut ret = Vec::new();

    for line in yaml.char_range(0..idx).lines().rev() {

        if let Some(cap) = regex.captures(line) {
            let indentation = cap.get(1).unwrap().len();
            if indentation < max_indentation {
                ret.push(cap.get(2).unwrap().as_str().to_owned());
            }
        }

        if let Some(m) = indentation_regex.find(line) {
            let indentation = m.as_str().len();
            if indentation < max_indentation {
                max_indentation = indentation;
            }
        }
    }
    if ret.is_empty() {
        None
    } else {
        ret.reverse();
        Some(ret.join("."))
    }
}