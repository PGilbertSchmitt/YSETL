use std::collections::HashSet;

use bytes::Bytes;

use crate::object::object::{BaseObject, IterKind, Object, ObjectOps};

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
        self.collection.get(self.pos).unwrap().clone()
    }
}

#[derive(Debug)]
pub enum Collector {
    Tuple(Vec<Object>),
    Set(HashSet<Object>),
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
        }
    }

    pub fn push(&mut self, obj: Object) {
        match self {
            Self::Tuple(v) => v.push(obj),
            Self::Set(_s) => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct YsetlIter {
    collections: Vec<IterCollection>,
    output: Collector,
}

#[derive(Debug)]
pub struct Frame {
    pub ins: Bytes,
    pub return_ptr: u64,
    pub stack_base: usize,
    pub closed_values: Vec<Object>,
    iterator: Option<YsetlIter>,
}

impl Frame {
    pub fn new_as_func(
        ins: Bytes,
        return_ptr: u64,
        stack_base: usize,
        closed_values: Vec<Object>,
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
        return_ptr: u64,
        stack_base: usize,
        closed_values: Vec<Object>,
        as_tuple: bool,
    ) -> Self {
        Self {
            ins,
            return_ptr,
            stack_base,
            closed_values,
            iterator: Some(YsetlIter {
                collections: Vec::new(),
                output: if as_tuple {
                    Collector::new_tuple()
                } else {
                    Collector::new_set()
                },
            }),
        }
    }

    pub fn make_iter(&mut self, obj: &Object) {
        self.iterator_mut()
            .collections
            .push(IterCollection::new(obj))
    }

    pub fn dup_iter(&mut self) {
        let collections = &mut self.iterator_mut().collections;
        collections.push(collections.last().unwrap().clone());
    }

    pub fn iter_next(&mut self, iter_idx: usize, mut on_continue: impl FnMut()) {
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
        } else {
            on_continue();
        }
    }

    pub fn get_iter_val(&self, iter_idx: usize) -> Object {
        self.iterator_at(iter_idx).current_value()
    }

    // Iterator keys are based on the type of collection
    pub fn get_iter_key(&self, iter_idx: usize) -> Object {
        let current_iter = self.iterator_at(iter_idx);
        match current_iter.kind {
            IterKind::Tuple | IterKind::String => current_iter.current_value(),
            IterKind::Set => BaseObject::Int(current_iter.pos as i64).wrap(),
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

    pub fn collector(self) -> Object {
        match self.iterator.unwrap().output {
            Collector::Tuple(v) => BaseObject::Tuple(v).wrap(),
            _ => todo!(),
        }
    }

    pub fn print_info(&self) {
        println!(
            "[ bytes: {}, return_to: {}, stack_base: {} ]",
            self.ins.len(),
            self.return_ptr,
            self.stack_base,
        );

        // println!("Closed Over: {{");
        // self.closed_values.iter().for_each(|v| {
        //     println!("\t{}", v.to_debug_string());
        // });
        // println!("}}");

        let iter = self.iterator.as_ref().unwrap();
        println!("Iterators: {{");
        println!("\tOutput: {:?}", iter.output);
        iter.collections.iter().for_each(|c| {
            println!("\t[{}/{}] {:?}", c.pos, c.size, c.collection);
        });
        println!("}}");
    }

    fn iterator(&self) -> &YsetlIter {
        self.iterator.as_ref().expect("Frame is missing")
    }

    fn iterator_mut(&mut self) -> &mut YsetlIter {
        self.iterator.as_mut().expect("Frame is missing")
    }

    fn iterator_at(&self, idx: usize) -> &IterCollection {
        self.iterator()
            .collections
            .get(idx)
            .expect("Went past collection bound")
    }

    fn iterator_at_mut(&mut self, idx: usize) -> &mut IterCollection {
        self.iterator_mut()
            .collections
            .get_mut(idx)
            .expect("Went past collection bound")
    }
}
