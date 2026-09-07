# 統合 LUFS ゲート

プリセット曲が小さすぎないかを、**統合 LUFS（ITU-R BS.1770-4、ロング）** で見る。基準を下回った曲だけ人間が keep / 再作成を判断する。

ソースの lint やスペアナ画像はこの手順には含めない。

## 準備

リポジトリのルートで:

```
pip install -r scripts/requirements-eval.txt
cargo build --release
```

`samples/` は Git LFS で実 WAV が入っていること。ポインタのままだと PCM が無音になり、LUFS が床を割る。

`target/release/strudel-rs` が古いと `unknown command: render` になる。そのときはもう一度 `cargo build --release`。

## 実行

```
python scripts/lufs_gate.py
python scripts/lufs_gate.py --songs songs/house
python scripts/lufs_gate.py --min-lufs -18
python scripts/lufs_gate.py --bin target/release/strudel-rs
python scripts/lufs_gate.py --keep-wav
```

既定:

- 探索: `songs/`（ジャンルフォルダ 1 段）
- レンダー: ウォームアップ 1 小節を捨て、そのあと 16 小節（48 kHz、デッキ A）
- しきい値: **−28.0 LUFS**（`--min-lufs` で変更）
- レポート: `eval/lufs/runs/<UTC時刻>/`
- 1 曲でも基準未満 or エラーなら終了コード 1（`--no-fail` で 0）

`strudel-rs render` は音声デバイスを使わない。`play --headless` とは別経路。

## レポート

| ファイル | 内容 |
| --- | --- |
| `summary.md` | 人間向け。below を先に、そのあとジャンル別の全曲 |
| `below-threshold.md` | 基準未満だけ（再作成キュー） |
| `summary.json` / `songs.json` | 同じ内容の JSON |
| `wav/` | `--keep-wav` のときだけ残る作業 WAV |

`eval/lufs/runs/` は gitignore。生成物はコミットしない。

## 判定

ゲートは統合 LUFS だけ。peak / RMS は参考値。クリップや「うるさいが中身が薄い」はこの床では落ちない。

16 小節で測った同梱 01 の目安（このエンジンの gain のまま、配信 Loudness には揃えていない）:

| 曲 | 統合 LUFS |
| --- | ---: |
| `songs/house/01.strudel` | 約 −23.7 |
| `songs/minimal-techno/01.strudel` | 約 −23.7 |
| `songs/ambient/01.strudel` | 約 −25.3 |

既定 −28 は、この「鳴っているプリセット」より数 dB 下の床。−20 にすると現行 01 も below になる。無音や PCM ミスはもっと下がる。初回カタログの分布を見て `--min-lufs` を上げ下げしてよい。ジャンル別の表は持たない。
