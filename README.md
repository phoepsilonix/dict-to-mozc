# 目的
mozcのパッケージ作成において、システム辞書として、有志が公開してくださっている辞書を含めることが目的です。  

現状、主にSudachiDictをシステム辞書として、組み込むことを目的とします。

sudachiフォルダ以下のrustプログラムが、SudachiDictをはじめとする辞書データを、Mozcのシステム辞書およびユーザー辞書型式へ変換するプログラムです。それ以外のスクリプトも残してありますが、sudachiフォルダのrustプログラムが現時点でメンテナンスされているものです。SudachiDict以外も、このプログラムで一応、変換できます。rustプログラムは、MITライセンスとしています。

## 変換プログラム概要
+ Mozcソースのid.defは更新されうるものなので、id.defは最新のものを用意してください。
+ id.defを読み込み、その品詞と、ユーザー辞書で用いられている品詞をマッピングさせます。  
ユーザー辞書の品詞の分類に変更がない限り有効です。
+ -Uオプションを用いると、ユーザー辞書型式で出力されます。省略するとシステム辞書に組み込むための型式で出力されます。
+ SudachiDictなどの辞書データの品詞判定が行えなかった場合、普通名詞と判定されます。  
id.defでの`名詞,一般,*,*,*,*,*`扱いになります。  
Mozcの内部的な品詞IDは変わることがありますので、その時点でのMozcのid.defを用いることが大事です。ただユーザー辞書型式での出力の場合には、品詞名がそのまま出力されますので、あまり意識することはないでしょう。  
+ -s SudachiDict型式を指定します。-u UtDict,-n Neologd型式を指定できます。  
+ UtDictは、それ自体が独自の品詞判定を行ったものを配布しています。そのデータが単純にユーザー辞書型式に変換されます。同じ時点id.defが使われている限りは、それなりに品詞判定が有効だと思います。  
+ Neologdやmecab-ipadicの型式も、多分、そのまま読み込んで、変換できます。品詞判定もそれなりにされると思います。
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

## 使用例
SudachiDictのそれぞれのファイルをまとめたものをall.csvファイルとした場合の使用例です。
```sh
cd sudachi
# id.defの最新のものを取得
curl -LO https://github.com/google/mozc/raw/refs/heads/master/src/data/dictionary_oss/id.def
# rustプログラムのビルド
cargo build --release
# システム辞書型式への変換
./target/release/dict-to-mozc -s -i ./id.def -f all.csv > all-dict.txt
# ユーザー辞書型式への変換
./target/release/dict-to-mozc -s -i ./id.def -f all.csv -U > all-userdict.txt
```

## Neologdの例
https://github.com/neologd/mecab-ipadic-neologd/
```sh
curl -LO https://github.com/neologd/mecab-ipadic-neologd/raw/refs/heads/master/seed/mecab-user-dict-seed.20200910.csv.xz
xz -k -d mecab-user-dict-seed.20200910.csv.xz
# システム辞書型式への変換
./target/release/dict-to-mozc -n -i ./id.def -f mecab-user-dict-seed.20200910.csv > mecab-dict.txt
# ユーザー辞書型式への変換
./target/release/dict-to-mozc -n -i ./id.def -f mecab-user-dict-seed.20200910.csv -U > mecab-userdict.txt
```
