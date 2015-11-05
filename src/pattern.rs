//! Module: musitrt
use portmidi::MidiEvent;

use event::SeqEvent;

pub type PatternId = usize;

pub enum PatternCommand{
	NewPattern(PatternId),
	AddEvent(PatternId, SeqEvent),
}

pub struct Pattern	{
	pub id: PatternId,
	pub events: Vec<MidiEvent>,
}

impl Pattern	{
	pub fn new(id: PatternId) -> Pattern {
		Pattern {id: id, events:vec!()}
	}

	pub fn add_event<'b>(&'b mut self, tick:u64, event: MidiEvent)	{
		println!("Pattern {:?} receive event  {:?}", self.id, event);
		self.events.push(event);
	}
}
