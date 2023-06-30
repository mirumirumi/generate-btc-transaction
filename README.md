# generate-btc-transaction

ビットコイン用の署名済みトランザクションを作成する CLI アプリケーションです。  
Rust で書かれています。

## 概要および制限事項

- テストネット専用
- Segwit 非対応
- ロックタイムの考慮はなし
- UTXO 関連のデータは自分で適切なものを用意し、アプリケーションに入力する
- 利用にネットワーク通信は必要ありません

## 動作確認の手順

※動的に入力パラメータを自動調整する機能はないため（特に UTXO 関連）、以下の実行は冪等性がなく二度目以降は有効なトランザクションにはなりません。  
（初回は動作するように既に書き込んであります）

入力はこちらです：

```sh
sh inputs/exec.sh
```

以下のように出力されます：

```txt
0x01000000011798d99e33691fe595ac0fb00224adf249657b5a8d8cf7574928accce7c4d70e010000006a473044022053f663276bf1673a32f55d213428983d5fdfa7146ac3884439475ee90257c21b02206b1ecc97e8601a67eaf13067a70386737b0f7d59b7e44817d927db17829275b301210303998660a6a026b2f8aa72d37a077b6a76b282b2d5b73fc582fdc274f66fa5bcffffffff0264000000000000001976a914a997f6d478624028ea1f36082e7ceb5d79d7567188acd41d0000000000001976a9143d927250d4a4744f5f99b499f750d85054dbf9fc88ac00000000
```

下記サイトにて手動でブロードキャストを実行できます（プレフィックスの `0x` は削除する必要があります）：

https://live.blockcypher.com/btc-testnet/pushtx/

## テストネットでの動作確認結果

下記の URL にて実際のブロードキャスト結果をご確認いただけます。  
（それぞれ別のトランザクションです）

https://blockstream.info/testnet/tx/6228dedb6c7f743114093934fce3577013d0673b9688ac1df30a942ca0bd999f

https://live.blockcypher.com/btc-testnet/tx/2a4bb3ed0533d9e1d4c896003e4ed0a1378e128c984632a0080b491ba316f3eb/

https://live.blockcypher.com/btc-testnet/tx/d73ebea9ad590316b5fbae5a176937178cdba72c1422a1636817a8f864a9c331/
