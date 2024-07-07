pub mod windows{
    use egui::{Modifiers, Slider, Ui};

    use crate::Sphere;

    #[derive(Clone)]
    pub struct SandboxWindow {
        pub spheres: Vec<Sphere>,
        pub new_sphere: Sphere,
        pub skycolor: [f32; 3],
    }
    
    impl SandboxWindow {
        pub fn new() -> Self {
            Self {
                spheres: Vec::new(),
                new_sphere: Sphere {
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
                },
                skycolor: [30.0,255.0,255.0],
            }
        }
    
    pub fn ui(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        let _ = ctx;
            self.sphere_list(ui);
            self.add_new_sphere(ui);
            self.scene_settings(ui);
        
    }
    
        pub fn sphere_list(&mut self, ui: &mut Ui) {
            ui.vertical_centered(|ui| {
                ui.label("Spheres:");
                ui.collapsing("Sphere List", |ui| {
                    for (i, sphere) in self.spheres.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(&format!("Sphere {}", i));
                            ui.add(
                                Slider::new(&mut sphere.position[0], -100.0..=100.0)
                                    .text("X"),
                            );
                            ui.add(
                                Slider::new(&mut sphere.position[1], -100.0..=100.0)
                                    .text("Y"),
                            );
                            ui.add(
                                Slider::new(&mut sphere.position[2], -100.0..=100.0)
                                    .text("Z"),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label(&format!("Sphere {}", i));
                            ui.add(
                                Slider::new(&mut sphere.velocity[0], -100.0..=100.0)
                                    .text("X"),
                            );
                            ui.add(
                                Slider::new(&mut sphere.velocity[1], -100.0..=100.0)
                                    .text("Y"),
                            );
                            ui.add(
                                Slider::new(&mut sphere.velocity[2], -100.0..=100.0)
                                    .text("Z"),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.add(
                                Slider::new(&mut sphere.radius, 0.1..=100.0)
                                    .text("Radius"),
                            );
                            ui.add(
                                Slider::new(&mut sphere.color[0], 0.0..=255.0)
                                    .text("R")
                                    .clamp_to_range(true),
                            );
                            ui.add(
                                Slider::new(&mut sphere.color[1], 0.0..=255.0)
                                    .text("G")
                                    .clamp_to_range(true),
                            );
                            ui.add(
                                Slider::new(&mut sphere.color[2], 0.0..=255.0)
                                    .text("B")
                                    .clamp_to_range(true),
                            );
                        });
                        ui.add(
                            Slider::new(&mut sphere.roughness, 0.0..=1.0)
                                .text("Roughness")
                                .clamp_to_range(true),
                        );
                        ui.add(
                            Slider::new(&mut sphere.emission, 0.0..=100.0)
                                .text("Emission"),
                        );
                    ui.add(egui::Checkbox::new(&mut sphere.is_static, "Make it Static"));
                    }
                });
            });
                    ui.separator();
    
        }
    
        pub fn add_new_sphere(&mut self, ui: &mut Ui) {
            ui.vertical_centered(|ui| {
                ui.collapsing("New Sphere", |ui| {
                    ui.label("New Sphere:");
                    ui.horizontal(|ui| {
                        ui.add(
                            Slider::new(&mut self.new_sphere.position[0], -100.0..=100.0)
                                .text("X"),
                        );
                        ui.add(
                            Slider::new(&mut self.new_sphere.position[1], -100.0..=100.0)
                                .text("Y"),
                        );
                        ui.add(
                            Slider::new(&mut self.new_sphere.position[2], -100.0..=100.0)
                                .text("Z"),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.add(
                            Slider::new(&mut self.new_sphere.radius, 0.1..=100.0)
                                .text("Radius"),
                        );
                        ui.add(
                            Slider::new(&mut self.new_sphere.color[0], 0.0..=255.0)
                                .text("R")
                                .clamp_to_range(true),
                        );
                        ui.add(
                            Slider::new(&mut self.new_sphere.color[1], 0.0..=255.0)
                                .text("G")
                                .clamp_to_range(true),
                        );
                        ui.add(
                            Slider::new(&mut self.new_sphere.color[2], 0.0..=255.0)
                                .text("B")
                                .clamp_to_range(true),
                        );
                    });
                    ui.add(
                        Slider::new(&mut self.new_sphere.roughness, 0.0..=1.0)
                            .text("Roughness")
                            .clamp_to_range(true),
                    );
                    ui.add(
                        Slider::new(&mut self.new_sphere.emission, 0.0..=100.0)
                            .text("Emission"),
                    );
                    ui.add(egui::Checkbox::new(&mut self.new_sphere.is_static, "Make it Static"));
    
                    if ui.button("Add Sphere").clicked() {
                        self.spheres.push(Sphere {
                            position: self.new_sphere.position,
                            radius: self.new_sphere.radius,
                            color: self.new_sphere.color,
                            roughness: self.new_sphere.roughness,
                            emission: self.new_sphere.emission,
                            is_static:self.new_sphere.is_static,
                            velocity: [0.0; 3],
                            acceleration: [0.0; 3],
                            angular_velocity: [0.0, 0.0, 0.0],
                            angular_acceleration: [0.0, 0.0, 0.0],
                            orientation: [1.0, 0.0, 0.0, 0.0],
                            mass: 1.0,
                        });
                        self.new_sphere= Sphere {
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
                    ui.hyperlink_to(
                        format!("{TWITTER} @ernerfeldt"),
                        "https://twitter.com/ernerfeldt",
                    );
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
                    if ui.button("Sandbox window").clicked() {
                        self.show_sandbox_window = !self.show_sandbox_window;
                    }
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