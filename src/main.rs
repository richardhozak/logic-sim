use std::collections::HashMap;

use macroquad::{hash, prelude::*, ui::root_ui};

fn is_point_inside_box(
    (point_x, point_y): (f32, f32),
    (box_x, box_y, box_w, box_h): (f32, f32, f32, f32),
) -> bool {
    point_x > box_x && point_x < box_x + box_w && point_y > box_y && point_y < box_y + box_h
}

enum GateMouseHover {
    Input(usize),
    Output(usize),
    Gate,
}

fn draw_gate(
    name: &str,
    x: f32,
    y: f32,
    inputs: &[bool],
    outputs: &[bool],
) -> Option<GateMouseHover> {
    let max_io_len = usize::max(inputs.len(), outputs.len()) as f32;
    let io_h = 20f32;
    let io_w = 20f32;
    let io_spacing = 5f32;
    let h = max_io_len * io_h + max_io_len * io_spacing + io_spacing;
    let w = h;

    let (font_size, font_scale, font_aspect) = camera_font_scale(h / 2.);
    let text_params = TextParams {
        font_size,
        font_scale,
        font_scale_aspect: font_aspect,
        color: BLACK,
        ..Default::default()
    };

    let text_dimensions = measure_text(name, None, font_size, font_scale);

    let whitish = Color::from_rgba(0xcc, 0xcc, 0xcc, 0xff);
    draw_rectangle(x, y, w, h, whitish);

    let mouse_pos = mouse_position();
    let mut mouse_hover = None;

    let dt = h / inputs.len() as f32;
    for (index, state) in inputs.iter().enumerate() {
        let t = 0.5 * dt + index as f32 * dt;
        let in_x = x - io_w / 2.;
        let in_y = y + t - (io_h / 2.);
        draw_rectangle(in_x, in_y, io_w, io_h, if *state { RED } else { GRAY });

        if is_point_inside_box(mouse_pos, (in_x, in_y, io_w, io_h)) {
            mouse_hover = Some(GateMouseHover::Input(index));
            draw_rectangle_lines(in_x, in_y, io_w, io_h, 4f32, WHITE);
        }
    }

    let dt = h / outputs.len() as f32;
    for (index, state) in outputs.iter().enumerate() {
        let t = 0.5 * dt + index as f32 * dt;
        let out_x = x + w - io_w / 2.;
        let out_y = y + t - (io_h / 2.);
        draw_rectangle(out_x, out_y, io_w, io_h, if *state { RED } else { GRAY });

        if is_point_inside_box(mouse_pos, (out_x, out_y, io_w, io_h)) {
            mouse_hover = Some(GateMouseHover::Output(index));
            draw_rectangle_lines(out_x, out_y, io_w, io_h, 4f32, WHITE);
        }
    }

    draw_text_ex(
        name,
        x + (w - text_dimensions.width) / 2.,
        y + (h - text_dimensions.height) / 2. + text_dimensions.offset_y,
        text_params,
    );

    if mouse_hover.is_some() {
        mouse_hover
    } else if is_point_inside_box(mouse_pos, (x, y, w, h)) {
        Some(GateMouseHover::Gate)
    } else {
        None
    }
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

struct And3;

impl Gate<3, 1> for And3 {
    const NAME: &'static str = "AND3";

    fn update(&self, inputs: &[bool; 3], outputs: &mut [bool; 1]) {
        outputs[0] = inputs[0] && inputs[1] && inputs[2];
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

    fn get_gate_state(&self, id: usize) -> (&[bool], &[bool]) {
        let gate = self.gates.get(&id).unwrap();
        (&gate.inputs, &gate.outputs)
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
    sim.add_gate(Or);
    sim.add_gate(Not);
    sim.add_gate(Xor);
    sim.add_gate(And3);
    let mut board_gates = Vec::<(usize, &'static str, f32, f32)>::new();
    board_gates.push((0, "AND", 200., 0.));
    board_gates.push((1, "OR", 250., 0.));
    board_gates.push((2, "NOT", 300., 0.));
    board_gates.push((3, "XOR", 350., 0.));
    board_gates.push((4, "AND3", 450., 0.));

    let mut dragging: Option<usize> = None;

    let blackish = Color::from_rgba(0x1e, 0x1e, 0x1e, 0xff);
    let mut last_update = get_time();
    let mut frequency = 10f32;
    let mut elapsed_remainder = 0f64;
    loop {
        if is_mouse_button_released(MouseButton::Left) && dragging.is_some() {
            dragging = None;
        }

        clear_background(blackish);

        let period = (1.0 / frequency) as f64;
        let elapsed = get_time() - last_update;
        let iterations = (elapsed / period) + elapsed_remainder;
        if iterations >= 1.0 {
            last_update = get_time();
            elapsed_remainder = iterations.fract();

            let iterations = iterations.trunc() as usize;

            // println!("{:.5} {:.5} {:.5} {:.5} {:<5} {:.5}", elapsed, period, elapsed / period, elapsed % period, iterations, elapsed_remainder);

            // println!("iterations {}", iterations);
            for _ in 0..iterations {
                // println!("tick");
                sim.simulate();
            }
        }

        for &mut (id, name, ref mut x, ref mut y) in &mut board_gates {
            if let Some(dragging_id) = dragging {
                if dragging_id == id {
                    let pos = mouse_position();
                    println!("setting pos {:?}", pos);

                    *x = pos.0;
                    *y = pos.1;
                }
            }

            let (inputs, outputs) = sim.get_gate_state(id);
            if let Some(mouse_hover) = draw_gate(name, *x, *y, inputs, outputs) {
                match mouse_hover {
                    GateMouseHover::Input(input_id) => {
                        println!("input id {}", input_id);
                    },
                    GateMouseHover::Output(output_id) => {
                        println!("output id {}", output_id);
                    },
                    GateMouseHover::Gate => {
                        if dragging.is_none() {
                            if is_mouse_button_pressed(MouseButton::Left) {
                                dragging = Some(id);
                            }
                        }
                    },
                }
            }
        }

        root_ui().window(hash!(), vec2(0.0, 0.0), vec2(200.0, 400.0), |ui| {
            ui.slider(hash!(), "Frequency Hz", 1f32..100f32, &mut frequency);
        });

        next_frame().await
    }
}
