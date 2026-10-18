use pivot_transport::{read_frame, tokio, write_frame};
use tokio::io::AsyncWriteExt;

#[tokio::test]
async fn sequential_frames_and_clean_disconnect() {
    let (mut sender, mut receiver) = tokio::io::duplex(64);
    let task = tokio::spawn(async move {
        write_frame(&mut sender, b"one", 16).await.unwrap();
        write_frame(&mut sender, b"two", 16).await.unwrap();
    });
    assert_eq!(
        read_frame(&mut receiver, 16).await.unwrap(),
        Some(b"one".to_vec())
    );
    assert_eq!(
        read_frame(&mut receiver, 16).await.unwrap(),
        Some(b"two".to_vec())
    );
    task.await.unwrap();
    assert_eq!(read_frame(&mut receiver, 16).await.unwrap(), None);
}
#[tokio::test]
async fn oversized_zero_and_truncated_frames_are_rejected() {
    for bytes in [
        u32::MAX.to_le_bytes().to_vec(),
        0u32.to_le_bytes().to_vec(),
        vec![1, 2],
        vec![3, 0, 0, 0, 1],
    ] {
        let (mut sender, mut receiver) = tokio::io::duplex(64);
        sender.write_all(&bytes).await.unwrap();
        drop(sender);
        assert!(read_frame(&mut receiver, 16).await.is_err());
    }
    let (mut sender, _) = tokio::io::duplex(16);
    assert!(write_frame(&mut sender, b"", 16).await.is_err());
    assert!(write_frame(&mut sender, b"large", 2).await.is_err());
}
