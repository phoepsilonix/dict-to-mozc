# [Mozc辞書型式への変換プログラ厶](https://github.com/phoepsilonix/dict-to-mozc)[^1]

## 目的
[Mozc](https://github.com/google/mozc)のパッケージ作成において、システム辞書として、有志が公開してくださっている辞書を含めることが目的です。  

現状、主に[SudachiDict](https://github.com/WorksApplications/SudachiDict)をシステム辞書として、組み込むことを目的とします。

このレポジトリでは、SudachiDictやMecabなどをはじめとする辞書データを、Mozcのシステム辞書およびユーザー辞書型式へ変換するプログラムを配布しています。  
対象としてSudachiDictをメインにメンテナンスしていますが、mecab-unidic-neologd,mecab-ipadic-neologdなどの型式のデータも、このプログラムで一応、変換できます。このプログラム自体は、MITライセンスとしています。

## 変換プログラム概要
+ Mozcソースのid.defは更新されうるものなので、id.defは最新のものを用意してください。
+ id.defを読み込み、その品詞と、ユーザー辞書で用いられている品詞をマッピングさせます。  
ユーザー辞書の品詞の分類に変更がない限り有効です。  
システム辞書型式に変換するためにも必要です。
+ -Uオプションを用いると、ユーザー辞書型式で出力されます。省略するとシステム辞書に組み込むための型式で出力されます。
+ SudachiDictなどの辞書データの品詞判定が行えなかった場合、普通名詞と判定されます。  
id.defでの`名詞,一般,*,*,*,*,*`扱いになります。  
Mozcの内部的な品詞IDは変わることがありますので、その時点でのMozcのid.defを用いることが大事です。ただユーザー辞書型式での出力の場合には、品詞名がそのまま出力されますので、あまり意識することはないでしょう。  
+ -s SudachiDict型式を指定します。-n Neologd,-u Ut Dictionary,-M Mozcユーザー辞書型式を指定できます。  
+ 辞書型式のどのオプション(-s,-n,-u,-M)も指定しない場合にも、若干、SudachiDictとスキップ条件を変更した基準で、データの変換を行います。
+ 重複チェックは、品詞ID、読み、表記の組み合わせで行っています。
ユーザー辞書型式への変換は、品詞IDが異なっても、同じ品詞名になりえるので、重複が残る場合があります。
+ Ut Dictionaryは、その時点のid.defの名詞の品詞IDを含めたデータとして以前は配布されていましたが、現在は品詞IDは`0000`となっているようです。品詞IDを`0`と判定すると品詞を`BOS/EOS`として扱ってしまいますが、それよりは名詞として扱ったほうが無難でしょうから、名詞判定にしています。
+ -pオプションを指定すると、出力データに地名も含めます。  
地域、地名として、分類されているデータへの扱いです。  
ただし日本語以外の地名は、オプション指定しなくても、そのまま出力されます。
+ -Sオプションは、出力に記号を含めます。  
記号、キゴウ、空白として、分類されているデータへの扱いです。  
キゴウの読みで、大量の類似のデータがあるため、それらを標準では対象外にしています。
なおオプション指定しなくても、固有名詞の場合は出力されます。
+ データにUnicode Escapeの記述が含まれる場合、それらも通常の文字に変換しています。
+ -P,-N,-W,-Cオプションを追加しました。読み込みフィールド位置を指定できます。
+ -dオプションでタブ区切りにも対応できます。  
読み込みにつかっているcsvクレートで用いるデリミタを指定できます。
```sh
Usage: dict-to-mozc [-f <csv-file>] [-i <id-def>] [-U] [-s] [-n] [-u] [-M] [-p] [-S] [-P <pronunciation-index>] [-N <notation-index>] [-W <word-class-index>] [-w <word-class-numbers>] [-C <cost-index>] [-d <delimiter>] [-D <debug>]

Dictionary to Mozc Dictionary Formats: a tool for processing dictionary files. (Mozc辞書型式への変換プログラム)

Options:
  -f, --csv-file    path to the dictionary CSV file(TSV with -d $'\t' or -d TAB)
  -i, --id-def      path to the Mozc id.def file(Default is ./id.def)
  -U, --user-dict   generate Mozc User Dictionary
                    formats(指定しない場合、Mozcシステム辞書型式で出力)
  -s, --sudachi     target SudachiDict
  -n, --neologd     target NEologd dictionary
  -u, --utdict      target UT dictionary
  -M, --mozcuserdict
                    target Mozc User Dictionary
  -p, --places      include place names (地名を含める)
  -S, --symbols     include symbols (記号を含める)
  -P, --pronunciation-index
                    pronunciation 読みフィールドの位置（0から始まる）
  -N, --notation-index
                    notation 表記フィールドの位置（0から始まる）
  -W, --word-class-index
                    word class 品詞判定フィールドの位置（0から始まる）
  -w, --word-class-numbers
                    word class 品詞判定フィールドのフィールド数
  -C, --cost-index  cost コストフィールドの位置（0から始まる）
  -d, --delimiter   delimiter デリミタ(初期値 ',' カンマ)
  -D, --debug       debug デバッグ(1: time, 2: config 3: DictonaryData)
  --help, help      display usage information
```

## 動作説明
下記のようなデータがあった場合、最初を0番目として、読みは11番目、表記は0番目または4番目、品詞の情報は5番目から10番目を使えば、良さそうです。

- 表記,品詞ID,品詞ID,COST,表記,品詞データ,,,,,,読み
```csv:test.csv
$,5152,5152,3757,$,名詞,普通名詞,助数詞可能,*,*,*,ドル,$,*,A,*,*,*,013707
$,5969,5969,2784,$,補助記号,一般,*,*,*,*,キゴウ,$,*,A,*,*,*,*
-,-1,-1,0,-,動詞,一般,*,*,五段-カ行,連体形-一般,ヒク,-,*,A,*,*,*,*
-,-1,-1,0,-,名詞,普通名詞,サ変可能,*,*,*,マイナス,-,*,A,*,*,*,*
-,-1,-1,0,-,名詞,普通名詞,一般,*,*,*,タイ,-,*,A,*,*,*,*
-,5969,5969,2625,-,補助記号,一般,*,*,*,*,-,-,*,A,*,*,*,*
.,-1,-1,0,.,名詞,普通名詞,一般,*,*,*,コンマ,.,*,A,*,*,*,*
.,-1,-1,0,.,名詞,普通名詞,一般,*,*,*,テン,.,*,A,*,*,*,*
```
上記のようなtest.csvがあった場合、下記のような手順でユーザー辞書型式へ変換できます。
```sh
# id.defの最新版を取得
curl -LO https://github.com/google/mozc/raw/refs/heads/master/src/data/dictionary_oss/id.def
# ユーザー辞書型式への変換
./target/release/dict-to-mozc -U -i ./id.def -f test.csv -P 11 -N 4 -W 5 -C 3 > user_dict.txt
```
-s,-n,-u,-Mオプションの、フィールドの扱いは、下記オプションと同等です。
```
# SudachiDict
-s -P 11 -N 12 -W 5 -w 6 -C 3
# Neologd(unidic)
-n -P 10 -N 12 -W 4 -w 6 -C 3
# Neologd(ipadic)
-n -P 12 -N 10 -W 4 -w 6 -C 3
# Ut Dictionary
-u -P 0 -N 4 -W 1 -w 1 -C 3
# Mozc User Dictionary
-M -P 0 -N 1 -W 2 -w 1 -C 3 -d $'\t'
```

## インストール

```sh
# ソースの取得
git clone --filter=tree:0 https://github.com/phoepsilonix/dict-to-mozc.git dict-to-mozc
cd dict-to-mozc

# rustプログラムのビルド
RUSTFLAGS="" cargo build --release -F use-mimalloc-rs
```
mimalloc-rustクレートの場合には、そのままビルドできるケースが多いとは思います。  
また特にFeaturesを指定しなくても、若干パフォーマンスが落ちるだけで、問題なく動作します。

```sh
RUSTFLAGS="" cargo build --release
```

## mimalloc-rs
```sh
RUSTFLAGS="" cargo build --release -F use-mimalloc-rs
```
## jemalloc
```sh
RUSTFLAGS="" cargo build --release -F use-jemalloc
```
## tcmalloc
```sh
RUSTFLAGS="" cargo build --release -F use-tcmalloc
```
* 実行時にtcmalloc.soが必要。  
  Ubuntu `libgoogle-perftools-dev` パッケージ
  ```sh
  sudo apt install libgoogle-perftools-dev
  ```
  Arch `gperftools` パッケージ
  ```sh
  sudo pacman install gperftools
  ```
## snmalloc
```sh
RUSTFLAGS="" cargo build --release -F use-snmalloc
```

* ビルド時にcmakeが必要。

## リリース版の設定
### v0.6.24
| プラットフォーム | OS | メモリアロケータ |
|----------------|----|----------------|
| x86_64, aarch64 | Linux | auto-allocator(mimalloc-rust) |
| x86_64, aarch64 | Windows | auto-allocator(mimalloc-rust) |
| x86_64, aarch64 | Mac | auto-allocator(mimalloc-rust) |

### v0.6.23
| プラットフォーム | OS | メモリアロケータ |
|----------------|----|----------------|
| x86_64, aarch64 | Linux | mimalloc-rust |
| x86_64, aarch64 | Windows | |
| x86_64, aarch64 | Mac | mimalloc-rust |

### v0.6.22
| プラットフォーム | OS | メモリアロケータ |
|----------------|----|----------------|
| x86_64, aarch64 | Linux | jemalloc |
| x86_64, aarch64 | Windows | |
| x86_64, aarch64 | Mac | jemalloc |

### v0.6.21
| プラットフォーム | OS | メモリアロケータ |
|----------------|----|----------------|
| x86_64 | Linux | tcmalloc |
| aarch64 | Linux | jemalloc |
| x86_64, aarch64 | Windows | |
| x86_64, aarch64 | Mac | jemalloc |

### ライブラリインストール
use-tcmalloc featuresを有効にした場合。
#### tcmalloc
##### Ubuntu, Debian
```sh
sudo apt install libgoogle-perftools-dev
```
##### Arch, Manjaro
```sh
sudo pacman -S gperftools
```

### ダウンロード
https://github.com/phoepsilonix/dict-to-mozc/releases からダウンロード
```sh
curl -LO https://github.com/phoepsilonix/dict-to-mozc/releases/latest/download/dict-to-mozc-x86_64-unknown-linux-gnu.tar.xz
tar xf dict-to-mozc-x86_64-unknown-linux-gnu.tar.xz --strip-components=1
ls -l ./dict-to-mozc
sudo cp ./dict-to-mozc /usr/local/bin/
```

### cargo binstallでインストール
cargo binstallをインストールしている場合。
`$HOME/.cargo/bin`にインストールされます。
```sh
cargo binstall --git https://github.com/phoepsilonix/dict-to-mozc.git dict-to-mozc
export PATH="$PATH":"$HOME/.cargo/bin"
```

### cargo-distで生成されたスクリプトでインストール
v0.5.6以降
```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/phoepsilonix/dict-to-mozc/releases/latest/download/dict-to-mozc-installer.sh | sh
```

### ソースからビルド＆インストール
Rustなどはあらかじめインストールしておいてください。  
`$HOME/.cargo/bin`にインストールされます。
#### その１
```sh
cargo install --git https://github.com/phoepsilonix/dict-to-mozc.git dict-to-mozc --profile release -F use-mimalloc-rs
which dict-to-mozc
```
#### その２
```sh
git clone https://github.com/phoepsilonix/dict-to-mozc.git
cd dict-to-mozc
RUSTFLAGS="" cargo build --release --target x86_64-unknown-linux-gnu -F use-mimalloc-rs
ls -l target/x86_64-unknown-linux-gnu/release/dict-to-mozc
cp target/x86_64-unknown-linux-gnu/release/dict-to-mozc ~/.cargo/bin/
```
私の環境だとtcmallocのほうが、若干パフォーマンスが良かったです。  
```sh
RUSTFLAGS="" cargo build --release --target x86_64-unknown-linux-gnu -F use-tcmalloc
```

#### 補足
gcc15などの場合、mimallocを有効にすると、cc-rsのビルドで失敗する場合があります。  
その場合、下記のような環境変数をCFLAGSに設定することで、gcc15でもmimallocを有効にしたままビルド可能です。
```sh
#環境変数CCにgccが含まれる場合、CFLAGSを設定する。
expr "$CC" : ".*gcc" >/dev/null && {
    for flag in -std=c11 -Wno-implicit-function-declaration -Wno-error=implicit-function-declaration; do
        case " $CFLAGS " in
            *" $flag "*) ;; # フラグが既にある場合はスキップ
            *) CFLAGS="$CFLAGS $flag" ;; # フラグを追加
        esac
    done
}
export CFLAGS
echo "$CFLAGS"
RUSTFLAGS="" cargo build --release --target x86_64-unknown-linux-gnu -F use-mimalloc-rs
```
#### 補足２
mold linkerを用いる場合のコマンド例。
```sh
RUSTFLAGS="-Clink-arg=-Bmold" cargo build --release --target x86_64-unknown-linux-gnu -F use-mimalloc-rs
```

### 使用例
https://github.com/WorksApplications/SudachiDict  
SudachiDictのそれぞれのファイルをまとめたものをsudachi.csvファイルとした場合の使用例です。
```sh
# SudachiDictダウンロード例
# 最新版の日付を確認
# curl -s https://api.github.com/repos/WorksApplications/SudachiDict/releases/latest|jq -r ".tag_name"|tr -d "v"
# curl -s 'http://sudachi.s3-website-ap-northeast-1.amazonaws.com/sudachidict-raw/' | htmlq "tbody tr:first-child td:first-child" --text
# curl -s 'http://sudachi.s3-website-ap-northeast-1.amazonaws.com/sudachidict-raw/' | xmllint --html --xpath '(//tbody/tr[1]/td[1]/text())[1]' - 2>/dev/null
# curl -s 'http://sudachi.s3-website-ap-northeast-1.amazonaws.com/sudachidict-raw/' | tail -n +2 | xq -r '.html.body.table.tbody.tr[0].td[0]'
# curl -s 'http://sudachi.s3-website-ap-northeast-1.amazonaws.com/sudachidict-raw/' | sed '/<!doctype.*>/Id' | xq -r '.html.body.table.tbody.tr[0].td[0]'

_sudachidict_date=$(curl -s 'http://sudachi.s3-website-ap-northeast-1.amazonaws.com/sudachidict-raw/' | grep -o '<td>[0-9]*</td>' | grep -o '[0-9]*' | sort -n | tail -n 1)
curl -LO "http://sudachi.s3-website-ap-northeast-1.amazonaws.com/sudachidict-raw/${_sudachidict_date}/small_lex.zip"
curl -LO "http://sudachi.s3-website-ap-northeast-1.amazonaws.com/sudachidict-raw/${_sudachidict_date}/core_lex.zip"
curl -LO "http://sudachi.s3-website-ap-northeast-1.amazonaws.com/sudachidict-raw/${_sudachidict_date}/notcore_lex.zip"
unzip small_lex.zip
unzip core_lex.zip
unzip notcore_lex.zip
cat small_lex.csv core_lex.csv notcore_lex.csv > sudachi.csv
```

```sh
# id.defの最新版を取得
curl -LO https://github.com/google/mozc/raw/refs/heads/master/src/data/dictionary_oss/id.def
# rustプログラムのビルド
RUSTFLAGS="" cargo build --release -F use-mimalloc-rs
# Mozcシステム辞書型式への変換
dict-to-mozc -s -i ./id.def -f sudachi.csv > sudachi-dict.txt
# Mozcユーザー辞書型式への変換
dict-to-mozc -U -s -i ./id.def -f sudachi.csv > sudachi-userdict.txt
```

### Neologdの例
https://github.com/neologd/mecab-unidic-neologd/  
https://github.com/neologd/mecab-ipadic-neologd/  
```sh
# unidic
curl -LO https://github.com/phoepsilonix/mecab-unidic-neologd/raw/refs/heads/master/seed/mecab-unidic-user-dict-seed.20200910.csv.xz
xz -k -d mecab-unidic-user-dict-seed.20200910.csv.xz
# Mozcシステム辞書型式への変換
dict-to-mozc -n -i ./id.def -f mecab-unidic-user-dict-seed.20200910.csv > mecab-unidic-dict.txt
# Mozcユーザー辞書型式への変換
dict-to-mozc -U -n -i ./id.def -f mecab-unidic-user-dict-seed.20200910.csv > mecab-unidic-userdict.txt
# ipadic
curl -LO https://github.com/phoepsilonix/mecab-ipadic-neologd/raw/refs/heads/master/seed/mecab-user-dict-seed.20200910.csv.xz
xz -k -d mecab-user-dict-seed.20200910.csv.xz
# Mozcシステム辞書型式への変換
dict-to-mozc -n -P 12 -N 10 -i ./id.def -f mecab-user-dict-seed.20200910.csv > mecab-ipadic-dict.txt
# Mozcユーザー辞書型式への変換
dict-to-mozc -U -n -P 12 -N 10 -i ./id.def -f mecab-user-dict-seed.20200910.csv > mecab-ipadic-userdict.txt
```

### Ut Dictionaryの例
https://github.com/utuhiro78/merge-ut-dictionaries
```sh
# 現在、品詞IDが`0000`とされているので、`0000`の場合には普通名詞と判定します。
# 1843が名詞,一般のid.defの取得
curl -L -o id2.def https://github.com/google/mozc/raw/8121eb870b66f26256995b42f069c9f4a8788953/src/data/dictionary_oss/id.def
# 例として、cannaのデータを取得
curl -LO https://github.com/utuhiro78/mozcdic-ut-alt-cannadic/raw/b35f78867c2853b552851f6ebba975860d938b55/mozcdic-ut-alt-cannadic.txt.tar.bz2
tar xf mozcdic-ut-alt-cannadic.txt.tar.bz2
# Mozcユーザー辞書型式への変換
dict-to-mozc -U -u -i ./id2.def -f mozcdic-ut-alt-cannadic.txt > canna-userdict.txt
# Mozcユーザー辞書型式からMozcシステム辞書型式への変換も加えたので、下記startideさんのスクリプトでcanna型式をMozcユーザー辞書型式に変換してから、dict-to-mozcでMozcシステ厶辞書型式へ変換すれば品詞情報も、それなりに残せます。
# また次のレポジトリでは、startideさんの品詞情報の対応表を元にした品詞判定を加えた変換スクリプトで、canna型式の辞書をMozcユーザー辞書型式に変換したあと、dict-to-mozcでMozcシステム辞書型式に変換することで、品詞情報をある程度残しています。
# https://github.com/phoepsilonix/mozcdic-ut-alt-cannadic

# 例としてskk jisyo
curl -LO https://github.com/utuhiro78/mozcdic-ut-skk-jisyo/raw/refs/heads/main/mozcdic-ut-skk-jisyo.txt.bz2
bzip2 -d mozcdic-ut-skk-jisyo.txt.bz2
# Mozcユーザー辞書型式への変換(id.defは最新のものでOK)
dict-to-mozc -U -u -i ./id.def -f mozcdic-ut-skk-jisyo.txt > skk-jisyo-userdict.txt
```

## 依存ライブラリの補足説明
読みのカタカナから平仮名への変換は、クレートの[kanaria](https://docs.rs/kanaria/latest/kanaria/)[^5]を用いています。  
なおkanariaについては、依存ライブラリを新しいライブラリへ対応させたものを用いました。  
クレートの[encoding_rs](https://docs.rs/encoding_rs/latest/encoding_rs/)と[unicode_normalization](https://docs.rs/unicode-normalization/latest/unicode_normalization/)を用いても、同等のことは可能です。ただkanariaを用いたほうがファイルサイズが小さくなりました。またパフォーマンス面も、ほぼ変わらないようです。  
データの読み込みは、[csv](https://docs.rs/csv/latest/csv/)クレートを用いてます。  
カナや英数記号の判定の正規表現の判定に、[lazy-regex](https://docs.rs/lazy-regex/latest/lazy_regex/)を用いてます。  
重複するエントリーを取り除くのに、[indexmap](https://docs.rs/indexmap/latest/indexmap/)を用いています。  
Global Allocatorとしてmimallocへのバインディングを提供している[mimalloc-rust](https://docs.rs/mimalloc-rust/latest/mimalloc_rust/)を用いることで、パフォーマンスの改善が行えました。cargo distで複数のアーキテクチャ向けにビルドする際、mimalloc-rustだと、どのアーキテクチャでもビルドエラーにならなかったので、こちらを採用しました。[現在](#%E3%83%AA%E3%83%AA%E3%83%BC%E3%82%B9%E7%89%88%E3%81%AE%E8%A8%AD%E5%AE%9Av0621)はビルド可能なものに関しては、tcmalloc、jemalloc、snmallocなども使用しています。


## ユーザー辞書として
SudachiDictをMozcユーザー辞書形式へ変換したものと、Neologdのunidic,ipadicを一つのMozcユーザー辞書形式にまとめたものの、2種類を次のサイトに公開しておきます。(Mozcのid.defに依存せずに品詞情報を含めた対応表として残せることが利点です。)  
[SudachiDict and Neologd Mozc ユーザー辞書](https://github.com/phoepsilonix/mozc-user-dictionary)

1. あまり巨大なファイルを取り込むと重くなるかもしれません。
1. 複数の辞書のエントリには、重複項目がかなりあることでしょう。

ただ上記2点のことを踏まえると、ユーザー辞書として、すべてを取り込むのは、使い勝手の面からも、よくないかもしれません。
実際に、全件を取り込むと、mozc_serverが応答しないなどの問題が発生しました。品詞などを選別して、特定のものに限ったほうがよさそうです。システム辞書に組み込む場合は、そこまで重くなっていないのですが、ユーザー辞書の用途として、やはり大量の件数を取り込むのは悪手なのでしょう。  
個人的な知り合いの氏名が出にくい、郵便番号辞書が更新されたが反映されていないなど、個別の案件で、そのユーザーが優先的に変換したいものを登録するのが、ユーザー辞書の本来のあり方だとも思います。システム辞書型式に変換するだけでなく、付属的にユーザー辞書への変換機能も追加しましたが、ユーザー辞書型式にして、取り込む場合には、元データをよく選別してから、使うほうが良さそうです。（2025年初頭に取り込まれた更新で、ユーザー辞書の機能も、若干、改善されているようです。）

下記サイトでMozcのシステム辞書として、SudachiDict[^6]とMeCab-unidic-NEologd[^7],MeCab-ipadic-NEologd[^8]を組み込んだものを用意しています。  
[Ubuntu:23.10(mantic) and Debian:12(bookworm)向けMozcパッケージ](https://github.com/phoepsilonix/mozc-deb/releases)[^2]  
[ArchLinux and ManjaroLinux向け Mozcパッケージ](https://github.com/phoepsilonix/mozc-arch/releases)[^3]  

タグに```with-jp-dict```がついているものが、SudachiDictやMeCab-unidic-neologd、MeCab-ipadic-neolodのデータをシステム辞書に組み込んだパッケージです。

## ユーザー辞書型式のデータの重複を取り除く場合
すべてのユーザー辞書のデータを、いったん一つのファイルにまとめておきます。
all.tsvという名前で保存しているとします。
### 読み、表記
品詞はあまり気にせず、読みと表記の組み合わせで重複を取り除く場合。
```sh
awk 'BEGIN{
    FS="\t"
    OFS="\t"
}
{
    if (!a[$1,$2]++) {
        print $0
    }
}' all.tsv > user_dict.txt
```

### 読み、表記、品詞
品詞も含めて、重複チェックする場合。
```sh
awk 'BEGIN{
    FS="\t"
    OFS="\t"
}
{
    if (!a[$1,$2,$3]++) {
        print $0
    }
}' all.tsv > user_dict.txt
```

### 行数が多いので、分割
1つのユーザー辞書に取り込める行数には上限があるので、分割します。  
分割したファイルをMozcの辞書ツールで取り込めます。
```sh
split --numeric-suffixes=1 -l 1000000 --additional-suffix=.txt user_dict.txt user_dict-
ls -l user_dict-*.txt
```

## その他のIMEとのユーザー辞書の相互変換
[Startide Project](https://startide.jp/)さんが公開されているRubyスクリプトを用いれば、Anthyなどの他IMEの辞書の型式との相互変換ができるみたいです。利用には、[オブジェクト指向スクリプト言語 Ruby](https://www.ruby-lang.org/ja/)とその拡張gemの[rexml](https://docs.ruby-lang.org/ja/latest/library/rexml.html)が必要です。
こちらを使う場合にも、それぞれのIMEの仕様にあわせて、上記のような形で`split`コマンドを用いて、件数（行数）を調整したほうがいいかもしれません。  
[userdic - 日本語入力ユーザー辞書変換スクリプト](https://startide.jp/comp/im/userdic/)[^4]

# その他、Mozc関連記事
[Mozc を応援するいくつかの方法](https://zenn.dev/komatsuh/articles/91def2bc633a8d)

[大概のLinuxで使えそうな日本語入力(Flatpak版Fcitx5-Mozc)](https://zenn.dev/phoepsilonix/articles/flatpak-mozc)  
[UbuntuでMozcの新しいバージョンをビルドするには](https://zenn.dev/phoepsilonix/articles/0c492a22a3c9d0)  
[Mozcをオフラインでビルドするには？](https://zenn.dev/phoepsilonix/articles/mozc-offiline-build)  

[郵便番号辞書 Mozc形式作成手順](https://zenn.dev/phoepsilonix/articles/japanese-zip-code-dictionary)  

[^1]: https://github.com/phoepsilonix/dict-to-mozc
[^2]: [Ubuntu:23.10(mantic) and Debian:12(bookworm)向けMozcパッケージ](https://github.com/phoepsilonix/mozc-deb/releases)
[^3]: [ArchLinux and ManjaroLinux向け Mozcパッケージ](https://github.com/phoepsilonix/mozc-arch/releases)
[^4]: [userdic - 日本語入力ユーザー辞書変換スクリプト](https://startide.jp/comp/im/userdic/)
[^5]: [samunohito/kanaria: このライブラリは、ひらがな・カタカナ、半角・全角の相互変換や判別を始めとした機能を提供します。](https://github.com/samunohito/kanaria)
[^6]: https://github.com/WorksApplications/SudachiDict
[^7]: https://github.com/neologd/mecab-unidic-neologd
[^8]: https://github.com/neologd/mecab-ipadic-neologd
