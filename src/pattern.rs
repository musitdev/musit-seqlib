//! Module: musitrt
use portmidi::MidiEvent;
use std::collections::BTreeMap;

use event::SeqEvent;

pub type PatternId = usize;

pub enum PatternCommand{
	NewPattern(PatternId),
	AddEvent(PatternId, SeqEvent),
}

pub struct Pattern	{
	pub id: PatternId,
	pub events: BTreeMap<u64, MidiEvent>,
}

impl Pattern	{
	pub fn new(id: PatternId) -> Pattern {
		Pattern {id: id, events:BTreeMap::new()}
	}

	pub fn add_event<'b>(&'b mut self, tick:u64, event: MidiEvent)	{
		println!("Pattern {:?} receive event  {:?}", self.id, event);
		self.events.insert(tick, event);
	}

	pub fn get_event_for_tick(&self, tick: &u64) -> Option<&MidiEvent> {
		self.events.get(tick)
	}
}
