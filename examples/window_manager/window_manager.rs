pub mod windows{
    use egui::{Modifiers, Slider, Ui};

    use crate::Object;

    #[derive(Clone)]
    pub struct SandboxWindow {
        pub Objects: Vec<Object>,
        pub new_Object: Object,
        pub skycolor: [f32; 3],
        pub is_fisheye:bool,
    }
    
    impl SandboxWindow {
        pub fn new() -> Self {
            Self {
                Objects: Vec::new(),
                new_Object: Object {
                    position: [0.0; 3],
                    radius: 1.0,
                    color: [120.0; 3],
                    roughness: 1.0,
                    emission: 0.0,
                    is_static: true,
                    velocity: [0.0; 3],
                    acceleration: [0.0; 3],
                    angular_velocity: [0.0; 3],
                    angular_acceleration: [0.0; 3],
                    orientation: [1.0, 0.0, 0.0, 0.0],
                    mass: 1.0,
                    is_cube: true,
                    size: [0.0;3],
                    is_glass: false,
                    reflectness: 0.0,
                },
                is_fisheye:false,
                skycolor: [30.0,255.0,255.0],
            }
        }
    
    pub fn ui(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        let _ = ctx;
            self.add_new_object(ui);
            self.scene_settings(ui);
        
    }

        pub fn add_new_object(&mut self, ui: &mut Ui) {
            ui.vertical_centered(|ui| {
                ui.collapsing("New Object", |ui| {
                    ui.label("New Object:");
                    ui.horizontal(|ui| {
                        ui.add(
                            Slider::new(&mut self.new_Object.position[0], -100.0..=100.0)
                                .text("X"),
                        );
                        ui.add(
                            Slider::new(&mut self.new_Object.position[1], -100.0..=100.0)
                                .text("Y"),
                        );
                        ui.add(
                            Slider::new(&mut self.new_Object.position[2], -100.0..=100.0)
                                .text("Z"),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.add(
                            Slider::new(&mut self.new_Object.radius, 0.1..=100.0)
                                .text("Radius"),
                        );
                        ui.add(
                            Slider::new(&mut self.new_Object.color[0], 0.0..=255.0)
                                .text("R")
                                .clamp_to_range(true),
                        );
                        ui.add(
                            Slider::new(&mut self.new_Object.color[1], 0.0..=255.0)
                                .text("G")
                                .clamp_to_range(true),
                        );
                        ui.add(
                            Slider::new(&mut self.new_Object.color[2], 0.0..=255.0)
                                .text("B")
                                .clamp_to_range(true),
                        );
                    });
                    ui.add(
                        Slider::new(&mut self.new_Object.roughness, 0.0..=1.0)
                            .text("Roughness")
                            .clamp_to_range(true),
                    );
                    ui.add(
                        Slider::new(&mut self.new_Object.emission, 0.0..=100.0)
                            .text("Emission"),
                    );
                    ui.add(egui::Checkbox::new(&mut self.new_Object.is_glass, "Make it Glass"));
                    if(self.new_Object.is_glass){
                        ui.add(
                            Slider::new(&mut self.new_Object.reflectness, 0.0..=10.0)
                                .text("Reflectness"),
                        );
                    }
                    ui.add(egui::Checkbox::new(&mut self.new_Object.is_static, "Make it Static"));
                    ui.add(egui::Checkbox::new(&mut self.new_Object.is_cube, "Make it Cube"));
                    if ui.button("Add Object").clicked() {
                        self.Objects.push(Object {
                            position: self.new_Object.position,
                            radius: self.new_Object.radius,
                            color: self.new_Object.color,
                            roughness: self.new_Object.roughness,
                            emission: self.new_Object.emission,
                            is_static:self.new_Object.is_static,
                            velocity: [0.0; 3],
                            acceleration: [0.0; 3],
                            angular_velocity: [0.0;3],
                            angular_acceleration: [0.0;3],
                            orientation: [1.0, 0.0, 0.0, 0.0],
                            mass: 1.0,
                            is_cube: self.new_Object.is_cube,
                            size:[1.0;3],
                            is_glass: self.new_Object.is_glass,
                            reflectness: self.new_Object.reflectness,
                        });
                        self.new_Object= Object {
                            position: [0.0; 3],
                            radius: 1.0,
                            color: [120.0; 3],
                            roughness: 1.0,
                            emission: 0.0,
                            is_static: true,
                            velocity: [0.0; 3],
                            acceleration: [0.0; 3],
                            angular_velocity: [0.0; 3],
                            angular_acceleration: [0.0; 3],
                            orientation: [1.0, 0.0, 0.0, 0.0],
                            mass: 1.0,
                            is_cube: true,
                            size:[1.0;3],
                            is_glass: false,
                            reflectness: 0.0,
                        };
                    }
                });
            });
        }
    
        pub fn scene_settings(&mut self, ui: &mut Ui) {
            ui.vertical_centered(|ui| {
                ui.label("Scene Settings:");
                ui.add(Slider::new(&mut self.skycolor[0], 0.0..=255.0).text("R"));
                ui.add(Slider::new(&mut self.skycolor[1], 0.0..=255.0).text("G"));
                ui.add(Slider::new(&mut self.skycolor[2], 0.0..=255.0).text("B"));
            });
            ui.add(egui::Checkbox::new(&mut self.is_fisheye, "Fisheye Effect"));
        }
        
    }
    pub struct MainWindow<'a> {
        pub show_sandbox_window: bool,
        pub sandbox_window: &'a mut SandboxWindow,
    }
    
    impl<'a> MainWindow<'a> {
        pub fn new(sandbox_window: &'a mut SandboxWindow) -> Self {
            Self {
                show_sandbox_window: false,
                sandbox_window,
            }
        }
    
        pub fn ui(&mut self, ctx: &egui::Context) {
            self.desktop_ui(ctx);
        }
    
        pub fn desktop_ui(&mut self, ctx: &egui::Context) {
            egui::SidePanel::left("egui_demo_panel")
                .resizable(true)
                .default_width(250.0)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("✒ Vetracer Engine");
                    });
                    ui.separator();
                    use egui::special_emojis::{GITHUB, TWITTER};
                    if self.show_sandbox_window {
                        egui::Window::new("Sandbox Window")
                            .resizable(true)
                            .default_width(400.0)
                            .show(ctx, |ui| {
                                self.sandbox_window.ui(ctx, ui);
                            });
                    }
                    ui.hyperlink_to(
                        format!("{GITHUB} Resource Code"),
                        "https://github.com/OmarDevX",
                    );
                    // ui.hyperlink_to(
                    //     format!("{TWITTER} @ernerfeldt"),
                    //     "https://twitter.com/ernerfeldt",
                    // );
                    ui.separator();
                    self.demo_list_ui(ui);
                });
    
            egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    file_menu_button(ui);
                });
            });
        }
    
        pub fn demo_list_ui(&mut self, ui: &mut egui::Ui) {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    ui.label("Placeholder");
                ui.vertical_centered(|ui| {
                    ui.label("Objects:");
                    ui.collapsing("Object List", |ui| {
                        for (i, object) in self.sandbox_window.Objects.iter_mut().enumerate() {
                            ui.push_id(i, |ui| {
                                ui.collapsing(format!("Object {}", i), |ui| {
                                    // Object Position Sliders
                                    ui.vertical(|ui| {
                                        ui.label(format!("Object Position"));
                                        ui.add(Slider::new(&mut object.position[0], -100.0..=100.0).text("Position X"));
                                        ui.add(Slider::new(&mut object.position[1], -100.0..=100.0).text("Position Y"));
                                        ui.add(Slider::new(&mut object.position[2], -100.0..=100.0).text("Position Z"));
                                    });

                                    // Cube Size Sliders if the object is a cube
                                    if object.is_cube {
                                        ui.vertical(|ui| {
                                            ui.label("Cube Size");
                                            ui.add(Slider::new(&mut object.size[0], -100.0..=100.0).text("Size X"));
                                            ui.add(Slider::new(&mut object.size[1], -100.0..=100.0).text("Size Y"));
                                            ui.add(Slider::new(&mut object.size[2], -100.0..=100.0).text("Size Z"));
                                        });
                                    }

                                    // Radius and Color Sliders
                                    ui.vertical(|ui| {
                                        ui.label("Object Color");
                                        ui.add(Slider::new(&mut object.radius, 0.1..=100.0).text("Radius"));
                                        ui.add(Slider::new(&mut object.color[0], 0.0..=255.0).text("Color R").clamp_to_range(true));
                                        ui.add(Slider::new(&mut object.color[1], 0.0..=255.0).text("Color G").clamp_to_range(true));
                                        ui.add(Slider::new(&mut object.color[2], 0.0..=255.0).text("Color B").clamp_to_range(true));
                                    });

                                    // Roughness and Emission Sliders
                                    ui.vertical(|ui| {
                                        ui.label("Object Material");
                                        ui.add(Slider::new(&mut object.roughness, 0.0..=1.0).text("Roughness").clamp_to_range(true));
                                        ui.add(Slider::new(&mut object.emission, 0.0..=100.0).text("Emission"));
                                        ui.add(Slider::new(&mut object.reflectness, 0.0..=100.0).text("reflect"));
                                    });

                                    // Static Checkbox
                                    ui.add(egui::Checkbox::new(&mut object.is_static, "Make it Static"));
                                });
                            });
                            ui.separator();
                        }
                    });
                    if ui.button("Add Sphere").clicked() {
                        self.show_sandbox_window = !self.show_sandbox_window;
                    }
                });
                    if ui.button("Organize windows").clicked() {
                        ui.ctx().memory_mut(|mem| mem.reset_areas());
                    }
                });
            });
        }
        pub fn get_sandbox_window(&self) -> &SandboxWindow {
                return self.sandbox_window;
        }
    }
    
        pub fn file_menu_button(ui: &mut Ui) {
        let organize_shortcut =
            egui::KeyboardShortcut::new(Modifiers::CTRL | Modifiers::SHIFT, egui::Key::O);
        let reset_shortcut =
            egui::KeyboardShortcut::new(Modifiers::CTRL | Modifiers::SHIFT, egui::Key::R);
    
        // NOTE: we must check the shortcuts OUTSIDE of the actual "File" menu,
        // or else they would only be checked if the "File" menu was actually open!
    
        if ui.input_mut(|i| i.consume_shortcut(&organize_shortcut)) {
            ui.ctx().memory_mut(|mem| mem.reset_areas());
        }
    
        if ui.input_mut(|i| i.consume_shortcut(&reset_shortcut)) {
            ui.ctx().memory_mut(|mem| *mem = Default::default());
        }
    
        ui.menu_button("File", |ui| {
            ui.set_min_width(220.0);
            ui.style_mut().wrap = Some(false);
    
            // On the web the browser controls the zoom
            #[cfg(not(target_arch = "wasm32"))]
            {
                egui::gui_zoom::zoom_menu_buttons(ui);
                ui.weak(format!(
                    "Current zoom: {:.0}%",
                    100.0 * ui.ctx().zoom_factor()
                ))
                .on_hover_text("The UI zoom level, on top of the operating system's default value");
                ui.separator();
            }
    
            if ui
                .add(
                    egui::Button::new("Organize Windows")
                        .shortcut_text(ui.ctx().format_shortcut(&organize_shortcut)),
                )
                .clicked()
            {
                ui.ctx().memory_mut(|mem| mem.reset_areas());
                ui.close_menu();
            }
    
            if ui
                .add(
                    egui::Button::new("Reset egui memory")
                        .shortcut_text(ui.ctx().format_shortcut(&reset_shortcut)),
                )
                .on_hover_text("Forget scroll, positions, sizes etc")
                .clicked()
            {
                ui.ctx().memory_mut(|mem| *mem = Default::default());
                ui.close_menu();
            }
        });
    }

}