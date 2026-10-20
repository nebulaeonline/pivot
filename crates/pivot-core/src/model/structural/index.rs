//! Deterministically balanced immutable AVL index; local edits copy O(log N) nodes.
use super::memory::{Admission, Footprint, Memory, Shared};
use std::borrow::Borrow;

type Link<K, V> = Option<Shared<Node<K, V>>>;
struct Node<K, V> {
    key: K,
    value: V,
    left: Link<K, V>,
    right: Link<K, V>,
    height: u8,
    len: usize,
}
pub trait Key: Ord + Clone {
    fn heap_bytes(&self) -> usize {
        0
    }
}
impl Key for super::NodeId {}
impl Key for String {
    fn heap_bytes(&self) -> usize {
        self.capacity()
    }
}
impl<K: Key, V: Footprint> Footprint for Node<K, V> {
    fn heap_bytes(&self) -> usize {
        self.key.heap_bytes() + self.value.heap_bytes()
    }
}
pub struct Index<K, V> {
    root: Link<K, V>,
    memory: Memory,
}
impl<K, V> Clone for Index<K, V> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
            memory: self.memory.clone(),
        }
    }
}
impl<K: Key, V: Clone + Footprint> Index<K, V> {
    pub fn new(memory: Memory) -> Self {
        Self { root: None, memory }
    }
    pub fn len(&self) -> usize {
        len(&self.root)
    }
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }
    pub fn get<Q: Ord + ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        let mut next = self.root.as_ref();
        while let Some(n) = next {
            match key.cmp(n.key.borrow()) {
                std::cmp::Ordering::Less => next = n.left.as_ref(),
                std::cmp::Ordering::Greater => next = n.right.as_ref(),
                std::cmp::Ordering::Equal => return Some(&n.value),
            }
        }
        None
    }
    pub fn contains_key<Q: Ord + ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
    {
        self.get(key).is_some()
    }
    pub fn insert(&mut self, key: K, value: V) -> Result<(), Admission> {
        self.root = put(&self.memory, &self.root, key, value)?;
        Ok(())
    }
    /// Internal private-candidate insertion. A failure requires discarding the candidate.
    /// Published/shared nodes remain immutable. Inline payloads can reuse unique nodes;
    /// heap-owning payloads retain the ordinary charged functional insertion.
    #[doc(hidden)]
    pub fn insert_private(&mut self, key: K, value: V) -> Result<(), Admission> {
        if key.heap_bytes() != 0 || value.heap_bytes() != 0 {
            return self.insert(key, value);
        }
        put_private(&self.memory, &mut self.root, key, value)
    }
    pub fn remove<Q: Ord + ?Sized>(&mut self, key: &Q) -> Result<(), Admission>
    where
        K: Borrow<Q>,
    {
        self.root = remove(&self.memory, &self.root, key)?;
        Ok(())
    }
    pub fn iter_from<Q: Ord + ?Sized>(&self, key: &Q) -> impl Iterator<Item = (&K, &V)>
    where
        K: Borrow<Q>,
    {
        let mut stack = Vec::new();
        let mut cursor = self.root.as_ref();
        while let Some(node) = cursor {
            if node.key.borrow() >= key {
                stack.push(node);
                cursor = node.left.as_ref();
            } else {
                cursor = node.right.as_ref();
            }
        }
        let mut next = None;
        std::iter::from_fn(move || {
            while let Some(node) = next {
                stack.push(node);
                next = node.left.as_ref();
            }
            let node = stack.pop()?;
            next = node.right.as_ref();
            Some((&node.key, &node.value))
        })
    }
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        let mut stack = Vec::new();
        let mut next = self.root.as_ref();
        std::iter::from_fn(move || {
            while let Some(n) = next {
                stack.push(n);
                next = n.left.as_ref();
            }
            let n = stack.pop()?;
            next = n.right.as_ref();
            Some((&n.key, &n.value))
        })
    }
}
fn height<K, V>(n: &Link<K, V>) -> u8 {
    n.as_ref().map_or(0, |n| n.height)
}
fn len<K, V>(n: &Link<K, V>) -> usize {
    n.as_ref().map_or(0, |n| n.len)
}
fn node<K: Key, V: Clone + Footprint>(
    m: &Memory,
    key: K,
    value: V,
    left: Link<K, V>,
    right: Link<K, V>,
) -> Result<Shared<Node<K, V>>, Admission> {
    crate::experiment_profile::copy(std::mem::size_of::<Node<K, V>>() + key.heap_bytes());
    m.alloc(Node {
        height: 1 + height(&left).max(height(&right)),
        len: 1 + len(&left) + len(&right),
        key,
        value,
        left,
        right,
    })
}
fn balance<K: Key, V: Clone + Footprint>(
    m: &Memory,
    key: K,
    value: V,
    mut left: Link<K, V>,
    mut right: Link<K, V>,
) -> Result<Shared<Node<K, V>>, Admission> {
    if height(&left) > height(&right) + 1 {
        let l = left.as_ref().unwrap();
        if height(&l.left) < height(&l.right) {
            let x = l.right.as_ref().unwrap();
            left = Some(node(
                m,
                x.key.clone(),
                x.value.clone(),
                Some(node(
                    m,
                    l.key.clone(),
                    l.value.clone(),
                    l.left.clone(),
                    x.left.clone(),
                )?),
                x.right.clone(),
            )?);
        }
        let l = left.as_ref().unwrap();
        return node(
            m,
            l.key.clone(),
            l.value.clone(),
            l.left.clone(),
            Some(node(m, key, value, l.right.clone(), right)?),
        );
    }
    if height(&right) > height(&left) + 1 {
        let r = right.as_ref().unwrap();
        if height(&r.right) < height(&r.left) {
            let x = r.left.as_ref().unwrap();
            right = Some(node(
                m,
                x.key.clone(),
                x.value.clone(),
                x.left.clone(),
                Some(node(
                    m,
                    r.key.clone(),
                    r.value.clone(),
                    x.right.clone(),
                    r.right.clone(),
                )?),
            )?);
        }
        let r = right.as_ref().unwrap();
        return node(
            m,
            r.key.clone(),
            r.value.clone(),
            Some(node(m, key, value, left, r.left.clone())?),
            r.right.clone(),
        );
    }
    node(m, key, value, left, right)
}
fn put<K: Key, V: Clone + Footprint>(
    m: &Memory,
    root: &Link<K, V>,
    key: K,
    value: V,
) -> Result<Link<K, V>, Admission> {
    let Some(n) = root else {
        return Ok(Some(node(m, key, value, None, None)?));
    };
    Ok(Some(match key.cmp(&n.key) {
        std::cmp::Ordering::Equal => node(m, key, value, n.left.clone(), n.right.clone())?,
        std::cmp::Ordering::Less => balance(
            m,
            n.key.clone(),
            n.value.clone(),
            put(m, &n.left, key, value)?,
            n.right.clone(),
        )?,
        std::cmp::Ordering::Greater => balance(
            m,
            n.key.clone(),
            n.value.clone(),
            n.left.clone(),
            put(m, &n.right, key, value)?,
        )?,
    }))
}
fn unique<K: Key, V: Clone + Footprint>(
    m: &Memory,
    root: &mut Shared<Node<K, V>>,
) -> Result<(), Admission> {
    if root.get_mut().is_none() {
        *root = node(
            m,
            root.key.clone(),
            root.value.clone(),
            root.left.clone(),
            root.right.clone(),
        )?;
    }
    Ok(())
}
fn refresh<K, V>(n: &mut Node<K, V>) {
    n.height = 1 + height(&n.left).max(height(&n.right));
    n.len = 1 + len(&n.left) + len(&n.right);
}
fn rotate_private<K: Key, V: Clone + Footprint>(
    m: &Memory,
    root: &mut Link<K, V>,
    left: bool,
) -> Result<(), Admission> {
    // Prepare ownership before taking any links; subsequent rotation cannot fail.
    let a = root.as_mut().unwrap();
    unique(m, a)?;
    let n = a.get_mut().unwrap();
    unique(
        m,
        if left {
            n.right.as_mut().unwrap()
        } else {
            n.left.as_mut().unwrap()
        },
    )?;
    let mut a = root.take().unwrap();
    let n = a.get_mut().unwrap();
    let mut b = if left {
        n.right.take().unwrap()
    } else {
        n.left.take().unwrap()
    };
    let pivot = b.get_mut().unwrap();
    if left {
        n.right = pivot.left.take();
    } else {
        n.left = pivot.right.take();
    }
    refresh(n);
    if left {
        pivot.left = Some(a);
    } else {
        pivot.right = Some(a);
    }
    refresh(pivot);
    *root = Some(b);
    Ok(())
}
fn put_private<K: Key, V: Clone + Footprint>(
    m: &Memory,
    root: &mut Link<K, V>,
    key: K,
    value: V,
) -> Result<(), Admission> {
    let Some(n) = root.as_mut() else {
        *root = Some(node(m, key, value, None, None)?);
        return Ok(());
    };
    if (**n).heap_bytes() != 0 {
        *root = put(m, root, key, value)?;
        return Ok(());
    }
    unique(m, n)?;
    let n = n.get_mut().unwrap();
    match key.cmp(&n.key) {
        std::cmp::Ordering::Equal => {
            n.key = key;
            n.value = value;
        }
        std::cmp::Ordering::Less => put_private(m, &mut n.left, key, value)?,
        std::cmp::Ordering::Greater => put_private(m, &mut n.right, key, value)?,
    }
    refresh(n);
    if height(&n.left) > height(&n.right) + 1 {
        let l = n.left.as_ref().unwrap();
        if height(&l.left) < height(&l.right) {
            rotate_private(m, &mut n.left, true)?;
        }
        rotate_private(m, root, false)?;
    } else if height(&n.right) > height(&n.left) + 1 {
        let r = n.right.as_ref().unwrap();
        if height(&r.right) < height(&r.left) {
            rotate_private(m, &mut n.right, false)?;
        }
        rotate_private(m, root, true)?;
    }
    Ok(())
}
fn remove<K: Key + Borrow<Q>, V: Clone + Footprint, Q: Ord + ?Sized>(
    m: &Memory,
    root: &Link<K, V>,
    key: &Q,
) -> Result<Link<K, V>, Admission> {
    let Some(n) = root else {
        return Ok(None);
    };
    Ok(match key.cmp(n.key.borrow()) {
        std::cmp::Ordering::Less => Some(balance(
            m,
            n.key.clone(),
            n.value.clone(),
            remove(m, &n.left, key)?,
            n.right.clone(),
        )?),
        std::cmp::Ordering::Greater => Some(balance(
            m,
            n.key.clone(),
            n.value.clone(),
            n.left.clone(),
            remove(m, &n.right, key)?,
        )?),
        std::cmp::Ordering::Equal => match (&n.left, &n.right) {
            (None, _) => n.right.clone(),
            (_, None) => n.left.clone(),
            (_, Some(r)) => {
                let mut successor = r;
                while let Some(l) = &successor.left {
                    successor = l;
                }
                Some(balance(
                    m,
                    successor.key.clone(),
                    successor.value.clone(),
                    n.left.clone(),
                    remove(m, &n.right, successor.key.borrow())?,
                )?)
            }
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    fn check<K: Key, V: Clone + Footprint>(root: &Link<K, V>) -> (usize, u8) {
        let Some(n) = root else { return (0, 0) };
        let (ll, lh) = check(&n.left);
        let (rl, rh) = check(&n.right);
        assert!(lh.abs_diff(rh) <= 1);
        assert_eq!(n.height, 1 + lh.max(rh));
        assert_eq!(n.len, 1 + ll + rl);
        (n.len, n.height)
    }
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct Inline(u64);
    impl Key for Inline {}
    #[test]
    fn private_rotations_shared_generations_and_deletions_match_ordered_map() {
        let memory = Memory::new(32 * 1024 * 1024);
        let mut index = Index::new(memory.clone());
        let mut oracle = BTreeMap::new();
        let mut retained = Vec::new();
        let mut seed = 73u64;
        for step in 0..4096 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let key = Inline((seed >> 16) % 512);
            if seed & 3 == 0 {
                index.remove(&key).unwrap();
                oracle.remove(&key);
            } else {
                let v = pivot_types::PivotDecimal::from(step);
                index.insert_private(key, v).unwrap();
                oracle.insert(key, v);
            }
            check(&index.root);
            assert!(
                index
                    .iter()
                    .map(|(k, v)| (*k, *v))
                    .eq(oracle.iter().map(|(k, v)| (*k, *v)))
            );
            if step % 128 == 0 {
                retained.push((index.clone(), oracle.clone()));
            }
        }
        for (old, expected) in retained {
            check(&old.root);
            assert!(old.iter().map(|(k, v)| (*k, *v)).eq(expected.into_iter()));
        }
        drop(index);
        assert_eq!(memory.stats().charged, 0);
    }
    #[test]
    fn private_admission_failure_preserves_published_root_and_releases_partial_candidate() {
        let memory = Memory::new(1024 * 1024);
        let mut published = Index::new(memory.clone());
        for i in 0..512 {
            published.insert_private(Inline(i), ()).unwrap();
        }
        let before = memory.stats().charged;
        let mut candidate = published.clone();
        let hold = memory.reserve(memory.stats().limit - before - 512).unwrap();
        assert!(candidate.insert_private(Inline(1000), ()).is_err());
        assert_eq!(published.len(), 512);
        assert!(!published.contains_key(&Inline(1000)));
        check(&published.root);
        drop((hold, candidate));
        assert_eq!(memory.stats().charged, before);
        drop(published);
        assert_eq!(memory.stats().charged, 0);
    }
    #[test]
    fn heap_payload_replacement_preserves_exact_charge_by_functional_fallback() {
        let memory = Memory::new(1024 * 1024);
        let mut index = Index::new(memory.clone());
        let text = "x".repeat(4096);
        index.insert_private(Inline(1), text).unwrap();
        let before = memory.stats().charged;
        index.insert_private(Inline(1), String::new()).unwrap();
        assert!(index.get(&Inline(1)).unwrap().is_empty());
        assert!(memory.stats().charged + 4096 <= before);
        drop(index);
        assert_eq!(memory.stats().charged, 0);
    }
    #[test]
    fn rotations_deletions_and_retained_roots_match_an_independent_ordered_map() {
        let memory = Memory::new(32 * 1024 * 1024);
        let mut index = Index::<String, ()>::new(memory.clone());
        let mut oracle = BTreeMap::new();
        let mut seed = 73u64;
        let mut retained = Vec::new();
        for step in 0..4096 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let key = format!("n{}", (seed >> 16) % 512);
            if seed & 3 == 0 {
                index.remove(&key).unwrap();
                oracle.remove(&key);
            } else {
                index.insert(key.clone(), ()).unwrap();
                oracle.insert(key, ());
            }
            check(&index.root);
            assert_eq!(
                index.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>(),
                oracle.keys().cloned().collect::<Vec<_>>()
            );
            if step % 256 == 0 {
                retained.push((index.clone(), oracle.clone()));
            }
        }
        for (old, expected) in retained {
            check(&old.root);
            assert_eq!(
                old.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>(),
                expected.keys().cloned().collect::<Vec<_>>()
            );
        }
        drop(index);
        assert_eq!(memory.stats().charged, 0);
    }
}
