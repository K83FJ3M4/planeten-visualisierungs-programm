use bytemuck::{Pod, Zeroable};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{vertex_attr_array, Buffer, BufferUsages, Device, Queue, VertexBufferLayout, VertexStepMode};
#[cfg(target_arch = "wasm32")]
use web_time::Duration;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct PlanetInstance {
    position: [f32; 4],
    color: [f32; 4],
}

pub(super) struct System {
    offset: usize,
    num_planets: usize,
    planets: Vec<Vec<PlanetInstance>>,
    colors: Vec<[f32; 4]>,
    interval: Duration,
    last_update: Instant,
    pub(super) planet_buffer: Buffer,
    scale: f64,
}

//Data Format
// x, y, z, x1, y1, z1, ... for every planet followed by new lines and the new cooridnates for the next time step


impl System {
    pub(super) fn speed_up(&mut self) {
        self.interval = self.interval.div_f64(2.0);
    }

    pub(super) fn slow_down(&mut self) {
        self.interval = self.interval.mul_f64(2.0);
    }

    pub(super)  fn new(device: &Device, content: String) -> System {

        let mut planets = Vec::new();
        let mut starting_planets = Vec::new();
        for line in content.lines() {
            let mut new_planets = Vec::new();
            let mut iter = line.split_whitespace()
                .map(|s| s.parse::<f64>().unwrap_or_default())
                .peekable();


            while iter.peek().is_some() {
                let x = iter.next().unwrap_or_default();
                let y = iter.next().unwrap_or_default();
                let z = iter.next().unwrap_or_default();

                new_planets.push(PlanetInstance {
                    position: [x as f32, y as f32, z as f32, 1.0],
                    color: [1.0, 1.0, 1.0, 1.0]
                });
            }

            if new_planets.len() > starting_planets.len() {
                starting_planets = new_planets.clone();
            }

            planets.push(new_planets);
        }

        if starting_planets.is_empty() {
            starting_planets.push(PlanetInstance {
                position: [0.0, 0.0, 0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0]
            });
        }

        let max_distance = starting_planets.iter()
            .map(|planet| (planet.position.iter().map(|&x| x * x).sum::<f32>()).sqrt())
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(1.0);

        let scale = if max_distance.abs() > 0.01 {
            10.0 / max_distance as f64
        } else {
            1.0
        };

        let mut colors = Vec::new();
        for i in 0..starting_planets.len() {
            let mut random = Self::random(i as u32).to_ne_bytes().map(|b| b as f32 / 256.0);
            random[3] = 0.0;
            colors.push(random);
        }

        let planet_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&starting_planets),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST
        });

        System {
            offset: 0,
            planets,
            planet_buffer,
            interval: Duration::from_millis(100),
            last_update: Instant::now(),
            num_planets: starting_planets.len(),
            scale,
            colors
        }
    }

    fn random(seed: u32) -> u32 {
        let state = seed;
        state.wrapping_mul(1664525).wrapping_add(1013904223)
    }

    pub(super) fn step(&mut self, queue: &Queue) -> u32 {
        let planets_len = self.planets.len();
        let Some(mut planets) = self.planets.get(self.offset).cloned() else {
            return 0;
        };

        for (planet, color) in planets.iter_mut().zip(self.colors.iter()) {
            planet.position[0] *= self.scale as f32;
            planet.position[1] *= self.scale as f32;
            planet.position[2] *= self.scale as f32;
            planet.color = *color;
        }

        let now = Instant::now();
        let intervals = (now - self.last_update).as_millis() / self.interval.as_millis();
        self.offset += intervals as usize;
        self.last_update += self.interval * intervals as u32;

        if self.offset >= planets_len {
            self.offset = 0;
        }


        queue.write_buffer(&self.planet_buffer, 0, bytemuck::cast_slice(planets.as_slice()));
        self.num_planets as u32
    }
}

impl PlanetInstance {
    pub(super) fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: const {
                &vertex_attr_array![
                    2 => Float32x4,
                    3 => Float32x4
                ]
            }
        }
    }
}