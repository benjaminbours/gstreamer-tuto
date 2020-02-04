extern crate gstreamer as gst;
use gst::prelude::*;

#[path = "../../tutorials-common.rs"]
mod tutorials_common;

fn tutorial_main() {
    // Initialize GStreamer
    gst::init().unwrap();

    // Create the elements
    let source = gst::ElementFactory::make("videotestsrc", Some("source"))
        .expect("Could not create source element.");
    let filter = gst::ElementFactory::make("vertigotv", Some("filter"))
        .expect("Could not create filter element.");
    let videoconvert = gst::ElementFactory::make("videoconvert", Some("element"))
        .expect("Could not create videoconvert element");
    let sink = gst::ElementFactory::make("autovideosink", Some("sink"))
        .expect("Could not create sink element.");

    // Create the empty pipeline
    let pipeline = gst::Pipeline::new(Some("test-pipeline"));

    // Build the pipeline
    pipeline.add_many(&[&source, &filter, &videoconvert, &sink]).unwrap();
    gst::Element::link_many(&[&source, &filter, &videoconvert, &sink]).unwrap();

    // Modify the source's properties
    source.set_property_from_str("pattern", "smpte");

    // Start playing
    pipeline
        .set_state(gst::State::Playing)
        .expect("Unable to set the pipeline to the `Playing` state");

    // Wait until error or EOS
    let bus = pipeline.get_bus().unwrap();
    for msg in bus.iter_timed(gst::CLOCK_TIME_NONE) {
        use gst::MessageView;

        match msg.view() {
            MessageView::Eos(..) => break,
            MessageView::Error(err) => {
                println!(
                    "Error from {:?}: {} ({:?})",
                    err.get_src().map(|s| s.get_path_string()),
                    err.get_error(),
                    err.get_debug()
                );
                break;
            }
            _ => (),
        }
    }

    // Shutdown pipeline
    pipeline
        .set_state(gst::State::Null)
        .expect("Unable to set the pipeline to the `Null` state");
}

fn main() {
    tutorials_common::run(tutorial_main);
}
