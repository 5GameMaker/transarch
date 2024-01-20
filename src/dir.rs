use std::collections::HashMap;

pub enum Entry {
    Dir(Dir),
    File(&'static [u8]),
}

pub struct Dir(HashMap<&'static str, Entry>);
impl Dir {
    pub fn file(&self, file: &str) -> &'static [u8] {
        let mut ptr = &self.0;

        for x in file.split('/') {
            let Some(entry) = ptr.get(x) else {
                panic!("file not found")
            };

            match entry {
                Entry::Dir(x) => ptr = &x.0,
                Entry::File(x) => return x,
            }
        }

        panic!("file not found")
    }
}
impl From<HashMap<&'static str, Entry>> for Dir {
    fn from(value: HashMap<&'static str, Entry>) -> Self {
        Self(value)
    }
}
