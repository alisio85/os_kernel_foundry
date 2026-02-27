//! Inter-process communication (IPC) primitives.
//!
//! This module contains minimal, test-focused abstractions for message-based
//! communication. It is deliberately agnostic about how tasks are scheduled
//! or how messages are delivered at the hardware level.

/// Represents a message that can be sent through an IPC channel.
pub trait Message {
    /// Returns a short, stable identifier for the message type.
    fn message_type(&self) -> &'static str;
}

/// A generic, lossy message endpoint.
///
/// In a real kernel, back-pressure and flow control are important concerns.
/// Here we focus on a simple, testable contract: senders can attempt to send
/// messages, and receivers can attempt to receive them.
pub trait MessageEndpoint<M: Message> {
    /// Attempts to send a message.
    ///
    /// Returns `true` on success, or `false` if the message could not be
    /// enqueued (for example because the channel is full).
    fn send(&mut self, msg: M) -> bool;

    /// Attempts to receive a message.
    fn recv(&mut self) -> Option<M>;

    /// Returns the number of messages currently buffered.
    fn len(&self) -> usize;

    /// Returns `true` if there are no messages buffered.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::{Message, MessageEndpoint};

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestMessage {
        kind: &'static str,
        payload: u32,
    }

    impl Message for TestMessage {
        fn message_type(&self) -> &'static str {
            self.kind
        }
    }

    /// A fixed-capacity, in-memory channel used for tests.
    struct FixedChannel<const N: usize> {
        buffer: std::collections::VecDeque<TestMessage>,
    }

    impl<const N: usize> FixedChannel<N> {
        fn new() -> Self {
            Self {
                buffer: std::collections::VecDeque::new(),
            }
        }
    }

    impl<const N: usize> MessageEndpoint<TestMessage> for FixedChannel<N> {
        fn send(&mut self, msg: TestMessage) -> bool {
            if self.buffer.len() >= N {
                return false;
            }
            self.buffer.push_back(msg);
            true
        }

        fn recv(&mut self) -> Option<TestMessage> {
            self.buffer.pop_front()
        }

        fn len(&self) -> usize {
            self.buffer.len()
        }
    }

    #[test]
    fn fixed_channel_sends_and_receives() {
        let mut chan: FixedChannel<2> = FixedChannel::new();

        assert!(chan.send(TestMessage { kind: "ping", payload: 1 }));
        assert!(chan.send(TestMessage { kind: "ping", payload: 2 }));
        assert!(!chan.send(TestMessage { kind: "ping", payload: 3 }));

        assert_eq!(
            chan.recv(),
            Some(TestMessage {
                kind: "ping",
                payload: 1
            })
        );
        assert_eq!(
            chan.recv(),
            Some(TestMessage {
                kind: "ping",
                payload: 2
            })
        );
        assert!(chan.recv().is_none());
    }
}

