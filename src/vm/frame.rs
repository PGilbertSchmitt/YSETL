use std::{collections::HashSet, rc::Rc};

use bytes::Bytes;

use crate::object::object::{FrameOps, IterKind, Object};

#[derive(Debug, Clone)]
struct IterCollection {
    collection: Vec<Object>,
    pos: usize,
    size: usize,
    kind: IterKind,
}

impl IterCollection {
    pub fn new(obj: &Object) -> Self {
        let collection = obj.to_vec();
        Self {
            size: collection.len(),
            collection: collection,
            pos: 0,
            kind: obj.iter_kind(),
        }
    }

    pub fn increment(&mut self) {
        self.pos += 1;
    }

    pub fn finished(&self) -> bool {
        self.pos >= self.size
    }

    pub fn reset(&mut self) {
        self.pos = 0;
    }

    pub fn current_value(&self) -> Object {
        self.collection[self.pos].clone()
    }
}

#[derive(Debug)]
pub enum Collector {
    Tuple(Vec<Object>),
    Set(HashSet<Object>),
    Accum(Object),
}

impl Collector {
    pub fn new_tuple() -> Self {
        Self::Tuple(Vec::new())
    }

    pub fn new_set() -> Self {
        Self::Set(HashSet::new())
    }

    pub fn size(&self) -> usize {
        match self {
            Self::Tuple(vec) => vec.len(),
            Self::Set(set) => set.len(),
            Self::Accum(_) => 1,
        }
    }

    pub fn push(&mut self, obj: Object) {
        match self {
            Self::Tuple(v) => v.push(obj),
            Self::Set(s) => {
                s.insert(obj);
            }
            Self::Accum(o) => *o = obj,
        }
    }
}

#[derive(Debug)]
pub struct YsetlIter {
    collections: Vec<IterCollection>,
    output: Collector,
    reducer: Option<Object>,
}

#[derive(Debug)]
pub struct Frame {
    pub ins: Bytes,
    pub return_ptr: usize,
    pub stack_base: usize,
    pub closed_values: Rc<Vec<Object>>,
    iterator: Option<YsetlIter>,
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
            iterator: None,
        }
    }

    pub fn new_as_iter(
        ins: Bytes,
        return_ptr: usize,
        stack_base: usize,
        closed_values: Rc<Vec<Object>>,
        collections: &Vec<Object>,
        collector: Collector,
        reducer: Option<Object>,
    ) -> Self {
        Self {
            ins,
            return_ptr,
            stack_base,
            closed_values,
            iterator: Some(YsetlIter {
                collections: collections
                    .into_iter()
                    .map(|c| IterCollection::new(c))
                    .collect(),
                output: collector,
                reducer,
            }),
        }
    }

    pub fn iter_next(&mut self, iter_idx: usize) -> bool {
        let sub_iter = self.iterator_at_mut(iter_idx);
        sub_iter.increment();
        if sub_iter.finished() {
            // When an iterator is finished, we reset its state and the state of all iterators
            // above it. If the iter_idx is 0, then we don't even need to reset, because we
            // would be done with all iteration and will be hitting the end
            if iter_idx > 0 {
                for iter in self.iterator_mut().collections[iter_idx..].iter_mut() {
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
        let current_iter = self.iterator_at(iter_idx);
        match current_iter.kind {
            IterKind::Tuple | IterKind::String => Object::Int(current_iter.pos as i64),
            IterKind::Set => current_iter.current_value(),
        }
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

    pub fn any_iter_empty(&self) -> bool {
        self.iterator()
            .collections
            .iter()
            .any(|collection| collection.size == 0)
    }

    pub fn into_collector(self) -> Object {
        match self.iterator.unwrap().output {
            Collector::Tuple(v) => Object::new_tuple(v),
            Collector::Set(s) => Object::new_set(s),
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

        let iter = self.iterator.as_ref().unwrap();
        println!("Iterators: {{");
        println!("\tOutput: {:?}", iter.output);
        iter.collections.iter().for_each(|c| {
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
        self.iterator.as_ref().expect("Frame is missing")
    }

    fn iterator_mut(&mut self) -> &mut YsetlIter {
        self.iterator.as_mut().expect("Frame is missing")
    }

    fn iterator_at(&self, idx: usize) -> &IterCollection {
        &self.iterator().collections[idx]
    }

    fn iterator_at_mut(&mut self, idx: usize) -> &mut IterCollection {
        &mut self.iterator_mut().collections[idx]
    }
}
