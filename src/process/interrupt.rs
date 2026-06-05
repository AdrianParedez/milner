use std::sync::atomic::{AtomicU8, Ordering};
#[cfg(test)]
use std::sync::{Mutex, MutexGuard};

const NONE: u8 = 0;
const CTRL_C: u8 = 1;
const CTRL_BREAK: u8 = 2;

static PENDING_INTERRUPT: AtomicU8 = AtomicU8::new(NONE);
#[cfg(test)]
static TEST_INTERRUPT_LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterruptKind {
    CtrlC,
    CtrlBreak,
}

impl std::fmt::Display for InterruptKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CtrlC => write!(f, "Ctrl+C"),
            Self::CtrlBreak => write!(f, "Ctrl+Break"),
        }
    }
}

pub fn request(kind: InterruptKind) {
    PENDING_INTERRUPT.store(kind.as_code(), Ordering::SeqCst);
}

pub fn take_pending() -> Option<InterruptKind> {
    InterruptKind::from_code(PENDING_INTERRUPT.swap(NONE, Ordering::SeqCst))
}

#[cfg(test)]
pub fn clear_pending_for_test() {
    PENDING_INTERRUPT.store(NONE, Ordering::SeqCst);
}

#[cfg(test)]
pub fn test_lock() -> MutexGuard<'static, ()> {
    TEST_INTERRUPT_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl InterruptKind {
    fn as_code(self) -> u8 {
        match self {
            Self::CtrlC => CTRL_C,
            Self::CtrlBreak => CTRL_BREAK,
        }
    }

    fn from_code(code: u8) -> Option<Self> {
        match code {
            CTRL_C => Some(Self::CtrlC),
            CTRL_BREAK => Some(Self::CtrlBreak),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{InterruptKind, clear_pending_for_test, request, take_pending, test_lock};

    #[test]
    fn pending_interrupt_is_taken_once() {
        let _guard = test_lock();
        clear_pending_for_test();
        request(InterruptKind::CtrlC);

        assert_eq!(take_pending(), Some(InterruptKind::CtrlC));
        assert_eq!(take_pending(), None);
    }

    #[test]
    fn newer_interrupt_replaces_older_pending_interrupt() {
        let _guard = test_lock();
        clear_pending_for_test();
        request(InterruptKind::CtrlC);
        request(InterruptKind::CtrlBreak);

        assert_eq!(take_pending(), Some(InterruptKind::CtrlBreak));
        assert_eq!(take_pending(), None);
    }
}
