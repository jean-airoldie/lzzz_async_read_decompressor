use {futures_lite::*, lzzzz::lz4f, piper::pipe, smol::Task};

const FRAME_SIZE: usize = 8192;

fn main() {
    smol::run(async {
        let (reader, writer) = pipe(53);

        let frame = vec![1u8; FRAME_SIZE];

        let frame2 = frame.clone();
        let task = Task::spawn(async move {
            let mut reader = lz4f::AsyncReadDecompressor::new(reader).unwrap();
            let mut buf = vec![];
            reader.read_to_end(&mut buf).await.unwrap();
            assert_eq!(buf, frame2);
        });

        let mut writer =
            lz4f::AsyncWriteCompressor::new(writer, lz4f::Preferences::default()).unwrap();
        writer.write_all(frame.as_ref()).await.unwrap();
        writer.flush().await.unwrap();

        // Signal EOF for the writer.
        drop(writer);

        task.await;
    })
}
