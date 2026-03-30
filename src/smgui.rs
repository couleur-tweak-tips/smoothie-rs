use crate::{
    cli::Arguments,
    recipe::{export_recipe, parse_recipe, Recipe, WidgetMetadata},
};
use std::{
    fs::File, io::Write,
    path::PathBuf,
    sync::mpsc::Sender // used to retrieve SmCommands
};
use copypasta::{ClipboardContext, ClipboardProvider};
use eframe::egui;
#[cfg(windows)]
use winit::raw_window_handle::HasWindowHandle;
use indexmap::map::IndexMap;

#[cfg(windows)]
type WinHWND = Option<windows::Win32::Foundation::HWND>;
#[cfg(not(windows))]
type WinHWND = ();

struct SmApp {
    first_frame: bool,
    save_new_recipe: bool,
    recipe_change_request: Option<String>,
    recipe: Recipe,
    metadata: WidgetMetadata,
    selected_files: Vec<PathBuf>,
    show_confirmation_dialog: bool,
    show_merge_dialog: bool,
    staging_merge: Option<(Recipe, IndexMap<String, IndexMap<String, bool>>)>,
    allowed_to_close: bool,
    show_about: bool,
    args: Arguments,
    start_rendering: bool,
    // yeah that's the damn typename
    recipe_saved: String,
    sender: Sender<(Recipe, Arguments, WinHWND)>,
    make_new_recipe: bool,
    new_recipe_filename: String,
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

    let recipe_path = if recipe_filename.clone().exists() {
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

#[allow(clippy::extra_unused_lifetimes)]
pub fn sm_gui<'gui>(
    recipe: Recipe,
    metadata: WidgetMetadata,
    args: Arguments,
    sender: Sender<(Recipe, Arguments, WinHWND)>,
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

           Ok(Box::new(
                SmApp {
                    show_merge_dialog: false,
                    staging_merge: None,
                    first_frame: true,
                    save_new_recipe: false,
                    recipe_change_request: None,
                    recipe_saved: format!("{:?}", recipe),
                    recipe,
                    metadata,
                    selected_files: vec![], // file select dialog with render button
                    show_confirmation_dialog: false,
                    allowed_to_close: false,
                    show_about: false,
                    args,
                    start_rendering: false,
                    sender,
                    make_new_recipe: false,
                    new_recipe_filename: String::new(),
            }
        ))
    }),
    )
}


impl eframe::App for SmApp {
    
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.start_rendering {
                let mut scoped_args = self.args.clone();
                scoped_args.input = self.selected_files.clone();

                #[cfg(windows)]
                let hwnd: Option<windows::Win32::Foundation::HWND> = {
                    let winit::raw_window_handle::RawWindowHandle::Win32(handle) = frame.window_handle().unwrap().as_raw() else {
                        panic!("Unsupported platform");
                    };
                    let ptr = handle.hwnd.get() as *mut std::ffi::c_void;
                    Some(windows::Win32::Foundation::HWND(ptr))
                };

                #[cfg(not(windows))]
                let hwnd: WinHWND = ();
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
            let ctrl_s_pressed = ctx.input_mut(|i| i.consume_shortcut(&ctrl_s));

            let ctrl_p = egui::KeyboardShortcut {
                modifiers: egui::Modifiers {
                    ctrl: true,
                    alt: false,
                    shift: false,
                    mac_cmd: false,
                    command: false,
                },
                logical_key: egui::Key::P,
            };
            let ctrl_p_pressed = ctx.input_mut(|i| i.consume_shortcut(&ctrl_p));


            egui::menu::bar(ui, |ui| {
                egui::widgets::global_theme_preference_switch(ui);
                if ui.button("README").clicked() {
                    self.show_about = true
                }

                let open_button = ui
                .button("open")
                .on_hover_text_at_pointer("open recipe file");
                
                if open_button
                    .clicked() || open_button.secondary_clicked()
                {
                    let to_open = PathBuf::from(self.args.recipe.clone()).display().to_string();
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
                        // used in an if statement later down
                        if !input_vids.is_empty() {
                            self.selected_files = input_vids;
                            self.start_rendering = true;
                        }
                    }
                }


                // declare buttons and immediately handle if it's clicked OR if keyboard shortcut is "consumed" (pressed)
                if ui
                    .button("save")
                    .on_hover_text_at_pointer("save recipe modifications to .ini")
                    .clicked()
                    || (ctrl_s_pressed && self.recipe_change_request.is_none())
                {
                    self.recipe_saved = format!("{:?}", self.recipe);
                    save_recipe(
                        &self.recipe,
                        &PathBuf::from(&self.args.recipe),
                        &self.metadata,
                    );
                }
                if ui
                    .button("copy")
                    .on_hover_text_at_pointer(concat!(
                        "SHIFT+click: wraps in a codeblock\n",
                        "CTRL+click: omits disabled categories\n",
                        "try to guess what CTRL+SHIFT+CLICK does :)"
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
            //egui::menu::bar(ui, |ui| {

            let binding = PathBuf::from(self.args.recipe.clone());
            let selected_recipe = binding
                .file_name().expect("Failed unwrapping file name from args.recipe")
                .to_str().expect("Failed unwrapping string from filename from args.recipe");

            egui::ComboBox::from_label("")
                .selected_text(selected_recipe)
                .show_ui(ui, |ui| {
                    let enum_values = crate::portable::get_config_filepaths();
                    for enum_value in enum_values {

                        let selected_value = enum_value.to_str().unwrap().to_string();

                        if selected_value == self.args.recipe {
                            continue
                        }

                        ui.selectable_value(
                            &mut self.args.recipe,
                            selected_value,
                            enum_value.file_name().to_owned().unwrap().to_str().unwrap()
                        );

                    }
                    if binding.to_str().unwrap() != self.args.recipe.as_str() {
                        self.recipe_change_request = Some(binding.to_str().unwrap().to_string());
                        // let (recipe, metadata) = crate::recipe::get_recipe(&mut self.args);
                        // self.recipe = recipe;
                        // self.metadata = metadata;                    
                    }
                    ui.selectable_value(
                        &mut self.make_new_recipe,
                        true,
                        "New recipe"
                    );
                });
             //});
            egui::menu::bar(ui, |_| {}); // <br>
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

                                if ui.hyperlink_to(cat, &url).secondary_clicked() {
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
                                let checkbox = ui.checkbox(&mut bool, key);
                                *value = if bool {
                                    if key == "stay on top" && (self.first_frame || checkbox.clicked()) {
                                        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop));
                                    }
                                    "yes".to_owned()
                                } else {
                                    if key == "enabled" {
                                        category_visibility = false;
                                    } else if key == "stay on top" && (self.first_frame || checkbox.clicked()) {
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
                                            .clamping(egui::SliderClamping::Never)
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
                                        .expect("Failed convertin file extension to string"),
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

                if !self.selected_files.is_empty() {
                    self.start_rendering = true;
                }
            });

            if self.make_new_recipe {
                let mut open = true;
                egui::Window::new("new recipe").open(&mut open).show(ctx, |ui|{
                    ui.label(format!("it will use current recipe ({}) as base", selected_recipe));
                    ui.horizontal(|ui| {
                        ui.label("file name: ");
                        ui.add(egui::TextEdit::singleline(&mut self.new_recipe_filename).desired_width(200.0)).request_focus();
                        ui.label(".ini")
                    });
                    if ui.button("create").clicked() {
                    }
                });
                if !open {
                    self.make_new_recipe = false;
                }
            }

            if ctrl_p_pressed {
                let clipboard = ClipboardContext::new().unwrap().get_contents().expect("Failed reading system clipboard");
                let lines = clipboard.split("\n");
                let mut cleaned: Vec<String>  = vec![];

                #[allow(non_camel_case_types)]
                enum _RecipeTypes {
                    smoothie_rs,
                    smoothie_py,
                    teres,
                    blur_18,
                    blur_19,
                    blur_20,
                }

                for line in lines {
                    if line.is_empty() || line.starts_with("#") ||  line.starts_with("//") ||  line.starts_with(";") {
                        continue
                    }
                    cleaned.push(line.to_owned());
                };

                if !cleaned.is_empty() {
                    let mut to_merge = Recipe::new();

                    parse_recipe(
                        PathBuf::from(self.args.recipe.clone()),
                        Some(cleaned.join("\n")), &mut to_merge,
                        &mut None,
                        false
                    );

                    let mut toggled: IndexMap<String, IndexMap<String, bool>> = IndexMap::new();

                    for section in to_merge.keys() {
                        toggled.insert(section.to_owned(), IndexMap::new());
                    }

                    for section in toggled.to_owned().keys() {
                        for key in to_merge.get_section(section){
                            toggled
                                .entry(section.to_owned())
                                .or_default()
                                .insert(key.0.to_owned(), true);
                        }
                    }

                    self.staging_merge = Some((to_merge, toggled));
                    self.show_merge_dialog = true;
                }

            }

            if self.show_merge_dialog {

                egui::Window::new("merge pasted config").show(ctx, |ui|{

                    let mut cancel = false;
                    let mut merge = false;
                    let mut invert_selection = false;
                    ui.horizontal(|ui| {
                        cancel = ui.button("cancel").clicked();
                        merge = ui.button("merge").clicked();
                        invert_selection = ui.button("invert").clicked();
                    });
                    let staging_recipe = self.staging_merge.as_mut().unwrap();
                    for section in staging_recipe.0.to_owned().keys() {

                        ui.horizontal(|ui| {

                            let section_selection: Vec<bool> = staging_recipe.1.entry(section.to_owned()).or_default().values().cloned().collect();

                            let mut select = !section_selection.contains(&false);

                            let toggle_section = ui.checkbox(&mut select, section);

                            let section = staging_recipe.1.entry(section.to_owned()).or_default();
                           

                            for (key, value) in section.to_owned().into_iter() {

                                if toggle_section.clicked() {
                                    section.insert(key.to_owned(), select);
                                } else if invert_selection {
                               
                                    section.insert(key.to_owned(), !value);
                                }
                            }
                        });

                        for (key , value) in staging_recipe.0.get_section_mut(section){

                            ui.horizontal(|ui| {

                                let enabled = staging_recipe.1
                                    .entry(section.to_owned())
                                    .or_default()
                                    .entry(key.to_owned())
                                    .or_insert(false);

                                ui.checkbox(enabled, key);
                                *enabled = enabled.to_owned();
                                ui.add(egui::TextEdit::singleline(&mut *value));
                            });
                        }
                    }
                    if merge {

                        self.show_merge_dialog = false;
                        for section in staging_recipe.0.to_owned().keys() {
                            for (key, value) in staging_recipe.0.get_section(section){
                                let enabled = 
                                staging_recipe.1.entry(section.to_owned()).or_default().entry(key.to_owned()).or_insert(false);

                                if !enabled.to_owned() { continue }
                                self.recipe.insert_value(section, key.to_owned(), value.to_owned());
                            }
                        }

                    }
                    if cancel || merge {
                        self.staging_merge = None;
                        self.show_merge_dialog = false;
                    }
                });
            }

            if self.show_about {
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
                            .on_hover_text_at_pointer("see /src/smgui.rs folder")
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
            if self.save_new_recipe {
                self.recipe_saved = format!("{:?}", self.recipe);
                self.save_new_recipe = false;
            }
            if self.recipe_change_request.is_some() {
                let old_recipe = self.recipe_change_request.clone().unwrap();
                if format!("{:?}", self.recipe) != self.recipe_saved {
                    egui::Window::new(
                        "Do you want to save changes to ".to_owned()
                            + PathBuf::from(old_recipe.clone())
                                .file_name()
                                .expect("Failed getting basename from recipe")
                                .to_str()
                                .unwrap()
                            + "?",
                    )
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Save").clicked() || ctrl_s_pressed {
                                save_recipe(
                                    &self.recipe,
                                    &PathBuf::from(old_recipe.clone()),
                                    &self.metadata,
                                );
                                self.recipe_change_request = None;
                                let (recipe, metadata) = crate::recipe::get_recipe(&mut self.args);
                                self.recipe = recipe.clone();
                                self.metadata = metadata;
                                // the recipe isn't formatted yet, let it go through a frame
                                // to normalize bools and int slider increments
                                //self.recipe_saved = format!("{:?}", recipe);
                                //self.save_new_recipe;
                            }
                            if ui.button("Don't Save").clicked() {
                                self.recipe_change_request = None;
                                let (recipe, metadata) = crate::recipe::get_recipe(&mut self.args);
                                self.recipe = recipe.clone();
                                self.recipe_saved = format!("{:?}", recipe);
                                self.metadata = metadata;
                            }
                            if ui.button("Cancel").clicked() {
                                self.recipe_change_request = None;
                                self.args.recipe = old_recipe;
                            }
                        });
                    });
                } else {
                    self.recipe_change_request = None;
                    let (recipe, metadata) = crate::recipe::get_recipe(&mut self.args);
                    self.recipe = recipe.clone();
                    self.recipe_saved = format!("{:?}", recipe);
                    self.metadata = metadata;
                }
            }
            if self.show_confirmation_dialog {
                if format!("{:?}", self.recipe) != self.recipe_saved {
                    egui::Window::new(
                        "Do you want to save changes to ".to_owned()
                            + PathBuf::from(self.args.recipe.clone())
                                .file_name()
                                .expect("Failed getting basename from recipe")
                                .to_str()
                                .unwrap()
                            + "?",
                    )
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Save").clicked() || ctrl_s_pressed {
                                save_recipe(
                                    &self.recipe,
                                    &PathBuf::from(&self.args.recipe),
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
            self.first_frame = false;
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
