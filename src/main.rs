extern crate time;
extern crate midi;
extern crate portmidi;
extern crate comm;

mod sequencer;
mod pattern;
mod event;
mod device;

use portmidi::{initialize, count_devices, get_device_info, terminate, OutputPort, MidiMessage};

use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

use sequencer::{sequencer_process, MusitCommand,SeqCommand};
use pattern::PatternCommand;

fn main() {

    portmidi::initialize().unwrap();
    let no = portmidi::count_devices();
    // use filter_map to discard None, and unwrap the Some(_)
    let devices = (0..no).filter_map(|i| portmidi::get_device_info(i)).collect::<Vec<_>>();
    for d in devices.into_iter() {
        println!("{:<3} {:<20} {:<6} {:<6}", d.device_id, d.name, d.input, d.output);
    }   

    let (tx, rx): (Sender<MusitCommand>, Receiver<MusitCommand>) = channel();

    thread::spawn(move|| {
	    sequencer_process(rx);
	});

	//Create new pattern:
	tx.send(MusitCommand::PatComm(PatternCommand::NewPattern("1".to_string())));
	//attach device
	tx.send(MusitCommand::SeqComm(SeqCommand::ConnectInput(1, "1".to_string())));

    thread::spawn(move|| {
		let mut output = OutputPort::new(0, 1024);
		output.open().unwrap();
    	loop {
			match output.write_message(MidiMessage{status: 129, data1: 64, data2: 0,})	{
				Ok(_) => (),
				Err(err) => panic!("error midi send {:?}", err),
			}
	    	thread::sleep_ms(1000);
    	}
	});

	thread::sleep_ms(30000);

	portmidi::terminate().unwrap();
}
