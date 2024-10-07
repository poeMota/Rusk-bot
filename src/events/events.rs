use once_cell::sync::Lazy;
pub use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub static EVENTMANAGER: Lazy<Arc<Mutex<EventManager>>> =
    Lazy::new(|| Arc::new(Mutex::new(EventManager::new())));

pub trait Event: Any + Send + Sync {
    fn raise(self);
    fn as_any(&self) -> &dyn Any;
}

pub struct EventManager {
    subscribers: HashMap<TypeId, Vec<Box<dyn Fn(&dyn Event) + Send + Sync>>>,
}

impl EventManager {
    fn new() -> Self {
        EventManager {
            subscribers: HashMap::new(),
        }
    }

    pub fn register<E: Event>(&mut self) {
        let type_id = TypeId::of::<E>();
        self.subscribers.entry(type_id).or_insert_with(Vec::new);
    }

    pub fn subscribe<E: Event>(&mut self, subscriber: Box<dyn Fn(&E) + Send + Sync>) {
        let type_id = TypeId::of::<E>();
        let callback = Box::new(move |event: &dyn Event| {
            if let Some(event) = event.as_any().downcast_ref::<E>() {
                subscriber(event);
            }
        }) as Box<dyn Fn(&dyn Event) + Send + Sync>;

        self.subscribers
            .entry(type_id)
            .or_insert_with(Vec::new)
            .push(callback);
    }

    pub fn raise_event<E: Event>(&self, event: E) {
        let type_id = TypeId::of::<E>();
        if let Some(subscribers) = self.subscribers.get(&type_id) {
            for subscriber in subscribers {
                subscriber(&event);
            }
        }
    }
}

pub fn subscribe_event<E: Event>(subscriber: Box<dyn Fn(&E) + Send + Sync>) {
    let mut ev_man = EVENTMANAGER.lock().unwrap();
    ev_man.subscribe::<E>(subscriber);
}

pub fn register_event<E: Event>() {
    let mut ev_man = EVENTMANAGER.lock().unwrap();
    ev_man.register::<E>();
}
