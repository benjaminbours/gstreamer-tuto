extern crate gstreamer as gst;
use gst::prelude::*;

#[path = "../../tutorials-common.rs"]
mod tutorials_common;

fn tutorial_main() {
    // Initialize GStreamer
    gst::init().unwrap();

    // Create the elements
    let source = gst::ElementFactory::make("uridecodebin", Some("source"))
        .expect("Could not create source element.");
    let audio_convert = gst::ElementFactory::make("audioconvert", Some("audio convert"))
        .expect("Could not create audio convert element.");
    let video_convert = gst::ElementFactory::make("videoconvert", Some("video convert"))
        .expect("Could not create video convert element.");
    let audio_sink = gst::ElementFactory::make("autoaudiosink", Some("audio sink"))
        .expect("Could not create audio sink element.");
    let video_sink = gst::ElementFactory::make("autovideosink", Some("video sink"))
        .expect("Could not create video sink element.");

    // Create the empty pipeline
    let pipeline = gst::Pipeline::new(Some("test-pipeline"));

    // Build the pipeline Note that we are NOT linking the source at this
    // point. We will do it later.
    pipeline
        .add_many(&[&source, &audio_convert, &audio_sink, &video_convert, &video_sink])
        .unwrap();
    audio_convert
        .link(&audio_sink)
        .expect("Elements could not be linked.");
    video_convert
        .link(&video_sink)
        .expect("Elements could not be linked.");

    // Set the URI to play
    let uri =
        "https://www.freedesktop.org/software/gstreamer-sdk/data/media/sintel_trailer-480p.webm";
    source
        .set_property("uri", &uri)
        .expect("Can't set uri property on uridecodebin");

    // Connect the pad-added signal
    let pipeline_weak = pipeline.downgrade();
    let audio_convert_weak = audio_convert.downgrade();
    let video_convert_weak = video_convert.downgrade();
    source.connect_pad_added(move |_, src_pad| {
        let pipeline = match pipeline_weak.upgrade() {
            Some(pipeline) => pipeline,
            None => return,
        };

        let audio_convert = match audio_convert_weak.upgrade() {
            Some(convert) => convert,
            None => return,
        };

        let video_convert = match video_convert_weak.upgrade() {
            Some(convert) => convert,
            None => return,
        };

        println!(
            "Received new pad {} from {}",
            src_pad.get_name(),
            pipeline.get_name()
        );

        let audio_sink_pad = audio_convert
            .get_static_pad("sink")
            .expect("Failed to get static sink pad from audio convert");

        let video_sink_pad = video_convert
            .get_static_pad("sink")
            .expect("Failed to get static sink pad from video convert");
        // If our converter is already linked, we have nothing to do here
        if audio_sink_pad.is_linked() && video_sink_pad.is_linked() {
            println!("We are already linked. Ignoring.");
            return;
        }

        // Check the new pad's type
        let new_pad_caps = src_pad
            .get_current_caps()
            .expect("Failed to get caps from new pad.");
        let new_pad_struct = new_pad_caps
            .get_structure(0)
            .expect("Failed to get first structure of caps");
        let new_pad_type = new_pad_struct.get_name();
        let is_audio = new_pad_type.starts_with("audio/x-raw");
        let is_video = new_pad_type.starts_with("video/x-raw");

        let sink_to_link = if is_audio {
            Some(&audio_sink_pad)
        } else if is_video {
            Some(&video_sink_pad)
        } else {
            None
        }.expect("new pad type is not video or audio");

        // Attempt the link
        let res = src_pad.link(sink_to_link);
        if res.is_err() {
            println!("Type is {} but link failed. {:?}", new_pad_type, res);
        } else {
            println!("Link succeeded (type {}).", new_pad_type);
        }
    });

    // Start playing
    pipeline
        .set_state(gst::State::Playing)
        .expect("Unable to set the pipeline to the `Playing` state");

    // Wait until error or EOS
    let bus = pipeline.get_bus().unwrap();
    for msg in bus.iter_timed(gst::CLOCK_TIME_NONE) {
        use gst::MessageView;

        match msg.view() {
            MessageView::Error(err) => {
                eprintln!(
                    "Error received from element {:?} {}",
                    err.get_src().map(|s| s.get_path_string()),
                    err.get_error()
                );
                eprintln!("Debugging information: {:?}", err.get_debug());
                break;
            }
            MessageView::StateChanged(state_changed) => {
                if state_changed
                    .get_src()
                    .map(|s| s == pipeline)
                    .unwrap_or(false)
                {
                    println!(
                        "Pipeline state changed from {:?} to {:?}",
                        state_changed.get_old(),
                        state_changed.get_current()
                    );
                }
            }
            MessageView::Eos(..) => break,
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
