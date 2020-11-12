use core::{
    pin::Pin,
    task::{Context, Poll},
};

use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use futures_lite::Stream;
use futures_util::{task::AtomicWaker, StreamExt};
use pc_keyboard::KeyEvent;

static SCANCODE_QUEUE: OnceCell<ArrayQueue<KeyEvent>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

/// Called by the keyboard interrupt handler
///
/// Must not block or allocate.
pub(crate) fn add_scancode(scancode: KeyEvent) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            warn!(target:"keyboard_handler", "WARNING: scancode queue full; dropping keyboard input");
        } else {
            WAKER.wake();
        }
    } else {
        warn!(target:"keyboard_handler", "WARNING: scancode queue uninitialized");
    }
}

/// Stream of input scancodes.
pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = KeyEvent;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<KeyEvent>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("scancode queue not initialized");

        // fast path
        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Some(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending,
        }
    }
}

pub async fn print_keypresses() -> u8 {
    let mut scancodes = ScancodeStream::new();
    // info!(target:"Keyboard", "Start processing keyboard events");

    while let Some(scancode) = scancodes.next().await {
        info!("KeyEvent: {:?}", scancode);
    }

    0
}
