use crate::recipe::{Recipe, WidgetMetadata};
use eframe::egui::{self};
use eframe::egui::{IconData, ViewportCommand};
use indexmap::map::Entry;
use std::sync::Arc;

struct SmApp {
    recipe: Recipe,
    metadata: WidgetMetadata,
    dropped_files: Vec<egui::DroppedFile>,
    show_confirmation_dialog: bool,
    allowed_to_close: bool,
    show_about: bool,
}

pub fn sm_gui(recipe: Recipe, metadata: WidgetMetadata) -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let mut state: SmApp = SmApp {
        recipe,
        metadata,
        dropped_files: vec![],
        show_confirmation_dialog: false,
        allowed_to_close: false,
        show_about: false,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_simple_native("smoothie-app", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let icon = include_bytes!("../../smoothie-32x.png");
            let image = image::load_from_memory(icon)
                .expect("Image could not be loaded from memory")
                .into_rgba8();

            ctx.send_viewport_cmd(ViewportCommand::Icon(Some(Arc::new(IconData {
                rgba: image.into_raw(),
                height: 32,
                width: 32,
            }))));

            ui.horizontal(|ui| {
                if ui.button("?").clicked() {
                    state.show_about = true
                }
                ui.label(" | ");
                egui::widgets::global_dark_light_mode_buttons(ui);
            });

            for cat in &mut state.metadata.keys() {
                ui.heading(cat);
                for def in &mut state.recipe.get_section(cat).keys() {
                    let def_metadata = &mut state.metadata.get(cat).unwrap().get(def).unwrap();
                    ui.label(def);

                    if def_metadata.get("type").is_none() {
                        dbg!(&cat);
                        dbg!(&def);
                        dbg!(&def_metadata);
                    }

                    match def_metadata.get("type").unwrap().as_str() {
                        "bool" => {}
                        "path" => {}
                        "float_slider" => {}
                        "int_slider" => {}
                        // "enum" => {

                        //     let values = def_metadata.get("values").unwrap();
                        //     let selected = my_app.recipe.get(cat, def);

                        //     egui::ComboBox::from_label("Take your pick")
                        //     .selected_text(format!("{selected:?}"))
                        //     .show_ui(ui, |ui| {
                        //         ui.style_mut().wrap = Some(false);
                        //         ui.set_min_width(60.0);
                        //         ui.selectable_value(&mut selected, selected, selected);
                        //         ui.selectable_value(&mut selected, selected, selected);
                        //         ui.selectable_value(&mut selected, selected, selected);
                        //     });
                        // }
                        "string" => {
                            // match &mut my_app.recipe.entry(cat.to_string()) {

                            //     Entry::Occupied(entry) => {

                            //         match entry.get().entry(def.to_string()) {

                            //             Entry::Occupied(entry) => {

                            //                 ui.add(
                            //                     egui::TextEdit::singleline(
                            //                         &mut *entry.get_mut())
                            //                         .hint_text(def),
                            //                 );
                            //             }
                            //             Entry::Vacant(_) => {todo!()}
                            //         }
                            //     }
                            //     Entry::Vacant(_) => {todo!()}
                            // }
                            let response =
                                ui.add(egui::TextEdit::singleline(&mut state.recipe.get(cat, def)));
                            if response.changed() {
                                dbg!(&response);
                            }
                        }

                        _ => {}
                    }
                }
            }

            if !state.dropped_files.is_empty() {
                for file in &state.dropped_files {
                    dbg!(&file);
                }
            }

            preview_files_being_dropped(ctx);

            if state.show_about {
                egui::Window::new("about smoothie app")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.heading("\nwhat is this");
                        ui.label(
                            "user interface to edit the recipe.ini text file more conveniently",
                        );
                        ui.heading("\nhow to use");
                        ui.label("set, save, launch smoothie from start menu/launch.bat/send to");
                        ui.heading("\nlinks");
                        ui.hyperlink_to("documentation", "https://ctt.cx/sm");
                        ui.hyperlink_to(
                            "github\n",
                            "https://github.com/couleur-tweak-tips/smoothie-rs",
                        )
                        .on_hover_text_at_pointer("see /smrs-egui/ folder");

                        if ui.button("ok").clicked() {
                            state.show_about = false;
                        }
                    });
            }

            if state.show_confirmation_dialog {
                egui::Window::new("Do you want to quit?")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("No").clicked() {
                                state.show_confirmation_dialog = false;
                                state.allowed_to_close = false;
                            }

                            if ui.button("Yes").clicked() {
                                state.show_confirmation_dialog = false;
                                state.allowed_to_close = true;
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        });
                    });
            }
        });
    })
}

/// Preview hovering files:
fn preview_files_being_dropped(ctx: &egui::Context) {
    use egui::*;
    use std::fmt::Write as _;

    if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
        let text = ctx.input(|i| {
            let mut text = "".to_owned();
            for file in &i.raw.hovered_files {
                if let Some(path) = &file.path {
                    write!(text, "\n{}", path.display()).ok();
                } else if !file.mime.is_empty() {
                    write!(text, "\n{}", file.mime).ok();
                } else {
                    text += "\n???";
                }
            }
            text
        });

        let painter =
            ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

        let screen_rect = ctx.screen_rect();
        painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
        painter.text(
            screen_rect.center(),
            Align2::CENTER_CENTER,
            text,
            TextStyle::Heading.resolve(&ctx.style()),
            Color32::WHITE,
        );
    }
}
