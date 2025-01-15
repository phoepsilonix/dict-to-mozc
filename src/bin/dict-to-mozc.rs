#[cfg(all(
    feature = "use-mimalloc",
    any(
        not(any(target_arch = "arm", target_arch = "aarch64")),
        all(target_arch = "aarch64", not(target_os = "windows"))
    )
))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(feature = "use-mimalloc-rs")]
#[global_allocator]
static GLOBAL_MIMALLOC: mimalloc_rust::GlobalMiMalloc = mimalloc_rust::GlobalMiMalloc;

#[cfg(all(
    feature = "use-snmalloc",
    any(
        not(any(target_arch = "arm", target_arch = "aarch64")),
        all(target_arch = "aarch64", not(target_os = "windows"))
    )
))]
#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

#[cfg(all(
    feature = "use-tcmalloc",
    any(
        not(any(target_arch = "arm", target_arch = "aarch64")),
        all(target_arch = "aarch64", not(target_os = "windows"))
    )
))]
#[global_allocator]
static GLOBAL: tcmalloc::TCMalloc = tcmalloc::TCMalloc;

extern crate argh;
extern crate lib_dict_to_mozc;

use argh::FromArgs;
use lib_dict_to_mozc::*;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use std::sync::{Arc, Mutex};

#[derive(FromArgs)]
/// Dictionary to Mozc Dictionary Formats: a tool for processing dictionary files.
/// (Mozc辞書型式への変換プログラム)
#[derive(Debug)]
struct Args {
    /// path to the dictionary CSV file(TSV with -d $'\t' or -d TAB)
    #[argh(option, short = 'f')]
    csv_file: Option<PathBuf>,

    /// path to the Mozc id.def file(Default is ./id.def)
    #[argh(option, short = 'i')]
    id_def: Option<PathBuf>,

    /// generate Mozc User Dictionary formats(指定しない場合、Mozcシステム辞書型式で出力)
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

    /// target Mozc User Dictionary
    #[argh(switch, short = 'M')]
    mozcuserdict: bool,

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

    /// word class 品詞判定フィールドのフィールド数
    #[argh(option, short = 'w')]
    word_class_numbers: Option<usize>,

    /// cost コストフィールドの位置（0から始まる）
    #[argh(option, short = 'C')]
    cost_index: Option<usize>,

    /// delimiter デリミタ(初期値 ',' カンマ)
    #[argh(option, short = 'd')]
    delimiter: Option<String>,

    /// threads
    #[argh(option, short = 'T')]
    threads: Option<usize>,

    /// chunk size
    #[argh(option, short = 'c')]
    chunk_size: Option<usize>,

    /// debug デバッグ(1: time, 2: config 3: DictonaryData)
    /// debug デバッグ(1: time, 2: config 3: DictonaryData)
    #[argh(option, short = 'D')]
    debug: Option<usize>,
}

enum DictType {
    Default,
    Sudachi,
    UTDict,
    NEologd,
    MozcUserDict,
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
        } else if self.mozcuserdict {
            DictType::MozcUserDict
        } else {
            DictType::Default
        };

        Ok(Config {
            csv_file: self.csv_file.unwrap_or_else(|| current_dir.join("all.csv")),
            id_def: self.id_def.unwrap_or_else(|| current_dir.join("id.def")),
            pronunciation_index: self
                .pronunciation_index
                .unwrap_or_else(|| dict_type.default_pronunciation_index()),
            notation_index: self
                .notation_index
                .unwrap_or_else(|| dict_type.default_notation_index()),
            word_class_index: self
                .word_class_index
                .unwrap_or_else(|| dict_type.default_word_class_index()),
            word_class_numbers: self
                .word_class_numbers
                .unwrap_or_else(|| dict_type.default_word_class_numbers()),
            cost_index: self
                .cost_index
                .unwrap_or_else(|| dict_type.default_cost_index()),
            delimiter: self
                .delimiter
                .unwrap_or_else(|| dict_type.default_delimiter()),
            sudachi: self.sudachi,
            utdict: self.utdict,
            neologd: self.neologd,
            mozcuserdict: self.mozcuserdict,
            user_dict: self.user_dict,
            places: self.places,
            symbols: self.symbols,
            threads: self.threads.unwrap_or(1),
            chunk_size: self.chunk_size.unwrap_or(10000),
            debug: self.debug.unwrap_or_else(|| dict_type.default_debug()),
        })
    }
}

impl DictType {
    fn default_pronunciation_index(&self) -> usize {
        match self {
            DictType::Default => 11,
            DictType::Sudachi => 11,
            DictType::NEologd => 10,
            DictType::UTDict => 0,
            DictType::MozcUserDict => 0,
        }
    }

    fn default_notation_index(&self) -> usize {
        match self {
            DictType::Default => 4,
            DictType::Sudachi => 12,
            DictType::NEologd => 12,
            DictType::UTDict => 4,
            DictType::MozcUserDict => 1,
        }
    }

    fn default_word_class_index(&self) -> usize {
        match self {
            DictType::Default => 5,
            DictType::Sudachi => 5,
            DictType::NEologd => 4,
            DictType::UTDict => 1,
            DictType::MozcUserDict => 2,
        }
    }

    fn default_word_class_numbers(&self) -> usize {
        match self {
            DictType::Default => 6,
            DictType::Sudachi => 6,
            DictType::NEologd => 6,
            DictType::UTDict => 1,
            DictType::MozcUserDict => 1,
        }
    }

    fn default_cost_index(&self) -> usize {
        match self {
            DictType::Default => 3,
            DictType::Sudachi => 3,
            DictType::NEologd => 3,
            DictType::UTDict => 3,
            DictType::MozcUserDict => 3,
        }
    }

    fn default_delimiter(&self) -> String {
        match self {
            DictType::Default => ",".to_owned(),
            DictType::Sudachi => ",".to_owned(),
            DictType::NEologd => ",".to_owned(),
            DictType::UTDict => "\t".to_owned(),
            DictType::MozcUserDict => "\t".to_owned(),
        }
    }

    fn default_debug(&self) -> usize {
        match self {
            DictType::Default => 0,
            DictType::Sudachi => 0,
            DictType::NEologd => 0,
            DictType::UTDict => 0,
            DictType::MozcUserDict => 0,
        }
    }
}

fn filter_args() -> Vec<OsString> {
    let args: Vec<OsString> = std::env::args_os().collect();
    let help_flags: Vec<OsString> = vec!["-h".into(), "--help".into(), "-?".into()];

    if args.len() <= 1 || args.iter().any(|arg| help_flags.contains(arg)) {
        vec![args[0].to_owned(), "--help".into()]
    } else {
        args
    }
}

/// WIP_main_function_description
pub fn main() -> ExitCode {
    let now = std::time::Instant::now();
    let filtered_args = filter_args();
    // OsStringを&strに変換する
    let args_slice: Vec<&str> = filtered_args
        .iter()
        .filter_map(|os_str| os_str.to_str())
        .collect();

    let cmd = args_slice.first().copied().unwrap_or("");

    // コマンド名のみでオプション指定がない場合、またはヘルプが指定されている場合、`--help`を渡す
    // それ以外は、すべてのオプションを渡す。
    let args = match Args::from_args(&[cmd], &args_slice[1..]) {
        Ok(args) => args,
        Err(early_exit) => {
            match early_exit.status {
                Ok(()) => {
                    println!("{}", early_exit.output);
                    return ExitCode::from(2); // ヘルプ表示時の終了コード
                }
                Err(()) => {
                    eprintln!(
                        "{}\nRun {} --help for more information.",
                        early_exit.output, cmd
                    );
                    return ExitCode::FAILURE; // コマンドオプションが不適切な場合の終了コード
                }
            }
        }
    };
    // argsを使ってconfigを生成
    let config = match args.into_config() {
        Ok(config) => config,
        Err(_) => {
            eprintln!("Failed to parse config");
            return ExitCode::from(3); // configのパースに失敗した場合の終了コード
        }
    };

    if config.debug > 1 {
        eprintln!("{:?}", config);
    }

    // ファイルの存在チェック
    if !config.csv_file.exists() {
        eprintln!("Error: CSV file not found at {:?}", &config.csv_file);
        return ExitCode::from(4);
    }

    // ファイルの存在チェック
    if !config.id_def.exists() {
        eprintln!("Error: id.def file not found at {:?}", &config.id_def);
        return ExitCode::from(5);
    }

    let dict_data = Arc::new(Mutex::new(DictionaryData::new()));

    // 辞書の読み込み処理
    let _processor: Arc<Box<dyn DictionaryProcessor>> = if config.sudachi {
        Arc::new(Box::new(SudachiProcessor))
    } else if config.neologd {
        Arc::new(Box::new(NeologdProcessor))
    } else if config.utdict {
        Arc::new(Box::new(UtDictProcessor))
    } else if config.mozcuserdict {
        Arc::new(Box::new(MozcUserDictProcessor))
    } else {
        Arc::new(Box::new(DefaultProcessor))
    };

    //let processor = Arc::new(Box::new(*_processor.as_ref()));
    let processor = Arc::clone(&_processor);
    let _ = process_dictionary(processor, dict_data.clone(), &config);

    let _ = dict_data.lock().unwrap().output(config.user_dict);

    if config.debug > 0 {
        let elp = now.elapsed();
        eprintln!("elapsed time: {elp:?}");
    }
    ExitCode::SUCCESS
}
