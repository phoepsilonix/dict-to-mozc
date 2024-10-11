# [Mozc辞書型式への変換プログラ厶](https://github.com/phoepsilonix/dict-to-mozc)[^1]

## 目的
[Mozc](https://github.com/google/mozc)のパッケージ作成において、システム辞書として、有志が公開してくださっている辞書を含めることが目的です。  

現状、主に[SudachiDict](https://github.com/WorksApplications/SudachiDict)をシステム辞書として、組み込むことを目的とします。

このレポジトリでは、SudachiDictやMecabなどをはじめとする辞書データを、Mozcのシステム辞書およびユーザー辞書型式へ変換するプログラムを配布しています。  
対象としてSudachiDictをメインにメンテナンスしていますが、mecab-ipadic-neologdなどの型式のデータも、このプログラムで一応、変換できます。このプログラム自体は、MITライセンスとしています。

## 変換プログラム概要
+ Mozcソースのid.defは更新されうるものなので、id.defは最新のものを用意してください。
+ id.defを読み込み、その品詞と、ユーザー辞書で用いられている品詞をマッピングさせます。  
ユーザー辞書の品詞の分類に変更がない限り有効です。  
システム辞書型式に変換するためにも必要です。
+ -Uオプションを用いると、ユーザー辞書型式で出力されます。省略するとシステム辞書に組み込むための型式で出力されます。
+ SudachiDictなどの辞書データの品詞判定が行えなかった場合、普通名詞と判定されます。  
id.defでの`名詞,一般,*,*,*,*,*`扱いになります。  
Mozcの内部的な品詞IDは変わることがありますので、その時点でのMozcのid.defを用いることが大事です。ただユーザー辞書型式での出力の場合には、品詞名がそのまま出力されますので、あまり意識することはないでしょう。  
+ -s SudachiDict型式を指定します。-n Neologd,-u Ut Dictionary型式を指定できます。  
+ 辞書型式のどのオプション(-s,-n,-u)も指定しない場合にも、若干、SudachiDictとスキップ条件を変更した基準で、データの変換を行います。
+ 重複チェックは、品詞ID、読み、表記の組み合わせで行っています。
+ mecab-ipadic-neologdの型式も、そのまま読み込んで、変換できます。品詞判定もそれなりにされると思います。
+ Ut Dictionaryは、それ自体が独自の品詞判定を行った上で、Mozcの内部型式の品詞IDを含めたデータとして配布されています。その品詞IDデータを用いて、ユーザー辞書型式に変換できます。同じ時点のid.defが使われている限りにおいて、それなりに品詞判定のマッピングが有効だと思います。つまり古めのアーカイブの場合には、その時点のid.defを取得して用いれば、品詞判定が改善するでしょう。実用性において、どの程度影響があるかは別ですが、id.defの番号がずれている場合、そのまま辞書に組み込むと、品詞としてのデータはミスマッチが起きる場合もあるでしょう。
+ -pオプションを指定すると、出力データに地名も含めます。  
地域、地名として、分類されているデータへの扱いです。  
ただし、SudachiDictの英語名の地名は、オプション指定しなくても、そのまま出力されます。
+ -Sオプションは、出力に記号を含めます。  
記号、キゴウ、空白として、分類されているデータへの扱いです。  
キゴウの読みで、大量の類似のデータがあるため、それらを標準では対象外にしています。
なおオプション指定しなくても、固有名詞の場合は出力されます。
+ データにUnicode Escapeの記述が含まれる場合、それらも通常の文字に変換しています。
+ -P,-N,-W,-Cオプションを追加しました。読み込みフィールド位置を指定できます。
+ -dオプションでタブ区切りにも対応できます。  
読み込みにつかっているcsvクレートで用いるデリミタを指定できます。
```
Usage: dict-to-mozc [-f <csv-file>] [-i <id-def>] [-U] [-s] [-n] [-u] [-p] [-S] [-P <pronunciation-index>] [-N <notation-index>] [-W <word-class-index>] [-C <cost-index>] [-d <delimiter>] [-D]

Dictionary to Mozc Dictionary Formats: a tool for processing dictionary files. (Mozc辞書型式への変換プログラム)

Options:
  -f, --csv-file    path to the dictionary CSV file(TSV with -d $'\t' or -d TAB)
  -i, --id-def      path to the Mozc id.def file(Default is ./id.def)
  -U, --user-dict   generate Mozc User Dictionary formats
  -s, --sudachi     target SudachiDict
  -n, --neologd     target NEologd dictionary
  -u, --utdict      target UT dictionary
  -p, --places      include place names (地名を含める)
  -S, --symbols     include symbols (記号を含める)
  -P, --pronunciation-index
                    pronunciation 読みフィールドの位置（0から始まる）
  -N, --notation-index
                    notation 表記フィールドの位置（0から始まる）
  -W, --word-class-index
                    word class 品詞判定フィールドの位置（0から始まる）
  -C, --cost-index  cost コストフィールドの位置（0から始まる）
  -d, --delimiter   delimiter デリミタ(初期値 ',' カンマ)
  -D, --debug       debug デバッグ
  --help            display usage information

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
# rustプログラムのビルド
cargo build --release
# ユーザー辞書型式への変換
./target/release/dict-to-mozc -U -i ./id.def -f test.csv -P 11 -N 4 -W 5 -C 3 > user_dict.txt
```
-s,-n,-uオプションの、フィールドの扱いは、下記オプションと同等です。
```
# SudachiDict
-s -P 11 -N 4 -W 5 -C 3
# Neologd
-n -P 12 -N 10 -W 4 -C 3
# Ut Dictionary
-u -P 0 -N 4 -W 1 -C 3
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
cargo build --release
# システム辞書型式への変換
./target/release/dict-to-mozc -s -i ./id.def -f sudachi.csv > sudachi-dict.txt
# ユーザー辞書型式への変換
./target/release/dict-to-mozc -s -i ./id.def -f sudachi.csv -U > sudachi-userdict.txt
```

### Neologdの例
https://github.com/neologd/mecab-ipadic-neologd/  
```sh
curl -LO https://github.com/neologd/mecab-ipadic-neologd/raw/refs/heads/master/seed/mecab-user-dict-seed.20200910.csv.xz
xz -k -d mecab-user-dict-seed.20200910.csv.xz
# システム辞書型式への変換
./target/release/dict-to-mozc -n -i ./id.def -f mecab-user-dict-seed.20200910.csv > mecab-dict.txt
# ユーザー辞書型式への変換
./target/release/dict-to-mozc -n -i ./id.def -f mecab-user-dict-seed.20200910.csv -U > mecab-userdict.txt
```

### Ut Dictionaryの例
https://github.com/utuhiro78/merge-ut-dictionaries
```sh
# 1843が名詞,一般のid.defの取得
curl -L -o id2.def https://github.com/google/mozc/raw/8121eb870b66f26256995b42f069c9f4a8788953/src/data/dictionary_oss/id.def
# 例として、cannaのデータを取得
curl -LO https://github.com/utuhiro78/mozcdic-ut-alt-cannadic/raw/refs/heads/main/mozcdic-ut-alt-cannadic.txt.tar.bz2
tar xf mozcdic-ut-alt-cannadic.txt.tar.bz2
# ユーザー辞書型式への変換
./target/release/dict-to-mozc -u -U -i ./id2.def -f mozcdic-ut-alt-cannadic.txt > canna-userdict.txt
```

## 依存ライブラリの補足説明
読みのカタカナから平仮名への変換は、クレートのkanariaを用いています。  
なおkanariaについては、依存ライブラリを新しいライブラリへ対応させたものを用いました。  
クレートのencoding_rsとunicode-normalizationを用いても、同等のことは可能です。ただkanariaを用いたほうがファイルサイズが小さくなりました。またパフォーマンス面も、ほぼ変わらないようです。
csvクレートで読み込んでいます。

## ユーザー辞書として
1. あまり巨大なファイルを取り込むと重くなるかもしれません。
1. 複数の辞書のエントリには、重複項目がかなりあることでしょう。

上記2点のことを踏まえると、ユーザー辞書として、すべてを取り込むのは、使い勝手の面からも、よくないかもしれません。
そのためMozcのシステム辞書として、SudachiDictを組み込んだものを下記に用意しています。

[Ubuntu:23.10(mantic) and Debian:12(bookworm)向けMozcパッケージ](https://github.com/phoepsilonix/mozc-deb/releases)[^2]  
[ArchLinux and ManjaroLinux向け Mozcパッケージ](https://github.com/phoepsilonix/mozc-arch)[^3]  

タグに```with-jp-dict```がついているものは、[SudachiDict](https://github.com/WorksApplications/SudachiDict)のデータをシステム辞書に組み込んだパッケージです。  

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
ユーザー辞書の取り込める行数には上限があるので、分割します。  
分割したファイルをMozcの辞書ツールで取り込めます。
```sh
split --numeric-suffixes=1 -l 1000000 --additional-suffix=.txt user_dict.txt user_dict-
ls -l user_dict-*.txt
```

# その他、Mozc関連記事
[Mozc を応援するいくつかの方法](https://zenn.dev/komatsuh/articles/91def2bc633a8d)

[大概のLinuxで使えそうな日本語入力(Flatpak版Fcitx5-Mozc)](https://zenn.dev/phoepsilonix/articles/flatpak-mozc)  
[UbuntuでMozcの新しいバージョンをビルドするには](https://zenn.dev/phoepsilonix/articles/0c492a22a3c9d0)  
[Mozcをオフラインでビルドするには？](https://zenn.dev/phoepsilonix/articles/mozc-offiline-build)  

[郵便番号辞書 Mozc形式作成手順](https://zenn.dev/phoepsilonix/articles/japanese-zip-code-dictionary)  

[^1]: https://github.com/phoepsilonix/dict-to-mozc
[^2]: [Ubuntu:23.10(mantic) and Debian:12(bookworm)向けMozcパッケージ](https://github.com/phoepsilonix/mozc-deb/releases)
[^3]: [ArchLinux and ManjaroLinux向け Mozcパッケージ](https://github.com/phoepsilonix/mozc-arch)
