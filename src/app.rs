/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct StickiesApp {
    draft: String,
    checked: bool,
    todos: Vec<Todo>,
}

impl Default for StickiesApp {
    fn default() -> Self {
        Self {
            draft: "Feed doge".to_owned(),
            checked: false,
            todos: vec![Todo::new(String::from("Water plants"))],
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Todo {
    label: String,
    checked: bool,
    edit_mode: bool,
}

impl Todo {
    fn new(label: String) -> Self {
        Self {
            label: label.into(),
            checked: false,
            edit_mode: false,
        }
    }
}

impl StickiesApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for StickiesApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("stickies");

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Add a sticky: ");
                ui.text_edit_singleline(&mut self.draft);

                // TODO: pressing Enter should also save a todo
                if ui.button("Save").clicked() {
                    self.todos.push(Todo::new(String::from(self.draft.clone())));
                    self.draft.clear();
                }
            });

            ui.add_space(10.0);

            let mut to_delete = None;
            for todo in &mut self.todos {
                // TODO: make this a draggable sticky instead of a row
                ui.horizontal(|ui| {
                    ui.checkbox(&mut todo.checked, "");
                    
                    if todo.edit_mode {
                        ui.text_edit_singleline(&mut todo.label);
                    } else {
                        ui.label(todo.label.clone());
                    }

                    let edit_button_text = if todo.edit_mode { "Save" } else { "Edit" };
                    if ui.button(edit_button_text).on_hover_text("Re-write sticky").clicked() {
                        todo.edit_mode = !todo.edit_mode;
                    }

                    if ui.button("Delete").on_hover_text("Throw away sticky").clicked() {
                        to_delete = Some(todo.label.clone());
                    }
                });
            }

            if let Some(to_delete) = to_delete {
                self.todos.retain(|t| t.label != to_delete);
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.add(egui::github_link_file!(
            "https://github.com/pmillspaugh/stickies/blob/main/",
            "Source code. "
        ));
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
