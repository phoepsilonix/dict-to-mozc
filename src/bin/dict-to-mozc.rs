use lib_dict_to_mozc::*;
use argh::FromArgs;
use std::process::ExitCode;
use std::ffi::OsString;
use std::path::PathBuf;


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

    /// debug デバッグ
    #[argh(switch, short = 'D')]
    debug: bool,

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
            pronunciation_index: self.pronunciation_index.unwrap_or_else(|| dict_type.default_pronunciation_index()),
            notation_index: self.notation_index.unwrap_or_else(|| dict_type.default_notation_index()),
            word_class_index: self.word_class_index.unwrap_or_else(|| dict_type.default_word_class_index()),
            word_class_numbers: self.word_class_numbers.unwrap_or_else(|| dict_type.default_word_class_numbers()),
            cost_index: self.cost_index.unwrap_or_else(|| dict_type.default_cost_index()),
            delimiter: self.delimiter.unwrap_or_else(|| dict_type.default_delimiter()),
            sudachi: self.sudachi,
            utdict: self.utdict,
            neologd: self.neologd,
            mozcuserdict: self.mozcuserdict,
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
            DictType::Default => ",".to_string(),
            DictType::Sudachi => ",".to_string(),
            DictType::NEologd => ",".to_string(),
            DictType::UTDict => "\t".to_string(),
            DictType::MozcUserDict => "\t".to_string(),
        }
    }
}

fn filter_args() -> Vec<OsString> {
    let args: Vec<OsString> = std::env::args_os().collect();

    let mut filtered_args = vec![args[0].clone()];

    let help_flags: Vec<OsString> = vec!["-h".into(), "--help".into(), "-?".into()];

    if args.len() <= 1 || args.iter().any(|arg| help_flags.contains(arg)) {
        filtered_args.push("--help".into());
    } else {
        filtered_args.extend(args.iter().skip(1).cloned());
    }

    filtered_args
}

pub fn main() -> ExitCode {
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
                },
                Err(()) => {
                    eprintln!("{}\nRun {} --help for more information.", early_exit.output, cmd);
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

    if config.debug {
        eprintln!("{:?}", config);
    }

    // CSVファイルとid.defファイルのパス取得
    let csv_path = config.csv_file.clone();
    let id_def_path = config.id_def.clone();

    // ファイルの存在チェック
    if !csv_path.exists() {
        eprintln!("Error: CSV file not found at {:?}", csv_path);
        return ExitCode::from(4);
    }

    if !id_def_path.exists() {
        eprintln!("Error: id.def file not found at {:?}", id_def_path);
        return ExitCode::from(5);
    }

    let mut dict_data = DictionaryData::new();

    // 辞書の読み込み処理
    let _processor: Box<dyn DictionaryProcessor> = if config.sudachi {
        Box::new(SudachiProcessor)
    } else if config.neologd {
        Box::new(NeologdProcessor)
    } else if config.utdict {
        Box::new(UtDictProcessor)
    } else if config.mozcuserdict {
        Box::new(MozcUserDictProcessor)
    } else {
        Box::new(DefaultProcessor)
    };

    let _ = process_dictionary(&csv_path, _processor.as_ref(), &id_def_path, &mut dict_data, &config);

    let _ = dict_data.output(config.user_dict);

    ExitCode::SUCCESS
}
