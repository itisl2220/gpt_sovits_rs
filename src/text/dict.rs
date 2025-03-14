use std::collections::HashMap;

use lazy_static::lazy_static;
use std::path::PathBuf;

lazy_static! {
    static ref ZN_DICT: HashMap<String, Vec<String>> = {
        if let Ok(word_dict_path) = std::env::var("GPT_SOVITS_DICT_PATH") {
            let path = PathBuf::from(word_dict_path.as_str()).join("zh_word_dict.json");
            let zh_word_dict = std::fs::read_to_string(path).unwrap();
            serde_json::from_str(&zh_word_dict).unwrap()
        } else {
            HashMap::default()
        }
    };
    static ref EN_DICT: HashMap<String, Vec<String>> = {
        if let Ok(word_dict_path) = std::env::var("GPT_SOVITS_DICT_PATH") {
            let path = PathBuf::from(word_dict_path.as_str()).join("en_word_dict.json");
            let en_word_dict = std::fs::read_to_string(path).unwrap();
            serde_json::from_str(&en_word_dict).unwrap()
        } else {
            HashMap::default()
        }
    };
}

pub fn zh_word_dict(word: &str) -> Option<&'static [String]> {
    ZN_DICT.get(word).map(|s| s.as_slice())
}

pub fn en_word_dict(word: &str) -> Option<&'static [String]> {
    EN_DICT.get(word).map(|s| s.as_slice())
}
