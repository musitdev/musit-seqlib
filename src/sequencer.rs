use std::sync::mpsc::{Receiver, TryRecvError};
use time::precise_time_ns;
use std::thread::sleep_ms;

pub enum SequencerCommand {
    SeqComm(String),
    PatComm(String),
}

pub fn sequencer_process(receiver: Receiver<SequencerCommand>) {
	let wait_time_ms: u64 = 1000;
	let mut time_ns: u64 = precise_time_ns();
	loop {
	    //manage extern command
	    let comm_rx_value = receiver.try_recv();
	    match comm_rx_value {
	    	Ok(command) => {
	        	match command {
			        SequencerCommand::SeqComm(content) =>  (),
			        SequencerCommand::PatComm(content) =>  (),
	        	}
			},	
	        Err(err) => {
		        match err   {
				    TryRecvError::Empty => (),
				    TryRecvError::Disconnected => panic!("Sequencer comm channel disconnect error."),
				}
			}
	    }


	    //manag wait time
	    let new_time_ns = precise_time_ns();
	    let diff = new_time_ns - time_ns;
	    if (diff) < wait_time_ms {
	    	sleep_ms((wait_time_ms - diff) as u32);
	    }
	    time_ns = precise_time_ns();
	}

}