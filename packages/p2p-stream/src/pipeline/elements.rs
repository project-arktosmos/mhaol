use crate::error::{Error, Result};
use gstreamer as gst;

/// Create a GStreamer element by factory name with an optional element name.
pub(crate) fn make_element(factory: &str, name: Option<&str>) -> Result<gst::Element> {
    let mut builder = gst::ElementFactory::make(factory);
    if let Some(n) = name {
        builder = builder.name(n);
    }
    builder
        .build()
        .map_err(|_| Error::ElementNotFound(factory.to_string()))
}
