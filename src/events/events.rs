use once_cell::sync::Lazy;
pub use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub static EVENTMANAGER: Lazy<Arc<RwLock<EventManager>>> =
    Lazy::new(|| Arc::new(RwLock::new(EventManager::new())));

pub trait Event: Any + Send + Sync {
    fn raise(self);
    fn as_any(&self) -> &dyn Any;
}

pub struct EventManager {
    subscribers: HashMap<TypeId, Vec<Box<dyn Fn(&dyn Event) + Send + Sync>>>,
}

impl EventManager {
    fn new() -> Self {
        Self {
            subscribers: HashMap::new(),
        }
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

    pub fn subscribe_method<E, S>(&mut self, instance: Arc<&'static S>, method: fn(&S, &E))
    where
        E: Event,
        S: Send + Sync + 'static,
    {
        let type_id = TypeId::of::<E>();
        let callback = Box::new(move |event: &dyn Event| {
            if let Some(event) = event.as_any().downcast_ref::<E>() {
                method(&instance, event);
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
            for subscriber in subscribers.iter() {
                subscriber(&event);
            }
        }
    }
}

pub fn subscribe_event<E: Event>(subscriber: Box<dyn Fn(&E) + Send + Sync>) {
    let mut ev_man = EVENTMANAGER.try_write().unwrap();
    ev_man.subscribe::<E>(subscriber);
}

pub fn subscribe_method<E, S>(instance: Arc<&'static S>, subscriber: fn(&S, &E))
where
    E: Event,
    S: Send + Sync + 'static,
{
    let mut ev_man = EVENTMANAGER.try_write().unwrap();
    ev_man.subscribe_method::<E, S>(instance, subscriber);
}
