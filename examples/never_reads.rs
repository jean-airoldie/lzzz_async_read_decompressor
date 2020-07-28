use {
    bytes::Bytes,
    futures_lite::*,
    lazy_static::lazy_static,
    lzzzz::lz4f,
    piper::pipe,
    rand::{distributions::Standard, rngs::SmallRng, Rng, SeedableRng},
    smol::Task,
};

const FRAME_SIZE: usize = 1024;

lazy_static! {
    static ref FRAME: Bytes = {
        let rng = SmallRng::seed_from_u64(42060);
        rng.sample_iter(Standard).take(FRAME_SIZE).collect()
    };
}

fn main() {
    smol::run(async {
        // Choose a size that is smaller than the compressed stream.
        let (reader, writer) = pipe(FRAME_SIZE / 4);

        let task = Task::spawn(async move {
            let mut reader = lz4f::AsyncReadDecompressor::new(reader).unwrap();
            let mut total = vec![];
            let mut buf = vec![0u8; FRAME_SIZE / 4];

            loop {
                let bytes_read = reader.read(&mut buf).await.unwrap();
                if bytes_read == 0 {
                    break;
                }
                println!("read {} bytes", bytes_read);
                total.extend_from_slice(&buf[..bytes_read]);
            }
            assert_eq!(total, *FRAME);
        });

        let mut writer =
            lz4f::AsyncWriteCompressor::new(writer, lz4f::Preferences::default()).unwrap();

        println!("writing {} bytes", FRAME.len());

        writer.write_all(FRAME.as_ref()).await.unwrap();
        writer.flush().await.unwrap();

        // Signal EOF for the writer.
        drop(writer);

        println!("done writing");

        task.await;

        println!("done reading");
    })
}
