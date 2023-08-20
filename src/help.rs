use eframe::egui::{TextBuffer, Ui};
use once_cell::sync::Lazy;
use regex::Regex;
use crate::model::{DocumentationStructure, FieldInfo, FieldType, get_documentation_structure_by_path, HelpDocumentation};
use crate::model::scene::Scene;

static DOCUMENTATION_STRUCTURE: Lazy<DocumentationStructure> = Lazy::new(Scene::get_documentation_structure);

pub fn show(ui: &mut Ui, yaml_cursor_idx: Option<usize>, yaml: &dyn TextBuffer) {
    if let Some(idx) = yaml_cursor_idx {
        let help_path = get_help_identifier(idx, yaml);
        if let Some(doc) = get_documentation_structure_by_path(&DOCUMENTATION_STRUCTURE, &help_path) {
            ui.heading(doc.description);

            let mut fields: Vec<(&String, &FieldInfo)> = doc.fields.iter().to_owned().collect();
            fields.sort_by_key(|f| f.0);
            for f in fields {
                let field_type_descr = match f.1.field_type {
                    FieldType::Normal => "",
                    FieldType::Optional => "(optional)",
                    FieldType::List => "(list)",
                };

                ui.label(format!("{} {}: {}", f.0, field_type_descr, f.1.description));
            }
        }
    }
}

fn get_help_identifier(idx: usize, yaml: &dyn TextBuffer) -> Vec<String> {
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
    ret.reverse();
    ret
}