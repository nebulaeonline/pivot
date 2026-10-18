use pivot_transport::{FrameTimeouts, read_frame_with_timeouts, write_frame_with_timeouts};
use pivot_transport::{read_frame, tokio, write_frame};
use std::time::Duration;
use tokio::io::AsyncWriteExt;

#[tokio::test(start_paused = true)]
async fn long_idle_does_not_consume_partial_frame_deadline() {
    let policy = FrameTimeouts {
        idle: Duration::from_secs(30),
        partial_read: Duration::from_secs(2),
        write: Duration::from_secs(1),
    };
    let (mut sender, mut receiver) = tokio::io::duplex(64);
    let task =
        tokio::spawn(async move { read_frame_with_timeouts(&mut receiver, 16, policy).await });
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(10)).await;
    assert!(!task.is_finished());
    write_frame(&mut sender, b"ok", 16).await.unwrap();
    assert_eq!(task.await.unwrap().unwrap(), Some(b"ok".to_vec()));
}

#[tokio::test(start_paused = true)]
async fn idle_and_partial_frame_deadlines_are_separate() {
    let policy = FrameTimeouts {
        idle: Duration::from_secs(30),
        partial_read: Duration::from_secs(2),
        write: Duration::from_secs(1),
    };
    let (_sender, mut receiver) = tokio::io::duplex(64);
    let task =
        tokio::spawn(async move { read_frame_with_timeouts(&mut receiver, 16, policy).await });
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(31)).await;
    assert!(
        task.await
            .unwrap()
            .unwrap_err()
            .to_string()
            .contains("idle timeout")
    );
    for header in [vec![3], vec![3, 0, 0, 0, 1]] {
        let (mut sender, mut receiver) = tokio::io::duplex(64);
        sender.write_all(&header).await.unwrap();
        let task =
            tokio::spawn(async move { read_frame_with_timeouts(&mut receiver, 16, policy).await });
        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_secs(3)).await;
        assert!(
            task.await
                .unwrap()
                .unwrap_err()
                .to_string()
                .contains("partial frame")
        );
    }
}

#[tokio::test(start_paused = true)]
async fn blocked_writes_have_their_own_deadline() {
    let policy = FrameTimeouts {
        write: Duration::from_secs(1),
        ..Default::default()
    };
    let (mut sender, _receiver) = tokio::io::duplex(1);
    let task = tokio::spawn(async move {
        write_frame_with_timeouts(&mut sender, b"blocked", 16, policy).await
    });
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(2)).await;
    assert!(
        task.await
            .unwrap()
            .unwrap_err()
            .to_string()
            .contains("write timeout")
    );
}

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
