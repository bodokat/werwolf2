use std::{
    borrow::Borrow,
    collections::HashMap,
    hash::{BuildHasher, Hash},
};

pub trait MapExt<K, Q, V, S>
where
    K: Eq + Hash,
    K: Borrow<Q>,
    Q: Hash + Eq,
    S: BuildHasher,
{
    fn swap(&mut self, x: &Q, y: &Q) -> bool;
}

impl<K, Q, V, S> MapExt<K, Q, V, S> for HashMap<K, V, S>
where
    K: Eq + Hash,
    K: Borrow<Q>,
    Q: Hash + Eq,
    S: BuildHasher,
{
    fn swap(&mut self, x: &Q, y: &Q) -> bool {
        let a = match self.get_mut(&x) {
            Some(a) => a,
            None => return false,
        } as *mut V;
        let b = match self.get_mut(&y) {
            Some(b) => b,
            None => return false,
        } as *mut V;
        // SAFETY: the only reason why we can't call std::mem::swap is that we would have to borrow self mutably twice
        unsafe {
            std::ptr::swap(a, b);
        }
        true
    }
}
