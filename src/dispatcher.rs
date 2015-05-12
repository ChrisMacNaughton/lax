use self::DispatchType::{ChangeCurrentChannel, OutgoingMessage, IncomingMessage};
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

#[derive(PartialEq, Debug, Clone)]
enum DispatchType {
    ChangeCurrentChannel,
    OutgoingMessage,
    IncomingMessage
}

#[derive(Clone)]
struct DispatchMessage {
   dispatch_type: DispatchType,
   payload: String
}

struct Dispatcher {
    // I heard you like types
    subscribers: HashMap<&'static str, Vec<mpsc::Sender<DispatchMessage>>>,
    broadcasters: Vec<Arc<Mutex<mpsc::Receiver<DispatchMessage>>>>
}

impl Dispatcher {
    pub fn new() -> Dispatcher {
        Dispatcher { subscribers: HashMap::new(), broadcasters: vec![] }
    }

    pub fn register_broadcaster(&mut self, broadcaster: &mut Broadcast) {
       let handle = Arc::new(Mutex::new(broadcaster.broadcast_handle()));
       self.broadcasters.push(handle);
    }

    pub fn register_subscriber(&mut self, subscriber: &Subscribe) {
       let sender = subscriber.subscribe();
       let type_key = type_to_str(&subscriber.what_subscribe());
       let new = match self.subscribers.get_mut(type_key) {
          Some(others) => {
             others.push(sender);
             None
          },
          None => {
             Some(vec![sender])
          }
       };
       // Improve me. Cant chain because double mut borrow not allowed
       new.and_then(|new_senders| self.subscribers.insert(type_key, new_senders));
    }

    pub fn start(&self) {
       for broadcaster in self.broadcasters.clone() {
          let subscribers = self.subscribers.clone();
          thread::spawn(move || {
             loop {
                let message = broadcaster.lock().unwrap().recv().ok().expect("Couldn't receive message in broadcaster or channel hung up");
                match subscribers.get(type_to_str(&message.dispatch_type)) {
                  Some(ref subs) => { 
                      for sub in subs.iter() { sub.send(message.clone()).unwrap(); }
                  },
                  None => ()
                }

             }
          });
       }
    }

    // fn shared_subscribers(&self) -> HashMap<&str, Arc<Mutex<mpsc::Sender<DispatchMessage>>>> {
    //    self.subscribers.iter().map(|v| Arc::new(Mutex::new(v.clone()))).collect()
    // }

    fn num_broadcasters(&self) -> usize {
       self.broadcasters.len()
    }

    fn num_subscribers(&self, dispatch_type: DispatchType) -> usize {
       match self.subscribers.get(type_to_str(&dispatch_type)) {
          Some(subscribers) => subscribers.len(),
          None => 0
       }
    }
}

// Convert to hashable for dispatchtype?
fn type_to_str(dispatch_type: &DispatchType) -> &'static str {
   match *dispatch_type {
       OutgoingMessage => "OutgoingMessage",
       ChangeCurrentChannel => "ChangeCurrentChannel",
       IncomingMessage => "IncomingMessage"
   }
}

trait Broadcast {
   fn broadcast(&self, dispatch_type: DispatchType, payload: String);
   fn broadcast_handle(&mut self) -> mpsc::Receiver<DispatchMessage>;
}

trait Subscribe {
   fn subscribe(&self) -> mpsc::Sender<DispatchMessage>;
   fn what_subscribe(&self) -> DispatchType;
}

#[cfg(test)]
mod test {
    use std::sync::mpsc;
    use super::{ Dispatcher, Broadcast, Subscribe, DispatchMessage};
    use super::DispatchType::{self, OutgoingMessage};

    #[test]
    fn test_register_broadcaster() {
        let mut dispatcher = Dispatcher::new();
        let mut brd = TestBroadcaster::new();
        assert_eq!(dispatcher.num_broadcasters(), 0);
        dispatcher.register_broadcaster(&mut brd);
        assert_eq!(dispatcher.num_broadcasters(), 1);
    }

    #[test]
    fn test_register_subscriber() {
        let mut dispatcher = Dispatcher::new();
        let sub = TestSubscriber::new();
        assert_eq!(dispatcher.num_subscribers(OutgoingMessage), 0);
        dispatcher.register_subscriber(&sub);
        assert_eq!(dispatcher.num_subscribers(OutgoingMessage), 1);
    }

    #[test]
    fn test_register_multiple_subscribers() {
        let mut dispatcher = Dispatcher::new();
        let sub = TestSubscriber::new();
        let sub2 = TestSubscriber::new();

        assert_eq!(dispatcher.num_subscribers(OutgoingMessage), 0);
        dispatcher.register_subscriber(&sub);
        dispatcher.register_subscriber(&sub2);
        assert_eq!(dispatcher.num_subscribers(OutgoingMessage), 2);
    }

    #[test]
    fn test_broadcast_simple_message() {
        let mut dispatcher = Dispatcher::new();
        let sub = TestSubscriber::new();
        let mut brd = TestBroadcaster::new();
        dispatcher.register_broadcaster(&mut brd);
        dispatcher.register_subscriber(&sub);

        dispatcher.start();

        brd.broadcast(OutgoingMessage, "Hello world!".to_string());
        let message = sub.receiver.recv().unwrap();
        assert_eq!(message.dispatch_type, OutgoingMessage);
        assert_eq!(message.payload, "Hello world!");
    }

    struct TestBroadcaster {
       sender: Option<mpsc::Sender<DispatchMessage>>
    }

    impl TestBroadcaster {
       fn new() -> TestBroadcaster {
         TestBroadcaster { sender: None }
      }
    }
    impl Broadcast for TestBroadcaster {
      fn broadcast_handle(&mut self) -> mpsc::Receiver<DispatchMessage> {
         let (tx, rx) = mpsc::channel::<DispatchMessage>();
         self.sender = Some(tx);
         rx
      }

      fn broadcast(&self, dispatch_type: DispatchType, payload: String) {
         let message = DispatchMessage { dispatch_type: dispatch_type, payload: payload };
         match self.sender {
            Some(ref s) => { s.send(message); },
            None => ()
         };
      }
    }

    struct TestSubscriber {
      receiver: mpsc::Receiver<DispatchMessage>,
      sender: mpsc::Sender<DispatchMessage>
    }

    impl TestSubscriber {
       fn new() -> TestSubscriber {
          let(tx, rx) = mpsc::channel::<DispatchMessage>();
          TestSubscriber { receiver: rx, sender: tx }
       }
    }

    impl Subscribe for TestSubscriber {
       fn subscribe(&self) -> mpsc::Sender<DispatchMessage> {
          self.sender.clone()
       }

       fn what_subscribe(&self) -> DispatchType {
          OutgoingMessage
       }
    }
}
