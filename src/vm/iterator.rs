use crate::object::{
    hashing_collection::{new_y_map_with_capacity, new_y_set_with_capacity, YsetlMap, YsetlSet},
    object::{FrameOps, Object},
};

#[derive(Debug, Clone)]
pub enum CollectionKind {
    ListLike(Vec<Object>),
    MapLike(Vec<(Object, Object)>),
}

impl CollectionKind {
    fn len(&self) -> usize {
        match self {
            Self::ListLike(v) => v.len(),
            Self::MapLike(v) => v.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SingleIterator {
    pub collection: CollectionKind,
    pub pos: usize,
    pub size: usize,
}

impl SingleIterator {
    pub fn new(obj: &Object) -> Self {
        let collection = obj.to_collection();
        Self {
            size: collection.len(),
            collection,
            pos: 0,
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
        match &self.collection {
            CollectionKind::ListLike(v) => v[self.pos].clone(),
            CollectionKind::MapLike(v) => v[self.pos].1.clone(),
        }
    }

    pub fn current_key(&self) -> Object {
        match &self.collection {
            CollectionKind::ListLike(_) => Object::Int(self.pos as i64),
            CollectionKind::MapLike(v) => v[self.pos].0.clone(),
        }
    }
}

// Should consider a special collector for Maps as well
#[derive(Debug)]
pub enum Collector {
    Tuple(Vec<Object>),
    Set(YsetlSet),
    Map(YsetlMap),
    Accum(Object),
}

impl Collector {
    pub fn new_tuple(capacity: usize) -> Self {
        Self::Tuple(Vec::with_capacity(capacity))
    }

    pub fn new_set(capacity: usize) -> Self {
        Self::Set(new_y_set_with_capacity(capacity))
    }

    pub fn new_map(capacity: usize) -> Self {
        Self::Map(new_y_map_with_capacity(capacity))
    }

    pub fn size(&self) -> usize {
        match self {
            Self::Tuple(vec) => vec.len(),
            Self::Set(set) => set.len(),
            Self::Map(map) => map.len(),
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
            Self::Map(_) => panic!("Cannot push single value into map"),
        }
    }

    pub fn insert(&mut self, key: Object, value: Object) {
        match self {
            Self::Map(m) => m.insert(key, value),
            _ => panic!("Can only insert values into a map"),
        };
    }
}

#[derive(Debug)]
pub struct YsetlIter {
    pub pre_collections: Vec<Object>,
    pub iterators: Vec<SingleIterator>,
    pub output: Collector,
    pub reducer: Option<Object>,
}
