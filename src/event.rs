use midi::Message;


pub struct Event {
	pub ts: u64,
	pub message: Message,
}