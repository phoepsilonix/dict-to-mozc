# Mozc辞書型式への変換プログラ厶

## 目的
mozcのパッケージ作成において、システム辞書として、有志が公開してくださっている辞書を含めることが目的です。  

現状、主にSudachiDictをシステム辞書として、組み込むことを目的とします。

このレポジトリでは、SudachiDictやMecabなどをはじめとする辞書データを、Mozcのシステム辞書およびユーザー辞書型式へ変換するプログラムを配布します。  
SudachiDictをメインにメンテナンスしていますが、mecab-ipadic-neologdなどの型式のデータも、このプログラムで一応、変換できます。このプログラム自体は、MITライセンスとしています。

## 変換プログラム概要
+ Mozcソースのid.defは更新されうるものなので、id.defは最新のものを用意してください。
+ id.defを読み込み、その品詞と、ユーザー辞書で用いられている品詞をマッピングさせます。  
ユーザー辞書の品詞の分類に変更がない限り有効です。  
システム辞書型式に変換するためには必要です。
+ -Uオプションを用いると、ユーザー辞書型式で出力されます。省略するとシステム辞書に組み込むための型式で出力されます。
+ SudachiDictなどの辞書データの品詞判定が行えなかった場合、普通名詞と判定されます。  
id.defでの`名詞,一般,*,*,*,*,*`扱いになります。  
Mozcの内部的な品詞IDは変わることがありますので、その時点でのMozcのid.defを用いることが大事です。ただユーザー辞書型式での出力の場合には、品詞名がそのまま出力されますので、あまり意識することはないでしょう。  
+ -s SudachiDict型式を指定します。-n Neologd,-u UtDict型式を指定できます。  
+ Neologdやmecab-ipadicの型式も、多分、そのまま読み込んで、変換できます。品詞判定もそれなりにされると思います。
+ Ut Dictionaryは、それ自体が独自の品詞判定を行った上で、Mozcの内部型式の品詞IDを含めたデータとして配布されています。その品詞IDデータを用いて、ユーザー辞書型式に変換できます。同じ時点のid.defが使われている限りにおいて、それなりに品詞判定のマッピングが有効だと思います。  
+ -Pオプションを指定すると、出力データに地名も含めます。  
ただし、英語名の地名は、オプション指定しなくても、そのまま出力されます。
地域、地名として、分類されているデータへの扱いです。
+ -Sオプションは、出力に記号を含めます。  
オプション指定しなくても、固有名詞の場合は出力されます。
記号、キゴウ、空白として、分類されているデータへの扱いです。
```
Usage: dict-to-mozc [-f <csv-file>] [-i <id-def>] [-U] [-s] [-n] [-u] [-P] [-S]

Dictionary to Mozc Dictionary Formats: a tool for processing dictionary files

Options:
  -f, --csv-file    path to the dictionary CSV file
  -i, --id-def      path to the Mozc id.def file
  -U, --user-dict   generate Mozc User Dictionary formats
  -s, --sudachi     target SudachiDict
  -n, --neologd     target NEologd dictionary
  -u, --utdict      target UT dictionary
  -P, --places      include place names (chimei)
  -S, --symbols     include symbols (kigou)
  --help            display usage information
```

### 使用例
https://github.com/WorksApplications/SudachiDict  
SudachiDictのそれぞれのファイルをまとめたものをsudachi.csvファイルとした場合の使用例です。
```sh
# SudachiDictダウンロード例
_sudachidict_date$(curl -s https://api.github.com/repos/WorksApplications/SudachiDict/releases/latest|jq ".tag_name")

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
# id.defの最新のものを取得
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

## 依存ライブラリの補足説明
読みのカタカナから平仮名への変換は、クレートのkanariaを用いています。  
なおkanariaについては、依存ライブラリを新しいライブラリへ対応させたものを用いました。  
クレートのencoding_rsとunicode-normalizationを用いても、同等のことは可能です。ただkanariaを用いたほうがファイルサイズが小さくなりました。またパフォーマンス面も、ほぼ変わらないようです。
