
use portmidi::{InputPort, PortMidiDeviceId, PortMidiResult, MidiEvent};
use comm::spmc;
use std::thread;
//use midi::{RawMessage, Status};


pub struct InputDevice<'a>	{
	pub id: PortMidiDeviceId,
	pub receiver: spmc::bounded_fast::Consumer<'a, MidiEvent>,
}

impl<'a> InputDevice<'a>	{
	pub fn new(device_id: PortMidiDeviceId) -> PortMidiResult<InputDevice<'static>>	{
		let (send, recv) = unsafe { spmc::bounded_fast::new(10)};
	
		let device = InputDevice{id: device_id, receiver:recv};
		start_read(device_id, send);

		Ok(device)
	}
}


fn start_read<'a>(device_id: PortMidiDeviceId, sender: spmc::bounded_fast::Producer<'static, MidiEvent>)	{
	//let clone_send = sender.clone();
	//let (send, recv) = unsafe { spmc::bounded_fast::new(10)};
	thread::spawn(move || {
		println!("Portmidi open input for port {:?}", device_id);
		let mut input = InputPort::new(device_id, 1024);
		input.open().unwrap();

		println!("Portmidi init ok");

		loop {
	        while let Some(event) = input.read().unwrap() {
	        	println!("Portmidi read available");
	            // filter midi time stamps and active sensing
	            if event.message.status != 248 && event.message.status != 254 {
	                println!("{} {:?}", event.timestamp, event.message);
	            }
	            sender.send_async(event).unwrap();
	        };
	        thread::sleep_ms(10);
		};
	});
}



