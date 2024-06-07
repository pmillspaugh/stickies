use std::{collections::HashMap, sync::mpsc};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct AppState {
    #[serde(skip_serializing, skip_deserializing)]
    effects_tx: mpsc::Sender<Effect>,
    #[serde(skip_serializing, skip_deserializing)]
    effects_rx: mpsc::Receiver<Effect>,

    draft: String,
    todos: Vec<Todo>,
    calculated: HashMap<String, f32>,
}

impl Default for AppState {
    fn default() -> Self {
        let (effects_tx, effects_rx) = mpsc::channel();

        Self {
            effects_tx,
            effects_rx,

            draft: "Feed doge".to_owned(),
            todos: vec![],
            calculated: HashMap::new(),
        }
    }
}

impl AppState {
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

    fn apply_effects(&mut self, ctx: &egui::Context) {
        while let Ok(effect) = self.effects_rx.try_recv() {
            match effect {
                Effect::DraftTodo(draft) => {
                    self.draft = draft;
                }
                Effect::AddTodo(label) => {
                    let id = ctx.memory_mut(|mem| {
                        let counter: &mut usize = mem.data.get_persisted_mut_or_default(egui::Id::new("counter"));
                        let id = egui::Id::new(*counter);
                        *counter += 1;
                        id
                    });

                    self.todos.push(Todo::new(id, label));
                    self.draft.clear();
                }
                Effect::EditTodo(index) => {
                    if let Some(todo) = self.todos.get_mut(index) {
                        todo.edit_mode = !todo.edit_mode;
                    }
                }
                Effect::SaveTodo(index, label) => {
                    if let Some(todo) = self.todos.get_mut(index) {
                        todo.label = label;
                    }
                }
                Effect::CheckTodo(index) => {
                    if let Some(todo) = self.todos.get_mut(index) {
                        todo.checked = !todo.checked;
                    }
                }
                Effect::DeleteTodo(index) => {
                    if index < self.todos.len() {
                        self.todos.remove(index);
                    }
                }

                Effect::InsertCalculated(name, value) => {
                    self.calculated.insert(name, value);
                    // self.calculated.clear();
                }
            }
        }
    }

    fn render(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Stickies");
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                // Center the elements using the stored width from the previous frame
                // TODO: to prevent flicker, the first frame should only calculate size and not actually render
                let id = "draft_todo";
                if let Some(stored_width) = self.calculated.get(id) {
                    let offset = (ui.available_width() - stored_width) / 2.0;
                    ui.add_space(offset);
                }

                ui.label("Add a sticky: ");

                let mut local_draft = self.draft.clone();
                if ui.text_edit_singleline(&mut local_draft).lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                {
                    self.effects_tx
                        .send(Effect::AddTodo(local_draft.clone()))
                        .unwrap();
                    local_draft.clear();
                }

                if ui.button("Save").clicked() {
                    self.effects_tx
                        .send(Effect::AddTodo(local_draft.clone()))
                        .unwrap();
                    local_draft.clear();
                }

                self.effects_tx
                    .send(Effect::DraftTodo(local_draft))
                    .unwrap();

                // Store the width for the next frame if this is the first frame
                if let None = self.calculated.get(id) {
                    self.effects_tx
                        .send(Effect::InsertCalculated(
                            id.to_string(),
                            ui.min_rect().width(),
                        ))
                        .unwrap();
                }
            });

            ui.add_space(10.0);

            for (index, todo) in self.todos.iter().enumerate() {
                let window = egui::Window::new(todo.label.clone())
                    .id(todo.id)
                    .resizable(false)
                    .collapsible(false)
                    .title_bar(false);
                
                window.show(ui.ctx(), |ui| {
                    // Note: I could not get `.fixed_size()`, `min_size()`, nor `.default_size()`
                    // to work on the Window instance itself, so this is a workaround
                    ui.set_min_size(egui::Vec2::new(150.0, 150.0));
                    ui.set_max_size(egui::Vec2::new(150.0, 150.0));

                    let mut local_label = todo.label.clone();
                    
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            let id = "todo_actions";
                            let container_width = ui.available_width();
                            
                            let mut local_checked = todo.checked;
                            if ui.checkbox(&mut local_checked, "").changed() {
                                self.effects_tx.send(Effect::CheckTodo(index)).unwrap();
                            }

                            // We want to right justify the Edit and Delete buttons
                            if let Some(stored_width) = self.calculated.get(id) {
                                let offset = container_width - stored_width;
                                ui.add_space(offset);
                            }

                            if todo.edit_mode {
                                if ui.button("Save").clicked() {
                                    self.effects_tx.send(Effect::EditTodo(index)).unwrap();
                                    self.effects_tx
                                        .send(Effect::SaveTodo(index, local_label.clone()))
                                        .unwrap();
                                }
                            } else {
                                if ui.button("Edit").clicked() {
                                    self.effects_tx.send(Effect::EditTodo(index)).unwrap();
                                }
                            }
    
                            if ui.button("Delete").clicked() {
                                self.effects_tx.send(Effect::DeleteTodo(index)).unwrap();
                            }

                            if let None = self.calculated.get(id) {
                                self.effects_tx
                                    .send(Effect::InsertCalculated(
                                        id.to_string(),
                                        ui.min_rect().width(),
                                    ))
                                    .unwrap();
                            }
                        });

                        let id = "todo_text";
                        let container_height = ui.available_height();
                        if let Some(stored_height) = self.calculated.get(id) {
                            let offset = (container_height - stored_height) / 2.0;
                            ui.add_space(offset);
                        }
                        
                        ui.vertical_centered(|ui| {
                            if todo.edit_mode {
                                if ui.text_edit_singleline(&mut local_label).lost_focus()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                                {
                                    self.effects_tx.send(Effect::EditTodo(index)).unwrap();
                                    self.effects_tx
                                        .send(Effect::SaveTodo(index, local_label.clone()))
                                        .unwrap();
                                }

                                self.effects_tx
                                    .send(Effect::SaveTodo(index, local_label.clone()))
                                    .unwrap();
                            } else {
                                ui.add(egui::Label::new(&todo.label).wrap(true));
                            }
                        });

                        if let None = self.calculated.get(id) {
                            self.effects_tx
                                .send(Effect::InsertCalculated(
                                    id.to_string(),
                                    container_height - ui.available_height(),
                                ))
                                .unwrap();
                        }
                    });
                });
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
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
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

enum Effect {
    DraftTodo(String),
    AddTodo(String),
    EditTodo(usize),
    SaveTodo(usize, String),
    CheckTodo(usize),
    DeleteTodo(usize),

    InsertCalculated(String, f32),
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Todo {
    id: egui::Id,
    label: String,
    checked: bool,
    edit_mode: bool,
}

impl Todo {
    fn new(id: egui::Id, label: String) -> Self {
        Self {
            id,
            label: label.into(),
            checked: false,
            edit_mode: false,
        }
    }
}

impl eframe::App for AppState {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render(ctx);
        self.apply_effects(ctx);
    }
}
