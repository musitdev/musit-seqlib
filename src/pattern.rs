//! Module: musitrt
use portmidi::MidiEvent;
use comm::{spmc, Error};

use event::Event;


pub enum PatternCommand{
	NewPattern(String),
	AddEvent(String, Event),
}

pub struct Pattern<'a>	{
	pub id: String,
	pub events: Vec<MidiEvent>,
	pub receiver: Option<spmc::bounded_fast::Consumer<'a, MidiEvent>>,
}

impl<'a> Pattern<'a>	{
	pub fn new(id: String) -> Pattern<'a> {
		Pattern {id: id, events:vec!(), receiver: None}
	}

	pub fn add_event<'b>(&'b mut self,  event: MidiEvent)	{
		println!("Pattern {:?} receive event  {:?}", self.id, event);
		self.events.push(event);
	}

	pub fn try_read_channel<'b>(&'b mut self)	{
		let rc_event = {
			if let Some(ref receive) = self.receiver {
				match receive.recv_async() 	{
					Ok(event) => Some(event),
					Err(err) =>  {
				        match err   {
						    Error::Empty => None,
						    _ => panic!("Error during pattern channel receive event: channel disconnect."),
						}
					},
				}
			} else {
				None
			}
		};
		if let Some(event) = rc_event	{
			self.add_event(event);
		};

	}
}
