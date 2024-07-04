use egui::{FullOutput, ViewportId};
use egui_backend::{
    egui::{self, ClippedPrimitive},
    epi::{Frame, IntegrationInfo},
    get_frame_time, gl, sdl2,
    sdl2::event::Event,
    sdl2::video::GLProfile,
    sdl2::video::SwapInterval,
    DpiScaling, ShaderVersion, Signal,
};

use std::{fs, os::unix::raw::time_t, sync::Arc, time::Instant};

mod camera;
use camera::*;

use epi::backend::FrameData;
use glm::{Vec3, Vector3};
use sdl2::{event::WindowEvent, keyboard::Keycode, sys::u_int};
// Alias the backend to something less mouthful
use egui_sdl2_gl::{self as egui_backend, painter::{compile_shader, link_program}};
use gl::types::*;
use std::ptr;
use std::ffi::CString;



fn main() {
    
    let mut SCREEN_WIDTH = 1280;
    let mut SCREEN_HEIGHT = 700;
    let my_position = glm::vec3(0.0, 0.0, 3.0);
    let my_up = glm::vec3(0.0, 1.0, 0.0);
    let my_yaw = -90.0;
    let my_pitch = 0.0;
    let mut moveCamera=false;
    let mut my_camera = Camera::new(my_position, my_up, my_yaw, my_pitch,45.0);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_framebuffer_srgb_compatible(true);
    gl_attr.set_double_buffer(true);
    gl_attr.set_multisample_samples(4);
    gl_attr.set_context_version(3, 2);
        let last_frame_time: Instant = Instant::now();
    let window = video_subsystem
        .window(
            "Demo: Egui backend for SDL2 + GL",
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
        )
        .opengl()
        .resizable()
        .build()
        .unwrap();

    let _ctx = window.gl_create_context().unwrap();
    debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
    debug_assert_eq!(gl_attr.context_version(), (3, 2));

    if let Err(error) = window.subsystem().gl_set_swap_interval(SwapInterval::VSync) {
        println!(
            "Failed to gl_set_swap_interval(SwapInterval::VSync): {}",
            error
        );
    }
    let (mut painter, mut egui_state) =
        egui_backend::with_sdl2(&window, ShaderVersion::Default, DpiScaling::Default);
    //let mut demo_windows = egui_demo_lib::DemoWindows::default();
    let egui_ctx = egui::Context::default();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let start_time: Instant = Instant::now();
    let repaint_signal = Arc::new(Signal::default());

    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);

    // Load GLSL shader source from files
    let compute_shader_source = fs::read_to_string("examples/shaders/compute_shader.glsl")
        .expect("Failed to read compute_shader.glsl");
    let quad_vertex_shader_source = fs::read_to_string("examples/shaders/quad_vertex_shader.glsl")
        .expect("Failed to read quad_vertex_shader.glsl");
    let quad_fragment_shader_source = fs::read_to_string("examples/shaders/quad_fragment_shader.glsl")
        .expect("Failed to read quad_fragment_shader.glsl");

    // Compile shaders
    let compute_shader = compile_shader(&compute_shader_source, gl::COMPUTE_SHADER);
    let quad_vertex_shader = compile_shader(&quad_vertex_shader_source, gl::VERTEX_SHADER);
    let quad_fragment_shader = compile_shader(&quad_fragment_shader_source, gl::FRAGMENT_SHADER);

    // Link shader programs
    let compute_shader_program = link_program(compute_shader, 0);
    let quad_shader_program = link_program(quad_vertex_shader, quad_fragment_shader);

    // Create a texture for the compute shader to write to
    let mut texture = create_texture(SCREEN_WIDTH,SCREEN_HEIGHT);

    // Set up a fullscreen quad
    let vertices: [f32; 8] = [
        -1.0, -1.0,
        1.0, -1.0,
        -1.0,  1.0,
        1.0,  1.0,
    ];

    let mut vao = 0;
    let mut vbo = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, (vertices.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr, vertices.as_ptr() as *const _, gl::STATIC_DRAW);

        let pos_attrib = gl::GetAttribLocation(quad_shader_program, CString::new("in_pos").unwrap().as_ptr());
        gl::EnableVertexAttribArray(pos_attrib as GLuint);
        gl::VertexAttribPointer(pos_attrib as GLuint, 2, gl::FLOAT, gl::FALSE, 2 * std::mem::size_of::<GLfloat>() as GLsizei, ptr::null());
        
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }
    
    // Create an instance of MyWindow
    let mut my_window = MyWindow::new();
    
    
    let now: Instant = Instant::now();
    let delta_time: f32 = now.duration_since(last_frame_time).as_secs_f32();
    'running: loop {
        let timernow: Instant = Instant::now();
        let timer: f32 = timernow.duration_since(last_frame_time).as_secs_f32();
        egui_state.input.time = Some(start_time.elapsed().as_secs_f64());
        egui_ctx.begin_frame(egui_state.input.take());

        let frame_time = get_frame_time(start_time);
        let frame = Frame::new(FrameData {
            info: IntegrationInfo {
                web_info: None,
                cpu_usage: Some(frame_time),
                native_pixels_per_point: Some(egui_state.native_pixels_per_point),
                prefer_dark_mode: None,
                name: "egui + sdl2 + gl",
            },
            output: Default::default(),
            repaint_signal: repaint_signal.clone(),
        });

        //demo_windows.ui(&egui_ctx);

        egui::Window::new("Sandbox Window")
            .resizable(true) // Allow resizing
            .default_size([800.0, 800.0]) // Set default size
            .min_size([200.0, 150.0]) // Set minimum size
            .max_size([800.0, 800.0]) // Set maximum size
            .hscroll(false) // Disable scrolling if not needed
            .show(&egui_ctx, |ui| {
                my_window.ui(ui);
            });

        // Update sphere position in compute shader
        let camera_pos= my_camera.position;
        let camera_front=my_camera.front;
        let camera_up=my_camera.up;
        let camera_right=my_camera.right;
        let camera_fov=my_camera.fov;
        let spheres_position=my_window.spheres_position.clone();
        let sphere_color=my_window.spheres_color.clone();
        let sphere_radius=my_window.spheres_radius.clone();
        let sphere_roughness=my_window.spheres_roughness.clone();
        let sphere_emission=my_window.spheres_emission.clone();
        let is_accumulation=my_window.is_accumulation.clone();
        let skycolor=my_window.skycolor.clone();
        // println!("{:?}", materials);

        unsafe {
            gl::UseProgram(compute_shader_program);
            let camera_position_loc = gl::GetUniformLocation(compute_shader_program, CString::new("camera_pos").unwrap().as_ptr());
            gl::Uniform3f(camera_position_loc, camera_pos[0], camera_pos[1], camera_pos[2]);

            let camera_front_loc = gl::GetUniformLocation(compute_shader_program, CString::new("camera_front").unwrap().as_ptr());
            gl::Uniform3f(camera_front_loc, camera_front[0], camera_front[1], camera_front[2]);

            let camera_up_loc = gl::GetUniformLocation(compute_shader_program, CString::new("camera_up").unwrap().as_ptr());
            gl::Uniform3f(camera_up_loc, camera_up[0], camera_up[1], camera_up[2]);
            
            let camera_right_loc = gl::GetUniformLocation(compute_shader_program, CString::new("camera_right").unwrap().as_ptr());
            gl::Uniform3f(camera_right_loc, camera_right[0], camera_right[1], camera_right[2]);

            let camera_fov_loc = gl::GetUniformLocation(compute_shader_program, CString::new("fov").unwrap().as_ptr());
            gl::Uniform1f(camera_fov_loc, camera_fov);

            let sphere_pos_loc = gl::GetUniformLocation(compute_shader_program, CString::new("spheres_position").unwrap().as_ptr());
            let sphere_color_loc = gl::GetUniformLocation(compute_shader_program, CString::new("spheres_color").unwrap().as_ptr());
            let sphere_radius_loc = gl::GetUniformLocation(compute_shader_program, CString::new("spheres_radius").unwrap().as_ptr());
            let roughness_loc = gl::GetUniformLocation(compute_shader_program, CString::new("spheres_roughness").unwrap().as_ptr());
            let emission_loc = gl::GetUniformLocation(compute_shader_program, CString::new("spheres_emission").unwrap().as_ptr());
            let time_loc = gl::GetUniformLocation(compute_shader_program, CString::new("currentTime").unwrap().as_ptr());

            let accumulation_loc = gl::GetUniformLocation(compute_shader_program, CString::new("is_accumulation").unwrap().as_ptr());
            let skycolor_loc = gl::GetUniformLocation(compute_shader_program, CString::new("skycolor").unwrap().as_ptr());
            let camera_vel_loc = gl::GetUniformLocation(compute_shader_program, CString::new("camera_velocity").unwrap().as_ptr());

            gl::Uniform1f(time_loc as GLint, timer);
            gl::Uniform1i(accumulation_loc as GLint, is_accumulation);
            gl::Uniform3f(skycolor_loc as GLint, skycolor.x/255.0, skycolor.y/255.0, skycolor.z/255.0);
            gl::Uniform3f(camera_vel_loc as GLint, my_camera.velocity.x, my_camera.velocity.y, my_camera.velocity.z);

            for i in 0..spheres_position.len() {
                    gl::Uniform3f(sphere_pos_loc + i as GLint, spheres_position[i].x, spheres_position[i].y, spheres_position[i].z);
                    gl::Uniform3f(sphere_color_loc + i as GLint, sphere_color[i].x, sphere_color[i].y, sphere_color[i].z);
                    gl::Uniform1f(sphere_radius_loc + i as GLint, sphere_radius[i]);
                    gl::Uniform1f(roughness_loc + i as GLint, sphere_roughness[i]);
                    gl::Uniform1f(emission_loc + i as GLint, sphere_emission[i]);
                }


            gl::DispatchCompute(SCREEN_WIDTH / 8, SCREEN_HEIGHT / 8, 1);
            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
        }
        //////
        let FullOutput {
            platform_output,
            textures_delta,
            shapes,
            pixels_per_point,
            viewport_output,
        } = egui_ctx.end_frame();
        egui_state.process_output(&window, &platform_output);

        if frame.take_app_output().quit {
            break 'running;
        }

        let repaint_after = viewport_output
            .get(&ViewportId::ROOT)
            .expect("Missing ViewportId::ROOT")
            .repaint_delay;
        my_camera.update(delta_time);


// Event handling loop
for event in event_pump.poll_iter() {
    match event {
        Event::Quit { .. } => break 'running,
        Event::Window{
            win_event: WindowEvent::Resized(width,hegith),
            ..
        }=>{
            SCREEN_HEIGHT=hegith as u32;
            SCREEN_WIDTH=width as u32;
            unsafe {
                gl::Viewport(0,0,SCREEN_WIDTH as i32,SCREEN_HEIGHT as i32);
            };
            texture = unsafe { create_texture(SCREEN_WIDTH, SCREEN_HEIGHT)}
        }
        Event::KeyDown { keycode: Some(Keycode::W), .. } => {
            my_camera.process_keyboard(CameraMovement::Forward, delta_time);
            my_window.is_accumulation=0;
        }
        Event::KeyDown { keycode: Some(Keycode::A), .. } => {
            my_camera.process_keyboard(CameraMovement::Left, delta_time);
            my_window.is_accumulation=0;
        }
        Event::KeyDown { keycode: Some(Keycode::D), .. } => {
            my_camera.process_keyboard(CameraMovement::Right, delta_time);
            my_window.is_accumulation=0;
        }
        Event::KeyDown { keycode: Some(Keycode::S), .. } => {
            my_camera.process_keyboard(CameraMovement::Backward, delta_time);
            my_window.is_accumulation=0;
        }
        //
        Event::KeyUp { keycode: Some(Keycode::W), .. } => {
            if moveCamera==false{
                my_window.is_accumulation=1;

            }        }
        Event::KeyUp { keycode: Some(Keycode::A), .. } => {
            if moveCamera==false{
                my_window.is_accumulation=1;
            }        }
        Event::KeyUp { keycode: Some(Keycode::D), .. } => {
            if moveCamera==false{
                my_window.is_accumulation=1;
            }        }
        Event::KeyUp { keycode: Some(Keycode::S), .. } => {
            if moveCamera==false{
                my_window.is_accumulation=1;
            }
        }
        Event::MouseButtonDown { timestamp, window_id, which, mouse_btn, clicks, x, y }=>{
             match mouse_btn {
                sdl2::mouse::MouseButton::Left => {
                    egui_state.process_input(&window, event, &mut painter);
                }
                sdl2::mouse::MouseButton::Right => {
                    moveCamera = true;
                    my_window.is_accumulation = 0;
                    // Handle right button down event if needed
                }
                _ => {}
        }}
        Event::MouseButtonUp { timestamp, window_id, which, mouse_btn, clicks, x, y }=>{
             match mouse_btn {
                sdl2::mouse::MouseButton::Left => {
                    egui_state.process_input(&window, event, &mut painter);
                }
                sdl2::mouse::MouseButton::Right => {
                    moveCamera = false;
                    my_window.is_accumulation = 1;
                    // Handle right button down event if needed
                }
                _ => {}
        }}
        // Evenst
        Event::MouseMotion { xrel, yrel,..} => {
            if moveCamera==false{
                egui_state.process_input(&window, event, &mut painter);
            }else{
                my_camera.process_mouse_movement(xrel as f32, -yrel as f32, true);
                my_window.is_accumulation=0;
            }
        }
        _ => {
            // Pass other SDL2 events to egui for processing
                egui_state.process_input(&window, event, &mut painter);

            }
        }
}



        // Use the compute shader program to process the texture
        unsafe {
            gl::UseProgram(compute_shader_program);
            gl::DispatchCompute(SCREEN_WIDTH / 8, SCREEN_HEIGHT / 8, 1);
            gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
        }

        // Render the texture to the screen
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::UseProgram(quad_shader_program);
            gl::BindVertexArray(vao);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }

        let paint_jobs: Vec<ClippedPrimitive> = egui_ctx.tessellate(shapes, pixels_per_point);
        painter.paint_jobs(None, textures_delta, paint_jobs);

        window.gl_swap_window();
    }

}

struct MyWindow {
    
    new_sphere_roughness: f32,
    spheres_roughness: Vec<f32>,
    spheres_position: Vec<Vector3<f32>>, // Vector of positions for spheres
    spheres_radius: Vec<f32>,            // Vector of radii for spheres
    spheres_color: Vec<Vector3<f32>>,    // Vector of colors for spheres
    new_sphere_position: Vector3<f32>,   // New sphere position to be added
    new_sphere_radius: f32,              // New sphere radius to be added
    new_sphere_color: Vector3<f32>,      // New sphere color to be added
    new_sphere_emission:f32,
    is_accumulation:i32,
    spheres_emission:Vec<f32>,
    skycolor:Vector3<f32>,
}

impl MyWindow {
    fn new() -> Self {
        Self {
            spheres_position: vec![Vector3::new(0.0, 0.0, 0.0),Vector3::new(12.4, 2.9, -14.2),Vector3::new(0.0, -100.0, 0.0)], // Initial position (example)
            spheres_radius: vec![1.0,9.1,99.0],                          // Initial radius (example)
            spheres_color: vec![Vector3::new(0.0, 0.0, 0.0),Vector3::new(204.0, 128.0, 51.0),Vector3::new(255.0, 170.0, 155.0)],    // Initial color (example)
            spheres_emission:vec![0.0,0.0,0.0],
            spheres_roughness: vec![0.2,1.0,1.0],
            new_sphere_position: Vector3::new(0.0,0.0,0.0),
            new_sphere_radius: 1.0,
            new_sphere_color: Vector3::new(1.0, 0.0, 1.0),
            new_sphere_roughness: 1.0,
            new_sphere_emission: 0.3,
            skycolor: Vector3::new(35.0,255.0,255.0),
            is_accumulation:1,
        }
    }

fn ui(&mut self, ui: &mut egui::Ui) {
    // Spheres section
    ui.label("Spheres:");
    for i in 0..self.spheres_position.len() {
        ui.collapsing(format!("Sphere {}", i + 1), |ui| {
            ui.label(format!("Sphere {}", i + 1));

            ui.vertical(|ui| {
                ui.add(
                    egui::Slider::new(&mut self.spheres_position[i].x, -100.0..=100.0)
                        .text("X")
                        .clamp_to_range(true),
                );
                ui.add(
                    egui::Slider::new(&mut self.spheres_position[i].y, -100.0..=100.0)
                        .text("Y")
                        .clamp_to_range(true),
                );
                ui.add(
                    egui::Slider::new(&mut self.spheres_position[i].z, -100.0..=100.0)
                        .text("Z")
                        .clamp_to_range(true),
                );
            });

            ui.vertical(|ui| {
                ui.add(
                    egui::Slider::new(&mut self.spheres_radius[i], 0.1..=100.0)
                        .text("Radius")
                );
                ui.add(
                    egui::Slider::new(&mut self.spheres_emission[i], 0.0..=100.0)
                        .text("Emission")
                );
                ui.add(
                 egui::Slider::new(&mut self.spheres_roughness[i], 0.0..=1.0)
                        .text("Roughness")
                        .clamp_to_range(true),
                );
                ui.add(
                    egui::Slider::new(&mut self.spheres_color[i].x, 0.0..=255.0)
                        .text("R")
                        .clamp_to_range(true),
                );
                ui.add(
                    egui::Slider::new(&mut self.spheres_color[i].y, 0.0..=255.0)
                        .text("G")
                        .clamp_to_range(true),
                );
                ui.add(
                    egui::Slider::new(&mut self.spheres_color[i].z, 0.0..=255.0)
                        .text("B")
                        .clamp_to_range(true),
                );
                if ui.add(egui::Button::new("Duplicate Sphere")).clicked() {
                    self.spheres_position.push(self.spheres_position[i]);
                    self.spheres_roughness.push(self.spheres_roughness[i]);
                    self.spheres_radius.push(self.spheres_radius[i]);
                    self.spheres_color.push(self.spheres_color[i]);
                    self.spheres_emission.push(self.spheres_emission[i]);
                }
                
            });
        });
    }

    // Add new sphere section
    ui.vertical_centered(|ui| {
        ui.collapsing("New Sphere", |ui| {
            ui.label("New Sphere:");

            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut self.new_sphere_position.x, -100.0..=100.0)
                        .text("X")
                );
                ui.add(
                    egui::Slider::new(&mut self.new_sphere_position.y, -100.0..=100.0)
                        .text("Y")
                );
                ui.add(
                    egui::Slider::new(&mut self.new_sphere_position.z, -100.0..=100.0)
                        .text("Z")
                );
            });

            ui.horizontal(|ui| {
                ui.add(
                    egui::Slider::new(&mut self.new_sphere_radius, 0.1..=100.0)
                        .text("Radius")
                );
                ui.add(
                    egui::Slider::new(&mut self.new_sphere_color.x, 0.0..=255.0)
                        .text("R")
                        .clamp_to_range(true),
                );
                ui.add(
                    egui::Slider::new(&mut self.new_sphere_color.y, 0.0..=255.0)
                        .text("G")
                        .clamp_to_range(true),
                );
                ui.add(
                    egui::Slider::new(&mut self.new_sphere_color.z, 0.0..=255.0)
                        .text("B")
                        .clamp_to_range(true),
                );
            });
            ui.add(
                    egui::Slider::new(&mut self.new_sphere_roughness, 0.0..=1.0)
                        .text("Roughness")
                        .clamp_to_range(true),
                );
            ui.add(
                    egui::Slider::new(&mut self.new_sphere_emission, 0.0..=100.0)
                        .text("Emission")
                );

            if ui.add(egui::Button::new("Add Sphere")).clicked() {
                self.spheres_position.push(self.new_sphere_position);
                self.spheres_roughness.push(self.new_sphere_roughness);
                self.spheres_radius.push(self.new_sphere_radius);
                self.spheres_color.push(self.new_sphere_color);
                self.spheres_emission.push(self.new_sphere_emission);
            }
            
        });
    });
    ui.label("Scene:");
    ui.collapsing(format!("Scene"), |ui| {

    ui.add(
    
    egui::Slider::new(&mut self.is_accumulation, 0..=1)
        .text("Enable accumulation")
    );
    
    ui.horizontal(|ui| {
    ui.add(
    egui::Slider::new(&mut self.skycolor.x, 0.0..=255.0)
        .text("R")
        .clamp_to_range(true),
    );
    ui.add(
    egui::Slider::new(&mut self.skycolor.y, 0.0..=255.0)
        .text("G")
        .clamp_to_range(true),
    );
    ui.add(
    egui::Slider::new(&mut self.skycolor.z, 0.0..=255.0)
        .text("B")
        .clamp_to_range(true),
    );});
});
}



}

fn create_texture(width: u32, height: u32) -> GLuint {
    let mut texture = 0;
    unsafe {
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA32F as i32,
            width as i32,
            height as i32,
            0,
            gl::RGBA,
            gl::FLOAT,
            std::ptr::null(),
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::BindImageTexture(0, texture, 0, gl::FALSE, 0, gl::WRITE_ONLY, gl::RGBA32F);
    }
    texture
}
