use chaos_framework::*;
use tokio::sync::Mutex;

use std::{sync::{Arc, LazyLock}};
use sysinfo::System;

static SYSTEM: LazyLock<Arc<Mutex<System>>> = LazyLock::new(|| {
    Arc::new(Mutex::new(System::new()))
});

async fn update_system() {
    {
        let mut system = SYSTEM.lock().await;
        system.refresh_cpu_usage();
    }

    tokio::time::sleep(std::time::Duration::from_millis(128)).await;
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let mut el = EventLoop::new(600, 600);
    let mut renderer = Renderer::new();

    el.window.glfw.set_swap_interval(SwapInterval::Sync(1));

    renderer.camera.set_projection(ProjectionType::Perspective);

    tokio::task::spawn(async {
        loop {
            update_system().await;
        }
    });

    unsafe {
        Enable(DEPTH_TEST);
    }

    let light_ofs = vec3(0.0, 0.0, 10.0);

    let l0 = renderer.add_light(Light {position: light_ofs, color: vec3(1.0, 1.0, 1.0)})
        .unwrap();
    let l1 = renderer.add_light(Light {position: -light_ofs, color: vec3(1.0, 1.0, 1.0)})
        .unwrap();


    let mut meter = Meter::new(&mut renderer).await.unwrap();

    renderer.update();
    while !el.window.should_close(){
        el.update();

        renderer.camera.update(renderer.camera.pos, &el);
        
        if el.is_key_down(glfw::Key::W){
            renderer.camera.pos.y += el.dt;
        }
        if el.is_key_down(glfw::Key::S){
            renderer.camera.pos.y -= el.dt;
        }
        if el.is_key_down(glfw::Key::A){
            renderer.camera.pos.x -= el.dt;
        }
        if el.is_key_down(glfw::Key::D){
            renderer.camera.pos.x += el.dt;
        }

        renderer.camera.pos.z -= el.event_handler.scroll[1];
        
        meter.update(&mut renderer, &mut el).await;

        // let mut light_position = renderer.camera.pos + light_ofs;
        // light_position.z = 10.0; // don't vary in depth
        // renderer.lights[l0].position = light_position;
        // renderer.lights[l1].position = -light_position;

        unsafe {
            Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
            ClearColor(0.1, 0.2, 0.3, 1.0);

            renderer.draw();
        }
    }
}

struct Dial {
    pointer: MeshHandle,
    _circle: MeshHandle,

    goal: Quat,
    sod: SecondOrderDynamics<Vec3>,
}

impl Dial {
    pub fn new(renderer: &mut Renderer, id: usize) -> Self {
        // Setting up position
        let mut x = 0.;
        if id % 2 != 0{
            x = 1.75;
        }

        let y = (id as f32/2.).floor()*1.75;

        let scale_factor = 0.05;

        // Setting up pointer

        let mut model = Model::new("src/objects/dial.obj");
        model.meshes[0].scale(Vec3::ONE*scale_factor);
        model.meshes[0].set_position(vec3(-1.+x, 1.-y, 0.));
        let mut pointer = model.meshes[0].clone();
        pointer.color = vec3(1.0, 0.0, 0.0);

        let pointer_handle = renderer.add_mesh(pointer).unwrap();

        // Setting up dial
        model.meshes[1].scale(Vec3::ONE*(scale_factor));
        model.meshes[1].set_position(vec3(-1.+x, 1.-y, 0.));
        let ring = model.meshes[1].clone();
        
        let circle_handle = renderer.add_mesh(ring).unwrap();
        //                              spring, damp, antecipador
        let sod = SecondOrderDynamics::new(1.0, 0.8, 0.0, Vec3::ZERO);
        
        Self {
            sod,
            pointer: pointer_handle,    
            _circle: circle_handle,
            goal: quat(0.0, 0.0, 0.0, 1.0),
        }
    }


    pub fn update(&mut self, renderer: &mut Renderer, usage: f32, el: &EventLoop) {
        self.goal = Quat::from_axis_angle(Vec3::Z, usage);

        let euler = self.goal.to_euler(EulerRot::XYZ);

        let y = self.sod.update(el.dt / 0.5, vec3(euler.0, euler.1, euler.2));

        renderer.meshes[self.pointer].rotation = Quat::from_euler(EulerRot::XYZ, y.x, y.y, y.z);
    }
}

struct Meter {
    dials: Vec<Dial>,
}

impl Meter {
    pub async fn new(renderer: &mut Renderer) -> Option<Self> {
        let mut dials = vec![];

        let mut system = SYSTEM.lock().await;

        system.refresh_cpu_usage();
        let cpus_len = system.cpus().len();

        if cpus_len == 0 {
            return None;
        }
        
        for i in 0..cpus_len {
            dials.push(Dial::new(renderer, i));
        }

        return Some(Self {
            dials,
        });

    }

    pub async fn update(&mut self, renderer: &mut Renderer, el: &mut EventLoop) {
        if let Ok(system) = SYSTEM.try_lock() {
            let cpus = system.cpus();
            
            for i in 0..cpus.len() {
                // cpus[i].cpu_usage()
                let usage = (cpus[i].cpu_usage() * 0.01) * 250.0_f32.to_radians();
                
                self.dials[i].update(renderer, 125.0_f32.to_radians()-usage, &el);
            }
        }

    }
}

struct Bar {
    bar_box: MeshHandle,
    move_bar: MeshHandle,

    sod: SecondOrderDynamics<Vec3>,
}

impl Bar {
    pub fn new(renderer: &mut Renderer, id: usize) -> Self {
        let sod = SecondOrderDynamics::new(0.5, 0.3, 0.0, Vec3::ZERO);

        let bar_box = renderer.add_mesh(Quad::new(Vec3::ONE, Vec4::ONE).mesh()).unwrap();
        let move_bar = renderer.add_mesh(Quad::new(Vec3::ONE, Vec4::ONE).mesh()).unwrap();
        
        Self {
            sod,
            bar_box,
            move_bar,
        }
    
    }

    pub fn update(&mut self, renderer: &mut Renderer, usage: f32, el: &EventLoop) {
        let bar = &mut renderer.meshes[self.bar_box];
        bar.scale = vec3(0.3, 0.1, 0.0);

        let usage_max = 1.0;
        bar.scale = vec3(usage/usage_max, 0.1, 0.0);


        let bar_box = &mut renderer.meshes[self.move_bar];
        bar_box.scale = vec3(0.3, 0.1, 0.0);
    }
}
