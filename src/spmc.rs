#[cfg(test)]
mod tests;

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

struct Channel<T: Send> {
    queue: Mutex<(VecDeque<T>, bool)>,
    queue_not_empty: Condvar,
}

pub fn channel<T: Send>() -> (Sender<T>, Receiver<T>) {
    let queue = Mutex::new((VecDeque::new(), false));
    let queue_not_empty = Condvar::new();

    let channel = Arc::new(Channel {
        queue,
        queue_not_empty,
    });

    (Sender(Arc::clone(&channel)), Receiver(channel))
}

pub struct Sender<T: Send>(Arc<Channel<T>>);

impl<T: Send> Sender<T> {
    pub fn send(&self, t: T) {
        let mut queue = self.0.queue.lock().unwrap();
        queue.0.push_back(t);
        self.0.queue_not_empty.notify_all();
    }

    pub fn queued(&self) -> usize {
        let queue = self.0.queue.lock().unwrap();

        queue.0.len()
    }
}

impl<T: Send> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut queue = self.0.queue.lock().unwrap();
        queue.1 = true;
    }
}

pub struct Receiver<T: Send>(Arc<Channel<T>>);

#[derive(Debug, PartialEq)]
pub enum RecvError {
    ChannelClosed,
}

#[derive(Debug, PartialEq)]
pub enum TryRecvError {
    ChannelClosed,
    NoValue,
}

type RecvResult<T> = Result<T, RecvError>;

type TryRecvResult<T> = Result<T, TryRecvError>;

impl<T: Send> Receiver<T> {
    pub fn recv(&self) -> RecvResult<T> {
        let mut queue = self.0.queue.lock().unwrap();

        Ok(loop {
            match queue.0.pop_front() {
                None => {
                    if queue.1 {
                        return Err(self::RecvError::ChannelClosed);
                    } else {
                        queue = self.0.queue_not_empty.wait(queue).unwrap();
                    }
                }
                Some(elem) => break elem,
            }
        })
    }

    pub fn try_recv(&self) -> TryRecvResult<T> {
        let mut queue = self.0.queue.lock().unwrap();
        match queue.0.pop_front() {
            None => {
                if queue.1 {
                    Err(self::TryRecvError::ChannelClosed)
                } else {
                    Err(self::TryRecvError::NoValue)
                }
            },
            Some(elem) => Ok(elem)
        }
    }

    pub fn queued(&self) -> usize {
        let queue = self.0.queue.lock().unwrap();

        queue.0.len()
    }
}

pub struct ReceiveIterator<T: Send> {
    receiver: Receiver<T>,
}

impl<T: Send> IntoIterator for Receiver<T> {
    type Item = T;
    type IntoIter = ReceiveIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        ReceiveIterator { receiver: self }
    }
}

impl<T: Send> Iterator for ReceiveIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        match self.receiver.recv() {
            Ok(elem) => Some(elem),
            Err(self::RecvError::ChannelClosed) => None,
            _ => panic!("Unknown error occured trying to receive message"),
        }
    }
}

impl<T: Send> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Receiver(Arc::clone(&self.0))
    }
}
