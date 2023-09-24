use eframe::egui::{Separator, Ui, WidgetText};

use crate::model::{DocumentationStructure, FieldInfo, FieldType};

pub fn show(ui: &mut Ui, documentation_structure: &Option<DocumentationStructure>) {
    if let Some(doc) = documentation_structure {
        ui.label(&doc.description);

        if !doc.fields.is_empty() {
            ui.add_space(10.);
            ui.add(Separator::default().spacing(10.));
        }

        let fields: Vec<(&String, &FieldInfo)> = doc.fields.iter().to_owned().collect();
        for f in fields {
            let field_type_descr = match f.1.field_type {
                FieldType::Normal => "",
                FieldType::Optional => "(optional)",
                FieldType::List => "(list)",
                FieldType::OptionalList => "(list) (optional)",
            };

            ui.add_space(10.);
            ui.horizontal(|ui| {
                ui.strong(format!("{}:", f.0));
                ui.label(WidgetText::from(field_type_descr).italics());
            });
            ui.label(&f.1.description);
        }
    }
}
