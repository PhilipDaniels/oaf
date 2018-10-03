use std::cmp;
use std::ops::Index;
use std::io::{self, Write, BufRead, BufWriter, BufReader};
use std::fs::File;
use std::path::{Path, PathBuf};
use path_encoding;
use paths;

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
pub struct MRUList<T> {
    is_changed: bool,
    max_items: usize,
    data: Vec<T>
}

impl<T> MRUList<T>
    where T: cmp::PartialEq
{
    pub fn new(max_items: usize) -> Self {
        MRUList {
            is_changed: false,
            max_items,
            data: Vec::<T>::with_capacity(max_items)
        }
    }

    pub fn is_changed(&self) -> bool {
        self.is_changed
    }

    pub fn clear_is_changed(&mut self) {
        self.is_changed = false;
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    // THOUGHT: Is there a way of taking a &V: Into<&MRUEntry> instead, so that this method
    // is as flexible as insert()? I think we need Borrow. See PRust, p. 381.
    pub fn remove(&mut self, value: &T)
    {
        if let Some(pos) = self.data.iter().position(|x| *x == *value) {
            self.data.remove(pos);
            self.is_changed = true;
        }
    }

    /// Adds a value into the MRUList. `value` is now the first item in the list.
    pub fn insert<V>(&mut self, value: V)
        where V: Into<T>
    {
        let value = value.into();
        self.remove(&value);
        self.data.insert(0, value);
        self.data.truncate(self.max_items);
        self.is_changed = true;
    }

    pub fn iter(&self) -> MRUIterator<T> {
        MRUIterator { data: &self.data, next: 0 }
    }
}

impl<T> Index<usize> for MRUList<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

pub struct MRUIterator<'mru, T: 'mru> {
    data: &'mru Vec<T>,
    next: usize,
}

impl<'mru, T> Iterator for MRUIterator<'mru, T> {
    type Item = &'mru T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next == self.data.len() {
            None
        } else {
            self.next += 1;
            Some(&self.data[self.next - 1])
        }
    }
}

impl<'mru, T> IntoIterator for &'mru MRUList<T>
    where T: PartialEq + 'mru
{
    type Item = &'mru T;
    type IntoIter = MRUIterator<'mru, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct OafMruList {
    filename: PathBuf,
    mru: MRUList<PathBuf>
}

impl OafMruList {
    pub fn new<P>(filename: P) -> Self
        where P: AsRef<Path>
    {
        OafMruList {
            filename: filename.as_ref().to_path_buf(),
            mru: MRUList::new(20)
        }
    }

    pub fn add_path<P>(&mut self, path: P)
        where P: AsRef<Path>
    {
        let path = paths::to_canon(path);
        self.mru.insert(path);
    }

    pub fn write_to_file(&mut self) -> io::Result<()> {
        let _timer = timer!("MRU.write");

        let file = File::create(&self.filename)?;
        let mut writer = BufWriter::new(file);

        for pbuf in self.mru.iter() {
            let encoded_path = path_encoding::encode_path(&pbuf);
            writeln!(writer, "{}", encoded_path);
        }

        self.mru.clear_is_changed();
        _timer.set_message(format!("Wrote {} entries to the MRU file '{}'", self.mru.len(), self.filename.display()));
        Ok(())
    }

    pub fn read_from_file(&mut self) -> io::Result<()> {
        let _timer = timer!("MRU.read");

        if Path::exists(&self.filename) {
            let file = File::open(&self.filename)?;
            let reader = BufReader::new(file);
            for line_result in reader.lines().take(self.mru.max_items) {
                let line = line_result?;
                if line.trim().is_empty() { continue };
                match path_encoding::decode_path(&line) {
                    Ok(decoded_path) => self.mru.insert(paths::from_canon(decoded_path)),
                    Err(_) => warn!("Skipping undecodable MRU entry '{}'", line)
                }
            }
            self.mru.data.reverse();
            self.mru.clear_is_changed();
            _timer.set_message(format!("Read {} MRU entries from '{}'",
                                       self.mru.len(), self.filename.display()));
        } else {
            _timer.set_message(format!("No MRU list loaded because the expected MRU file '{}' does not exist.",
                                      self.filename.display()));
        }

        Ok(())
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
        mru.clear_is_changed();
        mru
    }

    #[test]
    fn new_makes_empty_mru() {
        let mru = MRUList::<i32>::new(5);
        assert_eq!(0, mru.len());
        assert!(mru.is_empty());
        assert!(!mru.is_changed());
    }

    #[test]
    fn remove_can_remove_first_item() {
        let mut mru = make_simple_mru();
        mru.remove(&"a".to_string());
        assert!(mru.is_changed());
        assert_eq!(2, mru.len());
        assert_eq!(mru.iter().collect::<Vec<_>>(), ["b", "c"]);
    }

    #[test]
    fn remove_can_remove_last_item() {
        let mut mru = make_simple_mru();
        mru.remove(&"c".to_string());
        assert!(mru.is_changed());
        assert_eq!(2, mru.len());
        assert_eq!(mru.iter().collect::<Vec<_>>(), ["a", "b"]);
    }

    #[test]
    fn remove_for_item_that_is_not_in_list_is_noop() {
        let mut mru = make_simple_mru();
        mru.remove(&"z".to_string());
        assert!(!mru.is_changed());
        assert_eq!(3, mru.len());
        assert_eq!(mru.iter().collect::<Vec<_>>(), ["a", "b", "c"]);
    }

    #[test]
    fn insert_sets_changed_flag() {
        let mut mru = MRUList::<i32>::new(5);
        mru.insert(42);
        assert!(mru.is_changed());
    }

    #[test]
    fn insert_can_insert_exactly_max_items() {
        let mut mru = MRUList::<i32>::new(3);
        mru.insert(100);
        mru.insert(200);
        mru.insert(300);
        assert_eq!(3, mru.len());
        assert_eq!(300, mru[0]);
        assert_eq!(200, mru[1]);
        assert_eq!(100, mru[2]);
    }

    #[test]
    fn insert_can_insert_no_more_than_max_items() {
        let mut mru = MRUList::<i32>::new(3);
        mru.insert(100);
        mru.insert(200);
        mru.insert(300);
        mru.insert(400);
        assert_eq!(3, mru.len());
        assert_eq!(400, mru[0]);
        assert_eq!(300, mru[1]);
        assert_eq!(200, mru[2]);
    }

    #[test]
    fn iter_iterates_items_in_order() {
        let mut mru = MRUList::<i32>::new(3);
        mru.insert(100);
        mru.insert(200);
        mru.insert(300);
        let mut it = mru.iter();
        assert_eq!(Some(&300), it.next());
        assert_eq!(Some(&200), it.next());
        assert_eq!(Some(&100), it.next());
        assert_eq!(None, it.next());
    }
}

