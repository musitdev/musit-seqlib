use std::sync::mpsc::{Receiver, TryRecvError};
use portmidi::PortMidiDeviceId;
use time::precise_time_ns;
use std::thread::sleep_ms;
use std::collections::HashMap;
use std::cell::RefCell;

use portmidi::{MidiEvent};

use pattern::{Pattern, PatternCommand, PatternId};
use device::{PortMidiInputDevice, MidiInput, DeviceId, ProcessUnit, FilterUnit};

pub enum SeqCommand {
    CreateMidiInput(DeviceId, PortMidiDeviceId),
    //create the specified portMidiInput with the specified id
    ConnectProcess(DeviceId, ProcessUnit<MidiEvent>),
    ConnectFilter(DeviceId, FilterUnit<MidiEvent>),
    ConnectPatternToInput(DeviceId, Vec<PatternId>),
}

pub enum MusitCommand {
    SeqComm(SeqCommand),
    PatComm(PatternCommand),
}

pub fn sequencer_process(receiver: Receiver<MusitCommand>) {
    let wait_time_ns: u64 = 1000; //in microsecond
    let mut time_ns: u64 = precise_time_ns();
    let mut pattern_list: HashMap<PatternId, RefCell<Pattern>> = HashMap::new();
    let mut pattern_connection_list: HashMap<DeviceId, PatternId> = HashMap::new();
    let mut input_device_list: Vec<MidiInput> = vec!();
    let mut tick: u64 = 0;
    loop {
        //manage extern command
        let comm_rx_value = receiver.try_recv();
        match comm_rx_value {
            Ok(command) => {
                match command {
                    MusitCommand::SeqComm(comm) => {
                        match comm {
                            SeqCommand::CreateMidiInput(id, device_id) => {
                                //create the device if not created.
                                if let None = input_device_list.iter().find(|&d| d.id == id) {
                                    match PortMidiInputDevice::new(device_id as DeviceId,
                                                                   device_id) {
                                        Ok(new_dev) => {
                                            let midi_dev = MidiInput::new(id, new_dev);
                                            input_device_list.push(midi_dev);
                                        },
                                        Err(err) =>
                                            panic!("can\'t allocate new midi device {:?} because {:?}",id, err),
                                    }
                                }

                                //connect pattern to device.
                                /*	if let Some(patt) = pattern_list.get_mut(&id)	{
        							if let Some(device) = input_device_list.iter().find(|&d| d.midi_id == device_id)	{
        								patt.receiver = Some(device.receiver.clone());
        							}
        						}	*/
                            },
                            SeqCommand::ConnectProcess(device_id, process) => {
                                if let Some(input) =
                                   input_device_list.iter().find(|&d| d.id == device_id) {
                                    input.stream().process(process);
                                }
                            },
                            SeqCommand::ConnectFilter(device_id, filter) => {
                                if let Some(input) =
                                   input_device_list.iter().find(|&d| d.id == device_id) {
                                    input.stream().filter::<MidiEvent>(filter);
                                }
                            },
                            SeqCommand::ConnectPatternToInput(device_id, pattern_id) => {},

                        }
                    },
                    MusitCommand::PatComm(pat_command) => {
                        match pat_command {
                            PatternCommand::NewPattern(id) => {
                                let id2 = id.clone();
                                let pattern = Pattern::new(id);
                                pattern_list.insert(id2, RefCell::new(pattern));
                            },
                            PatternCommand::AddEvent(pattern_id, event) => (),
                        }
                    },
                }
            },
            Err(err) => {
                match err {
                    TryRecvError::Empty => (),
                    TryRecvError::Disconnected =>
                        panic!("Sequencer comm channel disconnect error."),
                }
            }
        };

        //manage input device and run receive all pattern
        //	    pattern_list.iter_mut().filter(|&(_,ref p)| p.receiver.is_some()).map(|(_,p)| p.try_read_channel()).collect::<Vec<()>>();
        for input in input_device_list.iter() {
            let event_list = input.process_input(tick);

            //send event to pattern connected to input.
            for event in event_list {
                let mut plist: Vec<&mut Pattern> = vec!();
                if pattern_connection_list.contains_key(&input.id) {
                    let pattern_id = pattern_connection_list.get(&(input.id)).unwrap();
                    if let Some(pattern) = pattern_list.get(pattern_id) {
                        pattern.borrow_mut().add_event(tick, event);
                    }
                }
            }
        }

        //manage wait time
        let new_time_ns = precise_time_ns();
        let diff = new_time_ns - time_ns;
        if (diff) < wait_time_ns {
            let wait = (wait_time_ns - diff) / 1000;
            //println!("wait: {:?}", wait);
            sleep_ms((wait) as u32);
        }
        time_ns = precise_time_ns();
        //println!("time: {:?}, diff: {:?}", time_ns, diff);
        tick += 1;
    }
}