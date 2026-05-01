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

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Source not found: {0}")]
    SourceNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
