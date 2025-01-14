extern crate csv;
extern crate hashbrown;
extern crate indexmap;
extern crate kanaria;
extern crate lazy_regex;
extern crate rayon;

use rayon::ThreadPool;
use rayon::ThreadPoolBuilder;
//use threadpool::ThreadPool;

use std::sync::{Arc, Mutex};

use lazy_regex::regex_replace_all;
use lazy_regex::Lazy;
use lazy_regex::Regex;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

use csv::StringRecord;
use csv::{Error as CsvError, ReaderBuilder};

use kanaria::string::{ConvertType, UCSStr};
use kanaria::utils::ConvertTarget;

use crate::utils::adjust_cost;
use crate::utils::convert_to_hiragana;
use crate::utils::unicode_escape_to_char;

use indexmap::IndexMap;
use std::ops::{Deref, DerefMut};

use hashbrown::DefaultHashBuilder as RandomState;
//use fxhash::FxBuildHasher as RandomState;

//use foldhash::fast::RandomState;
//use std::hash::RandomState;

/// MyIndexMap
/// IndexMapでwith_hasherの指定を、切り替えてテストするため。
pub struct MyIndexMap<K, V, S = RandomState>(IndexMap<K, V, S>);

impl<K, V> Default for MyIndexMap<K, V, RandomState> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> MyIndexMap<K, V, RandomState> {
    /// WIP_new_function_description
    pub fn new() -> Self {
        Self(IndexMap::with_hasher(RandomState::default()))
    }
}

impl<K, V, S> MyIndexMap<K, V, S>
where
    S: std::hash::BuildHasher + Default,
{
    /// WIP_with_hasher_function_description
    pub fn with_hasher(hash_builder: S) -> Self {
        Self(IndexMap::with_hasher(hash_builder))
    }
}

impl<K, V, S> Deref for MyIndexMap<K, V, S> {
    type Target = IndexMap<K, V, S>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, V, S> DerefMut for MyIndexMap<K, V, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, K, V, S> IntoIterator for &'a MyIndexMap<K, V, S>
where
    K: 'a,
    V: 'a,
{
    type Item = (&'a K, &'a V);
    type IntoIter = indexmap::map::Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

use std::fmt;
impl<K, V, S> fmt::Debug for MyIndexMap<K, V, S>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.0.iter()).finish()
    }
}

mod utils {
    use super::*;

    // カタカナから読みを平仮名へ
    pub(crate) fn convert_to_hiragana(text: &str) -> String {
        // 半角カタカナを全角カタカナへ
        let yomi = UCSStr::convert(
            &text.chars().collect::<Vec<char>>(),
            ConvertType::Wide,
            ConvertTarget::KATAKANA,
        );
        // ひらがなへ
        let hiragana: String = UCSStr::convert(&yomi, ConvertType::Hiragana, ConvertTarget::ALL)
            .iter()
            .collect();
        hiragana.replace("ゐ", "い").replace("ゑ", "え")
    }

    // Unicode Escapeの記述が含まれる場合、それを変換する。
    pub(crate) fn unicode_escape_to_char(text: &str) -> String {
        regex_replace_all!(r#"\\u([0-9a-fA-F]{4})"#, text, |_, num: &str| {
            let num: u32 = u32::from_str_radix(num, 16).unwrap();
            std::char::from_u32(num).unwrap().to_string()
        })
        .to_string()
    }

    // コスト計算
    pub(crate) fn adjust_cost(cost: i32) -> i32 {
        if cost < MIN_COST {
            8000
        } else if cost > MAX_COST {
            MAX_COST
        } else {
            DEFAULT_COST + (cost / COST_ADJUSTMENT)
        }
    }
}

/// 結果構造体
/// pronunciation,notation,word_class_idの組み合わせで重複チェックされる。
#[derive(Hash, Eq, PartialEq, Clone)]
pub struct DictionaryKey {
    pronunciation: String,
    notation: String,
    word_class_id: i32,
}

/// コストと品詞判定で判明した品詞の文字列
pub struct DictionaryEntry {
    key: DictionaryKey,
    cost: i32,
    word_class: String,
}

/// システム辞書型式とユーザー辞書型式
pub struct DictionaryData {
    entries: MyIndexMap<DictionaryKey, DictionaryEntry>,
    user_entries: MyIndexMap<DictionaryKey, DictionaryEntry>,
}

impl Default for DictionaryData {
    fn default() -> Self {
        Self::new()
    }
}

impl DictionaryData {
    /// WIP_new_function_description
    pub fn new() -> Self {
        Self {
            entries: MyIndexMap::with_hasher(RandomState::default()),
            user_entries: MyIndexMap::with_hasher(RandomState::default()),
        }
    }

    /// WIP_add_function_description
    pub fn add(&mut self, entry: DictionaryEntry, is_user_dict: bool) {
        let target = if is_user_dict {
            &mut self.user_entries
        } else {
            &mut self.entries
        };
        target.insert(entry.key.to_owned(), entry);
        //if insert_result.is_none() {
        //    return Some(entry);
        //}
        //None
    }

    /// WIP_output_function_description
    pub fn output(&self, is_user_dict: bool) -> io::Result<()> {
        // 非同期の標準出力を取得
        let mut writer = BufWriter::new(io::stdout());

        // -Uオプションが設定されている場合のみユーザー辞書を出力
        // ユーザー辞書のエントリーを出力
        if is_user_dict {
            for entry in self.user_entries.values() {
                writeln!(
                    writer,
                    "{}\t{}\t{}\t",
                    entry.key.pronunciation, entry.key.notation, entry.word_class
                )?;
            }
        } else {
            // システム辞書のエントリーを出力
            for entry in self.entries.values() {
                writeln!(
                    writer,
                    "{}\t{}\t{}\t{}\t{}",
                    entry.key.pronunciation,
                    entry.key.word_class_id,
                    entry.key.word_class_id,
                    entry.cost,
                    entry.key.notation
                )?;
            }
        }
        // バッファをフラッシュ
        writer.flush()
    }
}

/// Mozc ソースに含まれるsrc/data/dictionary_oss/id.defを読み込む
/// 更新される可能性がある。
type IdDef = MyIndexMap<String, i32>;

const DEFAULT_COST: i32 = 6000;
const MIN_COST: i32 = 0;
const MAX_COST: i32 = 10000;
const COST_ADJUSTMENT: i32 = 10;

fn id_expr(
    clsexpr: &str,
    _id_def: &mut IdDef,
    class_map: &mut MyIndexMap<String, i32>,
    _default_noun_id: i32,
) -> i32 {
    let mut expr: Vec<&str> = clsexpr.split(',').collect();
    // id.defの品詞文字列は7フィールド
    while expr.len() < 7 {
        expr.push("*");
    }
    let normalized_clsexpr = expr.join(",");

    if let Some((_, id)) = _id_def.iter().find(|(key, _)| *key == &normalized_clsexpr) {
        class_map.insert(normalized_clsexpr.to_owned(), *id);
        return *id;
    }

    let mut best_match = (0, -1); // (マッチ数, ID)

    for (key, id) in _id_def.iter() {
        let key_parts: Vec<&str> = key.split(',').collect();

        //       if expr.len() >= 2 && key_parts.len() >= 2 &&
        //           expr[0] == key_parts[0] && expr[1] == key_parts[1] {

        let mut match_count = 0; //2
        let mut expr_idx = 0; //2
        let mut key_idx = 0; //2

        while expr_idx < expr.len() && key_idx < key_parts.len() {
            if key_parts[key_idx] == "*" && expr[expr_idx] == "*" {
                // 両方が * の場合はカウント?
                //match_count += 1;
                expr_idx += 1;
                key_idx += 1;
            } else if key_parts[key_idx] == "*" {
                // key_parts が * の場合はカウントしない
                key_idx += 1;
            } else if expr[expr_idx] == "*" {
                // expr が * の場合はカウントしない
                expr_idx += 1;
            } else if expr[expr_idx] == key_parts[key_idx] {
                if key_parts[key_idx] != "一般"
                    && expr[expr_idx] != "接尾"
                    && expr[expr_idx] != "自立"
                    && expr[expr_idx] != "非自立"
                {
                    match_count += 1;
                }
                expr_idx += 1;
                key_idx += 1;
            } else {
                // 部分一致をチェック
                let mut found_partial_match = false;
                for (i, key_part) in key_parts.iter().enumerate().skip(key_idx) {
                    if expr[expr_idx] == *key_part {
                        match_count += 1;
                        key_idx = i + 1;
                        found_partial_match = true;
                        break;
                    }
                }
                if !found_partial_match {
                    break;
                }
                expr_idx += 1;
            }
        }
        // 動詞の特殊処理
        if expr[0] == "動詞" {
            let verb_type = expr.get(4).unwrap_or(&"");
            let key_verb_type = key_parts.get(4).unwrap_or(&"");

            if *key_verb_type != "一般" && verb_type == key_verb_type {
                match_count += 2; // 完全一致の場合、より高いスコアを与える
            } else {
                let verb_categories = ["五段", "一段", "四段", "カ変", "サ変", "ラ変"];
                let verb_rows = [
                    "カ行", "ガ行", "サ行", "タ行", "ナ行", "バ行", "マ行", "ラ行", "ワ行",
                ];
                for category in verb_categories.iter() {
                    if verb_type.contains(category) && key_verb_type.contains(category) {
                        match_count += 1;
                        break;
                    }
                }
                for row in verb_rows.iter() {
                    if verb_type.contains(row) && key_verb_type.contains(row) {
                        match_count += 1;
                        break;
                    }
                }
            }
        }

        if match_count > best_match.0 {
            best_match = (match_count, *id);
        }
    }

    let result_id = if best_match.1 == -1 {
        _default_noun_id
    } else {
        best_match.1
    };
    _id_def.insert(normalized_clsexpr.to_owned(), result_id);
    class_map.insert(normalized_clsexpr.to_owned(), result_id);
    result_id
}

/// id.defは更新されうるので、毎回、最新のものを読み込む。
/// 品詞判定が出来なかった場合、普通名詞とみなす。
/// _default_noun_idは、その普通名詞のIDを格納しておく。
fn read_id_def(path: &Path) -> Result<(IdDef, i32), CsvError> {
    let mut id_def = IdDef::with_hasher(RandomState::default());
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b' ')
        .from_path(path)?;
    let mut _default_noun_id: i32 = -1;

    for result in reader.records() {
        let record = result?;
        let id: i32 = record[0].parse().unwrap();
        let mut expr = record[1]
            .replace("名詞,一般,*,", "名詞,普通名詞,一般,")
            .replace("名詞,数,", "名詞,数詞,")
            .replace("名詞,接尾,助数詞,", "名詞,普通名詞,助数詞可能,")
            .replace("名詞,サ変接続,*,", "名詞,普通名詞,サ変可能,")
            .replace("名詞,副詞可能,*,", "名詞,普通名詞,副詞可能,")
            .replace("動詞,*,", "動詞,一般,")
            .replace("助動詞,*,", "助動詞,一般,")
            .replace("副詞,*,", "副詞,一般,")
            .replace("形容詞,*,", "形容詞,一般,")
            .replace("感動詞,*,", "感動詞,一般,")
            .replace("段・", "段,")
            .replace("形-", "形,")
            .replace("地域,", "地名,");

        // 名詞、一般名詞のIDを保存
        if expr == "名詞,普通名詞,一般,*,*,*,*" || expr == "名詞,一般,*,*,*,*,*" {
            _default_noun_id = id;
        }
        expr = regex_replace_all!(r"カ行([^,]*),", &expr, "カ行,$1").into_owned();
        expr = regex_replace_all!(r"サ行([^,]*),", &expr, "サ行,$1").into_owned();
        expr = regex_replace_all!(r"サ変([^,]*),", &expr, "サ変,$1").into_owned();
        expr = regex_replace_all!(r"ラ行([^,]*),", &expr, "ラ行,$1").into_owned();
        expr = regex_replace_all!(r"ワ行([^,]*),", &expr, "ワ行,$1").into_owned();

        id_def.insert(expr, id);
    }
    Ok((id_def, _default_noun_id))
}

// ユーザー辞書の品詞と、id.defの品詞のマッピングを作成する
#[derive(Debug)]
struct WordClassMapping {
    //user_to_id_def: MyIndexMap<String, String>,
    id_def_to_user: MyIndexMap<String, String>,
    id_to_user_word_class_cache: MyIndexMap<i32, String>,
}

impl WordClassMapping {
    fn new() -> Self {
        Self {
            id_def_to_user: MyIndexMap::with_hasher(RandomState::default()),
            id_to_user_word_class_cache: MyIndexMap::with_hasher(RandomState::default()),
        }
    }

    fn add_mapping(&mut self, user_word_class: &str, id_def_word_class: &str) {
        /*
        if self.user_to_id_def.get(user_word_class) == None {
        self.user_to_id_def.insert(user_word_class.to_owned(), id_def_word_class.to_owned());
        }
        */
        self.id_def_to_user
            .insert(id_def_word_class.to_owned(), user_word_class.to_owned());
    }

    fn get_first_id_def(&self, user_word_class: &String) -> Option<&String> {
        // id_def_to_userから最初にマッチしたものを取得
        for (id_def, user_class) in &self.id_def_to_user {
            if user_class == user_word_class {
                return Some(id_def);
            }
        }
        None
    }
}

// マッピング作成
fn create_word_class_mapping() -> WordClassMapping {
    let mut mapping = WordClassMapping::new();

    // ユーザー辞書の品詞とid.defの品詞のマッピングを追加
    mapping.add_mapping("名詞", "名詞,普通名詞,一般,*,*,*,*");
    mapping.add_mapping("名詞", "名詞,一般,*,*,*,*");
    mapping.add_mapping("名詞", "名詞,普通名詞,*,*,*,*,*");
    mapping.add_mapping("名詞", "名詞,代名詞,一般,*,*,*,*");
    mapping.add_mapping("固有名詞", "名詞,固有名詞,*,*,*,*,*");
    mapping.add_mapping("固有名詞", "名詞,固有名詞,一般,*,*,*,*");
    mapping.add_mapping("接尾人名", "接尾辞,人名,*,*,*,*,*");
    mapping.add_mapping("接尾人名", "接尾辞,人名,*,*,*,*,女史");
    mapping.add_mapping("接尾地名", "接尾辞,地名,*,*,*,*,*");
    mapping.add_mapping("接尾一般", "名詞,接尾,一般,*,*,*,*");
    mapping.add_mapping("地名", "名詞,固有名詞,地名,一般,*,*,*");
    mapping.add_mapping("地名", "名詞,固有名詞,地域,一般,*,*,*");
    mapping.add_mapping("地名", "名詞,接尾,地域,*,*,*,*");
    mapping.add_mapping("地名", "名詞,固有名詞,国,*,*,*,*");
    mapping.add_mapping("組織", "名詞,固有名詞,組織,*,*,*,*");
    mapping.add_mapping("人名", "名詞,固有名詞,人名,一般,*,*,*");
    mapping.add_mapping("名", "名詞,固有名詞,人名,名,*,*,*");
    mapping.add_mapping("姓", "名詞,固有名詞,人名,姓,*,*,*");
    mapping.add_mapping("動詞一段", "動詞,一般,*,*,一段,*,*");
    mapping.add_mapping("動詞サ変", "動詞,一般,*,*,サ変,*,*");
    mapping.add_mapping("動詞カ変", "動詞,一般,*,*,カ変,*,*");
    mapping.add_mapping("動詞ラ変", "動詞,自立,*,*,ラ変,*,*");
    mapping.add_mapping("動詞カ行五段", "動詞,一般,*,*,五段,カ行,*,*");
    mapping.add_mapping("動詞カ行五段", "動詞,一般,*,*,五段・カ行,*,*");
    mapping.add_mapping("動詞サ行五段", "動詞,一般,*,*,五段,サ行,*,*");
    mapping.add_mapping("動詞サ行五段", "動詞,一般,*,*,五段・サ行,*,*");
    mapping.add_mapping("動詞タ行五段", "動詞,一般,*,*,五段,タ行,*,*");
    mapping.add_mapping("動詞タ行五段", "動詞,一般,*,*,五段・タ行,*,*");
    mapping.add_mapping("動詞ナ行五段", "動詞,一般,*,*,五段,ナ行,*,*");
    mapping.add_mapping("動詞ナ行五段", "動詞,一般,*,*,五段・ナ行,*,*");
    mapping.add_mapping("動詞ハ行四段", "動詞,非自立,*,*,四段,ハ行,*,*");
    mapping.add_mapping("動詞ハ行四段", "動詞,非自立,*,*,四段・ハ行,*,*");
    mapping.add_mapping("動詞マ行五段", "動詞,一般,*,*,五段,マ行,*,*");
    mapping.add_mapping("動詞マ行五段", "動詞,一般,*,*,五段・マ行,*,*");
    mapping.add_mapping("動詞ラ行五段", "動詞,一般,*,*,五段,ラ行,*,*");
    mapping.add_mapping("動詞ラ行五段", "動詞,一般,*,*,五段・ラ行,*,*");
    mapping.add_mapping("動詞ガ行五段", "動詞,一般,*,*,五段,ガ行,*,*");
    mapping.add_mapping("動詞ガ行五段", "動詞,一般,*,*,五段・ガ行,*,*");
    mapping.add_mapping("動詞バ行五段", "動詞,一般,*,*,五段,バ行,*,*");
    mapping.add_mapping("動詞バ行五段", "動詞,一般,*,*,五段・バ行,*,*");
    mapping.add_mapping("動詞ワ行五段", "動詞,自立,*,*,五段,ワ行,*,*");
    mapping.add_mapping("動詞ワ行五段", "動詞,自立,*,*,五段・ワ行,*,*");
    mapping.add_mapping("名詞サ変", "名詞,普通名詞,サ変,可能,*,*,*");
    mapping.add_mapping("名詞サ変", "名詞,普通名詞,サ変,接続,*,*,*");
    mapping.add_mapping("名詞サ変", "名詞,普通名詞,サ変可能,*,*,*");
    mapping.add_mapping("名詞サ変", "名詞,普通名詞,サ変接続,*,*,*");
    mapping.add_mapping("名詞サ変", "名詞,サ変,可能,*,*,*");
    mapping.add_mapping("名詞サ変", "名詞,サ変,接続,*,*,*");
    mapping.add_mapping("名詞サ変", "名詞,サ変接続,*,*,*");
    mapping.add_mapping("名詞サ変", "名詞,サ変可能,*,*,*");
    mapping.add_mapping("形容詞", "形容詞,接尾,*,*,*,文語基本形,*");
    mapping.add_mapping("形容詞", "形容詞,一般,*,*,形容詞,*,*");
    mapping.add_mapping("形容詞", "形容詞,一般,*,*,*,*,*");
    mapping.add_mapping("感動詞", "感動詞,一般,*,*,*,*,*");
    mapping.add_mapping("感動詞", "感動詞,*,*,*,*,*,*");
    mapping.add_mapping("助動詞", "助動詞,一般,*,*,*,*,*");
    mapping.add_mapping("助動詞", "助動詞,*,*,*,*,*,*");
    mapping.add_mapping("終助詞", "助詞,終助詞,*,*,*,*,*");
    mapping.add_mapping("終助詞", "助詞,*,*,*,*,*,*");
    mapping.add_mapping("数", "名詞,数詞,*,*,*,*,*");
    mapping.add_mapping("助数詞", "名詞,普通名詞,助数詞可能,*,*,*");
    mapping.add_mapping("助数詞", "接尾辞,名詞的,助数詞,*,*,*,*");
    mapping.add_mapping("接続詞", "接続詞,*,*,*,*,*,*");
    mapping.add_mapping("接頭語", "接頭辞,*,*,*,*,*,*");
    mapping.add_mapping("副詞", "副詞,一般,*,*,*,*,*");
    mapping.add_mapping("副詞", "名詞,接尾,副詞可能,*,*,*,*");
    mapping.add_mapping("副詞", "接尾辞,名詞的,副詞可能,*,*,*,*");
    mapping.add_mapping("副詞", "副詞,*,*,*,*,*,*");
    mapping.add_mapping("形容詞", "形容詞,*,*,*,*,*,*");
    mapping.add_mapping("記号", "記号,*,*,*,*,*,*");
    mapping.add_mapping("記号", "補助記号,*,*,*,*,*,*");
    mapping.add_mapping("名詞形動", "名詞,形容動詞語幹,*,*,*,*,*");
    mapping.add_mapping("名詞形動", "形状詞,一般,*,*,*,*,*");
    mapping.add_mapping("名詞形動", "形状詞,*,*,*,*,*,*");
    mapping.add_mapping("接頭語", "形状詞,タリ,*,*,*,*,*");
    mapping.add_mapping("接尾一般", "接尾辞,名詞的,一般,*,*,*,*");
    mapping.add_mapping("接尾一般", "接尾辞,動詞的,*,*,*,*,*");
    mapping.add_mapping("接尾一般", "接尾辞,形状詞的,*,*,*,*,*");
    mapping.add_mapping("接尾一般", "接尾辞,*,*,*,*,*,*");
    mapping.add_mapping("形容詞", "接尾辞,形状詞的,*,*,*,*,*");
    mapping.add_mapping("連体詞", "連体詞,*,*,*,*,*,*");
    mapping.add_mapping("動詞", "動詞,*,*,*,*,*,*");
    mapping.add_mapping("フィラー", "感動詞,フィラー,*,*,*,*,*");
    mapping.add_mapping("BOS/EOS", "BOS/EOS,*,*,*,*,*,*");
    mapping.add_mapping("その他", "その他,*,*,*,*,*,*");
    mapping.add_mapping("その他", "その他,間投,*,*,*,*");

    mapping
}

// word_class_idからユーザー辞書の品詞の判定
fn get_user_word_class_by_id(
    mapping: &mut WordClassMapping,
    _id_def: &IdDef,
    word_class_id: i32,
) -> Option<String> {
    // キャッシュをチェック
    if let Some(cached_word_class) = mapping.id_to_user_word_class_cache.get(&word_class_id) {
        return Some(cached_word_class.to_owned());
    }
    let result = _id_def
        .iter()
        .find(|(_, id)| **id == word_class_id)
        .and_then(|(word_class, _)| {
            let parts: Vec<&str> = word_class.split(',').collect();
            let mut best_match: Option<(usize, &String)> = None;

            for (key, value) in &mapping.id_def_to_user {
                let key_parts: Vec<&str> = key.split(',').collect();
                let mut match_count = 0;

                // 特殊なケース（記号など）の処理
                if parts[0] == "記号" || parts[0] == "補助記号" {
                    if key_parts[0] == "記号" {
                        return Some(value.to_owned());
                    }
                    continue;
                }

                // 全項目のマッチングを試みる
                for (a, b) in parts.iter().zip(key_parts.iter()) {
                    if *b != "*" && *a == *b {
                        match_count += 1;
                    } else if *b != "*" && (a.contains(b) || b.contains(a)) {
                        match_count += 1;
                        continue;
                    } else if *b == "*" && *a == "*" {
                        continue;
                    } else {
                        break;
                    }
                }

                // 固有名詞の場合、より詳細なマッチングを要求
                //if parts[0] == "名詞" && parts[1] == "固有名詞" && match_count < 3 {
                //    is_valid_match = false;
                //}

                // 動詞の活用型のマッチング
                if parts[0] == "動詞" {
                    let verb_type = parts.get(4).unwrap_or(&"");
                    let verb_categories = ["五段", "一段", "四段", "カ変", "サ変", "ラ変"];
                    for category in verb_categories.iter() {
                        if verb_type.contains(category)
                            && key_parts.iter().any(|&k| k.contains(category))
                        {
                            match_count += 1;
                            break; // 最初にマッチしたら終了
                        }
                    }
                }

                if best_match.is_none() || match_count > best_match.unwrap().0 {
                    best_match = Some((match_count, value));
                }
            }

            best_match.map(|(_, v)| v.to_owned())
        });
    // 結果をキャッシュに保存
    if let Some(ref word_class) = result {
        mapping
            .id_to_user_word_class_cache
            .insert(word_class_id, word_class.to_owned());
    }

    result
}

// ユーザー辞書の品詞からid_defの品詞文字列へ
fn get_user_word_class(
    mapping: &WordClassMapping,
    _id_def: &IdDef,
    user_word_class: String,
) -> String {
    // キャッシュをチェック
    let word_class: String = match mapping.get_first_id_def(&user_word_class) {
        Some(class) => class.to_owned(),
        None => "名詞,一般,*,*,*,*,*".to_owned(),
    };
    word_class
}

// id.defからキーを検索
fn search_key(def: &IdDef, search: i32) -> String {
    for (key, value) in def {
        if value == &search {
            return key.to_owned();
        } else {
            continue;
        }
    }
    "".to_owned()
}

// 品詞idからユーザー辞書の品詞を判定
fn u_search_key(
    mapping: &mut WordClassMapping,
    _id_def: &IdDef,
    word_class_id: i32,
) -> Option<String> {
    get_user_word_class_by_id(mapping, _id_def, word_class_id)
}

// ユーザー辞書の品詞からid.defの品詞文字列へ
fn u_search_word_class(
    mapping: &WordClassMapping,
    _id_def: &mut IdDef,
    word_class: String,
) -> String {
    get_user_word_class(mapping, _id_def, word_class)
}

static KANA_CHECK: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[(ぁ-ゖ)ゐゑゐ゙ゑ゙(ァ-ヺ)ー・゛゜]+$").unwrap());
static START_SUUJI_CHECK: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\d|￥\d|¥\d|第\d)+").unwrap());
static KIGOU_CHECK: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z' ]+$").unwrap());
// 地名チェックに用いる日本語判定
// 漢字、ひらがな、カタカナから始まる単語を日本語とみなす。
// ２文字目以降は、漢字、ひらがな、カタカナ以外に、
// 句読点(Punct)、長音ー記号を含む修飾文字(Letter Modifier),
// (全角含む)空白(Zs),ラテン文字、数字などを容認する。
// (２文字目以降は任意の文字列にしてもいいかもしれない。)
static JAPANESE_CHECK: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[\x{3005}\x{3007}\x{303b}\x{3400}-\x{9FFF}\x{F900}-\x{FAFF}\x{20000}-\x{2FFFF}\p{Hiragana}\p{Katakana}][\x{3005}\x{3007}\x{303b}\x{3400}-\x{9FFF}\x{F900}-\x{FAFF}\x{20000}-\x{2FFFF}\p{Hiragana}\p{Katakana}\p{Lm}\p{Punct}\p{Zs}\p{Latin}\p{Number}]*$").unwrap()
});

fn is_kana(str: &str) -> bool {
    KANA_CHECK.is_match(str)
}

fn is_start_suuji(str: &str) -> bool {
    START_SUUJI_CHECK.is_match(str)
}

fn is_kigou(str: &str) -> bool {
    KIGOU_CHECK.is_match(str)
}

fn is_japanese(str: &str) -> bool {
    JAPANESE_CHECK.is_match(str)
}

/// WIP_DictValues_struct_description
pub struct DictValues<'a> {
    pronunciation: &'a mut String,
    notation: &'a mut String,
    word_class_id: &'a mut i32,
    cost: &'a mut i32,
}

/// WIP_WordClassValues_struct_description
#[derive(Debug)]
pub struct WordClassValues<'a> {
    class_map: &'a mut MyIndexMap<String, i32>,
    mapping: &'a mut WordClassMapping,
    id_def: &'a mut IdDef,
    default_noun_id: &'a mut i32,
}

/// WIP_DictionaryProcessor_trait_description
pub trait DictionaryProcessor: Send + Sync {
    fn should_skip(
        &self,
        _word_class_values: &mut WordClassValues,
        _dict_values: &mut DictValues,
        record: &StringRecord,
        _args: &Config,
    ) -> bool;
    fn word_class_analyze(
        &self,
        _word_class_values: &mut WordClassValues,
        _dict_values: &mut DictValues,
        record: &StringRecord,
        _args: &Config,
    ) -> bool;
}

fn skip_analyze(
    record: &StringRecord,
    _args: &Config,
    _word_class_values: &mut WordClassValues,
    _dict_values: &mut DictValues,
) -> bool {
    let _pronunciation = match record.get(_args.pronunciation_index) {
        Some(p) => p,
        None => return false,
    };
    let _notation = match record.get(_args.notation_index) {
        Some(n) => n,
        None => return false,
    };
    let mut word_class_parts = Vec::new();
    let start_index = _args.word_class_index;
    let end_index = std::cmp::min(start_index + _args.word_class_numbers, record.len());

    for i in start_index..end_index {
        if let Some(part) = record.get(i) {
            word_class_parts.push(part.trim());
        } else {
            break;
        }
    }

    if _args.sudachi {
        process_sudachi_skip(_args, _pronunciation, _notation, &word_class_parts)
    } else if _args.neologd {
        process_neologd_skip(_args, _pronunciation, _notation, &word_class_parts)
    } else if _args.utdict {
        process_utdict_skip(
            _args,
            _word_class_values,
            _dict_values,
            _pronunciation,
            _notation,
            &word_class_parts,
        )
    } else if _args.mozcuserdict {
        process_mozcuserdict_skip(
            _args,
            _word_class_values,
            _dict_values,
            _pronunciation,
            _notation,
            &word_class_parts,
        )
    } else {
        process_sudachi_skip(_args, _pronunciation, _notation, &word_class_parts)
    }
}

fn process_sudachi_skip(
    _args: &Config,
    _pronunciation: &str,
    _notation: &str,
    word_class: &[&str],
) -> bool {
    let mut _parts: Vec<String> = word_class.iter().map(|&s| s.to_owned()).collect();

    if !is_kana(_pronunciation) {
        return true;
    };
    if _notation.is_empty() {
        return true;
    };
    if _parts[0] == "空白" {
        return true;
    };
    if (!_args.symbols) && _pronunciation == "キゴウ" && _parts[0].contains("記号") {
        return true;
    };
    if _parts.len() > 1 && (!_args.symbols) && is_kigou(_notation) && _parts[1] != "固有名詞" {
        return true;
    };
    if _parts.len() > 2 && (!_args.places) && is_japanese(_notation) && _parts[2].contains("地名")
    {
        return true;
    };
    false
}

fn process_neologd_skip(
    _args: &Config,
    _pronunciation: &str,
    _notation: &str,
    word_class: &[&str],
) -> bool {
    let mut _parts: Vec<String> = word_class.iter().map(|&s| s.to_owned()).collect();

    if !is_kana(_pronunciation) {
        return true;
    };
    if _notation.is_empty() {
        return true;
    };
    if _parts[0] == "空白" {
        return true;
    };
    if (!_args.symbols) && _pronunciation == "キゴウ" && _parts[0].contains("記号") {
        return true;
    };
    if _parts.len() > 1 && (!_args.symbols) && is_kigou(_notation) && _parts[1] != "固有名詞" {
        return true;
    };
    if _parts.len() > 2 && (!_args.places) && _parts[2].contains("地域") {
        return true;
    };
    if _parts.len() > 2
        && _parts[0] == "名詞"
        && _parts[1] == "固有名詞"
        && _parts[2] == "一般"
        && is_start_suuji(_notation)
    {
        return true;
    };
    false
}

fn process_utdict_skip(
    _args: &Config,
    _word_class_values: &mut WordClassValues,
    _dict_values: &mut DictValues,
    _pronunciation: &str,
    _notation: &str,
    word_class: &[&str],
) -> bool {
    let mut _parts: Vec<String> = word_class.iter().map(|&s| s.to_owned()).collect();

    if !is_kana(_pronunciation) {
        return true;
    };
    if _notation.is_empty() {
        return true;
    };
    *_dict_values.word_class_id = _parts[0].parse::<i32>().unwrap();
    if *_dict_values.word_class_id == -1 || *_dict_values.word_class_id == 0 {
        *_dict_values.word_class_id = *_word_class_values.default_noun_id;
    }
    if (!_args.symbols)
        && is_kigou(_notation)
        && !search_key(_word_class_values.id_def, *_dict_values.word_class_id).contains("固有名詞")
    {
        return true;
    };
    if (!_args.places)
        && search_key(_word_class_values.id_def, *_dict_values.word_class_id).contains("地名")
    {
        return true;
    }
    false
}

fn process_mozcuserdict_skip(
    _args: &Config,
    _word_class_values: &mut WordClassValues,
    _dict_values: &mut DictValues,
    _pronunciation: &str,
    _notation: &str,
    word_class: &[&str],
) -> bool {
    let mut _parts: Vec<String> = word_class.iter().map(|&s| s.to_owned()).collect();

    if !is_kana(_pronunciation) {
        return true;
    };
    if _notation.is_empty() {
        return true;
    };
    // ユーザー辞書の品詞からID.defの品詞文字列へ
    let word_class = u_search_word_class(
        _word_class_values.mapping,
        _word_class_values.id_def,
        _parts.join(""),
    );
    *_dict_values.word_class_id = id_expr(
        &word_class,
        _word_class_values.id_def,
        _word_class_values.class_map,
        *_word_class_values.default_noun_id,
    );
    if (!_args.symbols)
        && is_kigou(_notation)
        && !search_key(_word_class_values.id_def, *_dict_values.word_class_id).contains("固有名詞")
    {
        return true;
    };
    if (!_args.places)
        && search_key(_word_class_values.id_def, *_dict_values.word_class_id).contains("地名")
    {
        return true;
    }
    false
}

fn process_word_class(
    record: &StringRecord,
    _args: &Config,
    _word_class_values: &mut WordClassValues,
    _dict_values: &mut DictValues,
) -> i32 {
    let mut word_class_parts = Vec::new();
    let start_index = _args.word_class_index;
    let end_index = std::cmp::min(start_index + _args.word_class_numbers, record.len());

    for i in start_index..end_index {
        if let Some(part) = record.get(i) {
            word_class_parts.push(part.trim());
        } else {
            break;
        }
    }
    let processed_class = if _args.sudachi {
        process_sudachi_word_class(&word_class_parts)
    } else if _args.neologd {
        process_neologd_word_class(&word_class_parts)
    } else if _args.utdict {
        return *_word_class_values.default_noun_id;
        //    process_utdict_word_class(&word_class_parts)
    } else if _args.mozcuserdict {
        u_search_word_class(
            _word_class_values.mapping,
            _word_class_values.id_def,
            process_mozcuserdict_word_class(&word_class_parts),
        )
    } else {
        process_sudachi_word_class(&word_class_parts)
    };

    id_expr(
        &processed_class,
        _word_class_values.id_def,
        _word_class_values.class_map,
        *_word_class_values.default_noun_id,
    )
}

fn process_sudachi_word_class(word_class: &[&str]) -> String {
    let mut parts: Vec<String> = word_class.iter().map(|&s| s.to_owned()).collect();

    parts[0] = parts[0].replace("補助記号", "記号");
    if parts.len() > 1 {
        parts[1] = parts[1].replace("非自立可能", "非自立");
    }
    if parts.len() > 4 {
        parts[4] = parts[4].replace("下一段", "一段");
    }
    if parts.len() > 5 {
        parts[5] = parts[5].replace("形-", "形,");
    }
    // 全体を文字列として結合
    let mut joined = parts.join(",");

    // 複数のフィールドを一度に置換
    joined = joined
        .replace("段-", "段,")
        .replace("接尾辞,名詞的,一般,", "名詞,接尾,一般,")
        .replace("接尾辞,名詞的,副詞可能,", "名詞,接尾,副詞可能,")
        .replace("接尾辞,名詞的,助数詞,", "名詞,普通名詞,助数詞可能,")
        .replace("接尾辞,名詞的,サ変可能,", "名詞,接尾,サ変接続,")
        .replace("接尾辞,動詞的,", "動詞,接尾,")
        .replace("接尾辞,形容詞的,", "形容詞,接尾,")
        .replace("接尾辞,形状詞的,", "名詞,接尾,助動詞語幹,")
        .replace("形状詞,助動詞語幹,", "名詞,接尾,助動詞語幹,")
        .replace("形状詞,一般,", "名詞,形容動詞語幹,")
        .replace("形状詞,タリ,", "接頭辞,形容詞接続,")
        .replace("代名詞,", "名詞,代名詞,一般,")
        .replace("接頭辞,", "接頭詞,");

    // 置換後の文字列を再度分割
    parts = joined.split(',').map(String::from).collect();
    parts.join(",")
}

fn process_neologd_word_class(word_class: &[&str]) -> String {
    let mut parts: Vec<String> = word_class.iter().map(|&s| s.to_owned()).collect();
    if parts.len() > 1 && parts[0] == "名詞" && parts[1] == "一般" {
        parts[1] = "普通名詞".to_owned();
    }
    parts.join(",")
}
fn process_mozcuserdict_word_class(parts: &[&str]) -> String {
    parts.join("")
}

/// WIP_DefaultProcessor_struct_description
pub struct DefaultProcessor;
impl DictionaryProcessor for DefaultProcessor {
    fn should_skip(
        &self,
        _word_class_values: &mut WordClassValues,
        _dict_values: &mut DictValues,
        record: &StringRecord,
        _args: &Config,
    ) -> bool {
        skip_analyze(record, _args, _word_class_values, _dict_values)
    }

    fn word_class_analyze(
        &self,
        _word_class_values: &mut WordClassValues,
        _dict_values: &mut DictValues,
        record: &StringRecord,
        _args: &Config,
    ) -> bool {
        let _pronunciation: String = match record.get(_args.pronunciation_index) {
            Some(p) => convert_to_hiragana(p),
            None => return false,
        };
        let _notation = match record.get(_args.notation_index) {
            Some(n) => n,
            None => return false,
        };
        *_dict_values.word_class_id =
            process_word_class(record, _args, _word_class_values, _dict_values);
        if (!_args.places)
            && search_key(_word_class_values.id_def, *_dict_values.word_class_id).contains("地名")
            && is_japanese(_dict_values.notation)
        {
            return false;
        }
        *_dict_values.pronunciation = unicode_escape_to_char(&_pronunciation);
        *_dict_values.notation = unicode_escape_to_char(_notation);
        let cost_str = record
            .get(_args.cost_index)
            .map_or(DEFAULT_COST.to_string(), |s| s.to_string());
        let cost = cost_str.parse::<i32>().unwrap_or(DEFAULT_COST);
        *_dict_values.cost = adjust_cost(cost);
        true
    }
}

/// WIP_SudachiProcessor_struct_description
pub struct SudachiProcessor;
impl DictionaryProcessor for SudachiProcessor {
    fn should_skip(
        &self,
        _word_class_values: &mut WordClassValues,
        _dict_values: &mut DictValues,
        record: &StringRecord,
        _args: &Config,
    ) -> bool {
        skip_analyze(record, _args, _word_class_values, _dict_values)
    }

    fn word_class_analyze(
        &self,
        _word_class_values: &mut WordClassValues,
        _dict_values: &mut DictValues,
        record: &StringRecord,
        _args: &Config,
    ) -> bool {
        let _pronunciation: String = match record.get(_args.pronunciation_index) {
            Some(p) => convert_to_hiragana(p),
            None => return false,
        };
        let _notation = match record.get(_args.notation_index) {
            Some(n) => n,
            None => return false,
        };
        *_dict_values.word_class_id =
            process_word_class(record, _args, _word_class_values, _dict_values);
        if (!_args.places)
            && search_key(_word_class_values.id_def, *_dict_values.word_class_id).contains("地名")
            && is_japanese(_dict_values.notation)
        {
            return false;
        }
        *_dict_values.pronunciation = unicode_escape_to_char(&_pronunciation);
        *_dict_values.notation = unicode_escape_to_char(_notation);
        let cost_str = record
            .get(_args.cost_index)
            .map_or(DEFAULT_COST.to_string(), |s| s.to_string());
        let cost = cost_str.parse::<i32>().unwrap_or(DEFAULT_COST);
        *_dict_values.cost = adjust_cost(cost);
        true
    }
}

/// WIP_NeologdProcessor_struct_description
pub struct NeologdProcessor;
impl DictionaryProcessor for NeologdProcessor {
    fn should_skip(
        &self,
        _word_class_values: &mut WordClassValues,
        _dict_values: &mut DictValues,
        record: &StringRecord,
        _args: &Config,
    ) -> bool {
        skip_analyze(record, _args, _word_class_values, _dict_values)
    }

    fn word_class_analyze(
        &self,
        _word_class_values: &mut WordClassValues,
        _dict_values: &mut DictValues,
        record: &StringRecord,
        _args: &Config,
    ) -> bool {
        let _pronunciation: String = match record.get(_args.pronunciation_index) {
            Some(p) => convert_to_hiragana(p),
            None => return false,
        };
        let _notation = match record.get(_args.notation_index) {
            Some(n) => n,
            None => return false,
        };
        *_dict_values.word_class_id =
            process_word_class(record, _args, _word_class_values, _dict_values);
        if (!_args.places)
            && search_key(_word_class_values.id_def, *_dict_values.word_class_id).contains("地名")
        {
            return false;
        }
        *_dict_values.pronunciation = unicode_escape_to_char(&_pronunciation);
        *_dict_values.notation = unicode_escape_to_char(_notation);
        let cost_str = record
            .get(_args.cost_index)
            .map_or(DEFAULT_COST.to_string(), |s| s.to_string());
        let cost = cost_str.parse::<i32>().unwrap_or(DEFAULT_COST);
        *_dict_values.cost = adjust_cost(cost);
        true
    }
}

/// WIP_UtDictProcessor_struct_description
pub struct UtDictProcessor;
impl DictionaryProcessor for UtDictProcessor {
    fn should_skip(
        &self,
        _word_class_values: &mut WordClassValues,
        _dict_values: &mut DictValues,
        record: &StringRecord,
        _args: &Config,
    ) -> bool {
        skip_analyze(record, _args, _word_class_values, _dict_values)
    }

    fn word_class_analyze(
        &self,
        _word_class_values: &mut WordClassValues,
        _dict_values: &mut DictValues,
        record: &StringRecord,
        _args: &Config,
    ) -> bool {
        let data = &record;
        let word_class = &data[_args.word_class_index];
        let mut word_class_id = word_class.parse::<i32>().unwrap();
        if word_class == "0000" || word_class_id == -1 || word_class_id == 0 {
            word_class_id = *_word_class_values.default_noun_id;
        }
        let _pronunciation: String = match record.get(_args.pronunciation_index) {
            Some(p) => convert_to_hiragana(p),
            None => return false,
        };
        let _notation = match record.get(_args.notation_index) {
            Some(n) => n,
            None => return false,
        };
        *_dict_values.pronunciation = unicode_escape_to_char(&_pronunciation);
        *_dict_values.notation = unicode_escape_to_char(_notation);
        let d: String = search_key(_word_class_values.id_def, word_class_id).to_owned();
        let word_class = _word_class_values.class_map.get(&d);
        if word_class.is_none() {
            *_dict_values.word_class_id = id_expr(
                &d,
                _word_class_values.id_def,
                _word_class_values.class_map,
                *_word_class_values.default_noun_id,
            );
        } else {
            *_dict_values.word_class_id = *word_class.unwrap();
        }
        let cost_str = record
            .get(_args.cost_index)
            .map_or(DEFAULT_COST.to_string(), |s| s.to_string());
        let cost = cost_str.parse::<i32>().unwrap_or(DEFAULT_COST);
        *_dict_values.cost = adjust_cost(cost);
        true
    }
}

/// WIP_MozcUserDictProcessor_struct_description
pub struct MozcUserDictProcessor;
impl DictionaryProcessor for MozcUserDictProcessor {
    fn should_skip(
        &self,
        _word_class_values: &mut WordClassValues,
        _dict_values: &mut DictValues,
        record: &StringRecord,
        _args: &Config,
    ) -> bool {
        skip_analyze(record, _args, _word_class_values, _dict_values)
    }

    fn word_class_analyze(
        &self,
        _word_class_values: &mut WordClassValues,
        _dict_values: &mut DictValues,
        record: &StringRecord,
        _args: &Config,
    ) -> bool {
        // ユーザー辞書型式から品詞IDに
        let mut word_class_parts = Vec::new();
        let start_index = _args.word_class_index;
        let end_index = std::cmp::min(start_index + _args.word_class_numbers, record.len());

        for i in start_index..end_index {
            if let Some(part) = record.get(i) {
                word_class_parts.push(part.trim());
            } else {
                break;
            }
        }
        // ユーザー辞書型式から品詞IDに
        *_dict_values.word_class_id =
            process_word_class(record, _args, _word_class_values, _dict_values);
        let _pronunciation: String = match record.get(_args.pronunciation_index) {
            Some(p) => convert_to_hiragana(p),
            None => return false,
        };
        let _notation = match record.get(_args.notation_index) {
            Some(n) => n,
            None => return false,
        };
        *_dict_values.pronunciation = unicode_escape_to_char(&_pronunciation);
        *_dict_values.notation = unicode_escape_to_char(_notation);
        let d: String =
            search_key(_word_class_values.id_def, *_dict_values.word_class_id).to_owned();
        let word_class = _word_class_values.class_map.get(&d);
        if word_class.is_none() {
            *_dict_values.word_class_id = id_expr(
                &d,
                _word_class_values.id_def,
                _word_class_values.class_map,
                *_word_class_values.default_noun_id,
            );
        } else {
            *_dict_values.word_class_id = *word_class.unwrap();
        }
        //let cost_str = record.get(_args.cost_index).map_or(DEFAULT_COST.to_string(), |s| s.to_string());
        //let cost = cost_str.parse::<i32>().unwrap_or(DEFAULT_COST);
        let cost = DEFAULT_COST;
        *_dict_values.cost = adjust_cost(cost);
        true
    }
}

fn add_dict_data(
    _data: &StringRecord,
    _word_class_values: &mut WordClassValues,
    _dict_values: &mut DictValues,
    _dict_data: Arc<Mutex<DictionaryData>>,
    _args: &Config,
) {
    let mut dict_data = _dict_data.lock().unwrap();
    if _args.user_dict {
        match u_search_key(
            _word_class_values.mapping,
            _word_class_values.id_def,
            *_dict_values.word_class_id,
        ) {
            Some(_word_class) => {
                dict_data.add(
                    DictionaryEntry {
                        key: DictionaryKey {
                            pronunciation: _dict_values.pronunciation.to_owned(),
                            notation: _dict_values.notation.to_owned(),
                            word_class_id: *_dict_values.word_class_id,
                        },
                        cost: *_dict_values.cost,
                        word_class: _word_class,
                    },
                    true,
                );
            }
            None => {
                dict_data.add(
                    DictionaryEntry {
                        key: DictionaryKey {
                            pronunciation: _dict_values.pronunciation.to_owned(),
                            notation: _dict_values.notation.to_owned(),
                            word_class_id: *_dict_values.word_class_id,
                        },
                        cost: *_dict_values.cost,
                        word_class: "名詞".to_owned(),
                    },
                    true,
                );
            }
        }
    } else {
        dict_data.add(
            DictionaryEntry {
                key: DictionaryKey {
                    pronunciation: _dict_values.pronunciation.to_owned(),
                    notation: _dict_values.notation.to_owned(),
                    word_class_id: *_dict_values.word_class_id,
                },
                cost: *_dict_values.cost,
                word_class: "".to_owned(),
            },
            false,
        );
    }
    /*
    match _result {
    Some(entry) => {
    dict_data.output_entry(writer, &entry.to_owned(), _args.user_dict);
    },
    None => todo!(),
    }
    */
}

fn parse_delimiter(s: &str, args: &Config) -> u8 {
    match s {
        "TAB" | "t" | "\\t" | "\t" => b'\t',
        "," => b',',
        ";" => b';',
        " " => b' ',
        _ => {
            let chars: Vec<char> = s.chars().collect();
            if chars.len() == 1 {
                chars[0] as u8
            } else {
                if args.debug > 1 {
                    eprintln!("Warning: Invalid delimiter '{}'. Using default ','.", s);
                }
                b','
            }
        }
    }
}

fn process_record(
    _processor: &dyn DictionaryProcessor,
    dict_data: Arc<Mutex<DictionaryData>>,
    _args: &Config,
    _word_class_values: &mut WordClassValues,
    _dict_values: &mut DictValues,
    data: &csv::StringRecord,
    _pool: &ThreadPool,
) {
    if !_processor.should_skip(_word_class_values, _dict_values, data, _args)
        && _processor.word_class_analyze(_word_class_values, _dict_values, data, _args)
    {
        //_pool.install(move || {
        add_dict_data(data, _word_class_values, _dict_values, dict_data, _args);
        //});
        //_pool.join();
    }
}

/// WIP_process_dictionary_function_description
pub fn process_dictionary(
    _processor: &dyn DictionaryProcessor,
    dict_data: Arc<Mutex<DictionaryData>>,
    _args: &Config,
) -> io::Result<()> {
    let (mut _id_def, mut _default_noun_id) = read_id_def(&_args.id_def)?;
    let mut class_map = MyIndexMap::<String, i32>::with_hasher(RandomState::default());
    let mut mapping = create_word_class_mapping();
    let mut pronunciation = String::new();
    let mut notation = String::new();
    let mut word_class_id = -1;
    let mut cost = -1;

    let mut _word_class_values = WordClassValues {
        class_map: &mut class_map,
        mapping: &mut mapping,
        id_def: &mut _id_def,
        default_noun_id: &mut _default_noun_id,
    };

    let mut _dict_values = DictValues {
        pronunciation: &mut pronunciation,
        notation: &mut notation,
        word_class_id: &mut word_class_id,
        cost: &mut cost,
    };

    let delimiter_char = parse_delimiter(&_args.delimiter, _args);

    let delimiter_str = if delimiter_char == b'\t' {
        "TAB".to_owned()
    } else {
        String::from_utf8(vec![delimiter_char]).unwrap_or_else(|_| "?".to_owned())
    };
    if _args.debug > 1 {
        eprintln!("Using delimiter: {} {}", delimiter_str, delimiter_char);
    }
    if _args.debug > 2 {
        dbg!(&_word_class_values);
    }

    let reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(delimiter_char)
        .from_path(&_args.csv_file);

    let num_threads = 0;
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    //let pool = ThreadPool::new(num_threads);
    //ThreadPoolBuilder::new().num_threads(num_threads).build_global().unwrap();

    for record in reader?.records() {
        let record = &record?;
        pool.install(|| {
            process_record(
                _processor,
                dict_data.clone(),
                _args,
                &mut _word_class_values,
                &mut _dict_values,
                record,
                &pool,
            );
        });
    }
    Ok(())
}

/// WIP_Config_struct_description
#[derive(Debug)]
pub struct Config {
    /// 変換元のテキストファイルのパス
    pub csv_file: PathBuf,
    /// Mozcソースにあるid.defファイルのパス
    pub id_def: PathBuf,
    /// 読みのフィールド位置。
    pub pronunciation_index: usize,
    /// 表記のフィールド位置。
    pub notation_index: usize,
    /// 品詞を示すフィールドの開始位置。
    pub word_class_index: usize,
    /// 品詞を示すフィールド数。
    pub word_class_numbers: usize,
    /// 品詞コストのフィールド位置。
    pub cost_index: usize,
    /// 読み取る変換元のテキストの区切り文字
    pub delimiter: String,
    /// 読み取り元をSudachiDictとみなす。
    pub sudachi: bool,
    /// 読み取り元をUtDictとみなす。
    pub utdict: bool,
    /// 読み取り元をNeologdとみなす。
    pub neologd: bool,
    /// 読み取り元をMozcユーザー辞書型式とみなす。
    pub mozcuserdict: bool,
    /// 出力する変換型式をMozcユーザー辞書型式にする。
    pub user_dict: bool,
    /// 出力に地名も含める。
    pub places: bool,
    /// 出力に記号も含める。
    pub symbols: bool,
    /// デバッグ情報の出力。
    pub debug: usize,
}
