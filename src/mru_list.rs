use std::ops::Index;
use std::io::{self, Write, BufRead, BufWriter, BufReader};
use std::fs::File;
use std::path::{Path, PathBuf};
use path_encoding;
use paths;
use std::slice;

/// A simple MRU-list data structure. Create a list of the appropriate
/// maximum size (which can be changed later) then use `insert` to add new
/// items. New items are always added at the front of the list. Adding
/// an item which is already in the list is ok - it is moved to the beginning
/// of the list. The list keeps track of whether its contents have changed,
/// to allow users to only persist the list if it actually changes.
///
/// The `MruList` is not intended to be a high-performance data
/// structure, it is intended for managing small numbers of items such as
/// might appear in an editor's MRU menu.
///
/// An MruList always holds paths in RAM in their expanded '/home/xyz' form,
/// but writes them out to disk in their friendlier '~' form.
pub struct MruList {
    filename: PathBuf,
    items: Vec<PathBuf>,
    max_items: usize,
    is_changed: bool,
}

impl MruList {
    pub fn new<P>(filename: P, max_items: usize) -> Self
        where P: AsRef<Path>
    {
        Self {
            filename: filename.as_ref().to_path_buf(),
            items: Vec::with_capacity(max_items),
            max_items: max_items,
            is_changed: false,
        }
    }

    pub fn is_changed(&self) -> bool {
        self.is_changed
    }

    pub fn clear_is_changed(&mut self) {
        self.is_changed = false;
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn filename(&self) -> &Path {
        &self.filename
    }

    /// Adds a path into the MRUList. `path` is now the first item in the list.
    pub fn insert<P>(&mut self, path: P)
        where P: AsRef<Path>
    {
        let path = paths::from_canon(path);
        self.insert_impl(path);
    }

    fn insert_impl(&mut self, path: PathBuf)
    {
        self.remove(&path);
        self.items.insert(0, path);
        self.items.truncate(self.max_items);
        self.is_changed = true;
    }

    /// Removes a path from the MRUList if it exists. A no-op if it doesn't.
    pub fn remove<P>(&mut self, path: P)
        where P: AsRef<Path>
    {
        let path = paths::from_canon(path);
        self.remove_impl(&path);
    }

    fn remove_impl(&mut self, path: &Path)
    {
        if let Some(pos) = self.items.iter().position(|x| *x == *path) {
            self.items.remove(pos);
            self.is_changed = true;
        }
    }

    pub fn write_to_file(&mut self) -> io::Result<()> {
        let _timer = timer!("MRU.write");

        let file = File::create(&self.filename)?;
        let mut writer = BufWriter::new(file);

        for pbuf in &self.items {
            let p = paths::to_canon(pbuf);
            let encoded_path = path_encoding::encode_path(&p);
            writeln!(writer, "{}", encoded_path);
        }

        self.clear_is_changed();
        _timer.set_message(format!("Wrote {} entries to the MRU file '{}'", self.len(), self.filename.display()));
        Ok(())
    }

    pub fn read_from_file(&mut self) -> io::Result<()> {
        let _timer = timer!("MRU.read");

        if Path::exists(&self.filename) {
            let file = File::open(&self.filename)?;
            let reader = BufReader::new(file);
            for line_result in reader.lines().take(self.max_items) {
                let line = line_result?;
                if line.trim().is_empty() { continue };
                match path_encoding::decode_path(&line) {
                    Ok(decoded_path) => self.insert(decoded_path),
                    Err(_) => warn!("Skipping undecodable MRU entry '{}'", line)
                }
            }
            self.items.reverse();
            self.clear_is_changed();
            _timer.set_message(format!("Read {} MRU entries from '{}'",
                                       self.len(), self.filename.display()));
        } else {
            _timer.set_message(format!("No MRU list loaded because the expected MRU file '{}' does not exist.",
                                      self.filename.display()));
        }

        Ok(())
    }

    pub fn iter(&self) -> slice::Iter<PathBuf> {
        self.items.iter()
    }
}

impl Index<usize> for MruList {
    type Output = PathBuf;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

impl<'a> IntoIterator for &'a MruList {
    type Item = &'a PathBuf;
    type IntoIter = slice::Iter<'a, PathBuf>;

    fn into_iter(self) -> slice::Iter<'a, PathBuf> {
        self.items.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_simple_mru() -> MruList {
        let mut mru = MruList::new("mru.txt", 20);
        // Insert in reverse order, so that the list is "a", "b", "c" when done.
        mru.insert("c");
        mru.insert("b");
        mru.insert("a");
        mru.clear_is_changed();
        mru
    }

    #[test]
    fn new_makes_empty_mru() {
        let mru = MruList::new("mru.txt", 20);
        assert_eq!(0, mru.len());
        assert!(mru.is_empty());
        assert!(!mru.is_changed());
    }

    #[test]
    fn remove_can_remove_first_item() {
        let mut mru = make_simple_mru();
        mru.remove("a");
        assert!(mru.is_changed());
        assert_eq!(2, mru.len());
        assert_eq!(mru[0], PathBuf::from("b"));
        assert_eq!(mru[1], PathBuf::from("c"));
    }

    #[test]
    fn remove_can_remove_last_item() {
        let mut mru = make_simple_mru();
        mru.remove("c");
        assert!(mru.is_changed());
        assert_eq!(2, mru.len());
        assert_eq!(mru[0], PathBuf::from("a"));
        assert_eq!(mru[1], PathBuf::from("b"));
    }

    #[test]
    fn remove_for_item_that_is_not_in_list_is_noop() {
        let mut mru = make_simple_mru();
        mru.remove("z");
        assert!(!mru.is_changed());
        assert_eq!(3, mru.len());
        assert_eq!(mru[0], PathBuf::from("a"));
        assert_eq!(mru[1], PathBuf::from("b"));
        assert_eq!(mru[2], PathBuf::from("c"));
    }

    #[test]
    fn insert_sets_changed_flag() {
        let mut mru = MruList::new("mru.txt", 20);
        mru.insert("a");
        assert!(mru.is_changed());
    }

    #[test]
    fn insert_can_insert_exactly_max_items() {
        let mut mru = MruList::new("mru.txt", 3);
        mru.insert("a");
        mru.insert("b");
        mru.insert("c");
        assert_eq!(3, mru.len());
        assert_eq!(mru[0], PathBuf::from("c"));
        assert_eq!(mru[1], PathBuf::from("b"));
        assert_eq!(mru[2], PathBuf::from("a"));
    }

    #[test]
    fn insert_can_insert_no_more_than_max_items() {
        let mut mru = MruList::new("mru.txt", 3);
        mru.insert("a");
        mru.insert("b");
        mru.insert("c");
        mru.insert("d");
        assert_eq!(3, mru.len());
        assert_eq!(mru[0], PathBuf::from("d"));
        assert_eq!(mru[1], PathBuf::from("c"));
        assert_eq!(mru[2], PathBuf::from("b"));
    }

    #[test]
    fn iter_iterates_items_in_order() {
        let mru = make_simple_mru();
        let mut it = mru.iter();
        assert_eq!(Some(&PathBuf::from("a")), it.next());
        assert_eq!(Some(&PathBuf::from("b")), it.next());
        assert_eq!(Some(&PathBuf::from("c")), it.next());
        assert_eq!(None, it.next());
    }
}
