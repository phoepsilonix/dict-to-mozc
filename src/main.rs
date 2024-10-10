use std::io::{Result as ioResult, stdout, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use regex::Regex;
use lazy_regex::regex_replace_all;
use lazy_regex::Lazy;

use csv::{ReaderBuilder, Error as CsvError};
use csv::StringRecord;

use kanaria::string::{UCSStr, ConvertType};
use kanaria::utils::ConvertTarget;

use crate::utils::convert_to_hiragana;
use crate::utils::unicode_escape_to_char;
use crate::utils::adjust_cost;

mod utils {
    use super::*;

    // カタカナから読みを平仮名へ
    pub fn convert_to_hiragana(text: &str) -> String {
        let target: Vec<char> = text.chars().collect();
        let mut pronunciation: String = UCSStr::convert(&target, ConvertType::Hiragana, ConvertTarget::ALL).iter().collect();
        pronunciation = pronunciation.replace("ゐ", "い").replace("ゑ", "え");
        pronunciation
    }

    // Unicode Escapeの記述が含まれる場合、それを変換する。
    pub fn unicode_escape_to_char(text: &str) -> String {
        regex_replace_all!(r#"\\u([0-9a-fA-F]{4})"#, text, |_, num: &str| {
            let num: u32 = u32::from_str_radix(num, 16).unwrap();
            std::char::from_u32(num).unwrap().to_string()
        }).to_string()
    }

    // コスト計算
    pub fn adjust_cost(cost: i32) -> i32 {
        if cost < MIN_COST {
            8000
        } else if cost > MAX_COST {
            MAX_COST
        } else {
            DEFAULT_COST + (cost / COST_ADJUSTMENT)
        }
    }
}

// 結果構造体
// pronunciation,notation,word_class_idの組み合わせで重複チェックされる。
#[derive(Hash, Eq, PartialEq, Clone)]
struct DictionaryKey {
    pronunciation: String,
    notation: String,
    word_class_id: i32,
}

// コストと品詞判定で判明した品詞の文字列
struct DictionaryEntry {
    key: DictionaryKey,
    cost: i32,
    word_class: String,
}

// システム辞書型式とユーザー辞書型式
struct DictionaryData {
    entries: HashMap<DictionaryKey, DictionaryEntry>,
    user_entries: HashMap<DictionaryKey, DictionaryEntry>,
}

impl DictionaryData {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            user_entries: HashMap::new(),
        }
    }

    fn add(&mut self, entry: DictionaryEntry, is_user_dict: bool) {
        let target = if is_user_dict { &mut self.user_entries } else { &mut self.entries };
        target.insert(entry.key.clone(), entry);
    }

    fn output(&self, _user_dict: bool) -> ioResult<()> {
        let mut writer = BufWriter::new(stdout());

        // システム辞書のエントリーを出力
        if ! _user_dict {
            for entry in self.entries.values() {
                writeln!(
                    writer,
                    "{}\t{}\t{}\t{}\t{}",
                    entry.key.pronunciation, entry.key.word_class_id, entry.key.word_class_id, entry.cost, entry.key.notation
                )?;
            }
        } else {
            // -Uオプションが設定されている場合のみユーザー辞書を出力
            for entry in self.user_entries.values() {
                if !self.entries.contains_key(&entry.key) {
                    writeln!(
                        writer,
                        "{}\t{}\t{}\t{}",
                        entry.key.pronunciation, entry.key.notation, entry.word_class, "".to_string()
                    )?;
                }
            }
        }

        writer.flush()
    }
}
// Mozc ソースに含まれるsrc/data/dictionary_oss/id.def
// 更新される可能性がある。
type IdDef = HashMap<String, i32>;

const DEFAULT_COST: i32 = 6000;
const MIN_COST: i32 = 0;
const MAX_COST: i32 = 10000;
const COST_ADJUSTMENT: i32 = 10;

// 辞書データの品詞情報とid.defを比較して品詞のidを確定する。
fn id_expr(clsexpr: &str, _id_def: &mut HashMap<String, i32>, class_map: &mut HashMap<String, i32>, _default_noun_id: i32) -> i32 {
    if let Some(&r) = _id_def.get(clsexpr) {
        class_map.insert(clsexpr.to_string(), r);
        return r;
    }

    let expr: Vec<&str> = clsexpr.split(',').collect();
    let mut best_match = (0, -1); // (マッチ数, ID)

    for (key, &id) in _id_def.iter() {
        let key_parts: Vec<&str> = key.split(',').collect();

        // 品詞の主要部分(最初の2-3項目)が一致するかを確認
        if expr.len() >= 2 && key_parts.len() >= 2 &&
            expr[0] == key_parts[0] && expr[1] == key_parts[1] {

                let mut match_count = 2; // 最初の2項目は既に一致している
                let mut is_valid_match = true;

                // 残りの項目をチェック
                for (i, (a, b)) in expr.iter().zip(key_parts.iter()).skip(2).enumerate() {
                    if *b != "*" && *a == *b {
                        match_count += 1;
                    } else if i < 1 { // 3番目の項目（小分類）まで厳密にチェック
                        is_valid_match = false;
                        break;
                    } else {
                        // 4番目以降の項目は部分一致も許容
                        if a.contains(b) || b.contains(a) {
                            match_count += 1;
                        }
                        break; // 最初の不一致で終了
                    }
                }

                // 特殊なケースの処理
                if expr[0] == "名詞" && expr[1] == "固有名詞" {
                    if match_count < 3 { // 固有名詞の場合、より詳細なマッチングを要求
                        is_valid_match = false;
                    }
                } else if expr[0] == "動詞" {
                    // 動詞の活用型のチェック
                    let verb_type = expr.get(4).unwrap_or(&"");
                    if verb_type.contains("五段") && key_parts.iter().any(|&k| k.contains("五段")) {
                        match_count += 1;
                    } else if verb_type.contains("四段") && key_parts.iter().any(|&k| k.contains("四段")) {
                        match_count += 1;
                    } else if verb_type.contains("一段") && key_parts.iter().any(|&k| k.contains("一段")) {
                        match_count += 1;
                    } else if verb_type.contains("カ変") && key_parts.iter().any(|&k| k.contains("カ変")) {
                        match_count += 1;
                    } else if verb_type.contains("サ変") && key_parts.iter().any(|&k| k.contains("サ変")) {
                        match_count += 1;
                    } else if verb_type.contains("ラ変") && key_parts.iter().any(|&k| k.contains("ラ変")) {
                        match_count += 1;
                    }
                }

                if is_valid_match && match_count > best_match.0 {
                    best_match = (match_count, id);
                }
            }
    }

    let result_id = if best_match.1 == -1 { _default_noun_id } else { best_match.1 };
    _id_def.insert(clsexpr.to_string(), result_id);
    class_map.insert(clsexpr.to_string(), result_id);
    result_id
}

// id.defは更新されうるので、毎回、最新のものを読み込む。
// 品詞判定が出来なかった場合、普通名詞とみなす。
// _default_noun_idは、その普通名詞のIDを格納しておく。
fn read_id_def(path: &Path) -> Result<(IdDef, i32), CsvError> {
    let mut hash = IdDef::new();
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b' ')
        .from_path(path)?;
    let mut _default_noun_id: i32 = -1;

    for result in reader.records() {
        let record = result?;
        let id: i32 = record[0].parse().unwrap();
        let mut expr = record[1].replace("名詞,一般", "名詞,普通名詞")
            .replace("名詞,数,", "名詞,数詞,")
            .replace("形-","形,")
            .replace("地域,","地名,");

        // 名詞、一般名詞のIDを保存
        if expr == "名詞,普通名詞,*,*,*,*,*" || expr == "名詞,一般,*,*,*,*,*" {
            _default_noun_id = id;
        }

        let mut re = Regex::new(r"五段・カ行[^,]*").unwrap();
        expr = re.replace(&expr, "五段・カ行").to_string();

        re = Regex::new(r"サ変([^,]*)").unwrap();
        let cap = match re.captures(&expr) {
            Some(i) => i.get(1).unwrap().as_str(),
            None => "",
        };
        if cap != "" {
            let mut s1 = String::from("サ変,");
            s1.push_str(cap);
            expr = re.replace(&expr, s1).to_string();
        };

        re = Regex::new(r"ラ行([^,]*)").unwrap();
        let cap = match re.captures(&expr) {
            Some(i) => i.get(1).unwrap().as_str(),
            None => "",
        };
        if cap != "" {
            let mut s1 = String::from("ラ行,");
            s1.push_str(cap);
            expr = re.replace(&expr, s1).to_string();
        };

        re = Regex::new(r"ワ行([^,]*)").unwrap();
        let cap = match re.captures(&expr) {
            Some(i) => i.get(1).unwrap().as_str(),
            None => "",
        };
        if cap != "" {
            let mut s1 = String::from("ワ行,");
            s1.push_str(cap);
            expr = re.replace(&expr, s1).to_string();
        };

        hash.insert(expr, id);
    }
    Ok((hash, _default_noun_id))
}

// ユーザー辞書の品詞と、id.defの品詞のマッピングを作成する
struct PosMapping {
    user_to_id_def: HashMap<String, Vec<String>>,
    id_def_to_user: HashMap<String, String>,
    id_to_user_word_class_cache: HashMap<i32, String>,
}

impl PosMapping {
    fn new() -> Self {
        Self {
            user_to_id_def: HashMap::new(),
            id_def_to_user: HashMap::new(),
            id_to_user_word_class_cache: HashMap::new(),
        }
    }

    fn add_mapping(&mut self, user_word_class: &str, id_def_word_class: &str) {
        self.user_to_id_def.entry(user_word_class.to_string())
            .or_insert_with(Vec::new)
            .push(id_def_word_class.to_string());
        self.id_def_to_user.insert(id_def_word_class.to_string(), user_word_class.to_string());
    }
}

// マッピング作成
fn create_word_class_mapping() -> PosMapping {
    let mut mapping = PosMapping::new();

    // ユーザー辞書の品詞とid.defの品詞のマッピングを追加
    mapping.add_mapping("固有名詞", "名詞,固有名詞,一般,*,*,*,*");
    mapping.add_mapping("組織", "名詞,固有名詞,組織,*,*,*,*");
    mapping.add_mapping("地名", "名詞,固有名詞,地名,一般,*,*,*");
    mapping.add_mapping("地名", "名詞,固有名詞,地域,一般,*,*,*");
    mapping.add_mapping("地名", "名詞,固有名詞,国,*,*,*,*");
    mapping.add_mapping("地名", "名詞,接尾,地域,*,*,*,*");
    mapping.add_mapping("人名", "名詞,固有名詞,人名,一般,*,*,*");
    mapping.add_mapping("名", "名詞,固有名詞,人名,名,*,*,*");
    mapping.add_mapping("姓", "名詞,固有名詞,人名,姓,*,*,*");
    mapping.add_mapping("接尾人名", "接尾辞,人名,*,*,*,*,*");
    mapping.add_mapping("接尾人名", "接尾辞,人名,*,*,*,*,女史");
    mapping.add_mapping("接尾地名", "接尾辞,地名,*,*,*,*,*");
    mapping.add_mapping("動詞カ行五段", "動詞,一般,*,*,五段・カ行,*,*");
    mapping.add_mapping("動詞カ変", "動詞,一般,*,*,カ変,*,*");
    mapping.add_mapping("動詞サ行五段", "動詞,一般,*,*,五段・サ行,*,*");
    mapping.add_mapping("動詞ハ行四", "動詞,非自立,*,*,四段・ハ行,*,*");
    mapping.add_mapping("動詞マ行五段", "動詞,一般,*,*,五段・マ行,*,*");
    mapping.add_mapping("動詞ラ行五段", "動詞,一般,*,*,五段・ラ行,*,*");
    mapping.add_mapping("動詞ワ行五段", "動詞,自立,*,*,五段・ワ行,*,*");
    mapping.add_mapping("動詞一段", "動詞,一般,*,*,一段,*,*");
    mapping.add_mapping("動詞サ変", "動詞,一般,*,*,サ変,*,*");
    mapping.add_mapping("動詞ラ変", "動詞,自立,*,*,ラ変,*,*");
    mapping.add_mapping("動詞五段", "動詞,一般,*,*,五段,*,*");
    mapping.add_mapping("名詞サ変", "名詞,普通名詞,サ変,可能,*,*,*");
    mapping.add_mapping("名詞サ変", "名詞,普通名詞,サ変,接続,*,*,*");
    mapping.add_mapping("名詞サ変", "名詞,普通名詞,サ変可能,*,*,*");
    mapping.add_mapping("名詞サ変", "名詞,普通名詞,サ変接続,*,*,*");

    mapping.add_mapping("形容詞", "形容詞,一般,*,*,形容詞,*,*");
    mapping.add_mapping("フィラー", "感動詞,フィラー,*,*,*,*,*");
    mapping.add_mapping("BOS/EOS", "BOS/EOS,*,*,*,*,*,*");
    mapping.add_mapping("その他", "その他,*,*,*,*,*,*");
    mapping.add_mapping("感動詞", "感動詞,*,*,*,*,*,*");
    mapping.add_mapping("助詞", "助詞,*,*,*,*,*,*");
    mapping.add_mapping("助動詞", "助動詞,*,*,*,*,*,*");
    mapping.add_mapping("終助詞", "助詞,終助詞,*,*,*,*,*");
    mapping.add_mapping("名詞", "名詞,普通名詞,*,*,*,*,*");
    mapping.add_mapping("固有名詞", "名詞,固有名詞,*,*,*,*,*");
    mapping.add_mapping("数", "名詞,数詞,*,*,*,*,*");
    mapping.add_mapping("助数詞", "名詞,数詞,*,*,*,*,*");
    mapping.add_mapping("接尾一般", "接尾辞,*,*,*,*,*,*");
    mapping.add_mapping("接続詞", "接続詞,*,*,*,*,*,*");
    mapping.add_mapping("接頭語", "接頭辞,*,*,*,*,*,*");
    mapping.add_mapping("副詞", "副詞,*,*,*,*,*,*");
    mapping.add_mapping("形容詞", "形容詞,*,*,*,*,*,*");
    mapping.add_mapping("記号", "補助記号,*,*,*,*,*,*");
    mapping.add_mapping("名詞形動", "形状詞,*,*,*,*,*,*");
    mapping.add_mapping("連体詞", "連体詞,*,*,*,*,*,*");
    mapping.add_mapping("動詞", "動詞,*,*,*,*,*,*");
    mapping.add_mapping("記号", "記号,*,*,*,*,*,*");

    mapping
}

// word_class_idからユーザー辞書の品詞の判定
fn get_user_word_class_by_id(mapping: &mut PosMapping, _id_def: &IdDef, word_class_id: i32) -> Option<String> {
    // キャッシュをチェック
    if let Some(cached_word_class) = mapping.id_to_user_word_class_cache.get(&word_class_id) {
        return Some(cached_word_class.clone());
    }
    let result = _id_def.iter()
        .find(|(_, &id)| id == word_class_id)
        .and_then(|(word_class, _)| {
            let parts: Vec<&str> = word_class.split(',').collect();
            let mut best_match: Option<(usize, &String)> = None;

            for (key, value) in &mapping.id_def_to_user {
                let key_parts: Vec<&str> = key.split(',').collect();
                let mut match_count = 0;
                let mut is_valid_match = true;

                // 特殊なケース（記号など）の処理
                if parts[0] == "記号" || parts[0] == "補助記号" {
                    if key_parts[0] == "記号" {
                        return Some(value.clone());
                    }
                    continue;
                }

                // 全項目のマッチングを試みる
                for (i, (a, b)) in parts.iter().zip(key_parts.iter()).enumerate() {
                    if *b != "*" && *a == *b {
                        match_count += 1;
                    } else if i < 2 { // 最初の2項目（品詞大分類、中分類）は必ずマッチする必要がある
                        is_valid_match = false;
                        break;
                    } else {
                        // 後半の項目（活用型など）が一致しない場合
                        // 完全一致でなくても、部分的な一致を許容する
                        if a.contains(b) || b.contains(a) {
                            match_count += 1;
                        }
                    }
                }

                // 固有名詞の場合、より詳細なマッチングを要求
                if parts[0] == "名詞" && parts[1] == "固有名詞" && match_count < 4 {
                    is_valid_match = false;
                }

                // 動詞の活用型のマッチング
                if parts[0] == "動詞" {
                    let verb_type = parts.get(4).unwrap_or(&"");
                    if verb_type.contains("五段") && key_parts.iter().any(|&k| k.contains("五段")) {
                        match_count += 1;
                    } else if verb_type.contains("四段") && key_parts.iter().any(|&k| k.contains("四段")) {
                        match_count += 1;
                    } else if verb_type.contains("一段") && key_parts.iter().any(|&k| k.contains("一段")) {
                        match_count += 1;
                    } else if verb_type.contains("カ変") && key_parts.iter().any(|&k| k.contains("カ変")) {
                        match_count += 1;
                    } else if verb_type.contains("サ変") && key_parts.iter().any(|&k| k.contains("サ変")) {
                        match_count += 1;
                    } else if verb_type.contains("ラ変") && key_parts.iter().any(|&k| k.contains("ラ変")) {
                        match_count += 1;
                    }
                }

                if is_valid_match && (best_match.is_none() || match_count > best_match.unwrap().0) {
                    best_match = Some((match_count, value));
                }
            }

            best_match.map(|(_, v)| v.clone())
        });
    // 結果をキャッシュに保存
    if let Some(ref word_class) = result {
        mapping.id_to_user_word_class_cache.insert(word_class_id, word_class.clone());
    }

    result
}

// id.defからキーを検索
fn search_key(def: &HashMap::<String, i32>, search: i32) -> String {
    for (key, value) in def {
        if value == &search {
            return key.to_string();
        } else {
            continue;
        }
    }
    return "".to_string();
}

// 品詞idからユーザー辞書の品詞を判定
fn u_search_key(mapping: &mut PosMapping, _id_def: &mut IdDef, word_class_id: i32) -> Option<String> {
    get_user_word_class_by_id(mapping, _id_def, word_class_id)
}

static KANA_CHECK: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[ぁ-ゖァ-ヺ]+$").unwrap());
static EISUU_CHECK: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z0-9\' ]+$").unwrap());
static KIGOU_CHECK: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z ]+$").unwrap());

fn is_kana(str: &str) -> bool {
    KANA_CHECK.is_match(&str)
}

fn is_eisuu(str: &str) -> bool {
    EISUU_CHECK.is_match(&str)
}

fn is_kigou(str: &str) -> bool {
    KIGOU_CHECK.is_match(&str)
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum DictionaryType {
    Sudachi,
    Neologd,
    UtDict,
    Default,
}

struct DictValues<'a> {
    id_def: &'a mut IdDef,
    default_noun_id: &'a mut i32,
    class_map: &'a mut HashMap::<String, i32>,
    mapping: &'a mut PosMapping,
    pronunciation: &'a mut String,
    notation: &'a mut String,
    word_class_id: &'a mut i32,
    cost: &'a mut i32,
}

trait DictionaryProcessor {
    fn should_skip(&self, _dict_values: &mut DictValues, record: &StringRecord, _args: &Config) -> bool;
    fn word_class_analyze(&self, _dict_values: &mut DictValues, record: &StringRecord, _args: &Config) -> bool;
}

struct DefaultProcessor;
impl DictionaryProcessor for DefaultProcessor {
    fn should_skip(&self, _dict_values: &mut DictValues, record: &StringRecord, _args: &Config) -> bool {
        let data = &record;
        if ! is_kana(&data[_args.pronunciation_index]) { return false };
        if ! _args.symbols && &data[_args.word_class_index] == "空白" { return false };
        if ! _args.symbols && &data[_args.pronunciation_index] == "キゴウ" { return false };
        if ! _args.symbols && is_kigou(&data[_args.notation_index]) { return false };
        if ! _args.places && data[_args.word_class_index+2].contains("地名") { return false };
        true
    }

    fn word_class_analyze(&self, _dict_values: &mut DictValues, record: &StringRecord, _args: &Config) -> bool {
        let data = &record;
        let mut _pronunciation: String = convert_to_hiragana(&data[_args.pronunciation_index]);
        let s1 = unicode_escape_to_char(&_pronunciation);
        let s2 = unicode_escape_to_char(&data[_args.notation_index]);
        let s3 = &data[_args.word_class_index].replace("補助記号", "記号"); //.replace("空白","記号");
        let s4 = &data[_args.word_class_index+1].replace("非自立可能","非自立"); //.replace(r"^数詞$", "数");
        let s5 = &data[_args.word_class_index+4].replace("下一段","一段").replace("一段-","一段,").replace("段-","段・");
        let s6 = &data[_args.word_class_index+5].replace("形-", "形,");
        let d: String = format!("{},{},{},{},{},{}", s3, s4, &data[_args.word_class_index+2], &data[_args.word_class_index+3], s5, s6);
        let word_class;
        word_class = _dict_values.class_map.get(&d);
        if word_class == None {
            *_dict_values.word_class_id = id_expr(&d, _dict_values.id_def, _dict_values.class_map, *_dict_values.default_noun_id);
        } else {
            *_dict_values.word_class_id = *word_class.unwrap();
        }
        if ! _args.places && search_key(_dict_values.id_def, *_dict_values.word_class_id).contains("地名") { return false }
        *_dict_values.pronunciation = s1;
        *_dict_values.notation = s2;
        let cost = data[_args.cost_index].parse::<i32>().unwrap();
        *_dict_values.cost = adjust_cost(cost);
        true
    }
}

struct SudachiProcessor;
impl DictionaryProcessor for SudachiProcessor {
    fn should_skip(&self, _dict_values: &mut DictValues, record: &StringRecord, _args: &Config) -> bool {
        let data = &record;
        if ! is_kana(&data[_args.pronunciation_index]) { return false };
        if ! _args.symbols && &data[_args.word_class_index] == "空白" { return false };
        if ! _args.symbols && &data[_args.pronunciation_index] == "キゴウ" && data[_args.word_class_index].contains("記号") { return false };
        if ! _args.symbols && is_kigou(&data[_args.notation_index]) && ! (&data[_args.word_class_index+1] == "固有名詞") { return false };
        // 地名を含む場合、オプション指定がなければ、英数のみの地名だけ残し、それ以外は省く。
        if ! _args.places {
            if ! is_eisuu(&data[_args.notation_index]) && data[_args.word_class_index+2].contains("地名") {
                return false;
            }
        };
        true
    }

    fn word_class_analyze(&self, _dict_values: &mut DictValues, record: &StringRecord, _args: &Config) -> bool {
        let data = &record;
        let mut _pronunciation: String = convert_to_hiragana(&data[_args.pronunciation_index]);
        let s1 = unicode_escape_to_char(&_pronunciation);
        let s2 = unicode_escape_to_char(&data[_args.notation_index]);
        let s3 = &data[_args.word_class_index].replace("補助記号", "記号"); //.replace("空白","記号");
        let s4 = &data[_args.word_class_index+1].replace("非自立可能","非自立"); //.replace(r"^数詞$", "数");
        let s5 = &data[_args.word_class_index+4].replace("下一段","一段").replace("一段-","一段,").replace("段-","段・");
        let s6 = &data[_args.word_class_index+5].replace("形-", "形,");
        let d: String = format!("{},{},{},{},{},{}", s3, s4, &data[_args.word_class_index+2], &data[_args.word_class_index+3], s5, s6);
        let word_class;
        word_class = _dict_values.class_map.get(&d);
        if word_class == None {
            *_dict_values.word_class_id = id_expr(&d, _dict_values.id_def, _dict_values.class_map, *_dict_values.default_noun_id);
        } else {
            *_dict_values.word_class_id = *word_class.unwrap();
        }
        *_dict_values.pronunciation = s1;
        *_dict_values.notation = s2;
        let cost = data[_args.cost_index].parse::<i32>().unwrap();
        *_dict_values.cost = adjust_cost(cost);
        true
    }
}

fn add_dict_data(_processor: &dyn DictionaryProcessor, _data: &StringRecord, _dict_values: &mut DictValues, dict_data: &mut DictionaryData, _args: &Config) {
        if _args.user_dict {
            match u_search_key(_dict_values.mapping, _dict_values.id_def, *_dict_values.word_class_id) {
                Some(word_class) => {
                    dict_data.add(DictionaryEntry {
                        key: DictionaryKey {
                            pronunciation: _dict_values.pronunciation.to_string(),
                            notation: _dict_values.notation.to_string(),
                            word_class_id: *_dict_values.word_class_id,
                        },
                        cost: *_dict_values.cost,
                        word_class: word_class,
                    }, true);
                },
                None => {
                    dict_data.add(DictionaryEntry {
                        key: DictionaryKey {
                            pronunciation: _dict_values.pronunciation.to_string(),
                            notation: _dict_values.notation.to_string(),
                            word_class_id: *_dict_values.word_class_id,
                        },
                        cost: *_dict_values.cost,
                        word_class: _dict_values.word_class_id.to_string(),
                    }, true);
                }
            }
        } else {
            dict_data.add(DictionaryEntry {
                key: DictionaryKey {
                    pronunciation: _dict_values.pronunciation.to_string(),
                    notation: _dict_values.notation.to_string(),
                    word_class_id: *_dict_values.word_class_id,
                },
                cost: *_dict_values.cost,
                word_class: "".to_string(),
            }, false);
        }
    }

struct NeologdProcessor;
impl DictionaryProcessor for NeologdProcessor {
    fn should_skip(&self, _dict_values: &mut DictValues, record: &StringRecord, _args: &Config) -> bool {
        let data = &record;
        if ! is_kana(&data[_args.pronunciation_index]) { return false };
        if &data[_args.word_class_index] == "空白" { return false };
        if &data[_args.pronunciation_index] == "キゴウ" && data[_args.word_class_index].contains("記号") { return false };
        if ! _args.symbols && is_kigou(&data[_args.notation_index]) && ! (&data[_args.word_class_index+1] == "固有名詞") { return false };
        if ! _args.places && data[_args.word_class_index+2].contains("地域") { return false };
        true
    }

    fn word_class_analyze(&self, _dict_values: &mut DictValues, record: &StringRecord, _args: &Config) -> bool {
        let data = &record;
        let mut _pronunciation: String = convert_to_hiragana(&data[_args.pronunciation_index]);
        let s1 = unicode_escape_to_char(&_pronunciation);
        let s2 = unicode_escape_to_char(&data[_args.notation_index]);
        let s3 = &data[_args.word_class_index];//.replace("補助記号", "記号"); //.replace("空白","記号");
        let s4 = if &data[_args.word_class_index] == "名詞" && &data[_args.word_class_index+1] == "一般" {
            "普通名詞"
        } else if &data[_args.word_class_index] == "名詞" && &data[_args.word_class_index+1] == "固有名詞" {
            &data[_args.word_class_index+1] // 固有名詞はそのまま保持
        } else {
            &data[_args.word_class_index+1]
        };
        let s5 = &data[_args.word_class_index+5];//.replace("形-", "形,");
        let d: String = format!("{},{},{},{},{},{}", s3, s4, &data[_args.word_class_index+2], &data[_args.word_class_index+3], &data[_args.word_class_index+4], s5);
        let word_class;
        word_class = _dict_values.class_map.get(&d);
        if word_class == None {
            *_dict_values.word_class_id = id_expr(&d, _dict_values.id_def, _dict_values.class_map, *_dict_values.default_noun_id);
        } else {
            *_dict_values.word_class_id = *word_class.unwrap();
        }
        if ! _args.places && search_key(_dict_values.id_def, *_dict_values.word_class_id).contains("地名") { return false }
        *_dict_values.pronunciation = s1;
        *_dict_values.notation = s2;
        let cost = data[_args.cost_index].parse::<i32>().unwrap();
        *_dict_values.cost = adjust_cost(cost);
        true
    }
}

struct UtDictProcessor;
impl DictionaryProcessor for UtDictProcessor {
    fn should_skip(&self, _dict_values: &mut DictValues, record: &StringRecord, _args: &Config) -> bool {
        let data = &record;
        if ! is_kana(&data[_args.pronunciation_index]) { return false };
        let word_class_id = data[_args.word_class_index].parse::<i32>().unwrap();
        *_dict_values.word_class_id = word_class_id;
        if ! _args.symbols && is_kigou(&data[_args.notation_index]) && ! search_key(_dict_values.id_def, word_class_id).contains("固有名詞") { return false };
        if ! _args.places && search_key(_dict_values.id_def, word_class_id).contains("地名") { return false }
        true
    }

    fn word_class_analyze(&self, _dict_values: &mut DictValues, record: &StringRecord, _args: &Config) -> bool {
        let data = &record;
        let word_class_id = data[_args.word_class_index].parse::<i32>().unwrap();
        let mut _pronunciation: String = convert_to_hiragana(&data[_args.pronunciation_index]);
        let s1 = unicode_escape_to_char(&_pronunciation);
        let s2 = unicode_escape_to_char(&data[_args.notation_index]);
        let d: String = format!("{}", search_key(_dict_values.id_def, word_class_id));
        let word_class;
        word_class = _dict_values.class_map.get(&d);
        if word_class == None {
            *_dict_values.word_class_id = id_expr(&d, _dict_values.id_def, _dict_values.class_map, *_dict_values.default_noun_id);
        } else {
            *_dict_values.word_class_id = *word_class.unwrap();
        }
        *_dict_values.pronunciation = s1;
        *_dict_values.notation = s2;
        let cost = data[_args.cost_index].parse::<i32>().unwrap();
        *_dict_values.cost = adjust_cost(cost);

        //let _word_class = search_key(_dict_values.id_def, word_class_id);
        //let d: String = format!("{}", _word_class);
        //let s1 = unicode_escape_to_char(&_pronunciation);
        //let s2 = unicode_escape_to_char(&data[_args.notation_index]);
        //*_dict_values.pronunciation = s1;
        //*_dict_values.notation = s2;
        //let cost = data[_args.cost_index].parse::<i32>().unwrap();
        //*_dict_values.cost = adjust_cost(cost);
        //eprintln!("{} {} {} {}",*_dict_values.pronunciation, *_dict_values.notation, *_dict_values.cost, _word_class);
        /*
        match u_search_key(_dict_values.mapping, _dict_values.id_def, *_dict_values.word_class_id) {
            Some(word_class) => {
                eprintln!("{} {} {} {}",*_dict_values.pronunciation, *_dict_values.notation, *_dict_values.word_class_id, word_class);
            }
            None => {
                eprintln!("None:{} {} {}",*_dict_values.pronunciation, *_dict_values.notation, *_dict_values.word_class_id);
            }
        }
        */
        true
    }
}

fn parse_delimiter(s: &str, args: &Config) -> u8 {
    match s {
        "t" | "\\t" | "\t" => b'\t',
        "," => b',',
        ";" => b';',
        " " => b' ',
        _ => {
            let chars: Vec<char> = s.chars().collect();
            if chars.len() == 1 {
                chars[0] as u8
            } else {
                if args.debug { eprintln!("Warning: Invalid delimiter '{}'. Using default ','.", s); }
                b','
            }
        }
    }
}

fn process_dictionary<P: AsRef<Path>>(
    path: P,
    _processor: &dyn DictionaryProcessor,
    id_def_path: &Path,
    dict_data: &mut DictionaryData,
    dict_type: DictionaryType,
    _args: &Config,
) -> ioResult<()> {
    let processor: Box<dyn DictionaryProcessor> = match dict_type {
        DictionaryType::Default => Box::new(DefaultProcessor),
        DictionaryType::Sudachi => Box::new(SudachiProcessor),
        DictionaryType::Neologd => Box::new(NeologdProcessor),
        DictionaryType::UtDict => Box::new(UtDictProcessor),
    };

    let (mut _id_def, mut _default_noun_id) = read_id_def(&id_def_path)?;
    let mut class_map = HashMap::<String, i32>::new();
    let mut mapping = create_word_class_mapping();
    let mut pronunciation = String::new();
    let mut notation = String::new();
    let mut word_class_id = -1;
    let mut cost = -1;

    let mut _dict_values = DictValues {
        id_def: &mut _id_def,
        default_noun_id: &mut _default_noun_id,
        class_map: &mut class_map,
        mapping: &mut mapping,
        pronunciation: &mut pronunciation,
        notation: &mut notation,
        word_class_id: &mut word_class_id,
        cost: &mut cost,
    };


    let delimiter_char = parse_delimiter(&_args.delimiter, &_args);

    let delimiter_str = if delimiter_char == b'\t' { 
        "TAB".to_string() 
    } else { 
        String::from_utf8(vec![delimiter_char]).unwrap_or_else(|_| "?".to_string())
    };
    if _args.debug { eprintln!("Using delimiter: {} {}", delimiter_str, delimiter_char.to_string()); }

    let reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(delimiter_char)
        .from_path(path);
    for result in reader?.records() {
        match result {
            Err(_err) => continue,
            Ok(record) => {
                let data = record;
                if ! processor.should_skip(&mut _dict_values, &data, &_args) { continue };
                if processor.word_class_analyze(&mut _dict_values, &data, &_args) {
                    add_dict_data(&*processor, &data, &mut _dict_values, dict_data, &_args);
                }
            }
        }
    }
    Ok(())
}

use argh::FromArgs;

#[derive(FromArgs)]
/// Dictionary to Mozc Dictionary Formats: a tool for processing dictionary files.
/// (Mozc辞書型式への変換プログラム)
struct Args {
    /// path to the dictionary CSV file
    #[argh(option, short = 'f')]
    csv_file: Option<PathBuf>,

    /// path to the Mozc id.def file
    #[argh(option, short = 'i')]
    id_def: Option<PathBuf>,

    /// generate Mozc User Dictionary formats
    #[argh(switch, short = 'U')]
    user_dict: bool,

    /// target SudachiDict
    #[argh(switch, short = 's')]
    sudachi: bool,

    /// target NEologd dictionary
    #[argh(switch, short = 'n')]
    neologd: bool,

    /// target UT dictionary
    #[argh(switch, short = 'u')]
    utdict: bool,

    /// include place names (地名を含める)
    #[argh(switch, short = 'p')]
    places: bool,

    /// include symbols (記号を含める)
    #[argh(switch, short = 'S')]
    symbols: bool,

    /// pronunciation 読みフィールドの位置（0から始まる）
    #[argh(option, short = 'P')]
    pronunciation_index: Option<usize>,

    /// notation 表記フィールドの位置（0から始まる）
    #[argh(option, short = 'N')]
    notation_index: Option<usize>,

    /// word class 品詞判定フィールドの位置（0から始まる）
    #[argh(option, short = 'W')]
    word_class_index: Option<usize>,

    /// cost コストフィールドの位置（0から始まる）
    #[argh(option, short = 'C')]
    cost_index: Option<usize>,

    /// delimiter デリミタ(初期値 ',' カンマ)
    #[argh(option, short = 'd')]
    delimiter: Option<String>,

    /// debug デバッグ
    #[argh(switch, short = 'D')]
    debug: bool,

}

#[derive(Debug)]
struct Config {
    csv_file: PathBuf,
    id_def: PathBuf,
    pronunciation_index: usize,
    notation_index: usize,
    word_class_index: usize,
    cost_index: usize,
    delimiter: String,
    sudachi: bool,
    utdict: bool,
    neologd: bool,
    user_dict: bool,
    places: bool,
    symbols: bool,
    debug: bool,
}

enum DictType {
    Sudachi,
    UTDict,
    NEologd,
    Default,
}

impl Args {
    fn into_config(self) -> std::io::Result<Config> {
        let current_dir = std::env::current_dir()?;
        let dict_type = if self.sudachi {
            DictType::Sudachi
        } else if self.utdict {
            DictType::UTDict
        } else if self.neologd {
            DictType::NEologd
        } else {
            DictType::Default
        };

        Ok(Config {
            csv_file: self.csv_file.unwrap_or_else(|| current_dir.join("all.csv")),
            id_def: self.id_def.unwrap_or_else(|| current_dir.join("id.def")),
            pronunciation_index: self.pronunciation_index.unwrap_or_else(|| dict_type.default_pronunciation_index()),
            notation_index: self.notation_index.unwrap_or_else(|| dict_type.default_notation_index()),
            word_class_index: self.word_class_index.unwrap_or_else(|| dict_type.default_word_class_index()),
            cost_index: self.cost_index.unwrap_or_else(|| dict_type.default_cost_index()),
            delimiter: self.delimiter.unwrap_or_else(|| dict_type.default_delimiter()),
            sudachi: self.sudachi,
            utdict: self.utdict,
            neologd: self.neologd,
            user_dict: self.user_dict,
            places: self.places,
            symbols: self.symbols,
            debug: self.debug,
        })
    }
}

impl DictType {
    fn default_pronunciation_index(&self) -> usize {
        match self {
            DictType::Sudachi => 11,
            DictType::NEologd => 12,
            DictType::UTDict => 0,
            DictType::Default => 11,
        }
    }

    fn default_notation_index(&self) -> usize {
        match self {
            DictType::Sudachi => 4,
            DictType::NEologd => 10,
            DictType::UTDict => 4,
            DictType::Default => 4,
        }
    }

    fn default_word_class_index(&self) -> usize {
        match self {
            DictType::Sudachi => 5,
            DictType::NEologd => 4,
            DictType::UTDict => 1,
            DictType::Default => 5,
        }
    }

    fn default_cost_index(&self) -> usize {
        match self {
            DictType::Sudachi => 3,
            DictType::NEologd => 3,
            DictType::UTDict => 3,
            DictType::Default => 3,
        }
    }

    fn default_delimiter(&self) -> String {
        match self {
            DictType::Sudachi => ",".to_string(),
            DictType::NEologd => ",".to_string(),
            DictType::UTDict => "\t".to_string(),
            DictType::Default => ",".to_string(),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = argh::from_env();
    let config = args.into_config()?;

    if config.debug { eprintln!("Config: {:?}", config); }

    // CSVファイルのパスを取得
    let csv_path = config.csv_file.clone();

    // id.defファイルのパスを取得
    let id_def_path = config.id_def.clone();

    // ファイルの存在チェック
    if !csv_path.exists() {
        eprintln!("Error: CSV file not found at {:?}", csv_path);
        return Err("CSV file not found".into());
    }

    if !id_def_path.exists() {
        eprintln!("Error: id.def file not found at {:?}", id_def_path);
        return Err("id.def file not found".into());
    }

    let mut dict_data = DictionaryData::new();

    // 辞書の読み込み処理
    if config.sudachi {
        process_dictionary(&csv_path, &SudachiProcessor, &id_def_path, &mut dict_data, DictionaryType::Sudachi, &config)?;
    } else if config.neologd {
        process_dictionary(&csv_path, &NeologdProcessor, &id_def_path, &mut dict_data, DictionaryType::Neologd, &config)?;
    } else if config.utdict {
        process_dictionary(&csv_path, &UtDictProcessor, &id_def_path, &mut dict_data, DictionaryType::UtDict, &config)?;
    } else {
        process_dictionary(&csv_path, &DefaultProcessor, &id_def_path, &mut dict_data, DictionaryType::Default, &config)?;
    }
    dict_data.output(config.user_dict)?;

    Ok(())
}
