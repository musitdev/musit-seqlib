extern crate time;
extern crate midi;
extern crate portmidi;
extern crate comm;

mod sequencer;
mod pattern;
mod event;
mod device;
mod process;
mod filter;

use portmidi::{PortMidi, OutputPort, MidiMessage, MidiEvent, DeviceInfo};

use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

use sequencer::{sequencer_process, MusitCommand,SeqCommand};
use pattern::PatternCommand;
//use device::MidiExecUnit;

fn main() {

    let portmidi = PortMidi::new().ok().expect("Can't init port midi");
    let no = portmidi.device_count();
    // use filter_map to discard None, and unwrap the Some(_)
    let devices = portmidi.devices().ok().expect("Can't get device info");;
    for d in devices.into_iter() {
        println!("{:<3} {:<20} {:<6} {:<6}", d.id(), d.name(), d.is_input(), d.is_output());
    }

	let (tx, rx): (Sender<MusitCommand>, Receiver<MusitCommand>) = channel();

    thread::spawn(move|| {
	    sequencer_process(rx);
	});

	//Create new pattern:
	tx.send(MusitCommand::PatComm(PatternCommand::NewPattern(1))); //patternid
	//attach device
	tx.send(MusitCommand::SeqComm(SeqCommand::CreateMidiInput(1, 1))); //deviceid, midi id
	tx.send(MusitCommand::SeqComm(SeqCommand::ConnectPatternToInput(1, 1))); //deviceid, patternid

	tx.send(MusitCommand::SeqComm(SeqCommand::ConnectProcess(1, Box::new(|tick: u64, event: MidiEvent| {println!("event :{:?}", event); Some(event)}))));
	tx.send(MusitCommand::SeqComm(SeqCommand::ConnectFilter(1, Box::new(|event: &MidiEvent| event.message.data1 > 60))));
	tx.send(MusitCommand::SeqComm(SeqCommand::ConnectProcess(1, Box::new(|tick: u64, event: MidiEvent| {println!("event after filter :{:?}", event); Some(event)}))));

	thread::sleep_ms(100);

    thread::spawn(move|| {
    	let device_info = DeviceInfo::new(0).unwrap();
		let mut output = OutputPort::new(device_info, 1024).unwrap();
    	loop {
			match output.write_message(MidiMessage{status: 129, data1: 64, data2: 0,})	{
				Ok(_) => (),
				Err(err) => panic!("error midi send {:?}", err),
			}
			match output.write_message(MidiMessage{status: 129, data1: 59, data2: 0,})	{
				Ok(_) => (),
				Err(err) => panic!("error midi send {:?}", err),
			}
	    	thread::sleep_ms(1000);
    	}
	});

	thread::sleep_ms(30000);
}
