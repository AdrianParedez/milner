use std::sync::atomic::{AtomicU8, Ordering};

const NONE: u8 = 0;
const CTRL_C: u8 = 1;
const CTRL_BREAK: u8 = 2;

static PENDING_INTERRUPT: AtomicU8 = AtomicU8::new(NONE);

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
    use super::{InterruptKind, request, take_pending};

    #[test]
    fn pending_interrupt_is_taken_once() {
        request(InterruptKind::CtrlC);

        assert_eq!(take_pending(), Some(InterruptKind::CtrlC));
        assert_eq!(take_pending(), None);
    }

    #[test]
    fn newer_interrupt_replaces_older_pending_interrupt() {
        request(InterruptKind::CtrlC);
        request(InterruptKind::CtrlBreak);

        assert_eq!(take_pending(), Some(InterruptKind::CtrlBreak));
        assert_eq!(take_pending(), None);
    }
}
