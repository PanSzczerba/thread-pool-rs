use super::*;

#[test]
fn simple_channel() {
    let (tx, rx) = channel();

    tx.send(1);

    assert_eq!(rx.recv().unwrap(), 1);
}

#[test]
fn dual_channel() {
    let (tx, rx1) = channel();

    let rx2 = rx1.clone();

    tx.send(1);
    tx.send(2);

    assert_eq!(rx1.recv().unwrap(), 1);
    assert_eq!(rx2.recv().unwrap(), 2);
}

#[test]
fn dual_channel_one_receives() {
    let (tx, rx1) = channel();

    let rx2 = rx1.clone();

    tx.send(1);
    tx.send(2);

    assert_eq!(rx1.try_recv().unwrap(), 1);
    assert_eq!(rx1.try_recv().unwrap(), 2);

    assert_eq!(rx2.try_recv().unwrap_err(), TryRecvError::NoValue);
    assert_eq!(rx1.try_recv().unwrap_err(), TryRecvError::NoValue);
}

#[test]
fn receiver_iter() {
    let (tx, rx) = channel();

    tx.send(1);

    drop(tx);
    for i in rx {
        assert_eq!(i, 1);
    }
}

#[test]
fn queue_depth() {
    let (tx, rx1) = channel();

    let rx2 = rx1.clone();

    tx.send(1);
    tx.send(2);

    assert_eq!(tx.queued(), 2);
    assert_eq!(rx1.queued(), 2);
    assert_eq!(rx2.queued(), 2);
}
