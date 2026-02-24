use gstreamer as gst;
use gstreamer::prelude::*;
use tokio::sync::mpsc;
use tracing::{debug, error, warn};

/// Events extracted from the GStreamer pipeline bus.
#[derive(Debug, Clone)]
pub enum BusEvent {
    Eos,
    Error {
        message: String,
        debug: Option<String>,
    },
    Warning {
        message: String,
        debug: Option<String>,
    },
    StateChanged {
        old: gst::State,
        new: gst::State,
    },
}

/// Spawn a dedicated thread that monitors the GStreamer pipeline bus and forwards
/// events to a tokio channel.
pub(crate) fn spawn_bus_monitor(pipeline: &gst::Pipeline) -> mpsc::UnboundedReceiver<BusEvent> {
    let (tx, rx) = mpsc::unbounded_channel();
    let bus = pipeline.bus().expect("Pipeline must have a bus");

    std::thread::Builder::new()
        .name("gst-bus-monitor".into())
        .spawn(move || {
            for msg in bus.iter_timed(gst::ClockTime::NONE) {
                let event = match msg.view() {
                    gst::MessageView::Eos(_) => {
                        debug!("Pipeline reached end of stream");
                        Some(BusEvent::Eos)
                    }
                    gst::MessageView::Error(err) => {
                        let msg = err.error().to_string();
                        let dbg = err.debug().map(|d| d.to_string());
                        error!("Pipeline error: {msg}");
                        Some(BusEvent::Error {
                            message: msg,
                            debug: dbg,
                        })
                    }
                    gst::MessageView::Warning(warn_msg) => {
                        let msg = warn_msg.error().to_string();
                        let dbg = warn_msg.debug().map(|d| d.to_string());
                        warn!("Pipeline warning: {msg}");
                        Some(BusEvent::Warning {
                            message: msg,
                            debug: dbg,
                        })
                    }
                    gst::MessageView::StateChanged(sc) => Some(BusEvent::StateChanged {
                        old: sc.old(),
                        new: sc.current(),
                    }),
                    _ => None,
                };

                if let Some(event) = event {
                    if tx.send(event).is_err() {
                        break;
                    }
                }
            }
        })
        .expect("Failed to spawn bus monitor thread");

    rx
}
