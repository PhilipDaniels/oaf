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

    pub fn clear_is_changed_flag(&mut self) {
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
        let it = MRUIterator { data: &self.data, next: 0 };
        it
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

    /// Makes a non-UTF8 path as might be seen on Unix or Windows.
    /// We are using such a path to test our serialization technique - we want
    /// to write paths into a human-readable file (which will be UTF-8) or ASCII.
    /// Therefore, we will be writing Rust String types to that file, and String
    /// is always valid UTF-8.
    ///
    /// Therefore, to test writing of that file with *paths that might occur on
    /// the particular operating system* but are *not valid Rust Strings* we must
    /// form some non-UTF8 paths.
    ///
    /// On Unix, only the '\0' and '/' are invalid in filenames but any
    /// other byte sequence is valid.
    ///
    /// On Windows, the following characters are banned:
    ///     < (less than)
    ///     > (greater than)
    ///     : (colon - sometimes works, but is actually NTFS Alternate Data Streams)
    ///     " (double quote)
    ///     / (forward slash)
    ///     \ (backslash)
    ///     | (vertical bar or pipe)
    ///     ? (question mark)
    ///     * (asterisk)
    /// However, note that these are all printable characters.
    /// Windows also bans bytes 0..31 (the ASCII control characters) - so no
    /// tabs, bells or newlines in files.
    /// 
    /// Taken together, what this means is that for both OSes we can create a
    /// byte sequence that will be a valid Path but not a valid String. In particular,
    /// the following bytes are simply not permitted in UTF-8, no matter
    /// where they appear in the byte sequence:
    ///
    ///     0xC0 (192), 0xC1 (193)
    ///     0xF5 (245) to 0xFF (255) inclusive
    ///
    /// The above is from https://en.wikipedia.org/wiki/UTF-8#Codepage_layout
    /// This makes it trivial to make a non-String-representable path on Unix.
    ///
    /// On Windows, paths are UTF-16-le, not UTF-8. So we need to make a UTF-16
    /// string that is not a valid UTF-8 string.
    ///
    #[test]
    fn make_non_utf8_path() {
        //let haystack = OsString::from_wide(&[0xd800, 0xdc00, 0xd800, 0xdc01, 0xd800, 0xdc02]);
        use std::path::PathBuf;
        use std::ffi::OsString;
        use std::os::unix::ffi::{OsStrExt, OsStringExt};

//        let x = b"Hello"; bytes is also "Hello" plus a trailing invalid char which stops
//        conversion to a string.
        let bytes: Vec<u8> = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F, 0xC0];
        let os = OsString::from_vec(bytes);
        let path = PathBuf::from(os);

        match path.to_str() {
            Some(s) => {
                // A valid string.
                assert_eq!("A valid string", s);
            },
            None => {
                // Get a ref to the inner OsStr slice.
                let inner_os = path.as_os_str();
                // Get a &[u8]
                let new_bytes = inner_os.as_bytes();
                let new_vec = new_bytes.iter().collect::<Vec<_>>();
                // This confirs that we are looking at the original bytes.
                // We can then base64 encode them using
                //     base64::encode() -> String
                // and decode using
                //     base64::decode(String) -> Vec<u8>
                // and then convert that into a Path like we did at the start.
                println!("new_vec = {:x?}", new_vec);
            }
        }

        // Going the other way
        // path.into_os_string();
        // path.fromm(os_string)
        //
        //
        // Printing
        // path.display()
    }
}

