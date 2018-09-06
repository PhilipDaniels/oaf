use std::cmp;
use std::ops::Index;

/// A simple MRU-list data structure. Create a list of the appropriate
/// maximum size (which can be changed later) then use `insert` to add new
/// items. New items are always added at the front of the list. Adding
/// an item which is already in the list is ok - it is moved to the beginning
/// of the list. The list keeps track of whether its contents have changed,
/// to allow users to only persist the list if it actually changes.
///
/// The `MRUList` is not intended to be a high-performance data
/// structure, it is intended for managing small numbers of items such as
/// might appear in an editor's MRU menu.
pub struct MRUList<MRUEntry> {
    is_changed: bool,
    max_items: usize,
    data: Vec<MRUEntry>
}

impl<MRUEntry> MRUList<MRUEntry>
    where MRUEntry: cmp::PartialEq
{
    pub fn new(max_items: usize) -> Self {
        MRUList {
            is_changed: false,
            max_items,
            data: Vec::<MRUEntry>::with_capacity(max_items)
        }
    }

    pub fn is_changed(&self) -> bool {
        self.is_changed
    }

    pub fn clear_is_changed_flag(&mut self) {
        self.is_changed = false;
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Adds a value into the MRUList. `value` is now the first item in the list.
    pub fn insert<V>(&mut self, value: V)
        where V: Into<MRUEntry>
    {
        let value = value.into();
        self.remove(&value);
        self.data.insert(0, value);
        self.data.truncate(self.max_items);
        self.is_changed = true;
    }

    // THOUGHT: Is there a way of taking a &V: Into<&MRUEntry> instead, so that this method
    // is as flexible as insert()?
    pub fn remove(&mut self, value: &MRUEntry)
    {
        if let Some(pos) = self.data.iter().position(|x| *x == *value) {
            self.data.remove(pos);
            self.is_changed = true;
        }
    }

    pub fn iter(&self) -> MRUIterator<MRUEntry> {
        let it = MRUIterator { data: &self.data, next: 0 };
        it
    }
}

impl<MRUEntry> Index<usize> for MRUList<MRUEntry> {
    type Output = MRUEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

pub struct MRUIterator<'mru, MRUEntry: 'mru> {
    data: &'mru Vec<MRUEntry>,
    next: usize,
}

impl<'mru, MRUEntry> Iterator for MRUIterator<'mru, MRUEntry> {
    type Item = &'mru MRUEntry;

    fn next(&mut self) -> Option<&'mru MRUEntry> {
        if self.next == self.data.len() {
            None
        } else {
            self.next += 1;
            Some(&self.data[self.next - 1])
        }
    }
}

impl<'mru, MRUEntry> IntoIterator for &'mru MRUList<MRUEntry>
    where MRUEntry: PartialEq + 'mru
{
    type Item = &'mru MRUEntry;
    type IntoIter = MRUIterator<'mru, MRUEntry>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_simple_mru() -> MRUList<String> {
        let mut mru = MRUList::new(20);
        // Insert in reverse order, so that the list is "a", "b", "c" when done.
        mru.insert("c");
        mru.insert("b");
        mru.insert("a");
        mru.clear_is_changed_flag();
        mru
    }

    #[test]
    pub fn remove_can_remove_first_item() {
        let mut mru = make_simple_mru();
        mru.remove(&"a".to_string());
        assert!(mru.is_changed());
        assert_eq!(2, mru.len());
        assert_eq!(mru.iter().collect::<Vec<_>>(), ["b", "c"]);
    }
}

