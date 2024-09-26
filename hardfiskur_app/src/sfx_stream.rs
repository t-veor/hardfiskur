use std::io::Cursor;

use rodio::{Decoder, OutputStream, OutputStreamHandle, Source};

pub struct SFXStream {
    _output_stream: OutputStream,
    output_stream_handle: OutputStreamHandle,
}

impl SFXStream {
    pub fn new() -> Self {
        let (stream, handle) = OutputStream::try_default().unwrap();

        Self {
            _output_stream: stream,
            output_stream_handle: handle,
        }
    }

    pub fn play_capture(&self) {
        let sound = Decoder::new(Cursor::new(include_bytes!("Capture.ogg").as_slice()))
            .unwrap()
            .amplify(0.2)
            .convert_samples();

        self.output_stream_handle.play_raw(sound).unwrap();
    }

    pub fn play_move(&self) {
        let sound = Decoder::new(Cursor::new(include_bytes!("Move.ogg").as_slice()))
            .unwrap()
            .amplify(0.2)
            .convert_samples();

        self.output_stream_handle.play_raw(sound).unwrap();
    }
}
