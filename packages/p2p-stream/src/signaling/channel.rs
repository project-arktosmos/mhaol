use crate::signaling::messages::{PeerId, SignalingMessage};
use tokio::sync::mpsc;

/// The consumer-facing half of the signaling channel.
///
/// Consumers read outgoing messages from `outgoing_rx` and forward them
/// over their transport (WebSocket, HTTP, etc.). They push incoming messages
/// from the remote peer into `incoming_tx`.
pub struct SignalingChannel {
    pub outgoing_rx: mpsc::UnboundedReceiver<(PeerId, SignalingMessage)>,
    pub incoming_tx: mpsc::UnboundedSender<(PeerId, SignalingMessage)>,
}

/// The library-internal half of the signaling channel.
///
/// The library writes outgoing signaling messages to `outgoing_tx` and
/// reads incoming messages from `incoming_rx`.
pub struct SignalingBridge {
    pub outgoing_tx: mpsc::UnboundedSender<(PeerId, SignalingMessage)>,
    pub incoming_rx: mpsc::UnboundedReceiver<(PeerId, SignalingMessage)>,
}

/// Create a new signaling channel pair.
///
/// Returns `(consumer_side, library_side)`. The consumer reads outgoing
/// messages and pushes incoming messages. The library does the reverse.
pub fn signaling_channel() -> (SignalingChannel, SignalingBridge) {
    let (outgoing_tx, outgoing_rx) = mpsc::unbounded_channel();
    let (incoming_tx, incoming_rx) = mpsc::unbounded_channel();
    (
        SignalingChannel {
            outgoing_rx,
            incoming_tx,
        },
        SignalingBridge {
            outgoing_tx,
            incoming_rx,
        },
    )
}
