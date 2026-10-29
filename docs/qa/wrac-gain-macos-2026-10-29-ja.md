# WRAC Gain の macOS ホスト動作確認（2026-10-29）

## 対象と範囲

- 検証対象は `c770b6a76b712779ab9553b108106ed80179ea7e` の WRAC Gain 0.1.0、release ビルド。
- macOS 14.8.5、Apple Silicon、Xcode 15.2、AAX SDK 2.9.0。AU / VST3 は Apple Silicon と Rosetta / Intel のバイナリーも確認した。
- この記録は実施した条件での結果を示す。全ホストや全サンプルレート、全バッファー長の正常動作を保証するものではない。
- 小さな数値・検査結果を [evidence](evidence/wrac-gain-macos-2026-10-29/) に保存した。ホストのプロジェクト、音声、スクリーンショット、ホストの詳細ログ、個人を特定し得る環境情報は公開対象に含めていない。

## ホスト別の結果

| ホスト・形式 | 確認結果と範囲 |
| --- | --- |
| GarageBand 10.4.8 / AU | 挿入、専用 GUI、ゲイン設定、再生、バイパス、保存・再読込を確認。44.1 kHz / 24 bit / stereo 書き出しの入力に対する倍率は、Gain −6 dB で左右とも約 0.501187、バイパスで 1.0。4096 samples の遅延を状態復元し、再生継続と保存後の復元を確認。遅延 0 と 4096 のオフライン書き出しは左右それぞれ 529,200 samples すべてが一致。オートメーションと GUI の遅延選択操作は未確認。 |
| REAPER 6.46 / AU・VST3 | GUI とホストのゲイン操作の相互反映、保存・再読込を確認。再生中に遅延を 0 ↔ 4096 samples と変えた際の表示更新も確認。44.1 kHz / 24 bit / stereo のオフライン書き出しで、遅延 4096 samples の経路と逆位相の基準経路は完全に相殺し、同相の対照は非ゼロ。VST3 では 4096 samples の保存・再読込も確認。 |
| Ableton Live 12.4.6 / AU・VST3 | GUI、ゲイン処理、デバイス無効化、ゲインの保存・再読込、オートメーション値の追従を確認。約 −7 dB の設定でトラックのピークは約 7 dB 減衰した。AU の新規挿入時に以下の表示不一致を観測。オートメーションの音声はサンプル単位では比較していない。 |
| Pro Tools Developer 23.6.0.5 / AAX Native | 未署名の検証ビルドを挿入し、専用 GUI とゲイン処理を確認。ホストのバイパスを操作しても処理が続く問題を再現。詳細は下記。使用した Developer 版では Save と Bounce Mix が無効だったため、セッションの再読込や書き出し比較は未確認。 |

GarageBand の [ゲイン・バイパスの数値](evidence/wrac-gain-macos-2026-10-29/garageband-gain-bypass-analysis.txt)、[遅延時の書き出し比較](evidence/wrac-gain-macos-2026-10-29/garageband-latency-analysis.txt)、[REAPER の逆位相比較](evidence/wrac-gain-macos-2026-10-29/reaper-latency-analysis.txt) を保存した。Pro Tools の結果は DAW の丸められたピーク表示によるものである。

## 問題・要調査

### Pro Tools / AAX：ホストのバイパスが音声に反映されない

1. ステレオトラックに WRAC Gain を挿入し、1 kHz / peak −20 dBFS の入力と Gain −6.0 dB を設定する。ホストにも Gain −6.0 dB が表示され、再生時のピークは −26.0 dBFS。
2. 挿入欄のホストバイパスをオンにする。挿入欄は `bypassed`、プラグインウィンドウの Effect Bypass は `on` と表示されるが、ピークは −26.0 dBFS のまま。公開 Parameter Bypass は `Off` のまま。
3. プラグインを `inactive` にするとピークは −20.0 dBFS に戻る。再度アクティブにすると、バイパス表示がオンのままピークは再び −26.0 dBFS。各測定前にピークをリセットした。

検証対象の AAX wrapper は master bypass ID として `MasterBypass` を登録する。一方、AAX SDK 2.9.0 の既定 `GetMasterBypassParameter` は `MasterBypassID` が登録されたときに同 ID を返す。SDK 実装を使った [独立検査](evidence/wrac-gain-macos-2026-10-29/aax-master-bypass-id-check.log) では、`MasterBypass` を登録した場合は空文字、`MasterBypassID` を登録した場合はその ID が返った。この不一致は有力な原因候補である。

ID だけを変えた比較 AAX は作成したが、ホストへの挿入を完了できず、実 DAW での修正効果は未確認。上記の再生結果は全サンプルの波形比較ではない。[測定の記録](evidence/wrac-gain-macos-2026-10-29/aax-bypass-observation.txt)

### Ableton Live / AU：新規挿入直後のバイパス表示と処理の不一致

新規 AU インスタンス 2 回で、ホストのパラメーター表示は初めから Bypass `On` だったが、Gain 約 −7 dB の音声は減衰した。Bypass を 0 にしてから 1 にすると、音声は元のレベルに戻った。

一方、[直接 AU 検査](evidence/wrac-gain-macos-2026-10-29/au-bypass-enum-check.log) では、初期バイパスパラメーターと BypassEffect プロパティはともに 0、音声処理と 0 → 1 → 0 の切り替えは期待どおりだった。値から文字列への変換は 0 → `Off`、1 → `On`。Indexed パラメーターの `ParameterValueStrings` は未提供だったが、このプロパティは任意であり、表示不一致との因果関係も未確認。原因をホストまたは wrapper のどちらかに断定していない。

### AU 再起動テストの不足

既存の再起動テストは遅延値の通知・適用を確認するが、並行 render が返す無音バッファーの内容を判定していない。追加の直接 AU 検査では非ゼロ入力と事前に非ゼロで埋めた出力を使って 60 回の遅延変更を実行した。無音フラグ付きブロックを Apple Silicon で 60 / 60 区間、Rosetta で 53 / 60 区間捕捉し、両チャンネルの全サンプルがゼロであることを確認した（[Apple Silicon](evidence/wrac-gain-macos-2026-10-29/au-restart-silence-arm64.log)、[Rosetta](evidence/wrac-gain-macos-2026-10-29/au-restart-silence-x86_64.log)）。

この追加検査は恒久テストではない。また、各ブロックが activate / deactivate の瞬間と重なったことや、リアルタイムの期限を満たしたことは保証しない。既存の状態復元テストでは ClassInfo 通知が発生せず、並行復元分岐は SKIP した。

### Pro Tools の挿入メニューで一度観測した例外

別の試行では挿入メニュー操作中にホストが応答を止め、クラッシュレポートにホストの `TMenu::PopUp` 内での NULL 参照が記録された。スタックには WRAC の処理関数がなく、NULL の発生原因は未特定。WRAC を置かない比較では同じ例外を観測しなかったが、操作順は完全には一致しない。後の WRAC 挿入・再生・バイパス確認では再発しなかったため、WRAC との因果関係は判断できない。

## 自動・直接ホスト検証

| 検査 | 結果 |
| --- | --- |
| `cargo test --workspace` | 78 passed / 0 failed。 |
| `cargo xtask validate --release --target=clap,vst3` | CLAP validator 34 passed / 0 failed / 10 skipped、VST3 SDK validator 47 passed / 0 failed。 |
| pluginval 1.0.4、strictness 8 | Apple Silicon の AU・VST3 と Rosetta / Intel の VST3 が成功。Intel AU は登録一覧で発見できず未完了。 |
| auval | 成功。 |
| AU 状態復元・再起動テスト | Apple Silicon / Intel とも基本ケースは成功。状態復元の並行分岐は上記のとおり SKIP。 |
| 直接 CLAP ホスト | 44.1 / 48 / 96 kHz × 64 / 128 / 512 / 1024 frames の 12 組で、ステレオのゲイン、サンプル位置を指定したパラメーター切り替え、バイパス、状態保存・復元を確認。全サンプルを期待値と比較した（[記録](evidence/wrac-gain-macos-2026-10-29/clap-basic-check.log)）。 |
| 直接 CLAP ホストの遅延 | 48 kHz / 512 frames で状態復元により 0 → 4096 → 0 samples に変更し、再起動要求、申告値と左右のインパルス遅延を確認（[記録](evidence/wrac-gain-macos-2026-10-29/clap-latency-check.log)）。 |

## 未確認・制約

- 使用した REAPER 6.46 は CLAP 非対応のため、実 DAW / CLAP は未確認。直接ホストによる CLAP 検査は実 DAW の代替ではない。
- Fender Studio Pro はこの macOS 環境で利用できず未確認。Logic Pro / AU と Cubase / VST3 は対象外とし、それぞれ GarageBand と別途利用可能なホストで確認した。
- WaveLab、LUNA、Renoise、Vienna Ensemble、RX 7 は、起動時の制約によりプラグインの確認に至らなかった。
- AAX のオートメーション、保存・復元、動的遅延の通知・補償、オフライン書き出し、AAX validator は未確認。Developer 版に同梱された DSH は、必要な `DynamicXPC.framework` が見つからず起動できなかった。
- デバイス出力を録音するツールは今回の再生時に入力信号を検出できず、Pro Tools のピーク表示に代わる録音波形比較はできなかった。
- 全形式・全ホストでの GUI 開閉中の変更、連続操作、サンプルレートとバッファー長の網羅、イベント消失や競合の有無は未確認。
