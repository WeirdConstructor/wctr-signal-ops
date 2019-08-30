use crate::signals::{OpIn, Op, OpPort, OpIOSpec};

pub struct AudioSend {
        volume_l: OpIn,
        volume_r: OpIn,
        cur_vol_l: f32,
        cur_vol_r: f32,
    pub out:    usize,
}

impl AudioSend {
    pub fn new() -> Self {
        AudioSend {
            volume_l:  OpIn::Constant(1.0),
            volume_r:  OpIn::Constant(1.0),
            cur_vol_l: 1.0,
            cur_vol_r: 1.0,
            out:       0,
        }
    }
}

impl Op for AudioSend {
    fn io_spec(&self, index: usize) -> OpIOSpec {
        OpIOSpec {
            inputs: vec![
                OpPort::new("vol_l", 0.0, 1.0),
                OpPort::new("vol_r", 0.0, 1.0),
            ],
            input_values:     vec![self.volume_l, self.volume_r],
            input_defaults:   vec![OpIn::Constant(1.0), OpIn::Constant(1.0)],
            outputs:          vec![],
            output_regs:      vec![],
            audio_out_groups: vec![self.out],
            index,
        }
    }

    fn init_regs(&mut self, _start_reg: usize, _regs: &mut [f32]) { }
    fn get_output_reg(&mut self, _name: &str) -> Option<usize> { None }

    fn set_input(&mut self, name: &str, to: OpIn, _as_default: bool) -> bool {
        match name {
            "vol_l" => { self.volume_l = to; true },
            "vol_r" => { self.volume_r = to; true },
            _       => false,
        }
    }

    fn exec(&mut self, _t: f32, regs: &mut [f32]) {
        self.cur_vol_l = self.volume_l.calc(regs);
        self.cur_vol_r = self.volume_r.calc(regs);
    }

    fn render(&mut self, num_samples: usize, offs: usize, input_idx: usize, bufs: &mut Vec<[Vec<f32>; 2]>)
    {
        for i in offs..(offs + num_samples) {
            bufs[self.out][0][i] += self.cur_vol_l * bufs[input_idx][0][i];
            bufs[self.out][1][i] += self.cur_vol_r * bufs[input_idx][1][i];
        }
    }
}


