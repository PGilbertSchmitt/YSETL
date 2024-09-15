use std::rc::Rc;

use bytes::Bytes;

use crate::object::object::Object;

use super::iterator::{Collector, SingleIterator, YsetlIter};

#[derive(Debug)]
pub struct Frame {
    pub ins: Bytes,
    pub return_ptr: usize,
    pub stack_base: usize,
    pub closed_values: Rc<Vec<Object>>,
    iterator_state: Option<YsetlIter>,
}

impl Frame {
    pub fn new_as_func(
        ins: Bytes,
        return_ptr: usize,
        stack_base: usize,
        closed_values: Rc<Vec<Object>>,
    ) -> Self {
        Self {
            ins,
            return_ptr,
            stack_base,
            closed_values,
            iterator_state: None,
        }
    }

    pub fn new_as_iter(
        ins: Bytes,
        return_ptr: usize,
        stack_base: usize,
        closed_values: Rc<Vec<Object>>,
        collections: Vec<Object>,
        collector: Collector,
        reducer: Option<Object>,
    ) -> Self {
        Self {
            ins,
            return_ptr,
            stack_base,
            closed_values,
            iterator_state: Some(YsetlIter {
                pre_collections: collections,
                iterators: Vec::new(),
                output: collector,
                reducer,
            }),
        }
    }

    pub fn make_iter(&mut self, coll_idx: usize) {
        let iterator = self.iterator_mut();
        let collection = &iterator.pre_collections[coll_idx];
        iterator.iterators.push(SingleIterator::new(&collection));
    }

    pub fn iter_next(&mut self, iter_idx: usize) -> bool {
        let sub_iter = self.iterator_at_mut(iter_idx);
        sub_iter.increment();
        if sub_iter.finished() {
            // When an iterator is finished, we reset its state and the state of all iterators
            // above it. If the iter_idx is 0, then we don't even need to reset, because we
            // would be done with all iteration and will be hitting the end
            if iter_idx > 0 {
                for iter in self.iterator_mut().iterators[iter_idx..].iter_mut() {
                    iter.reset();
                }
            }
            false
        } else {
            true
        }
    }

    pub fn get_iter_val(&self, iter_idx: usize) -> Object {
        self.iterator_at(iter_idx).current_value()
    }

    // Iterator keys are based on the type of collection
    pub fn get_iter_key(&self, iter_idx: usize) -> Object {
        self.iterator_at(iter_idx).current_key()
    }

    pub fn get_collection(&self) -> Object {
        match &self.iterator().output {
            Collector::Accum(o) => o.clone(),
            _ => unreachable!(),
        }
    }

    pub fn iter_collect(&mut self, obj: Object) {
        self.iterator_mut().output.push(obj);
    }

    pub fn iter_collect_kv(&mut self, key: Object, value: Object) {
        self.iterator_mut().output.insert(key, value);
    }

    pub fn any_iter_empty(&self) -> bool {
        self.iterator()
            .iterators
            .iter()
            .any(|collection| collection.size == 0)
    }

    pub fn into_collector(self) -> Object {
        match self.iterator_state.unwrap().output {
            Collector::Tuple(v) => Object::new_tuple(v),
            Collector::Set(s) => Object::new_set(s),
            Collector::Map(m) => Object::new_map(m),
            Collector::Accum(o) => o,
        }
    }

    pub fn print_info(&self) {
        println!(
            "[ bytes: {}, return_to: {}, stack_base: {} ]",
            self.ins.len(),
            self.return_ptr,
            self.stack_base,
        );

        let iter = self.iterator_state.as_ref().unwrap();
        println!("Iterators: {{");
        println!("\tOutput: {:?}", iter.output);
        iter.iterators.iter().for_each(|c| {
            println!("\t[{}/{}] {:?}", c.pos, c.size, c.collection);
        });
        println!("}}");
    }

    pub fn get_reducer(&self) -> Object {
        self.iterator()
            .reducer
            .as_ref()
            .expect("Tried to reduce without a reducer present")
            .clone()
    }

    fn iterator(&self) -> &YsetlIter {
        self.iterator_state.as_ref().expect("Frame is missing")
    }

    fn iterator_mut(&mut self) -> &mut YsetlIter {
        self.iterator_state.as_mut().expect("Frame is missing")
    }

    fn iterator_at(&self, idx: usize) -> &SingleIterator {
        &self.iterator().iterators[idx]
    }

    fn iterator_at_mut(&mut self, idx: usize) -> &mut SingleIterator {
        &mut self.iterator_mut().iterators[idx]
    }
}
