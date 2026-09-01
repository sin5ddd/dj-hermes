# 展示用: HP 上のローカル STT（オプション 2）

既定の F12 / VAD は Surface 上の Hermes 内蔵 Whisper です（`STRUDEL_STT_BASE_URL` 不要）。
この文書は issue #46 の選択肢 2 で、HP に HTTP STT を置く場合です。

`STRUDEL_STT_BASE_URL` を設定すると、Surface は HP の HTTP サーバへ 16 kHz WAV を送り、JSON の `text` を受け取ります。
xAI や ElevenLabs などのクラウド音声 API は使いません。

関連: [issue #46](https://github.com/sin5ddd/strudel-rust/issues/46)

## 役割分担

| 機械 | 担当 |
| --- | --- |
| Surface Pro 3 | `strudel-rs dj`（音 + TUI + マイク録音） |
| HP ProDesk（i5-9500T / 32GB） | Qwen 4B Q4 と、この STT サーバ |

strudel-rs の MCP（`127.0.0.1:17878`）は LAN に出しません。STT だけ Surface → HP の outbound です。

## HP 側

1. [whisper.cpp](https://github.com/ggml-org/whisper.cpp) をビルドし、`whisper-cli` を PATH に置く
2. 多言語モデルを置く（展示の出発点は `ggml-base.bin`。会場ノイズが荒ければ `small`）
3. リポジトリのラップを起動する（既定 2 スレッド。残りは Qwen 用）

```bash
python3 scripts/stt_server.py \
  --host 192.168.x.x \
  --port 8090 \
  --model /path/to/ggml-base.bin \
  --whisper-cli whisper-cli \
  --threads 2 \
  --language ja
# 任意: --token booth-token
```

動作確認:

```bash
curl -s http://192.168.x.x:8090/health
# 短い WAV があれば:
curl -s -F file=@clip.wav -F language=ja \
  http://192.168.x.x:8090/v1/audio/transcriptions
```

`--host` の既定は `127.0.0.1` です。ブース LAN から受けるときだけ HP の IPv4 を渡してください。公開インターネットには出さない想定です。

## Surface 側

```bash
export STRUDEL_STT_BASE_URL=http://192.168.x.x:8090
# 任意（HP で --token を付けたとき）
# export STRUDEL_STT_API_KEY=booth-token
# export STRUDEL_STT_LANGUAGE=ja
# export STRUDEL_VOICE_MODE=push    # または vad
# export STRUDEL_VOICE_SILENCE_THRESHOLD=500
# export STRUDEL_VOICE_SILENCE_SECS=1.2
strudel-rs dj songs/techno1.strudel songs/ambient1.strudel
```

`STRUDEL_STT_BASE_URL` が無いときは既定の Hermes STT です（キーボードの自然文は従来どおり）。

| `STRUDEL_VOICE_MODE` | 動き |
| --- | --- |
| `push`（既定） | F12 で録音開始 / 停止。会場の再生音がマイクに入るので展示の既定 |
| `vad` | 常時聞いて RMS で発話を切る。F12 は一時停止 / 再開 |

再生しながら常時聞きすると曲が誤発火しやすいです。ブースではまず `push` を使ってください。
