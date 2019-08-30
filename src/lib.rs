pub mod sample_row;
pub mod signals;
pub mod ops;

pub use signals::{
    OpIn,
    Op,
    OpPort,
    OpIOSpec,
    OpInfo,
    Simulator,
    SimulatorUIEvent,
    SimulatorUIInput,
    SimulatorCommunicator,
    SimulatorCommunicatorEndpoint};

//#[cfg(test)]
//mod tests {
//    #[test]
//    fn it_works() {
//        assert_eq!(2 + 2, 4);
//    }
//}
