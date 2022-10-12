use std::collections::HashMap;

use macroquad::{hash, prelude::*, ui::root_ui};

fn draw_gate(name: &str, x: f32, y: f32, w: f32, h: f32) {
    let whitish = Color::from_rgba(0xcc, 0xcc, 0xcc, 0xff);
    draw_rectangle(x, y, w, h, whitish);
    let (font_size, font_scale, font_aspect) = camera_font_scale(h / 2.);
    let text_params = TextParams {
        font_size,
        font_scale,
        font_scale_aspect: font_aspect,
        color: BLACK,
        ..Default::default()
    };
    let dimensions = measure_text(name, None, font_size, font_scale);
    draw_text_ex(
        name,
        x + (w - dimensions.width) / 2.,
        y + (h - dimensions.height) / 2. + dimensions.offset_y,
        text_params,
    );
}

trait Gate<const INPUTS: usize, const OUTPUTS: usize> {
    const NAME: &'static str;

    fn update(&self, inputs: &[bool; INPUTS], outputs: &mut [bool; OUTPUTS]);
}

struct And;

impl Gate<2, 1> for And {
    const NAME: &'static str = "AND";

    fn update(&self, inputs: &[bool; 2], outputs: &mut [bool; 1]) {
        outputs[0] = inputs[0] && inputs[1];
    }
}

struct Or;

impl Gate<2, 1> for Or {
    const NAME: &'static str = "OR";

    fn update(&self, inputs: &[bool; 2], outputs: &mut [bool; 1]) {
        outputs[0] = inputs[0] || inputs[1];
    }
}

struct Xor;

impl Gate<2, 1> for Xor {
    const NAME: &'static str = "XOR";

    fn update(&self, inputs: &[bool; 2], outputs: &mut [bool; 1]) {
        outputs[0] = inputs[0] != inputs[1];
    }
}

struct Not;

impl Gate<1, 1> for Not {
    const NAME: &'static str = "NOT";

    fn update(&self, inputs: &[bool; 1], outputs: &mut [bool; 1]) {
        outputs[0] = !inputs[0];
    }
}

type UpdateFn = Box<dyn Fn(&[bool], &mut [bool])>;

struct GateState {
    inputs: Box<[bool]>,
    outputs: Box<[bool]>,
    update_fn: UpdateFn,
}

impl GateState {
    fn update(&mut self) {
        (self.update_fn)(&self.inputs, &mut self.outputs);
    }
}

struct Simulation {
    counter: usize,
    gates: HashMap<usize, GateState>,
    connections: Vec<(usize, usize, usize, usize)>,
}

impl Simulation {
    fn new() -> Simulation {
        Simulation {
            counter: 0,
            gates: HashMap::new(),
            connections: Vec::new(),
        }
    }

    fn add_gate<const INPUTS: usize, const OUTPUTS: usize>(
        &mut self,
        gate: impl Gate<INPUTS, OUTPUTS> + 'static,
    ) {
        let inputs = Box::new([false; INPUTS]);
        let outputs = Box::new([false; OUTPUTS]);
        let id = self.counter;

        let update_fn: UpdateFn = Box::new(move |inputs, outputs| {
            gate.update(inputs.try_into().unwrap(), outputs.try_into().unwrap())
        });

        self.gates.insert(
            id,
            GateState {
                inputs,
                outputs,
                update_fn,
            },
        );
        self.counter += 1;
    }

    fn add_connection(&mut self, from: usize, output: usize, to: usize, input: usize) {
        self.connections.push((from, output, to, input));
    }

    fn simulate(&mut self) {
        for (from, output, to, input) in &self.connections {
            let output_state = self.gates.get(from).unwrap().outputs[*output];
            self.gates.get_mut(to).unwrap().inputs[*input] = output_state;
        }

        for (_, state) in &mut self.gates {
            state.update();
        }
    }
}

#[macroquad::main("logic-sim")]
async fn main() {
    let mut sim = Simulation::new();
    sim.add_gate(And);

    let blackish = Color::from_rgba(0x1e, 0x1e, 0x1e, 0xff);
    let mut last_update = get_time();
    let mut frequency = 10f32;
    let mut elapsed_remainder = 0f64;
    loop {
        clear_background(blackish);

        let period = (1.0 / frequency) as f64;
        let elapsed = get_time() - last_update;
        let iterations = (elapsed / period) + elapsed_remainder;
        if iterations >= 1.0 {
            last_update = get_time();
            elapsed_remainder = iterations.fract();

            let iterations = iterations.trunc() as usize;

            // println!("{:.5} {:.5} {:.5} {:.5} {:<5} {:.5}", elapsed, period, elapsed / period, elapsed % period, iterations, elapsed_remainder);

            println!("iterations {}", iterations);
            for _ in 0..iterations {
                println!("tick");
                sim.simulate();
            }
        }

        draw_gate("AND", 0., 0., 50.0, 50.0);
        draw_gate("OR", 50., 0., 50.0, 50.0);
        draw_gate("NOT", 100., 0., 50.0, 50.0);
        draw_gate("XOR", 150., 0., 50.0, 50.0);

        // root_ui().window(hash!(), vec2(0.0, 0.0), vec2(200.0, 400.0), |ui| {
        //     ui.slider(hash!(), "Frequency Hz", 1f32..100f32, &mut frequency);
        // });

        next_frame().await
    }
}
