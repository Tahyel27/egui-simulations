use std::{ops::Deref, sync::mpsc::{self, Receiver}, thread};
use crate::mpscsingle;
use tokio::sync::watch;

#[derive(Default)]
pub struct SimulationContext {
    step: usize
}

impl SimulationContext {
    fn increment_step(&mut self) { self.step = self.step + 1; }

    pub fn get_step(&self) -> usize {self.step}
}

pub trait SimulationData: Send {
    type SimRes: Send + Sync + Clone;

    fn update(&mut self, ctx: &SimulationContext) -> ();

    fn send_result(&self, ctx: &SimulationContext) -> Self::SimRes;
}

#[derive(Default)]
pub struct SimulationHandler<T: SimulationData + Clone + 'static> {
    data: Option<T>,
    //rx: Option<Receiver<T::SimRes>>,
    rx: Option<mpscsingle::Receiver<T::SimRes>>,
    rx_tk: Option<watch::Receiver<Option<T::SimRes>>>,
    send_freq: usize   
}

impl<SimData: SimulationData + Clone + 'static> SimulationHandler<SimData> {
    pub fn new(initial_data: SimData) -> Self {
        Self { data: Some(initial_data), rx: None, rx_tk: None, send_freq: 1 }
    }

    pub fn send_frequency(mut self, frequency: usize) -> Self {
        self.send_freq = frequency;
        self
    }

    pub fn run(&mut self) {
        if let Some(data) = &mut self.data {
            
            let mut data_clone = data.clone();

            let send_freq = self.send_freq;

            //let (tx, rx) = mpsc::channel();
            //let (tx, rx) = watch::channel(None);
            let (tx, rx) = mpscsingle::channel();

            self.rx = Some(rx);

            thread::spawn(move || {
                let mut ctx = SimulationContext{ step: 0};
                
                loop {
                    data_clone.update(&ctx);

                    let res = data_clone.send_result(&ctx);

                    if ctx.step % send_freq == 0 {
                        if tx.send(res).is_err() {
                            break ;
                        }
                    }

                    ctx.increment_step();
                }

            });
        }
    }

    pub fn try_receive(&self) -> Option<SimData::SimRes> {
        match &self.rx {
            Some(rx) => rx.try_recv(),
            None => None
        }

        /*match &self.rx_tk {
            Some(rx) => rx.borrow().deref().clone() ,
            None => None
        }*/

    }
}