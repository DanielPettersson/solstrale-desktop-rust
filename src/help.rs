use eframe::egui::Ui;

use crate::model::{DocumentationStructure, FieldInfo, FieldType};

pub fn show(ui: &mut Ui, documentation_structure: &Option<DocumentationStructure>) {
    if let Some(doc) = documentation_structure {
        ui.heading(&doc.description);

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