use std::{
    collections::{HashMap, VecDeque},
    ops::DerefMut,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

use futures::Stream;

#[derive(Debug)]
pub enum SendError<T> {
    ReceiverDropped(T),
}

pub struct Inner<T> {
    /// The buffer containing the messages.
    buffer: VecDeque<T>,
    /// Tracks the number of messages that have been processed and cleared so far.
    clear_count: usize,
    /// The number of created `Sender`s that are not yet dropped
    txs_left: usize,
    /// The metadatas of created `Receiver`s that are not yet dropped. Hash key is id for `Receiver`.
    rxs_metas: HashMap<ReceiverID, ReceiverMeta>,
}

struct ReceiverMeta {
    /// The absolute index of next message across all historical messages:
    /// `self.next_index_abs` - `inner.clear_count` = `index in inner.buffer of next message to be read`
    next_index_abs: usize,
    /// The waker used to wake the Receiver `Future`
    waker: Option<Waker>,
}

type ReceiverID = usize;

pub struct Receiver<T> {
    inner: Arc<Mutex<Inner<T>>>,
    /// Used to track corresponding receiver metadata.
    id: ReceiverID,
}

impl<T: Clone> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut inner = self.inner.lock().unwrap();
        let Inner {
            buffer,
            clear_count: processed_count,
            txs_left,
            rxs_metas,
        } = inner.deref_mut();

        let meta = rxs_metas.get_mut(&self.id).unwrap();
        let next_index_rel = meta.next_index_abs - *processed_count;
        match buffer.get(next_index_rel) {
            Some(v) => {
                meta.next_index_abs += 1;
                Poll::Ready(Some(v.clone()))
            }
            None => {
                if *txs_left == 0 {
                    Poll::Ready(None)
                } else {
                    meta.waker = Some(cx.waker().clone());
                    Poll::Pending
                }
            }
        }
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        let mut inner = self.inner.lock().unwrap();
        let Inner {
            rxs_metas,
            clear_count: processed_count,
            ..
        } = inner.deref_mut();
        let id = rxs_metas.len();
        rxs_metas.insert(
            id,
            ReceiverMeta {
                next_index_abs: *processed_count,
                waker: None,
            },
        );
        Self {
            inner: self.inner.clone(),
            id,
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        inner.rxs_metas.remove(&self.id);
    }
}

pub struct Sender<T> {
    inner: Arc<Mutex<Inner<T>>>,
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        let mut inner = self.inner.lock().unwrap();
        let Inner {
            buffer,
            clear_count: processed_count,
            rxs_metas,
            ..
        } = inner.deref_mut();

        if rxs_metas.len() == 0 {
            return Err(SendError::ReceiverDropped(value));
        }
        // The minimum number of messages (in current buffer) already processed by all receivers
        let min_rx_processed = rxs_metas
            .iter()
            .map(|(_, rx_meta)| rx_meta.next_index_abs - *processed_count)
            .min()
            .unwrap_or_default();
        buffer.drain(..min_rx_processed);
        *processed_count += min_rx_processed;
        buffer.push_back(value);
        for (_, rx_meta) in rxs_metas {
            if let Some(waker) = rx_meta.waker.take() {
                waker.wake();
            }
        }
        Ok(())
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        let mut inner = self.inner.lock().unwrap();
        inner.txs_left += 1;
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        inner.txs_left -= 1;
        for (_, rx_meta) in &mut inner.rxs_metas {
            if let Some(waker) = rx_meta.waker.take() {
                waker.wake();
            }
        }
    }
}

/// Create a new broadcast channel
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Inner {
        buffer: VecDeque::new(),
        clear_count: 0,
        txs_left: 1,
        rxs_metas: HashMap::from([(
            0,
            ReceiverMeta {
                next_index_abs: 0,
                waker: None,
            },
        )]),
    };
    let inner = Arc::new(Mutex::new(inner));
    let tx = Sender {
        inner: inner.clone(),
    };
    let rx = Receiver { inner, id: 0 };
    (tx, rx)
}

#[cfg(test)]
mod tests {
    use futures::{future::join_all, StreamExt};
    use tokio::task;

    use super::channel;

    #[tokio::test]
    async fn test_send_recv() {
        let (tx, rx) = channel();

        let n = 5;
        let tx_num = 3;
        let rx_num = 2;

        for i in 0..tx_num {
            task::spawn({
                let tx = tx.clone();
                async move {
                    for j in 0..n {
                        let v = i * n + j;
                        tx.send(v).unwrap();
                    }
                }
            });
        }
        drop(tx);

        let rxs: Vec<_> = (0..rx_num)
            .map(|_| {
                task::spawn({
                    let mut rx = rx.clone();
                    async move {
                        let mut msgs = vec![];
                        while let Some(msg) = rx.next().await {
                            msgs.push(msg);
                        }
                        msgs
                    }
                })
            })
            .collect();

        for msgs in join_all(rxs).await {
            let msgs = msgs.unwrap();
            let expect: Vec<_> = (0..tx_num)
                .flat_map(|i| (0..n).map(move |j| i * n + j))
                .collect();
            assert_eq!(msgs, expect);
        }
    }
}
