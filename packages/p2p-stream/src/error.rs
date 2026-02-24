use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("GStreamer initialization failed: {0}")]
    GstreamerInit(String),

    #[error("GStreamer error: {0}")]
    Gstreamer(#[from] gstreamer::glib::Error),

    #[error("GStreamer state change failed: {0}")]
    StateChange(String),

    #[error("Pipeline construction failed: {0}")]
    PipelineConstruction(String),

    #[error("Element not found: {0}")]
    ElementNotFound(String),

    #[error("Element linking failed: {source_element} -> {sink_element}")]
    ElementLinkFailed {
        source_element: String,
        sink_element: String,
    },

    #[error("Session error: {0}")]
    Session(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Signaling error: {0}")]
    Signaling(String),

    #[error("SDP parsing error: {0}")]
    SdpParse(String),

    #[error("ICE error: {0}")]
    Ice(String),

    #[error("Media source error: {0}")]
    MediaSource(String),

    #[error("Codec not available: {0}")]
    CodecUnavailable(String),

    #[error("Pipeline is not in expected state. Expected: {expected}, actual: {actual}")]
    InvalidState { expected: String, actual: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
