use std::sync::{Arc, RwLock};
use std::thread;

use comm::{spmc, Error};

use portmidi::{self, InputPort, PortMidiDeviceId, MidiEvent, DeviceInfo};

use pattern::Pattern;
//use midi::{RawMessage, Status};
pub type DeviceId = usize;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ExecUnitError {
    BadProcess,
    Poisoned,
}

pub type ExecUnitResult<T=()> = Result<T, ExecUnitError>;
pub type MidiExecUnit = ExecUnitResult<MidiEvent>;
pub type ProcessUnit<A> = Box<Fn(u64, A) -> Option<A> + Send + Sync + 'static,>;
pub type FilterUnit<A> = Box<Fn(&A) -> bool + Send + Sync + 'static,>;

//npput = input.new(device)
//input.stream().map(f).filter(f).ouput(device); f(eventlist) (send et sync)
//input.tick(t)

pub struct Input<A,I>{
	pub id: DeviceId,
	input:I,
	to_exec: Arc<RwLock<Vec<ProcessUnit<A>>>>,
	pub active: bool,
}

impl<A,I> Input<A,I>{
	pub fn new(id: DeviceId, dev: I) -> Input<A,I>	{
		Input {id: id, input: dev, to_exec: Arc::new(RwLock::new(vec![])), active: true}
	}

	pub fn stream(&self) -> Stream<A>	{
		Stream{to_exec: self.to_exec.clone()}
	}

	pub fn process(&self, tick: u64, input: A) -> Option<A> {
		let mut res = Some(input);
		for exec in &*self.to_exec.read().unwrap()	{
			if let None = res {
				break;
			}
			else {
				res = exec(tick, res.unwrap());
			}
		}
		res
	}
}

pub struct Stream<A>{
	to_exec: Arc<RwLock<Vec<ProcessUnit<A>>>>,
}

impl<A: 'static> Stream<A>{

	pub fn process(&self, f: ProcessUnit<A>) -> Stream<A>
        //where F: <Fn(A) -> Option<A> + Send + Sync + 'static,
    {
    	self.to_exec.write().unwrap().push(f);
    	Stream{to_exec: self.to_exec.clone()}
    }

	pub fn filter<F>(&self, f: FilterUnit<A>) -> Stream<A>
    //    where F: Fn(&A) -> bool+ Send + Sync + 'static,
    {
    	self.to_exec.write().unwrap().push(Box::new(move |_, a|{
    		if (&f)(&a) {
    			Some(a)
    		} else	{
    			None
    		}
    	}));
    	Stream{to_exec: self.to_exec.clone()}
    }
}

pub type MidiInput = Input<MidiEvent, PortMidiInputDevice<'static>>;

impl MidiInput {

	pub fn process_input(& self, tick: u64) -> Vec<MidiEvent>	{
		let mut event_list = vec!();
		loop {
			let rc_event = {
				match self.input.receiver.recv_async() 	{
					Ok(event) => Some(event),
					Err(err) =>  {
				        match err   {
						    Error::Empty => None,
						    _ => panic!("Error during pattern channel receive event: channel disconnect."),
						}
					},
				}
			};
			if let Some(event) = rc_event	{
				let new_event_opt = self.process(tick, event);
				if let Some(new_event) = new_event_opt {
					event_list.push(new_event);
				}
			} else {
				break;
			}
		}
		event_list
	}
}


pub struct PortMidiInputDevice<'a>	{
	pub id: DeviceId,
	pub midi_id: PortMidiDeviceId,
	pub receiver: spmc::bounded_fast::Consumer<'a, MidiEvent>,
}

impl<'a> PortMidiInputDevice<'a>	{
	pub fn new(id: DeviceId, portmidi_id: PortMidiDeviceId) -> portmidi::Result<PortMidiInputDevice<'static>>	{
		let (send, recv) = unsafe { spmc::bounded_fast::new(10)};
	
		let device = PortMidiInputDevice{id: id, midi_id: portmidi_id, receiver:recv};
		start_read(portmidi_id, send);

		Ok(device)
	}
}


fn start_read<'a>(device_id: PortMidiDeviceId, sender: spmc::bounded_fast::Producer<'static, MidiEvent>) -> portmidi::Result<()>	{
	//let clone_send = sender.clone();
	//let (send, recv) = unsafe { spmc::bounded_fast::new(10)};
	thread::spawn(move || {
		println!("Portmidi open input for port {:?}", device_id);
		let device_info = DeviceInfo::new(device_id).unwrap();
		let mut input = InputPort::new(device_info, 1024).unwrap();
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
	Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_simple_input() {
    	let input = Input::new(InputDevice);
    	input.stream().process(|i| Some(i+1)).process(|i| Some(i+2));
    	let res = input.process(1).unwrap();
    	assert_eq!(res, 4);
    }
    #[test]
    fn test_process_filter_input() {
    	let input = Input::new(InputDevice);
    	input.stream().process(Box::new(|i| Some(i+1))).filter(Box::new(|a| *a > 4)).process(Box::new(|i| Some(i+2)));
    	let res = input.process(1);
    	assert_eq!(res.is_some(), false);
    	let res = input.process(2);
    	assert_eq!(res.is_some(), false);
    	let res = input.process(4);
        assert_eq!(res.is_some(), true);
	    assert_eq!(res.unwrap(), 7);
    }

}



