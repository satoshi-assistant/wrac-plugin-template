# WRAC Gain の Windows ホスト動作確認（2026-10-29）

## 対象と範囲

- Windows 11 Pro build 26300、`novonotes-internal` の `c770b6a76b712779ab9553b108106ed80179ea7e` を基準とした WRAC Gain の release ビルドを確認した。
- GUI の追加確認では、`select` 操作時のフォーカス制御と選択肢の色を変更した修正版も使用した。これらと、遅延変更時の未保存通知の修正コードはこのブランチに含む。
- 数値データを [evidence](evidence/wrac-gain-windows-2026-10-29/) に保存した。`reaper-null-results.json` に含まれていたローカル絶対パスはファイル名に置き換えている。音声、スクリーンショット、ホストのプロジェクト、詳細ログなど、公開対象に含めていない元の証跡は検証環境に保管している。
- 以下の問題は観測結果であり、原因が WRAC、ホスト、設定、検証環境のどれにあるかを確定したものではない。成功した別の試行と問題を観測した試行を区別して記録する。

## ホスト別の結果

| ホスト・形式 | 結果と確認範囲 |
| --- | --- |
| REAPER 7.82、CLAP / VST3 | ホスト API から状態を復元し、停止中と Dummy Audio 再生中に 0→64→256→1024→4096→0 samples と変更。選択値とホスト PDC は 24 / 24 ケースで一致。44.1 kHz の逆位相並列書き出しは各遅延で完全相殺した。ゲイン 0.5 と PDC 無効の対照では音が残った。ゲインオートメーションと保存・再読込も確認。GUI 操作は下記のとおり一部のみ。 |
| Ableton Live 12.4.6、VST3 | 保存状態で 0 / 64 / 256 / 1024 / 4096 samples を読み込み、44.1 kHz・32 bit float・8 秒の単一経路書き出しが全サンプル一致。ゲイン 0.5 の対照はちょうど半分。4096 samples のホスト表示は 92.9 ms。並列経路は未確認。 |
| FL Studio 2026 26.1.7.5653、VST3 | 44.1 kHz・16 bit stereo の単一経路を書き出し、各遅延値を基準と比較。256 samples の初回試行だけに下記の音欠けを観測。並列経路の試行はエフェクトが片方に適用されたことを確認できず、無効とした。 |
| Cakewalk Sonar 2026.10、VST3 | 検出・挿入・GUI と、0 / 64 / 256 / 1024 / 4096 samples の PDC 表示を確認。単一経路の 44.1 kHz・32 bit float 書き出しは 0 samples に対し最大絶対差約 2.0e-15 以下。4096 samples は保存・再読込後も保持した。64 samples の逆位相並列書き出しは約 1.0e-15 まで相殺。 |
| Fender Studio Pro 8.1.2、VST3 | 検出・挿入・GUI、ゲイン 0.5 の対照、保存状態の 256 / 4096 samples とホスト表示を確認。単一・並列のオフライン書き出しでは下記のずれを観測。同じ 256 samples の並列経路はリアルタイム出力の録音で相殺した。 |
| Audacity 4.0、VST3 | 検出・GUI・Apply・Undo を確認。0 dB で適用後の 32 bit float 出力は元音源と全サンプル一致。16 bit 出力の微小な差は下記参照。 |
| Bitwig Studio 6.1.3 デモ、CLAP / VST3 | 検出・挿入・GUI、ホスト側ゲイン操作と GUI の同期、Test Tone に対する後段メーターの変化を確認。マスター出力はミュート。デモ版のため保存・書き出し・波形比較・PDC は未確認。 |
| 開発ホスト、CLAP / VST3 | 44.1 / 48 / 96 kHz × block 64 / 1024 × 遅延 0 / 4096 / 0 の 36 ケースで音声遅延とゲインを確認。状態復元とオートメーションも確認した。 |

主な数値証跡: [REAPER 切替](evidence/wrac-gain-windows-2026-10-29/reaper-dynamic-pdc.csv)、[REAPER 並列相殺](evidence/wrac-gain-windows-2026-10-29/reaper-null-results.json)、[REAPER オートメーション](evidence/wrac-gain-windows-2026-10-29/reaper-automation-results.json)、[Ableton 波形比較](evidence/wrac-gain-windows-2026-10-29/ableton-delay-comparison.json)、[Sonar 遅延比較](evidence/wrac-gain-windows-2026-10-29/sonar-delay-comparison.json)、[Sonar 並列比較](evidence/wrac-gain-windows-2026-10-29/sonar-parallel-analysis.json)、[開発ホストのマトリクス](evidence/wrac-gain-windows-2026-10-29/devhost-matrix-results.json)。

## 問題・要調査

### REAPER：遅延メニューが閉じる / 変更が未保存と判定されない

REAPER 7.82 の VST3 Settings で `LATENCY` をクリックすると選択肢がすぐ閉じ、値を変更できなかった。状態監視を止めても同じ現象を確認した。

有力な原因候補はフォーカス制御。設定画面は HTML `select` であり、既存の `pointerup` 処理はホストにフォーカスを戻す。`select` を入力中の要素とみなさない判定との組み合わせにより、選択途中でフォーカス復帰が起きる可能性がある。[ソース単位のイベント模擬](evidence/wrac-gain-windows-2026-10-29/focus-select-probe.json)では `focus_host_window` の呼び出しを確認したが、実際の WebView2 がどのイベントを届けたかは計測していない。

ローカル修正版では `select` の `pointerup` 時にはフォーカスを戻さず、`change` 後に戻した。関連パッケージのビルドと 9 テストに成功し、REAPER の VST3 GUI で 0→64→256 samples に変更でき、ホスト PDC も追従した。[記録](evidence/wrac-gain-windows-2026-10-29/reaper-fixed-gui-pdc.csv) 残りの値、再生中、CLAP の GUI 操作は未確認。

選択肢は明るい灰色背景に白文字で見づらいとの報告もあった。ローカル修正版で背景色と文字色を指定し、独立した Chromium 検証画面で暗い背景に明るい文字を確認した。REAPER / WebView2 の実際のポップアップでの見た目は未確認。

さらに、GUI で VST3 の遅延を 64→256 samples に変更すると、保存状態とホスト PDC は 256 だったが、REAPER の `IsProjectDirty` は 0 のままだった。明示的に別名保存して再読込すると 256 が復元された。GUI の `set_latency` は再起動を要求するが、未保存通知を送っていなかった。

ローカルで、GUI の選択値が変わった時にホストへ未保存を通知する修正を加えた。REAPER の VST3 / CLAP でテスト用フロントエンドから同じ GUI コマンドを `0→64→64→256` と送り、変更時の未保存状態、同じ値を再選択した時の無変更、保存・再読込後の 256 samples と未保存状態の解除を確認した。[結果](evidence/wrac-gain-windows-2026-10-29/reaper-dirty-test-summary.json)。これはポインターによるドロップダウンの直接操作ではない。Rust のテストと `cargo xtask quality` は成功し、テスト用コードを含まない通常ビルドを再生成した。修正コードはこのブランチに含む。他のホストは未確認。

### Studio Pro：オフライン書き出しとリアルタイム再生の差

44.1 kHz、8 秒のステレオ音源、WRAC Gain 0 dB、保存状態の遅延 256 / 4096 samples で確認した。ホストにはそれぞれ 5.8 / 92.9 ms が表示された。セッション全体を 32 bit float でオフライン書き出しすると、先頭は指定した samples 数だけ無音で、残りは 0 samples の出力を同じ数だけ後ろへ移動した波形と全サンプル一致し、末尾がその分切れた。[解析](evidence/wrac-gain-windows-2026-10-29/studio-pro-delay-analysis.json)

256 samples で同じ音源を 2 トラックに複製し、片方に WRAC Gain、もう片方に左右の位相を反転する Mixtool を置いて書き出した。片方の WRAC Gain をバイパス、または削除した場合、2 経路は相殺されず、最大絶対振幅は約 0.36093。両経路の WRAC Gain を有効にした対照では、352800 フレーム・両チャンネルで完全に相殺した。[並列解析](evidence/wrac-gain-windows-2026-10-29/studio-pro-parallel-analysis.json)

同じ 256 samples とプラグインなしの逆位相並列経路を FL Studio ASIO の既定出力へリアルタイム再生し、システム出力を 48 kHz・32 bit float で録音した。メイン出力は -31 dB。並列再生の録音は全サンプル 0、片方をソロにした対照のピークは約 0.00516。同一テイク中にソロから並列に切り替えても、音がある状態から無音になった。[録音の数値解析](evidence/wrac-gain-windows-2026-10-29/system-audio-analysis.json)

この条件では、リアルタイム再生とオフライン書き出しの結果が異なる。書き出しの設定、補正、プラグイン、ラッパー、ホストのどこに原因があるかは未確定。

### FL Studio：256 samples の無音を一度観測

ゲイン 1.0、遅延 256 samples の保存状態からオフライン書き出しした初回のみ、基準波形の frame 76689〜76944 に相当する連続 256 samples（約 5.8 ms）が両チャンネルで無音になった。それ以外は基準と一致した。[初回結果](evidence/wrac-gain-windows-2026-10-29/fl-offline-results.json)

同条件の直後の再試行 1 回、追加 10 回、さらに追加 30 回は一致した。0 samples の追加 10 回も一致した。最後の 30 回はログを info レベルにしたため、各回の遅延 256 の有効化ログはない。原因は未特定。[追加 20 回](evidence/wrac-gain-windows-2026-10-29/fl-repeat-results.json) / [追加 30 回](evidence/wrac-gain-windows-2026-10-29/fl-stress256-results.json)

### その他の観測

- VST3 の pluginval は strictness 10 と 5 の両方で成功したが、両ログに `wxp WebView evaluate_script failed` と `HRESULT(0x8007139F)` が記録された。通常操作への影響は未確認。
- Ableton Live のプラグインエディターが一時的に白く見えることがあった。別のホストのエディターを閉じて開き直すと通常表示になった。同環境では Live 自身の書き出し画面も白く描画されたことがあり、原因は未確定。
- Audacity の 16 bit 書き出しでは、0 dB 適用後の元音源との差が最大 9 LSB だった。Undo 後の 16 bit は元音源と一致し、適用後の 32 bit float は元音源を浮動小数点に換算した値と一致した。ディザリングの可能性はあるが設定と原因は未確認。[比較結果](evidence/wrac-gain-windows-2026-10-29/audacity-comparison.json)

## 未確認・制約

- Windows の `cargo xtask validate --target=clap,vst3,aax --release --continue-on-error` は CLAP 33 pass / 11 skipped、VST3 47 pass / 0 fail。AAX Validator / DSH の Windows アーカイブがなく AAX 自動検証は未実施。
- macOS CI の auval 成功は確認したが、Logic などの AU 実ホスト確認ではない。AU の pluginval、状態復元・再起動の回帰テスト、AAX の実ホスト確認も未実施。
- 各 DAW での GUI による全遅延値の切替、サンプルレート・ブロック長の全組み合わせ、並行処理の競合やイベント消失は網羅していない。上記の結果を、すべてのホスト・形式・条件の検証完了とは扱わない。
