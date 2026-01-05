このファイルはめちゃメモファイル
どこか間違ってるかも
# 次
- cargo install dora-cliでやりたい
  - cargo 重すぎて raspi Zero 2W に入らん
  - uv なら入る
- operator は コールバック ぽい

# コマンド
- dora coordinator
  - coordinator起動 (多分メイン機)
  - デフォポート 53290
- dora daemon --coordinator-addr [ipアドレス]
  - [ipアドレス]:53290 のcoordinatorに接続 
  - --coordinator-addrナシならlocalhost
  - --machine-id でマシンid指定可能
  - dataflow.yaml内で_unstable_deploy:から マシンid対応
- dora start [dataflowパス]
  - [dataflowパス] でスタート
  - デフォはlocalhostのcoordinatorで起動
- dora build [dataflowパス]
  - [dataflowパス]内の build: コマンドを実行
  - --local で このコマンド実行機体でbuild
  - coordinator起動時は各daemonで実行するため、--local使用したい
- dora up
  - coordinator + daemon 起動(localhostで)
- dora destroy
  - coordinator + daemon 終了
- dora run [dataflowパス]
  - coordinator + daemon + start 
  - オールインワンだが、なんか実行パスが変わる。[dataflowパス]内のpath 要変更


# 実行
## ソロ実行(dataflow_debug)
dora build ./dataflows/dataflow_debug.yaml
dora run ./dataflows/dataflow_debug.yaml

## マルチ実行
### メイン機
dora coordinator
dora daemon --machine-id pc

### 子機
dora daemon --machine-id rpi --coordinator-addr <coordinator_ip>


# うまくいかないとき
- Docker側のForwardAddressが原因
- ラズパイの実行権限
- dora coordinatorに接続後 dora buildすると、相手マシンでbuildしちゃう -> --local オプションで解決
- tracing-subscriber が エラー <- dora-node-api の tracing features が default で ON になっている。これを OFF にする。
- coordinator でWARN 時間の同期ずれ サービス内Windows Time を ON にして ターミナルで w32tm /resync
- https://zenoh.io/docs/getting-started/quick-test/
- udpのポートがランダムで変わる



# 開始方法 (どの機体でも デーモンを接続していたら？)
dora start ./dataflow.yaml

# コントローラー
- 同時押し+長押しで入力が残っちゃうバグあり <- おそらくOSの問題

# ログ
- 基本的に out/<dataflow_uuid>/log_<node_id>.txt に保存される。
- dora list で dataflow_name と dataflow_uuid を見れる。
- dora logs <name_or_uuid> <node_id> でログファイルを取り寄せる。
- 常にログを見てるわけではない と思う

# i2c
- sudo apt install i2c-tools
  sudo i2cdetect -y 1

# ラズパイ Rust インストール
- curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  - 1 Proceed with installation (default)
- cargo -V (source "$HOME/.cargo/env")
  - 失敗時
  - rustup toolchain remove stable-aarch64-unknown-linux-gnu
  - export RUSTUP_DIST_SERVER=https://rsproxy.cn
  - export RUSTUP_UPDATE_ROOT=https://rsproxy.cn/rustup
  - rustup toolchain install stable
  - rustup default stable