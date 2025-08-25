use std::{collections::BTreeMap, sync::atomic::AtomicUsize};

use crate::Value::Value;


#[derive(Debug, Clone)]
pub struct DataSection {
    dotdata: BTreeMap<String, Value>
}

impl DataSection {
    pub fn new() -> Self {
        Self { dotdata: BTreeMap::new() }
    }
    pub fn append_string(&mut self,s: String) {
        static STR_COUNTER: AtomicUsize = AtomicUsize::new(0);

        // name: str1 -> "hello world", str2 -> str1
        //self.dotdata.push(Data { name: , value: () });

        let name = format!("str{}",STR_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
        self.dotdata.entry(name).or_insert_with(|| Value::Str(s));
    }

    pub fn return_data(&self) -> &BTreeMap<String, Value> {
        &self.dotdata
    }

}
