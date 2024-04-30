use crate::{
    cli::Arguments,
    recipe::{export_recipe, Recipe, WidgetMetadata},
};
use std::{
    fs::File, io::Write,
    path::PathBuf,
    sync::mpsc::Sender // used to retrieve SmCommands
};
use copypasta::{ClipboardContext, ClipboardProvider};
use eframe::egui;
use winit::raw_window_handle::HasWindowHandle;

struct SmApp {
    recipe: Recipe,
    metadata: WidgetMetadata,
    selected_files: Vec<PathBuf>,
    show_confirmation_dialog: bool,
    allowed_to_close: bool,
    show_about: bool,
    recipe_filepath: String,
    args: Arguments,
    start_rendering: bool,
    // yeah that's the damn typename
    recipe_saved: String,
    sender: Sender<(Recipe, Arguments, Option<windows::Win32::Foundation::HWND>)>
}

pub const WINDOW_NAME: &str = "smoothie-app";

pub fn save_recipe(recipe: &Recipe, recipe_filename: &PathBuf, metadata: &WidgetMetadata) {
    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(e) => panic!("Could not resolve Smoothie's binary path: {}", e),
    };

    let bin_dir = match exe.parent() {
        Some(bin_dir) => bin_dir.parent().unwrap(),
        None => panic!("Could not resolve Smoothie's binary directory `{exe:?}`"),
    };

    let recipe_path = if PathBuf::from(recipe_filename.clone()).exists() {
        PathBuf::from(recipe_filename)
    } else {
        let cur_dir_rc = bin_dir.join(recipe_filename);
        if !cur_dir_rc.exists() {
            panic!(
                "Recipe filepath does not exist (expected at {})",
                cur_dir_rc.display()
            )
        }
        cur_dir_rc
    };

    let content = export_recipe(recipe.clone(), &metadata.clone(), false, true, false);

    // Attempt to create a new file, or truncate an existing file
    let mut file = match File::create(recipe_path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Error creating file: {}", err);
            return;
        }
    };

    // Write the string to the file
    match file.write_all(content.as_bytes()) {
        Ok(()) => println!("String successfully written to file."),
        Err(err) => eprintln!("Error writing to file: {}", err),
    }
    println!("{}", &content);
}

pub fn sm_gui<'gui>(
    recipe: Recipe,
    metadata: WidgetMetadata,
    recipe_filepath: String,
    args: Arguments,
    sender: Sender<(Recipe, Arguments, Option<windows::Win32::Foundation::HWND>)>,
) -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("smoothie-32x.png")[..]).unwrap(),
            )
            .with_inner_size([320.0, 900.0]),
        ..Default::default()
    };

    eframe::run_native(
        WINDOW_NAME,
        options,
        Box::new(|_cc|{

            Box::new(
                SmApp {
                    recipe_saved: format!("{:?}", recipe),
                    recipe,
                    metadata,
                    selected_files: vec![], // file select dialog with render button
                    show_confirmation_dialog: false,
                    allowed_to_close: false,
                    show_about: false,
                    recipe_filepath,
                    args,
                    start_rendering: false,
                    sender: sender
            }
        )
    }),
    )
}


impl eframe::App for SmApp {
    
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.start_rendering {
                let mut scoped_args = self.args.clone();
                scoped_args.input = self.selected_files.clone();


                let hwnd: Option<windows::Win32::Foundation::HWND> = if cfg!(windows) {
                    let winit::raw_window_handle::RawWindowHandle::Win32(handle) = _frame.window_handle().unwrap().as_raw() else {
                        panic!("Unsupported platform");
                    };
                    Some(windows::Win32::Foundation::HWND(handle.hwnd.into()))
                    
                } else {
                    None
                };

                let send_result = self.sender.send((self.recipe.clone(), scoped_args, hwnd));

                if let Err(e) = send_result {
                    eprintln!("Retrieving filepaths from GUI panicked: {:?}", e);
                }

                self.selected_files.clear();
                self.start_rendering = false;
       
                // self.show_confirmation_dialog = true;
                // self.allowed_to_close = true;
                // let ctx = ctx.clone();
                // std::thread::spawn(move || {
                //     ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                // });
                self.selected_files.clear();
                self.start_rendering = false;

                // self.show_confirmation_dialog = false;
                // self.allowed_to_close = true;
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
                if ui.button("README").clicked() {
                    self.show_about = true
                }
                // ui.label(" | ");
                let open_button = ui
                .button("open")
                .on_hover_text_at_pointer("select videos to render with");
                
                if open_button
                    .clicked() || open_button.secondary_clicked()
                {
                    let to_open = PathBuf::from(self.recipe_filepath.clone()).display().to_string();
                    dbg!(&to_open);
                    if open_button.secondary_clicked() {
                        let mut ctx = ClipboardContext::new().unwrap();
                        ctx.set_contents(to_open.clone()).unwrap();
                    } else {
                        match opener::open(&to_open){
                            Ok(()) =>{}
                            Err(e) => {
                                panic!("Error {e}\n\nFailed opening file {:?}", to_open);
                            }
                        }
                    }
                };
                if ui
                    .button("render")
                    .on_hover_text_at_pointer("select videos to render with")
                    .clicked()
                {
                    let input = rfd::FileDialog::new()
                        .add_filter("Video file", crate::VIDEO_EXTENSIONS)
                        .set_title("Select video(s) to queue to Smoothie")
                        .set_directory("/")
                        .pick_files();

                    if let Some(input_vids) = input {
                        // used later in an if statement later down
                        if !input_vids.is_empty() {
                            self.selected_files = input_vids;
                            self.start_rendering = true;
                        }
                    }
                }

                let ctrl_s = egui::KeyboardShortcut {
                    modifiers: egui::Modifiers {
                        ctrl: true,
                        alt: false,
                        shift: false,
                        mac_cmd: false,
                        command: false,
                    },
                    logical_key: egui::Key::S,
                };
                // if ui.button("hash").clicked() {
                //     println!("{:?}", self.recipe);
                // }

                // declare buttons and immediately handle if it's clicked OR if keyboard shortcut is "consumed" (pressed)
                if ui
                    .button("save")
                    .on_hover_text_at_pointer("save recipe modifications to .ini")
                    .clicked()
                    || ctx.input_mut(|i| i.consume_shortcut(&ctrl_s))
                {
                    self.recipe_saved = format!("{:?}", self.recipe);
                    save_recipe(
                        &self.recipe,
                        &PathBuf::from(&self.recipe_filepath),
                        &self.metadata,
                    );
                }
                if ui
                    .button("copy")
                    .on_hover_text_at_pointer(concat!(
                        "SHIFT+click: wraps in a codeblock\n",
                        "CTRL+click: omits disabled categories\n",
                        "yes, they can be combined"
                    ))
                    .clicked()
                {
                    let mut recipe_txt = if ui.input(|i| i.modifiers.ctrl) {
                        export_recipe(self.recipe.clone(), &self.metadata.clone(), true, false, false)
                    } else {
                        export_recipe(self.recipe.clone(), &self.metadata.clone(), false, false, false)
                    };

                    if ui.input(|i| i.modifiers.shift) {
                        recipe_txt = "```yml\n".to_owned() + &recipe_txt + "```"
                    }
                    let mut ctx = ClipboardContext::new().unwrap();

                    ctx.set_contents(recipe_txt).unwrap();
                }
            });
            egui::ScrollArea::vertical().show(ui, |ui| {
                for cat in &mut self.metadata.keys() {
                    let mut first_run: bool = true;
                    let mut category_visibility: bool = true;

                    // fn disable_category(flag: &mut bool) {
                    //     *flag = false;
                    // }

                    if self.metadata.get(cat).unwrap().get("_sm_category").unwrap().get("display").unwrap() == &"no".to_string() {
                        continue
                    }

                    let sections = self.recipe.get_section_mut(cat);

                    for (key, value) in sections {

                        if self.metadata.get(cat).unwrap().get(key).unwrap().get("display").unwrap() == &"no".to_string() {
                            continue
                        }
                        if first_run {
                            // make the ui different
                            ui.horizontal(|ui| {
                                let url = format!(
                                    "https://ctt.cx/recipe#{}",
                                    &cat.clone().replace(" ", "-")
                                );

                                let _ = if ui.hyperlink_to(cat, &url).secondary_clicked() {
                                    dbg!(&url);
                                    println!("Copied `{}` to clipboard", &url);
                                    let mut ctx = ClipboardContext::new().unwrap();
                                    ctx.set_contents(url.to_owned()).unwrap();
                                };

                                if key == "enabled" {
                                    let mut bool: bool = crate::YES.contains(&value.as_str());
                                    ui.horizontal(|ui| {
                                        // ui.label(key.to_owned() + ":");
                                        ui.checkbox(&mut bool, "\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t");
                                    });
                                    *value = if bool {
                                        "yes".to_owned()
                                    } else {
                                        if key == "enabled" {
                                            category_visibility = false;
                                            // disable_category(&mut category_visibility);
                                        }
                                        "no".to_owned()
                                    };
                                }
                            });

                            first_run = false;
                            if key == "enabled" {
                                continue;
                            }
                        }

                        if !category_visibility {
                            continue;
                        };
                        let def_metadata = &mut self.metadata.get(cat).unwrap().get(key).unwrap();

                        match def_metadata.get("type").unwrap().as_str() {
                            "enum" => {
                                let enum_values: Vec<String> = def_metadata
                                    .get("values")
                                    .expect("Failed getting values from enum")
                                    .split(",")
                                    .map(|s| s.trim().to_owned())
                                    .collect();

                                egui::ComboBox::from_label(key)
                                    .selected_text(value.clone())
                                    .show_ui(ui, |ui| {
                                        for enum_value in enum_values {
                                            ui.selectable_value(
                                                value,
                                                enum_value.clone(),
                                                enum_value + " " + key,
                                            );
                                        }
                                    });
                            }
                            "bool" => {
                                let mut bool: bool = crate::YES.contains(&value.as_str());
                                ui.horizontal(|ui| {
                                    // ui.label(key.to_owned() + ":");
                                    ui.checkbox(&mut bool, key);
                                });
                                *value = if bool {
                                    if key == "stay on top" {
                                        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop));
                                    }
                                    "yes".to_owned()
                                } else {
                                    if key == "enabled" {
                                        category_visibility = false;
                                    } else if key == "stay on top" {
                                        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(egui::WindowLevel::Normal));
                                    }
                                    "no".to_owned()
                                };
                            }
                            "int_slider" | "float_slider" => {
                                if value.is_empty() {
                                    continue;
                                }

                                let mut int = if let Ok(int) =
                                    value.clone().replace("x", "").parse::<f32>()
                                {
                                    int
                                } else {
                                    panic!(
                                        "{}",
                                        format!("Failed converting {:?} to string", value)
                                    );
                                };

                                let min = def_metadata
                                    .get("min")
                                    .unwrap()
                                    .parse::<f32>()
                                    .expect("No min");
                                let max = def_metadata
                                    .get("max")
                                    .unwrap()
                                    .parse::<f32>()
                                    .expect("No max");
                                let increment = def_metadata
                                    .get("increment")
                                    .unwrap()
                                    .parse::<f64>()
                                    .expect("No increment");

                                ui.horizontal(|ui| {
                                    ui.label(key.to_owned() + ":");
                                    let response = ui.add(
                                        egui::Slider::new(&mut int, min..=max)
                                            //.text(key)
                                            .step_by(increment),
                                    );

                                    if response.changed() {
                                        *value = int.to_string();
                                    }
                                });
                            }
                            "string" | "path" | "filepath" | "folderpath" | "exepath" => {
                                ui.horizontal(|ui| {
                                    ui.label(key.to_owned() + ":");
                                    ui.add(egui::TextEdit::singleline(&mut *value));
                                });
                            }
                            _ => {
                                ui.label(key.to_owned() + "- TODO");
                            }
                        }
                    }
                    if category_visibility {
                        ui.separator();
                    }
                }
            });

            preview_files_being_dropped(ctx);

            ctx.input(|i| {
                if !i.raw.dropped_files.is_empty() {
                    for file in i.raw.dropped_files.clone() {
                        if let Some(path) = file.path {
                            if let Some(ext_str) = path.extension() {
                                if crate::VIDEO_EXTENSIONS.contains(
                                    &ext_str
                                        .to_ascii_lowercase()
                                        .as_os_str()
                                        .to_str()
                                        .expect("Failed "),
                                ) {
                                    self.selected_files.push(path)
                                } else {
                                    eprintln!("Skipping file with no extension:");
                                    dbg!(&path);
                                }
                            }
                        } else {
                            eprintln!("Skipping DroppedFile with no path attribute:");
                            dbg!(&file);
                        }
                    }
                }

                // if i.consume_shortcut(&Ctrls){
                //     println!("you pressed ctrl+s");
                // }

                if !self.selected_files.is_empty() {
                    self.start_rendering = true;
                }
            });

            if self.show_about {
                egui::Window::new("about smoothie app")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.heading("\nknown bugs");
                        ui.label(
                            concat!(
                                "\n- Starting to render causes the GUI to freeze instead of closing, just minimize it tbh"
                            ),
                        );
                        ui.heading("\nwhat is this");
                        ui.label(
                            "user interface to edit the recipe.ini text file more conveniently",
                        );
                        ui.heading("\nhow to use");
                        ui.label("set, save, launch smoothie from start menu/launch.bat/send to");
                        ui.heading("links");
                        if ui
                            .hyperlink_to("documentation", "https://ctt.cx/sm")
                            .on_hover_text_at_pointer("https://ctt.cx/sm")
                            .secondary_clicked()
                        {
                            let url = "https://ctt.cx/sm";
                            println!("Copied `{}` to clipboard", &url);
                            let mut ctx = ClipboardContext::new().unwrap();
                            ctx.set_contents(url.to_owned()).unwrap();
                        };
                        if ui
                            .hyperlink_to(
                                "github\n",
                                "https://github.com/couleur-tweak-tips/smoothie-rs",
                            )
                            .on_hover_text_at_pointer("see /smrs-egui/ folder")
                            .secondary_clicked()
                        {
                            let url = "https://github.com/couleur-tweak-tips/smoothie-rs";
                            println!("Copied `{}` to clipboard", &url);
                            let mut ctx = ClipboardContext::new().unwrap();
                            ctx.set_contents(url.to_owned()).unwrap();
                        };

                        if ui.button("ok").clicked() {
                            self.show_about = false;
                        }
                    });
            }
            
            if ctx.input(|i| i.viewport().close_requested()) && !self.allowed_to_close {
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                    self.show_confirmation_dialog = true;
                }
            }
            if self.show_confirmation_dialog {
                if format!("{:?}", self.recipe) != self.recipe_saved {
                    // sorry, gotta do long ass title inline because rust ownership sucks
                    egui::Window::new(
                        "Do you want to save changes to ".to_owned()
                            + PathBuf::from(self.recipe_filepath.clone())
                                .file_name()
                                .expect("Failed getting basename from recipe_filepath")
                                .to_str()
                                .unwrap()
                            + "?",
                    )
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Save").clicked() {
                                save_recipe(
                                    &self.recipe,
                                    &PathBuf::from(&self.recipe_filepath),
                                    &self.metadata,
                                );
                                self.show_confirmation_dialog = false;
                                self.allowed_to_close = true;
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                            if ui.button("Don't Save").clicked() {
                                self.show_confirmation_dialog = false;
                                self.allowed_to_close = true;
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                            if ui.button("Cancel").clicked() {
                                self.show_confirmation_dialog = false;
                            }
                        });
                    });
                } else {
                    self.show_confirmation_dialog = false;
                    self.allowed_to_close = true;
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        });
    }
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
