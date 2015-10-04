
use std::sync::mpsc::{Receiver, TryRecvError};
use portmidi::PortMidiDeviceId;
use time::precise_time_ns;
use std::thread::sleep_ms;
use std::collections::HashMap;

use pattern::{Pattern, PatternCommand};
use device::InputDevice;


pub enum SeqCommand {
    ConnectInput(PortMidiDeviceId, String), //connect specified midi input to the specified Pattern id.
}

pub enum MusitCommand {
    SeqComm(SeqCommand),
    PatComm(PatternCommand),
}

pub fn sequencer_process(receiver: Receiver<MusitCommand>) {
	let wait_time_ns: u64 = 1000000;  //in nanosecond
	let mut time_ns: u64 = precise_time_ns();
	let mut pattern_list: HashMap<String, Pattern> = HashMap::new();
	let mut input_device_list: Vec<InputDevice> = vec!();
	let mut tick = 0;
	loop {
	    //manage extern command
	    let comm_rx_value = receiver.try_recv();
	    match comm_rx_value {
	    	Ok(command) => {
	        	match command {
			        MusitCommand::SeqComm(comm) =>  {
			        	match comm {
			        		SeqCommand::ConnectInput(device_id, id) => {
			        			//create the device if not created.
			        			if let None = input_device_list.iter().find(|&d| d.id == device_id)	{
			        				match InputDevice::new(device_id) {
			        					Ok(new_dev) => {
			        						input_device_list.push(new_dev);
			        					},
			        					Err(err) => panic!("can\'t allocate new midi device {:?} because {:?}",id, err),
			        				}			        							        					
			        			}

			        			//connect pattern to device.
        						if let Some(patt) = pattern_list.get_mut(&id)	{
        							if let Some(device) = input_device_list.iter().find(|&d| d.id == device_id)	{
        								patt.receiver = Some(device.receiver.clone());
        							}
        						}			        			
			        		},
			        	}
			        },
			        MusitCommand::PatComm(pat_command) =>  {
			        	match pat_command {
			        		PatternCommand::NewPattern(id) => {
			        			let id2 = id.clone();			        			
			        			let pattern = Pattern::new(id);
			        			pattern_list.insert(id2, pattern);
			        		},
							PatternCommand::AddEvent(pattern_id, event) => (),
			        	}
			        },
	        	}
			},	
	        Err(err) => {
		        match err   {
				    TryRecvError::Empty => (),
				    TryRecvError::Disconnected => panic!("Sequencer comm channel disconnect error."),
				}
			}
	    };

	    //manage input device and run receive all pattern
	    pattern_list.iter_mut().filter(|&(_,ref p)| p.receiver.is_some()).map(|(_,p)| p.try_read_channel()).collect::<Vec<()>>();


	    //manage wait time
	    let new_time_ns = precise_time_ns();
	    let diff = new_time_ns - time_ns;
	    if (diff) < wait_time_ns {
	    	let wait = (wait_time_ns - diff) / 1000;
	    	//println!("wait: {:?}", wait);
	    	sleep_ms((wait) as u32);
	    }
	    time_ns = precise_time_ns();
	    //println!("time: {:?}, diff: {:?}", time_ns, diff);
	    tick += 1;
	}

}