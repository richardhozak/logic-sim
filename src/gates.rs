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

pub struct And3;

impl Gate<3, 1> for And3 {
    const NAME: &'static str = "AND3";

    fn update(&self, inputs: &[bool; 3], outputs: &mut [bool; 1]) {
        outputs[0] = inputs[0] && inputs[1] && inputs[2];
    }
}

pub struct Or;

impl Gate<2, 1> for Or {
    const NAME: &'static str = "OR";

    fn update(&self, inputs: &[bool; 2], outputs: &mut [bool; 1]) {
        outputs[0] = inputs[0] || inputs[1];
    }
}

pub struct Xor;

impl Gate<2, 1> for Xor {
    const NAME: &'static str = "XOR";

    fn update(&self, inputs: &[bool; 2], outputs: &mut [bool; 1]) {
        outputs[0] = inputs[0] != inputs[1];
    }
}

pub struct Not;

impl Gate<1, 1> for Not {
    const NAME: &'static str = "NOT";

    fn update(&self, inputs: &[bool; 1], outputs: &mut [bool; 1]) {
        outputs[0] = !inputs[0];
    }
}
