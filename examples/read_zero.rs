use {
    bytes::Bytes, futures_lite::*, lazy_static::lazy_static, lzzzz::lz4f, piper::pipe, smol::Task,
};

lazy_static! {
    static ref FRAME: Bytes = vec![1u8; 8192].into();
}

fn main() {
    smol::run(async {
        let (reader, writer) = pipe(53);

        let task = Task::spawn(async move {
            let mut reader = lz4f::AsyncReadDecompressor::new(reader).unwrap();
            let mut buf = vec![];
            reader.read_to_end(&mut buf).await.unwrap();
            assert_eq!(buf, *FRAME);
        });

        let mut writer =
            lz4f::AsyncWriteCompressor::new(writer, lz4f::Preferences::default()).unwrap();
        writer.write_all(FRAME.as_ref()).await.unwrap();
        writer.flush().await.unwrap();

        // Signal EOF for the writer.
        drop(writer);

        task.await;
    })
}
