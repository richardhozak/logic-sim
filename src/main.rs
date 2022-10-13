use std::collections::HashMap;

use gates::*;
use logic_simulation::LogicSimulation;
use macroquad::{hash, prelude::*, ui::root_ui};

mod gates;
mod logic_simulation;

fn is_point_inside_box(
    (point_x, point_y): (f32, f32),
    (box_x, box_y, box_w, box_h): (f32, f32, f32, f32),
) -> bool {
    point_x > box_x && point_x < box_x + box_w && point_y > box_y && point_y < box_y + box_h
}

enum GateMouseHover {
    Input(usize, Vec2),
    Output(usize, Vec2),
    Gate(Vec2),
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
            mouse_hover = Some(GateMouseHover::Input(index, (x, in_y + io_h / 2.).into()));
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
            mouse_hover = Some(GateMouseHover::Output(
                index,
                (x + w, out_y + io_h / 2.).into(),
            ));
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
        Some(GateMouseHover::Gate(mouse_pos.into()))
    } else {
        None
    }
}

struct BoardSimulation {
    sim: LogicSimulation,
    gates: HashMap<usize, Vec2>,
    connections: Vec<((usize, usize, Vec2), (usize, usize, Vec2))>,
}

impl BoardSimulation {
    fn new() -> BoardSimulation {
        BoardSimulation {
            sim: LogicSimulation::new(),
            gates: HashMap::new(),
            connections: Vec::new(),
        }
    }

    fn add_gate<const INPUTS: usize, const OUTPUTS: usize>(
        &mut self,
        gate: impl Gate<INPUTS, OUTPUTS> + 'static,
        pos: Vec2,
    ) {
        let gate_id = self.sim.add_gate(gate);
        self.gates.insert(gate_id, pos);
    }

    fn remove_gate(&mut self, gate_id: usize) {
        self.sim.remove_gate(gate_id);
        if let Some(_) = self.gates.remove(&gate_id) {
            self.connections
                .retain(|(output, input)| output.0 != gate_id && input.0 != gate_id);
        }
    }

    fn add_connection(
        &mut self,
        (input_gate_id, input_id, input_offset): (usize, usize, Vec2),
        (output_gate_id, output_id, output_offset): (usize, usize, Vec2),
    ) {
        self.sim
            .add_connection(output_gate_id, output_id, input_gate_id, input_id);
        self.connections.push((
            (output_gate_id, output_id, output_offset),
            (input_gate_id, input_id, input_offset),
        ));
    }

    fn remove_connection(&mut self, input: (usize, usize, Vec2), output: (usize, usize, Vec2)) {
        self.sim
            .remove_connection(output.0, output.1, input.0, input.1);
        self.connections
            .retain(|connection| *connection != (output, input));
    }
}

#[macroquad::main("logic-sim")]
async fn main() {
    let mut simulation = BoardSimulation::new();
    simulation.add_gate(And, vec2(220., 20.));
    simulation.add_gate(Or, vec2(300., 20.));
    simulation.add_gate(Not, vec2(390., 20.));
    simulation.add_gate(Yes, vec2(390., 60.));
    simulation.add_gate(Xor, vec2(460., 20.));
    simulation.add_gate(And3, vec2(550., 20.));

    let mut dragging: Option<(usize, Vec2)> = None;
    let mut selected_input: Option<(usize, usize, Vec2)> = None;
    let mut selected_output: Option<(usize, usize, Vec2)> = None;
    let mut to_remove: Option<usize> = None;
    let mut connection_to_remove: Option<((usize, usize, Vec2), (usize, usize, Vec2))> = None;

    let blackish = Color::from_rgba(0x1e, 0x1e, 0x1e, 0xff);
    let mut last_update = get_time();
    let mut frequency = 10f32;
    let mut elapsed_remainder = 0f64;

    loop {
        if is_mouse_button_released(MouseButton::Left) && dragging.is_some() {
            dragging = None;
        }

        if is_mouse_button_released(MouseButton::Right) {
            selected_input = None;
            selected_output = None;
        }

        if let (Some(input), Some(output)) = (selected_input, selected_output) {
            simulation.add_connection(input, output);
            selected_input = None;
            selected_output = None;
        }

        clear_background(blackish);

        let period = (1.0 / frequency) as f64;
        let elapsed = get_time() - last_update;
        let iterations = (elapsed / period) + elapsed_remainder;
        if iterations >= 1.0 {
            last_update = get_time();
            elapsed_remainder = iterations.fract();

            let iterations = iterations.trunc() as usize;

            for _ in 0..iterations {
                simulation.sim.simulate();
            }
        }

        for (&id, gate_pos) in &mut simulation.gates {
            if let Some((dragging_id, drag_pos_offset)) = dragging {
                if dragging_id == id {
                    let pos: Vec2 = mouse_position().into();
                    *gate_pos = pos - drag_pos_offset;
                }
            }

            let (inputs, outputs) = simulation.sim.get_gate_state(id);
            let name = simulation.sim.get_gate_name(id);
            if let Some(mouse_hover) = draw_gate(name, gate_pos.x, gate_pos.y, inputs, outputs) {
                match mouse_hover {
                    GateMouseHover::Input(input_id, input_pos) => {
                        if is_mouse_button_pressed(MouseButton::Left) {
                            selected_input = Some((id, input_id, input_pos - *gate_pos));
                        }
                    }
                    GateMouseHover::Output(output_id, output_pos) => {
                        if is_mouse_button_pressed(MouseButton::Left) {
                            selected_output = Some((id, output_id, output_pos - *gate_pos));
                        }
                    }
                    GateMouseHover::Gate(drag_pos) => {
                        if dragging.is_none() {
                            if is_mouse_button_pressed(MouseButton::Left) {
                                let offset = drag_pos - *gate_pos;
                                dragging = Some((id, offset));
                            }
                        }

                        if is_mouse_button_pressed(MouseButton::Right) {
                            to_remove = Some(id);
                        }
                    }
                }
            }
        }

        for (output, input) in &simulation.connections {
            let (output_gate_id, output_id, output_pos_gate_offset) = output;
            let (input_gate_id, _, input_pos_gate_offset) = input;

            let (_, outputs) = simulation.sim.get_gate_state(*output_gate_id);
            let output_active = outputs[*output_id];

            let output_gate_pos = simulation.gates[output_gate_id];
            let input_gate_pos = simulation.gates[input_gate_id];

            let output_pos = output_gate_pos + *output_pos_gate_offset;
            let input_pos = input_gate_pos + *input_pos_gate_offset;

            let opos = Vec2::new(output_pos.x, output_pos.y);
            let ipos = Vec2::new(input_pos.x, input_pos.y);
            let mpos: Vec2 = mouse_position().into();

            let dm = mpos - opos;
            let d1 = ipos - opos;
            let cross = dm.perp_dot(d1);

            let is_between = if d1.x.abs() > d1.y.abs() {
                if d1.x > 0.0 {
                    opos.x <= mpos.x && mpos.x <= ipos.x
                } else {
                    ipos.x <= mpos.x && mpos.x <= opos.x
                }
            } else {
                if d1.y > 0.0 {
                    opos.y <= mpos.y && mpos.y <= ipos.y
                } else {
                    ipos.y <= mpos.y && mpos.y <= opos.y
                }
            };

            let mouse_over_line = is_between && cross.abs() < 1000.;

            if mouse_over_line {
                if is_mouse_button_pressed(MouseButton::Right) {
                    connection_to_remove = Some((*input, *output));
                }
            }

            draw_line(
                output_pos.x,
                output_pos.y,
                input_pos.x,
                input_pos.y,
                if mouse_over_line { 4. } else { 2. },
                if output_active { RED } else { WHITE },
            );
        }

        if let Some(gate_id) = to_remove.take() {
            simulation.remove_gate(gate_id);
        }

        if let Some((input, output)) = connection_to_remove.take() {
            simulation.remove_connection(input, output);
        }

        match (selected_input, selected_output) {
            (Some((gate_id, _, offset)), None) => {
                let (mouse_x, mouse_y) = mouse_position();
                let gate_pos = simulation.gates[&gate_id];
                let pos = gate_pos + offset;
                draw_line(pos.x, pos.y, mouse_x, mouse_y, 2., WHITE);
            }
            (None, Some((gate_id, _, offset))) => {
                let (mouse_x, mouse_y) = mouse_position();
                let gate_pos = simulation.gates[&gate_id];
                let pos = gate_pos + offset;
                draw_line(pos.x, pos.y, mouse_x, mouse_y, 2., WHITE);
            }
            _ => {}
        }

        root_ui().window(hash!(), vec2(0.0, 0.0), vec2(200.0, 400.0), |ui| {
            ui.slider(hash!(), "Frequency Hz", 1f32..100f32, &mut frequency);
            ui.label(None, "Add Gate:");
            if ui.button(None, "AND") {
                simulation.add_gate(And, Vec2::new(screen_width() / 2., screen_height() / 2.));
            }

            if ui.button(None, "OR") {
                simulation.add_gate(Or, Vec2::new(screen_width() / 2., screen_height() / 2.));
            }

            if ui.button(None, "NOT") {
                simulation.add_gate(Not, Vec2::new(screen_width() / 2., screen_height() / 2.));
            }

            if ui.button(None, "YES") {
                simulation.add_gate(Yes, Vec2::new(screen_width() / 2., screen_height() / 2.));
            }

            if ui.button(None, "XOR") {
                simulation.add_gate(Xor, Vec2::new(screen_width() / 2., screen_height() / 2.));
            }

            if ui.button(None, "AND3") {
                simulation.add_gate(And3, Vec2::new(screen_width() / 2., screen_height() / 2.));
            }
        });

        next_frame().await
    }
}
