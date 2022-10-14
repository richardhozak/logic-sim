pub trait Gate<const INPUTS: usize, const OUTPUTS: usize> {
    const NAME: &'static str;

    fn update(&self, inputs: &[bool; INPUTS], outputs: &mut [bool; OUTPUTS]);

    fn name(&self) -> &'static str {
        Self::NAME
    }
}

pub struct And;

impl Gate<2, 1> for And {
    const NAME: &'static str = "AND";

    fn update(&self, inputs: &[bool; 2], outputs: &mut [bool; 1]) {
        outputs[0] = inputs[0] && inputs[1];
    }
}

pub struct Nand;

impl Gate<2, 1> for Nand {
    const NAME: &'static str = "NAND";

    fn update(&self, inputs: &[bool; 2], outputs: &mut [bool; 1]) {
        outputs[0] = !(inputs[0] && inputs[1]);
    }
}

pub struct Or;

impl Gate<2, 1> for Or {
    const NAME: &'static str = "OR";

    fn update(&self, inputs: &[bool; 2], outputs: &mut [bool; 1]) {
        outputs[0] = inputs[0] || inputs[1];
    }
}

pub struct Nor;

impl Gate<2, 1> for Nor {
    const NAME: &'static str = "NOR";

    fn update(&self, inputs: &[bool; 2], outputs: &mut [bool; 1]) {
        outputs[0] = !(inputs[0] || inputs[1]);
    }
}

pub struct Xor;

impl Gate<2, 1> for Xor {
    const NAME: &'static str = "XOR";

    fn update(&self, inputs: &[bool; 2], outputs: &mut [bool; 1]) {
        outputs[0] = inputs[0] != inputs[1];
    }
}

pub struct Xnor;

impl Gate<2, 1> for Xnor {
    const NAME: &'static str = "XNOR";

    fn update(&self, inputs: &[bool; 2], outputs: &mut [bool; 1]) {
        outputs[0] = !(inputs[0] != inputs[1]);
    }
}

pub struct Not;

impl Gate<1, 1> for Not {
    const NAME: &'static str = "NOT";

    fn update(&self, inputs: &[bool; 1], outputs: &mut [bool; 1]) {
        outputs[0] = !inputs[0];
    }
}

pub struct Yes;

impl Gate<1, 1> for Yes {
    const NAME: &'static str = "YES";

    fn update(&self, inputs: &[bool; 1], outputs: &mut [bool; 1]) {
        outputs[0] = inputs[0];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // test are just checks against truth tables

    const Y: bool = true;
    const N: bool = false;

    struct TruthTable<const INPUTS: usize, const OUTPUTS: usize, const ROWS: usize>(
        [([bool; INPUTS], [bool; OUTPUTS]); ROWS],
    );

    fn test_gate<const INPUTS: usize, const OUTPUTS: usize>(
        gate: impl Gate<INPUTS, OUTPUTS>,
        io: ([bool; INPUTS], [bool; OUTPUTS]),
    ) {
        let (inputs, expected_outputs) = io;

        // invert the expected outputs, so we always check changed values, some
        // test may pass with default 'false' values
        let mut outputs = expected_outputs.map(|output| !output);

        gate.update(&inputs, &mut outputs);

        assert_eq!(outputs, expected_outputs);
    }

    #[test]
    fn and() {
        #[rustfmt::skip]
        let table = TruthTable([
            ([N, N], [N]),
            ([N, Y], [N]),
            ([Y, N], [N]),
            ([Y, Y], [Y]),
        ]);

        for row in table.0 {
            test_gate(And, row);
        }
    }

    #[test]
    fn nand() {
        #[rustfmt::skip]
        let table = TruthTable([
            ([N, N], [Y]),
            ([N, Y], [Y]),
            ([Y, N], [Y]),
            ([Y, Y], [N]),
        ]);

        for row in table.0 {
            test_gate(Nand, row);
        }
    }

    #[test]
    fn or() {
        #[rustfmt::skip]
        let table = TruthTable([
            ([N, N], [N]),
            ([N, Y], [Y]),
            ([Y, N], [Y]),
            ([Y, Y], [Y]),
        ]);

        for row in table.0 {
            test_gate(Or, row);
        }
    }

    #[test]
    fn nor() {
        #[rustfmt::skip]
        let table = TruthTable([
            ([N, N], [Y]),
            ([N, Y], [N]),
            ([Y, N], [N]),
            ([Y, Y], [N]),
        ]);

        for row in table.0 {
            test_gate(Nor, row);
        }
    }

    #[test]
    fn xor() {
        #[rustfmt::skip]
        let table = TruthTable([
            ([N, N], [N]),
            ([N, Y], [Y]),
            ([Y, N], [Y]),
            ([Y, Y], [N]),
        ]);

        for row in table.0 {
            test_gate(Xor, row);
        }
    }

    #[test]
    fn xnor() {
        #[rustfmt::skip]
        let table = TruthTable([
            ([N, N], [Y]),
            ([N, Y], [N]),
            ([Y, N], [N]),
            ([Y, Y], [Y]),
        ]);

        for row in table.0 {
            test_gate(Xnor, row);
        }
    }

    #[test]
    fn not() {
        #[rustfmt::skip]
        let table = TruthTable([
            ([N], [Y]),
            ([Y], [N]),
        ]);

        for row in table.0 {
            test_gate(Not, row);
        }
    }

    #[test]
    fn yes() {
        #[rustfmt::skip]
        let table = TruthTable([
            ([N], [N]),
            ([Y], [Y]),
        ]);

        for row in table.0 {
            test_gate(Yes, row);
        }
    }
}
