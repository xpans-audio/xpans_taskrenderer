use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
    mpsc::Receiver,
};

/// Message type to control render processes.
#[derive(Debug, Clone, Copy)]
pub enum Control {
    Pause,
    Resume,
    Cancel,
}

/**
Atomic bitflags indicating when the render process should be paused or
cancelled.
*/
#[derive(Debug, Clone, Default)]
pub struct AtomicStatus {
    inner: Arc<AtomicU8>,
}

impl AtomicStatus {
    pub fn get(&self) -> Status {
        Status {
            inner: self.inner.load(Ordering::Acquire),
        }
    }
    pub fn set(&self, status: Status) {
        self.inner.store(status.inner, Ordering::Release);
    }
    pub fn cancel(&self) {
        self.inner.fetch_or(CANCELLED_BIT, Ordering::SeqCst);
    }
    pub fn pause(&self) {
        self.inner.fetch_or(PAUSED_BIT, Ordering::SeqCst);
    }
    pub fn resume(&self) {
        self.inner.fetch_and(!PAUSED_BIT, Ordering::SeqCst);
    }
}
pub struct Status {
    inner: u8,
}

const CANCELLED_BIT: u8 = 0b_0000_0001;
const PAUSED_BIT: u8 = 0b_0000_0010;

impl Status {
    pub fn cancelled(&self) -> bool {
        (self.inner & CANCELLED_BIT) == CANCELLED_BIT
    }
    pub fn paused(&self) -> bool {
        (self.inner & PAUSED_BIT) == PAUSED_BIT
    }
}

/// A process that receives control messages and updates the `AtomicStatus`
/// to match.
///
/// This functions loops for the entire time the receiver is available.
pub fn manage_control(receiver: Receiver<Control>, atomic_status: AtomicStatus) {
    while let Ok(control) = receiver.recv() {
        match control {
            Control::Pause => atomic_status.pause(),
            Control::Resume => atomic_status.resume(),
            Control::Cancel => atomic_status.cancel(),
        }
    }
}
