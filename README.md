# wakamore

音楽に合わせて遊べる高速モグラ叩きゲーム

## このプロジェクトの目標

### 長期目標

より多い複雑なパターンのモグラ(ノーツ)を正確に叩くことにフォーカスした機能を実装する(記録管理、トレーニングモードなど)
BMSの主要なフォーマットを再生できる

### 短期目標

上からやる

#### ゲームエンジンをセットアップして、画面を表示できる

#### ダミーの譜面を表示できる

ここで使ったシーンがゲームプレイシーンになる

#### BMSファイルを読み込んで再生できる

#### キー入力を受け付ける

#### ゲームプレイができる

#### リザルトシーンを追加する

#### and more

## ワークスペース構成と開発手順

このリポジトリは Cargo workspace として構成されています。主な crate は以下の通りです。

- `common`: 共通型（イベント、トレイト）。Bevy の `Event` / `Message` を利用する型を定義します。
- `input`: キー入力の収集を担当し、低レベルの `common::InputEvent` を発行します（Bevy system）。
- `emitter`: `common::InputEvent` をゲーム側の `LaneInputEvent` 等に変換して発行します（ゲームパッドマッピングもここにあります）。
- `wakamore_render`: 描画・表示層の crate。カメラ初期化、UI 初期化、レーン表示、判定表示、タイトル/リザルト表示、および描画系 Plugin (`WakamoreRenderPlugin`) を提供します。
- ルート (`wakamore`): 実際のゲーム本体（`src/`）で、`input` / `emitter` / `common` を利用します。

簡単な開発コマンド:

```powershell
cargo check
cargo run --package wakamore
```

システムの連携:

- `input::poll_key_events` — キーのポーリングを `common::InputEvent` に書き込む Bevy system
- `emitter::input_events_to_lane_events` — `InputEvent` を `LaneInputEvent` に変換して発行
- `emitter::emit_gamepad_button_lane_input` — ゲームパッド入力を lane イベントに変換

移行ノート:

入力収集とイベント発行の責務は `input` / `emitter` に分離され、表示更新処理は `wakamore_render` に移設されています。`wakamore` はゲーム進行ロジックと状態遷移を中心に担当します。

問題や改善点:

- `common` は軽量に保ち、Bevy 型（`KeyCode` 等）を直接持たせています。将来的にエンジン依存を排除したい場合は型の抽象化を検討してください。

## テスト実行方法

ワークスペース全体の単体テストを実行するには、リポジトリルートで次のコマンドを実行してください。

```powershell
cargo test
```

特定のクレート（例: `emitter`）だけを実行する場合は次のようにします。

```powershell
cargo test -p emitter
```

テストは Bevy の `App` を利用したシステム単体／統合テストを含みます。テスト実行中にグラフィックやサウンドは使用せず、主に ECS とイベントの振る舞いを検証します。
