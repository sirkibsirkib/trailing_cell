use super::*;
    
//For these tests, I use the example where my states T have type `Vec<u32>`

#[derive(Clone)]
// one of these enums represents any change I want to make to my Vector.
enum Change {
    Push(u32),
    Pop,
}

// Next, I implement `TakesMessage` such that the vector can interpret a new
// change object, and mutate itself accordingly.
impl TakesMessage<Change> for Vec<u32> {
    fn take_message(&mut self, t: &Change) {
        match t {
            &Change::Push(x) => { self.push(x); },
            &Change::Pop     => { self.pop();   },
        }
    }
}

#[test]
fn wrapping_unwrapping() {
    // the message buffer has a capacity of 10, when full, `apply_change`
    // calls wll block; `try_apply_change` calls will return Err.
    let w = TcWriter::new(10);
    // the reader starts with local [1,2,3] 
    let r = w.add_reader(vec![1,2,3]);
    let cmp : Vec<u32> = vec![1,2,3];
    // no messages are sent. So this state is as fresh as can be without any
    // synchronization calls.
    assert_eq!(&cmp, &*r);
}

#[test]
fn staleness() {
    let w = TcWriter::new(10);
    let mut r = w.add_reader(vec![1,2,3]);
    w.apply_change(Change::Pop);
    // r's state is [1,2,3]. It is stale!
    r.update();
    // r's state is [1,2]
    w.apply_change(Change::Pop);
    w.apply_change(Change::Pop);
    let cmp : Vec<u32> = vec![1,2];
    assert_eq!(cmp, r.into_inner());
    // r's state is still [1,2], very stale.
}

#[test]
fn w1_r1_multithreaded() {
    use std::time::Duration;
    use std::thread;

    let w = TcWriter::new(10);
    let mut r = w.add_reader(vec![]);
    let ten_millis = Duration::from_millis(10);
    let mut handles = vec![];
	// this demonstrates how to distribute readers over threads.
    for i in 0..5 {
        let w_clone = w.clone();
        handles.push(thread::spawn(move || {
            thread::sleep(ten_millis);
            w_clone.clone().apply_change(Change::Push(i));
            thread::sleep(ten_millis);
        }));
    }
    // r's state is still stale, regardless of what the writers are doing.
    assert_eq!(r.len(), 0);
    thread::sleep(ten_millis);
    // We can't statically make any guarantees here except that the number 
    // of Push messages that have arrives lies on the interval [0,5]
    r.update();
    assert!(r.len() <= 5);
    for h in handles {
        h.join().is_ok();
    }
    r.update();
    // Now we can be sure all 5 messages have arrived, but we don't know the
    // order of vector elements.
    assert_eq!(r.len(), 5);
}

#[test]
fn limited_sync() {
    use std::thread;
    // Here our message queue is capped at 16.
    let w1 = TcWriter::new(16);
    let mut r = w1.add_reader(vec![]);
    let w2 = w1.clone();
    // Two threads attempt to write a total of 32 writes
    let h1 = thread::spawn(move || {
        for i in 0..16 {
            let _ = w1.try_apply_change(Change::Push(i));
        }
    });
    let h2 = thread::spawn(move || {
        for i in 0..16 {
            let _ = w2.try_apply_change(Change::Push(i));
        }
    });
    h1.join().is_ok();
    h2.join().is_ok();
    // All write tries must be done now. Only 16 have succeeded.
    
    // The reader will try (and succeed) to read up to 5 messages.
    r.update_limited(5);
    assert_eq!(r.len(), 5);

    // The reader will synchronize and get the remaining messages
    r.update();
    assert_eq!(r.len(), 16);
}

#[test]
fn deref_mut() {
    let w1 = TcWriter::new(16);
    let mut r = w1.add_reader(vec![]);

    // DerefMut is implemented for TcReader. 
    (*r) = vec![1,2,3];
    w1.apply_change(Change::Pop);
    r.update();
    let cmp: Vec<u32> = vec![1,2];
    assert_eq!(cmp, r.into_inner());
}

#[test]
fn deref() {
    let w1 = TcWriter::new(16);
    let mut r = w1.add_reader(vec![]);
    w1.apply_change(Change::Push(1));
    {
        let b1 = &*r;
        // writer writes are independent of reader
        w1.apply_change(Change::Push(2));
        let b2 = &*r;
        let b3 = &*r;
        // r.update() here would NOT compile!
        w1.apply_change(Change::Push(3));

        // three immutable borrows of the TcWriter's T field are A-ok!
        println!("{:?} {:?} {:?}", b1, b2, b3);

        //b1,b2,b3 borrows dropped
    }
    // can mutate r now
    r.update();

    let mut cmp: Vec<u32> = vec![1,2,3];

    //can rely on Deref and Deref mut to use `*r`
    assert_eq!(&mut cmp, &mut *r);
    assert_eq!(&mut cmp, &*r);
    assert_eq!(&cmp, &mut *r);
    assert_eq!(&cmp, &*r);

    assert_eq!(cmp, *r);
}