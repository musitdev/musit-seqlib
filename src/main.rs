extern crate time;

mod sequencer;
mod pattern;
mod note;

use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

use sequencer::{sequencer_process, SequencerCommand};

fn main() {
    let (tx, rx): (Sender<SequencerCommand>, Receiver<SequencerCommand>) = channel();

    thread::spawn(move|| {
	    sequencer_process(rx);
	});

	thread::sleep_ms(10000);
}
