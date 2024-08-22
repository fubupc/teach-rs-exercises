use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

use futures::Stream;

#[derive(Debug)]
pub enum SendError<T> {
    ReceiverDropped(T),
}

pub struct Inner<T> {
    /// The buffer containing the messages
    buffer: VecDeque<T>,
    /// The waker used to wake the Receiver `Future`
    waker: Option<Waker>,
    /// Indicates whether the `Receiver` was dropped
    rx_dropped: bool,
    /// The number of created `Sender`s that are not yet dropped
    txs_left: u32,
}

pub struct Receiver<T> {
    inner: Arc<Mutex<Inner<T>>>,
}

impl<T> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut inner = self.inner.lock().unwrap();

        match inner.buffer.pop_front() {
            Some(v) => Poll::Ready(Some(v)),
            None => {
                if inner.txs_left == 0 {
                    Poll::Ready(None)
                } else {
                    inner.waker = Some(cx.waker().clone());
                    Poll::Pending
                }
            }
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        inner.rx_dropped = true;
    }
}

pub struct Sender<T> {
    inner: Arc<Mutex<Inner<T>>>,
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        let mut inner = self.inner.lock().unwrap();
        if inner.rx_dropped {
            return Err(SendError::ReceiverDropped(value));
        }
        inner.buffer.push_back(value);
        if let Some(waker) = inner.waker.take() {
            waker.wake();
        }
        Ok(())
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        let inner = self.inner.clone();
        inner.lock().unwrap().txs_left += 1;
        Sender { inner }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        inner.txs_left -= 1;
        if let Some(waker) = inner.waker.take() {
            waker.wake();
        }
    }
}

/// Create a new mpsc channel
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Inner {
        buffer: VecDeque::new(),
        waker: None,
        rx_dropped: false,
        txs_left: 1,
    };
    let inner = Arc::new(Mutex::new(inner));
    let tx = Sender {
        inner: inner.clone(),
    };
    let rx = Receiver { inner };
    (tx, rx)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use futures::StreamExt;
    use tokio::task::{self};

    use crate::mpsc::{channel, SendError};

    #[tokio::test]
    async fn test_send_recv() {
        let (tx, mut rx) = channel();
        for i in 0..100 {
            tx.send(i).unwrap();
        }
        for i in 0..100 {
            assert_eq!(rx.next().await.unwrap(), i);
        }
    }

    #[tokio::test]
    async fn test_drop() {
        let (tx, mut rx) = channel::<()>();
        drop(tx);
        assert!(rx.next().await.is_none());

        let (tx, rx) = channel::<()>();
        drop(rx);
        assert!(matches!(tx.send(()), Err(SendError::ReceiverDropped(()))));
    }

    #[tokio::test]
    async fn test_multiple_tx() {
        let (tx, mut rx) = channel();

        for i in 0..10 {
            task::spawn({
                let tx = tx.clone();
                async move {
                    tx.send(i).unwrap();
                }
            });
        }
        drop(tx);
        let mut received_msgs = BTreeSet::new();
        while let Some(msg) = rx.next().await {
            received_msgs.insert(msg);
        }
        assert_eq!(received_msgs.len(), 10);

        received_msgs
            .into_iter()
            .enumerate()
            .for_each(|(i, msg)| assert_eq!(i, msg));
    }
}
