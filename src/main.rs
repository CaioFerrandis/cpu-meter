use chaos_framework::*;
use tokio::sync::Mutex;

use std::sync::{Arc, LazyLock};
use sysinfo::System;

static SYSTEM: LazyLock<Arc<Mutex<System>>> = LazyLock::new(|| {
    Arc::new(Mutex::new(System::new()))
});

async fn update_system() {
    {
        let mut system = SYSTEM.lock().await;
        system.refresh_cpu_usage();
    }

    tokio::time::sleep(std::time::Duration::from_millis(160)).await;
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let mut el = EventLoop::new(600, 600);
    let mut renderer = Renderer::new();

    el.window.glfw.set_swap_interval(SwapInterval::Sync(1));

    // renderer.camera.set_projection(ProjectionType::Orthographic);

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
        renderer.camera.input(&el);

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
        let mut model = Model::new("src/objects/pointer.obj");
        model.meshes[0].scale(Vec3::ONE*0.1);
        model.meshes[0].set_position(vec3(-1., 0., 0.));
        let mut pointer = model.meshes[0].clone();
        pointer.color = vec3(1.0, 0.0, 0.0);

        let pointer_handle = renderer.add_mesh(pointer).unwrap();

        renderer.get_mesh_mut(pointer_handle).unwrap().add_position(vec3(1.75*id as f32, 0., 0.));

        let mut model = Model::new("src/objects/dial.obj");
        model.meshes[0].scale(Vec3::ONE*0.1);
        model.meshes[0].set_position(vec3(-1., 0., 0.));
        let ring = model.meshes[0].clone();
        
        
        let circle_handle = renderer.add_mesh(ring).unwrap();
        
        renderer.get_mesh_mut(circle_handle).unwrap().add_position(vec3(1.75*id as f32, 0., 0.));

        let sod = SecondOrderDynamics::new(0.5, 0.3, 0.0, Vec3::ZERO);
        
        Self {
            sod,
            pointer: pointer_handle,    
            _circle: circle_handle,
            goal: quat(0.0, 0.0, 0.0, 1.0),
        }
    }


    pub fn update(&mut self, renderer: &mut Renderer, usage: f32) {
        self.goal = Quat::from_axis_angle(Vec3::Z, usage);

        let euler = self.goal.to_euler(EulerRot::XYZ);

        let y = self.sod.update(0.08, vec3(euler.0, euler.1, euler.2));

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
            if i == 0 {
                dials.push(Dial::new(renderer, i));
            }
        }

        return Some(Self {
            dials,
        });

    }

    pub async fn update(&mut self, renderer: &mut Renderer, el: &mut EventLoop) {
        if let Ok(system) = SYSTEM.try_lock() {
            let usage = (system.global_cpu_usage()* 0.01 ) * 4.5;

            self.dials[0].update(renderer, 2.25 - usage);
            
            // for i in 0..cpus.len() {
            //     // let usage = (cpus[i].cpu_usage()) * 0.001; // normalize range from 0..100 to 0..1
            //     let usage = (cpus[i].cpu_usage() * 0.01 ) * 4.5;
            //     
            //     self.dials[0].update(renderer, 2.25 - usage);
            // }
        }

    }
}
