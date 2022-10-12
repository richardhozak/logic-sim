use std::collections::HashMap;

use crate::gates::Gate;

type UpdateFn = Box<dyn Fn(&[bool], &mut [bool])>;

struct GateState {
    inputs: Box<[bool]>,
    outputs: Box<[bool]>,
    update_fn: UpdateFn,
    name: &'static str,
}

impl GateState {
    fn update(&mut self) {
        (self.update_fn)(&self.inputs, &mut self.outputs);
    }
}

pub struct LogicSimulation {
    counter: usize,
    gates: HashMap<usize, GateState>,
    connections: Vec<(usize, usize, usize, usize)>,
}

impl LogicSimulation {
    pub fn new() -> LogicSimulation {
        LogicSimulation {
            counter: 0,
            gates: HashMap::new(),
            connections: Vec::new(),
        }
    }

    pub fn add_gate<const INPUTS: usize, const OUTPUTS: usize>(
        &mut self,
        gate: impl Gate<INPUTS, OUTPUTS> + 'static,
    ) {
        let inputs = Box::new([false; INPUTS]);
        let outputs = Box::new([false; OUTPUTS]);
        let id = self.counter;
        let name = gate.name();

        let update_fn: UpdateFn = Box::new(move |inputs, outputs| {
            gate.update(inputs.try_into().unwrap(), outputs.try_into().unwrap())
        });

        self.gates.insert(
            id,
            GateState {
                inputs,
                outputs,
                update_fn,
                name,
            },
        );
        self.counter += 1;
    }

    pub fn add_connection(&mut self, from: usize, output: usize, to: usize, input: usize) {
        self.connections.push((from, output, to, input));
    }

    pub fn get_gate_state(&self, id: usize) -> (&[bool], &[bool]) {
        let gate = self.gates.get(&id).unwrap();
        (&gate.inputs, &gate.outputs)
    }

    pub fn get_gate_name(&self, id: usize) -> &'static str {
        self.gates.get(&id).unwrap().name
    }

    pub fn simulate(&mut self) {
        for (from, output, to, input) in &self.connections {
            let output_state = self.gates.get(from).unwrap().outputs[*output];
            self.gates.get_mut(to).unwrap().inputs[*input] = output_state;
        }

        for (_, state) in &mut self.gates {
            state.update();
        }
    }
}
