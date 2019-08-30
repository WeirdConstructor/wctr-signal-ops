use crate::sample_row::SampleRow;
use serde::Serialize;
use serde::Deserialize;

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum OpIn {
    Constant(f32),
    Reg(usize),
    RegMix2(usize, usize, f32),
    RegAdd(usize,f32),
    RegMul(usize,f32),
    RegAddMul(usize,f32,f32),
    RegMulAdd(usize,f32,f32),
    RegLerp(usize,f32,f32),
    RegSStep(usize,f32,f32),
    RegMap(usize,f32,f32,f32,f32),
}

impl OpIn {
    pub fn calc(&self, regs: &[f32]) -> f32 {
        match self {
            OpIn::Constant(v)            => *v,
            OpIn::Reg(i)                 => regs[*i],
            OpIn::RegMix2(ia, ib, am)    => regs[*ia] * am + regs[*ib] * (1.0 - am),
            OpIn::RegAdd(i, v)           => v + regs[*i],
            OpIn::RegMul(i, v)           => v * regs[*i],
            OpIn::RegAddMul(i, a, v)     => v * (regs[*i] + a),
            OpIn::RegMulAdd(i, v, a)     => (v * regs[*i]) + a,
            OpIn::RegLerp(i, a, b)       => (a * regs[*i]) + (b * (1.0 - regs[*i])),
            OpIn::RegSStep(i, a, b)      => {
                let x = (regs[*i] - a) / (b - a);
                let x = if x < 0.0 { 0.0 } else { x };
                let x = if x > 1.0 { 1.0 } else { x };
                x * x * (3.0 - 2.0 * x)
            },
            OpIn::RegMap(i, a_frm, b_frm, a_to, b_to) => {
                let x = (regs[*i] - a_frm) / (b_frm - a_frm);
                (a_to * x) + (b_to * (1.0 - x))
            },
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct DemOpPort {
    pub min: f32,
    pub max: f32,
    pub name: String,
}

impl DemOpPort {
    pub fn new(name: &str, min: f32, max: f32) -> Self {
        DemOpPort { name: name.to_string(), min, max }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct DemOpIOSpec {
    pub index:              usize,
    pub inputs:             Vec<DemOpPort>,
    pub input_values:       Vec<OpIn>,
    pub input_defaults:     Vec<OpIn>,
    pub audio_out_groups:   Vec<usize>,
    pub outputs:            Vec<DemOpPort>,
    pub output_regs:        Vec<usize>,
}

pub trait DemOp {
    fn io_spec(&self, index: usize) -> DemOpIOSpec;

    fn init_regs(&mut self, start_reg: usize, regs: &mut [f32]);

    fn get_output_reg(&mut self, name: &str) -> Option<usize>;
    fn set_input(&mut self, name: &str, to: OpIn, as_default: bool) -> bool;
    fn exec(&mut self, t: f32, regs: &mut [f32]);

    fn does_render(&self) -> bool { false }
    fn render(&mut self, _num_samples: usize, _offs: usize, _input_idx: usize, _bufs: &mut Vec<[Vec<f32>; 2]>) { }

    fn input_count(&self) -> usize { self.io_spec(0).inputs.len() }
    fn output_count(&self) -> usize { self.io_spec(0).outputs.len() }

    fn deserialize_inputs(&mut self, inputs: &Vec<(String, OpIn)>) {
        for (p, v) in inputs.iter() { self.set_input(p, *v, false); }
    }

    fn serialize_inputs(&self) -> Vec<(String, OpIn)> {
        let spec = self.io_spec(0);
        let vals : Vec<(String, OpIn)> =
            spec.inputs.iter()
                .zip(spec.input_values.iter())
                .map(|(p, v)| (p.name.clone(), *v))
                .collect();
        vals
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct OpGroup {
    pub name: String,
    pub index: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct OpInfo {
    pub name:  String,
    pub does_render: bool,
    pub group: OpGroup,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SimulatorUIInput {
    Refresh,
    SetOpInput(usize, String, OpIn, bool),
    SaveInputs,
    LoadInputs(Vec<(String, Vec<(String, OpIn)>)>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum SimulatorUIEvent {
    OpSpecUpdate(Vec<(DemOpIOSpec, OpInfo)>),
    SerializedInputValues(Vec<(String, Vec<(String, OpIn)>)>),
}

#[derive(Debug)]
pub struct SimulatorCommunicatorEndpoint {
    tx: std::sync::mpsc::Sender<SimulatorUIEvent>,
    rx: std::sync::mpsc::Receiver<SimulatorUIInput>,
}

impl SimulatorCommunicatorEndpoint {
    pub fn handle_ui_messages(&mut self, sim: &mut Simulator)
    {
        let r = self.rx.try_recv();
        match r {
            Ok(SimulatorUIInput::SetOpInput(idx, in_name, op_in, def)) => {
                println!("SETINPUT: {}", in_name);
                if !sim.set_op_input(idx, &in_name, op_in, def) {
                    panic!(format!("Expected op input name {}/{}/{:?}", idx, in_name, op_in));
                }
            },
            Ok(SimulatorUIInput::Refresh) => {
                self.tx.send(SimulatorUIEvent::OpSpecUpdate(sim.get_specs()))
                    .expect("communication with ui thread");
            },
            Ok(SimulatorUIInput::LoadInputs(inputs)) => {
                sim.deserialize_inputs(inputs);
            },
            Ok(SimulatorUIInput::SaveInputs) => {
                self.tx.send(SimulatorUIEvent::SerializedInputValues(
                                sim.serialize_inputs()))
                    .expect("communication with ui thread");
            },
            Err(std::sync::mpsc::TryRecvError::Empty) => (),
            Err(std::sync::mpsc::TryRecvError::Disconnected) => (),
        }
    }

}

#[derive(Debug)]
pub struct SimulatorCommunicator {
    tx: std::sync::mpsc::Sender<SimulatorUIInput>,
    rx: std::sync::mpsc::Receiver<SimulatorUIEvent>,
    ep: Option<SimulatorCommunicatorEndpoint>,
}

impl SimulatorCommunicator {
    pub fn new() -> Self {
        let (simuiin_tx, simuiin_rx) = std::sync::mpsc::channel::<SimulatorUIInput>();
        let (simuiev_tx, simuiev_rx) = std::sync::mpsc::channel::<SimulatorUIEvent>();

        SimulatorCommunicator {
            tx: simuiin_tx,
            rx: simuiev_rx,
            ep: Some(SimulatorCommunicatorEndpoint {
                tx: simuiev_tx,
                rx: simuiin_rx,
            }),
        }
    }

    pub fn get_endpoint(&mut self) -> SimulatorCommunicatorEndpoint {
        std::mem::replace(&mut self.ep, None)
        .expect("SimulatorCommunicatorEndpoint can only be retrieved once")
    }

    pub fn set_op_input(&mut self, op_index: usize, input_name: &str, op_in: OpIn, as_default: bool) {
        self.tx.send(SimulatorUIInput::SetOpInput(
                        op_index, input_name.to_string(), op_in, as_default))
            .expect("communication with backend thread");
    }

    pub fn save_input_values(&mut self) -> Vec<(String, Vec<(String, OpIn)>)> {
        self.tx.send(SimulatorUIInput::SaveInputs)
            .expect("communication with backend thread");
        let r = self.rx.recv();
        if let Ok(SimulatorUIEvent::SerializedInputValues(v)) = r {
            v
        } else {
            vec![]
        }
    }

    pub fn load_input_values(&mut self, inputs: &Vec<(String, Vec<(String, OpIn)>)>) {
        self.tx.send(SimulatorUIInput::LoadInputs(inputs.clone()))
            .expect("communication with backend thread");
    }

    pub fn update<F, T>(&mut self, mut cb: F) -> Option<T>
        where F: FnMut(SimulatorUIEvent) -> T {

        self.tx.send(SimulatorUIInput::Refresh)
            .expect("communication with backend thread");
        let r = self.rx.recv();
        if let Ok(ev) = r {
            Some(cb(ev))
        } else {
            None
        }
    }
}

pub struct Simulator {
    pub regs:               Vec<f32>,
    pub ops:                Vec<Box<dyn DemOp>>,
    pub op_infos:           Vec<OpInfo>,
    pub op_groups:          Vec<OpGroup>,
    pub render_groups:      Vec<Vec<usize>>,
    pub sample_row:         SampleRow,
    pub scope_sample_len:   usize,
    pub scope_sample_pos:   usize,
}

impl Simulator {
    pub fn new() -> Self {
        let sim = Simulator {
            regs:               Vec::new(),
            ops:                Vec::new(),
            op_groups:          Vec::new(),
            op_infos:           Vec::new(),
            render_groups:      Vec::new(),
            sample_row:         SampleRow::new(),
            scope_sample_len:   128, // SCOPE_SAMPLES
            scope_sample_pos:   0,
        };
        sim
    }

    pub fn deserialize_inputs(&mut self, op_inputs: Vec<(String, Vec<(String, OpIn)>)>) {
        for (k, v) in op_inputs.iter() {
            if let Some((idx, _)) =
                    self.op_infos.iter().enumerate()
                                 .find(|(_, i)| i.name == *k) {

                self.ops[idx].deserialize_inputs(v);
            }
        }
    }

    pub fn serialize_inputs(&self) -> Vec<(String, Vec<(String, OpIn)>)> {
        let mut valmap : Vec<(String, Vec<(String, OpIn)>)> = Vec::new();
        for (o, info) in self.ops.iter().zip(self.op_infos.iter()) {
            valmap.push((info.name.clone(), o.serialize_inputs()));
        }
        valmap
    }

    pub fn add_group(&mut self, name: &str) -> usize {
        self.op_groups.push(OpGroup { name: name.to_string(), index: self.op_groups.len() });
        self.render_groups.push(Vec::new());
        self.op_groups.len() - 1
    }

    pub fn get_specs(&self) -> Vec<(DemOpIOSpec, OpInfo)> {
        self.ops
            .iter()
            .enumerate()
            .map(|(i, o)|
                (o.io_spec(i), self.op_infos[i].clone()))
            .collect()
    }

    pub fn get_op_index(&self, op_name: &str) -> Option<usize> {
        let on = op_name.to_string();
        if let Some((i, _)) =
            self.op_infos.iter().enumerate().find(|(_i, o)| o.name == on) {

            Some(i)
        } else {
            None
        }
    }

    pub fn add_op(&mut self, mut op: Box<dyn DemOp>, op_name: String, group_index: usize) -> Option<usize> {
        let new_start_reg = self.regs.len();
        let new_reg_count = self.regs.len() + op.output_count();
        self.regs.resize(new_reg_count, 0.0);
        op.init_regs(new_start_reg, &mut self.regs[..]);
        let out_reg = op.get_output_reg("out");

        self.op_infos.push(OpInfo {
            name: op_name,
            does_render: op.does_render(),
            group: self.op_groups[group_index].clone()
        });
        self.ops.push(op);
        self.render_groups[group_index].push(self.ops.len() - 1);

        out_reg
    }

    pub fn set_reg(&mut self, idx: usize, v: f32) -> bool {
        if self.regs.len() > idx {
            self.regs[idx] = v;
            true
        } else {
            false
        }
    }

    pub fn get_reg(&self, idx: usize) -> f32 {
        if self.regs.len() > idx {
            self.regs[idx]
        } else {
            0.0
        }
    }

    pub fn set_op_input(&mut self, idx: usize, input_name: &str, to: OpIn, as_default: bool) -> bool {
        println!("SETSET {} {} {:?}", idx, input_name, to);
        if idx >= self.ops.len() {
            return false;
        }
        self.ops[idx].set_input(input_name, to, as_default)
    }

    pub fn exec(&mut self, t: f32, ext_scopes: std::sync::Arc<std::sync::Mutex<SampleRow>>) {
        for r in self.ops.iter_mut() {
            r.as_mut().exec(t, &mut self.regs[..]);
        }

        self.sample_row.read_from_regs(&self.regs[..], self.scope_sample_pos);
        self.scope_sample_pos =
            (self.scope_sample_pos + 1) % self.scope_sample_len;

        if let Ok(ref mut m) = ext_scopes.try_lock() {
//            use std::ops::DerefMut;
            std::mem::swap(&mut self.sample_row, &mut *m);
        }
    }

    pub fn get_group_sample_buffers(&self, size: usize) -> Vec<[Vec<f32>; 2]> {
        let mut v : Vec<[Vec<f32>; 2]> = Vec::with_capacity(self.op_groups.len());
        for _ in self.op_groups.iter() {
            let mut n : Vec<f32> = Vec::new();
            n.resize(size, 0.0);
            let mut n2 : Vec<f32> = Vec::new();
            n2.resize(size, 0.0);
            v.push([n, n2]);
        }
        v
    }

    pub fn render(&mut self, num_samples: usize, sample_offs: usize,
                  grp_bufs: &mut Vec<[Vec<f32>; 2]>) {

        for gb in grp_bufs.iter_mut() {
            for i in sample_offs..(sample_offs + num_samples) {
                gb[0][i] = 0.0;
                gb[1][i] = 0.0;
            }
        }

        for (ig, grp) in self.render_groups.iter().enumerate() {
            for i in grp.iter() {
                self.ops[*i].render(num_samples, sample_offs, ig, grp_bufs);
            }
        }
    }
}

pub struct DebugRegisters {
    pub debug_regs: Vec<(String, OpIn)>,
}

impl DebugRegisters {
    pub fn new() -> Self {
        DebugRegisters { debug_regs: Vec::new() }
    }

    pub fn add(&mut self, name: String, op_in: OpIn) {
        self.debug_regs.push((name, op_in));
    }

    pub fn show<T>(&self, regs: &[f32], view: &mut T) where T: RegisterView {
        view.start_print_registers();
        for r in self.debug_regs.iter() {
            view.print_register(&r.0, r.1.calc(regs));
        }
        view.end_print_registers();
    }
}

pub trait RegisterView {
    fn start_print_registers(&mut self);
    fn print_register(&mut self, name: &str, value: f32);
    fn end_print_registers(&mut self);
}

