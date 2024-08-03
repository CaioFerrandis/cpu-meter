use std::f32::consts::PI;

use chaos_framework::*;

use sysinfo::System;

fn main() {
    let mut el = EventLoop::new(600, 600);
    let mut renderer = Renderer::new();
    renderer.camera.set_projection(ProjectionType::Orthographic);

    unsafe {
        Enable(DEPTH_TEST);
    }

    renderer.add_light(Light {position: vec3(0.0, 0.0, 10.0), color: vec3(1.0, 1.0, 1.0)});

    let mut meter = Meter::new(&mut renderer);

    while !el.window.should_close(){
        el.update();
        renderer.update();

        meter.update(&mut renderer, &mut el);

        unsafe {
            Clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
            ClearColor(0.1, 0.2, 0.3, 1.0);
            renderer.draw();
        }
    }    
}

struct Dial {
    pointer: MeshHandle,
    circle: MeshHandle,
}

impl Dial {
    pub fn new(renderer: &mut Renderer, id: usize) -> Self {
        let mut model = Model::new("src/objects/pointer.obj");
        model.meshes[0].scale(Vec3::ONE*0.1);
        model.meshes[0].set_position(vec3(-1., 0., 0.));
        let pointer = model.meshes[0].clone();

        let pointer_handle = renderer.add_mesh(pointer).unwrap();

        renderer.get_mesh_mut(pointer_handle).unwrap().add_position(vec3(0.5*id as f32, 0., 0.));

        let mut model = Model::new("src/objects/dial.obj");
        model.meshes[0].scale(Vec3::ONE*0.1);
        model.meshes[0].set_position(vec3(-1., 0., 0.));
        let pointer = model.meshes[0].clone();

        let circle_handle = renderer.add_mesh(pointer).unwrap();
        
        renderer.get_mesh_mut(circle_handle).unwrap().add_position(vec3(0.5*id as f32, 0., 0.));

        Self {
            pointer: pointer_handle,
            circle: circle_handle,
        }
    }

    pub fn update(&mut self, renderer: &mut Renderer, usage: f32) {
        renderer.meshes[self.pointer]
                .set_rotation(Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, usage*std::f32::consts::PI*2.));
    }
}

struct Meter {
    system: System,
    dials: Vec<Dial>,
}

impl Meter {
    pub fn new(renderer: &mut Renderer) -> Self {
        let mut system = System::new();
        let mut dials = vec![];

        system.refresh_cpu_usage();

        let cpus_len = system.cpus().len();

        for i in 0..cpus_len {
            dials.push(Dial::new(renderer, i));
        }

        Self {
            system,
            dials,
        }
    }

    pub fn update(&mut self, renderer: &mut Renderer, el: &mut EventLoop) {
        self.system.refresh_cpu_usage();
        let cpus = self.system.cpus();

        for i in 0..cpus.len() {
            let usage = cpus[i].cpu_usage() * 0.01; // normalize range from 0..100 to 0..1
            if i == 0 {}
            self.dials[i].update(renderer, usage);
        }   
    }
}
