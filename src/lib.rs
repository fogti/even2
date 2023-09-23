//! original source: https://github.com/willcrichton/types-over-strings/blob/master/src/event.rs

#![forbid(unsafe_code)]

use as_any::{AsAny, Downcast};
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub trait Event: AsAny {}

type OneOneED<E> = Box<dyn FnMut(&E) + 'static>;
type OneEventDispatcher<E> = Vec<OneOneED<E>>;

pub struct EventDispatcher(HashMap<TypeId, (fn(&mut dyn Any, &dyn Event), Box<dyn Any>)>);

fn dyn_delegate<E: Event>(listeners: &mut dyn Any, event: &dyn Event) {
    let listeners = listeners
        .downcast_mut::<OneEventDispatcher<E>>()
        .expect("dyn_delegate: mismatching listeners type");
    let event: &E = event
        .downcast_ref()
        .expect("dyn_delegate: mismatching event type");

    for callback in listeners {
        callback(event);
    }
}

impl EventDispatcher {
    #[inline]
    pub fn new() -> EventDispatcher {
        EventDispatcher(HashMap::new())
    }

    pub fn add_event_listener<E, F>(&mut self, f: F)
    where
        E: Event,
        F: FnMut(&E) + 'static,
    {
        (*self
            .0
            .entry(TypeId::of::<E>())
            .or_insert_with(|| (dyn_delegate::<E>, Box::new(Vec::<OneOneED<E>>::new())))
            .1)
            .downcast_mut::<OneEventDispatcher<E>>()
            .unwrap()
            .push(Box::new(f));
    }

    #[inline]
    pub fn clear<E: Event>(&mut self) {
        self.0.remove(&TypeId::of::<E>());
    }

    #[inline]
    pub fn clear_all(&mut self) {
        self.0.clear();
    }

    pub fn trigger<E: Event>(&mut self, event: &E) {
        if let Some(listeners) = self.0.get_mut(&TypeId::of::<E>()) {
            for callback in (*listeners.1).downcast_mut::<OneEventDispatcher<E>>().unwrap() {
                callback(event);
            }
        }
    }

    pub fn trigger_dyn(&mut self, event: &dyn Event) {
        if let Some(listeners) = self.0.get_mut(&event.type_id()) {
            let d2 = listeners.0;
            d2(&mut *listeners.1, event);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{cell::RefCell, rc::Rc};

    struct OnClick {
        mouse_x: i32,
        mouse_y: i32,
    }

    impl Event for OnClick {}

    #[test]
    fn basic() {
        let mut node = EventDispatcher::new();
        let x = Rc::new(RefCell::new(0));
        let x2 = Rc::clone(&x);
        node.add_event_listener(move |event: &OnClick| {
            *x2.borrow_mut() += 1;
            assert_eq!(event.mouse_x, 10);
            assert_eq!(event.mouse_y, 5);
        });
        let e = OnClick {
            mouse_x: 10,
            mouse_y: 5,
        };
        node.trigger(&e);
        node.trigger_dyn(&e);
        assert_eq!(*x.borrow(), 2);
    }
}
