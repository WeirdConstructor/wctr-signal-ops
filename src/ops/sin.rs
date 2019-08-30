use crate::signals::{OpIn, DemOp, DemOpPort, DemOpIOSpec};

pub struct Sin {
    values:   [OpIn; 4],
    defaults: [OpIn; 4],
    out:      usize,
}

impl Sin {
    pub fn new() -> Self {
        let defs = [
            OpIn::Constant(1.0),
            OpIn::Constant(0.0),
            OpIn::Constant(0.0),
            OpIn::Constant(9.1),
        ];
        Sin {
            values:   defs,
            out:      0,
            defaults: [
                OpIn::Constant(1.0),
                OpIn::Constant(0.0),
                OpIn::Constant(0.0),
                OpIn::Constant(1.0)
            ],
        }
    }
}

impl DemOp for Sin {
    fn io_spec(&self, index: usize) -> DemOpIOSpec {
        DemOpIOSpec {
            inputs: vec![
                DemOpPort::new("amp",    0.0, 9999.0),
                DemOpPort::new("phase", -2.0 * std::f32::consts::PI,
                                         2.0 * std::f32::consts::PI),
                DemOpPort::new("vert",  -9999.0,  9999.0),
                DemOpPort::new("freq",      0.0, 11025.0),
            ],
            input_values: self.values.to_vec(),
            input_defaults: self.defaults.to_vec(),
            outputs: vec![
                DemOpPort::new("out", -9999.0, 9999.0),
            ],
            output_regs: vec![self.out],
            audio_out_groups: vec![],
            index,
        }
    }

    fn init_regs(&mut self, start_reg: usize, regs: &mut [f32]) {
        regs[start_reg] = 0.0;
        self.out = start_reg;
    }

    fn get_output_reg(&mut self, name: &str) -> Option<usize> {
        match name {
            "out"   => Some(self.out),
            _       => None,
        }
    }

    fn set_input(&mut self, name: &str, to: OpIn, as_default: bool) -> bool {
        let s = if as_default { &mut self.defaults } else { &mut self.values };
        match name {
            "amp"   => { s[0] = to; true },
            "phase" => { s[1] = to; true },
            "vert"  => { s[2] = to; true },
            "freq"  => { s[3] = to; true },
            _       => false,
        }
    }

    fn exec(&mut self, t: f32, regs: &mut [f32]) {
        let a = self.values[0].calc(regs);
        let p = self.values[1].calc(regs);
        let v = self.values[2].calc(regs);
        let f = self.values[3].calc(regs);
        regs[self.out] = a * (((f * t) + p).sin() + v);
        //d// println!("OUT: {}, {}", regs[self.out], self.out);
    }
}


