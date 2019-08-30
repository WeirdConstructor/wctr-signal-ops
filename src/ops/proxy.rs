use crate::signals::{OpIn, Op, OpPort, OpIOSpec};

pub struct OutProxy {
    pub values:   std::rc::Rc<std::cell::RefCell<Vec<f32>>>,
    out_regs: Vec<usize>,
}

impl OutProxy {
    pub fn new(num_outputs: usize) -> Self {
        OutProxy {
            values: std::rc::Rc::new(std::cell::RefCell::new(vec![0.0; num_outputs])),
            out_regs: vec![0; num_outputs],
        }
    }
}

impl Op for OutProxy {
    fn io_spec(&self, index: usize) -> OpIOSpec {
        OpIOSpec {
            inputs:         vec![],
            input_values:   vec![],
            input_defaults: vec![],
            outputs:        self.values.borrow().iter().enumerate()
                                       .map(|(i, _v)|
                                            OpPort::new(&format!("out{}", i), -9999.0, 9999.0))
                                       .collect(),
            output_regs:    self.out_regs.clone(),
            audio_out_groups: vec![],
            index,
        }
    }

    fn init_regs(&mut self, start_reg: usize, regs: &mut [f32]) {
        for (i, o) in self.out_regs.iter_mut().enumerate() {
            *o = i + start_reg;
            regs[*o] = 0.0;
        }
    }

    fn get_output_reg(&mut self, name: &str) -> Option<usize> {
        for i in 0..self.out_regs.len() {
            if name == format!("out{}", i) {
                return Some(self.out_regs[i])
            }
        }

        None
    }

    fn set_input(&mut self, _name: &str, _to: OpIn, _as_default: bool) -> bool {
        false
    }

    fn exec(&mut self, _t: f32, regs: &mut [f32]) {
        let v = self.values.borrow();
        for (i, or) in self.out_regs.iter().enumerate() {
            regs[*or] = v[i];
        }
    }
}

