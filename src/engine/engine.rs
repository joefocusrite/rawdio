use std::sync::{atomic::AtomicI64, Arc};

use crate::{realtime::Processor, AudioProcess, Command, CommandQueue, Context, Timestamp};

use lockfree::channel::mpsc;

use super::context::NotifierStatus;

pub struct Engine {
    sample_rate: usize,
    timestamp: Arc<AtomicI64>,
    command_tx: CommandQueue,
    notifiers: Vec<Box<dyn Fn() -> NotifierStatus>>,
}

impl Context for Engine {
    fn start(&mut self) {
        let _ = self.command_tx.send(Command::Start);
    }

    fn stop(&mut self) {
        let _ = self.command_tx.send(Command::Stop);
    }

    fn current_time(&self) -> Timestamp {
        Timestamp::from_raw_i64(self.timestamp.load(std::sync::atomic::Ordering::Acquire))
    }

    fn get_sample_rate(&self) -> usize {
        self.sample_rate
    }

    fn get_command_queue(&self) -> CommandQueue {
        self.command_tx.clone()
    }

    fn add_notifier(&mut self, notifier: Box<dyn Fn() -> NotifierStatus>) {
        self.notifiers.push(notifier);
    }

    fn process_notifications(&mut self) {
        self.notifiers
            .retain(|notifier| (notifier)() == NotifierStatus::Continue);
    }
}

pub fn create_engine(sample_rate: usize) -> (Box<dyn Context>, Box<dyn AudioProcess + Send>) {
    let (command_tx, command_rx) = mpsc::create();
    let timestamp = Arc::new(AtomicI64::new(0));
    let processor = Box::new(Processor::new(
        sample_rate,
        command_rx,
        Arc::clone(&timestamp),
    ));

    let engine = Box::new(Engine {
        sample_rate,
        timestamp,
        command_tx,
        notifiers: Vec::new(),
    });

    (engine, processor)
}