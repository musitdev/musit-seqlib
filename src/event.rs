use midi::Message;

pub struct SeqEvent {
    pub tick: u64,
    pub message: Message,
}