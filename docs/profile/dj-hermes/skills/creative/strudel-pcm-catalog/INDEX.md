# rust-fm-synthe PCM index

strudel-rs の呼び出しは `s("<part>:<slug>")`。 音程楽器は `note(...).scale("C4:…").s("<part>:<slug>")`。 `in_bank` が no の行はファイルが `samples/` に無い（長尺は既定オフ、または skip）。

生成: `python scripts/import_fm_pcm.py`

## `bd`

| call | in_bank | id | name | description | note | dur |
| --- | --- | --- | --- | --- | --- | --- |
| `bd:8b` | yes | `bd-808-boom` | 808 boom | クラシックな808ブーム。長い正弦の胴と大きなピッチ落下。サブ寄りワンショット。 | 32 | 1.4 |
| `bd:8d` | yes | `bd-808-dist` | 808 dist | 歪んだ808。トラップ／EDMのハイブリッド。サブは残しつつミッドを飽和。 | 33 | 0.88 |
| `bd:8t` | yes | `bd-808-tight` | 808 tight | 短いトラップ寄り808。クリック多め、胴は短め。ハイハットの下に置く用。 | 36 | 0.52 |
| `bd:9p` | yes | `bd-909-punch` | 909 punch | 909風のパンチ。ミッドのクリックと短い胴。四つ打ちやブレイクの芯。 | 36 | 0.42 |
| `bd:cn` | yes | `bd-cinematic` | cinematic boom | 長いシネマティックブーム。低い余韻と大きなピッチ落下。トレーラー向き。 | 24 | 1.5 |
| `bd:ck` | yes | `bd-click` | click layer | 胴なしクリック。レイヤー用のアタックだけ。本体キックの上に重ねる。 | 48 | 0.28 |
| `bd:dc` | yes | `bd-disco-dry` | disco dry | ファンキー／ディスコのドライキック。ブーム無し、短いアタック。生ドラム寄り。 | 36 | 0.38 |
| `bd:dn` | yes | `bd-dnb-tight` | dnb tight | Amen隣接のタイトなDnBキック。短くミッドが出る。ブレイクの芯。 | 41 | 0.32 |
| `bd:ez` | yes | `bd-electro-zap` | electro zap | 短いエレクトロのザップキック。レーザー気味のピッチ落下。フィル向き。 | 48 | 0.28 |
| `bd:fn` | yes | `bd-fm-noise` | fm noise | 実験的なFMノイズキック。高フィードバックの砂状だが、ピッチ落下でキックとして使える。 | 36 | 0.5 |
| `bd:fc` | yes | `bd-frenchcore` | frenchcore | フレンチコア／ハードコア。攻撃的なミッドとクリック。サブは削って裂けるように。 | 45 | 0.3 |
| `bd:gb` | yes | `bd-gabber-stomp` | gabber stomp | ガバ／インダストリアルのストンプ。歪んだミッドと短い踏み込み。 | 40 | 0.36 |
| `bd:hs` | yes | `bd-hardstyle` | hardstyle | ハードスタイルのピッチ感あるキック。逆再生っぽいスイープとミッドのパンチ。 | 46 | 0.4 |
| `bd:hf` | yes | `bd-house-floor` | house floor | アナログハウスの4つ打ち。暖かく乾いたフロアキック。長いサブは出さない。 | 36 | 0.5 |
| `bd:jg` | yes | `bd-jungle-round` | jungle round | ブレイクビーツ／ジャングルの丸いキック。Amenより太く、クリックは控えめ。 | 38 | 0.44 |
| `bd:lf` | yes | `bd-lofi-dust` | lo-fi dust | 柔らかいローファイ／ダスト。こもったワンショット。テープっぽい揺れ。 | 38 | 0.68 |
| `bd:mt` | yes | `bd-metal-ping` | metal ping | 金属的なFMピンキック。インダストリアル。固定周波数のリン＋短い胴。 | 40 | 0.48 |
| `bd:ng` | yes | `bd-neuro-growl` | neuro growl | フィルタしたFMのグロウルキック。ニューロ寄りの喉声ミッド。 | 35 | 0.58 |
| `bd:sb` | yes | `bd-sub` | sub layer | サブだけ。レイヤー用の正弦ブーム。クリック無し、胴の下に置く。 | 24 | 1.25 |
| `bd:tc` | yes | `bd-techno-thud` | techno thud | 深いテクノのドサッとした胴。クリック少なめ、ローが重いワンショット。 | 26 | 0.72 |

## `bs`

| call | in_bank | id | name | description | note | dur |
| --- | --- | --- | --- | --- | --- | --- |
| `bs:8s` | yes | `bs-808-sub` | 808 sub bass | 長い正弦の808サブ。キック用ブームではなくベースワンショット。クリックほぼなし、大きなピッチ落下。既存 sub-bass より胴が長くクリックが薄い。 | 36 | 1.85 |
| `bs:ac` | yes | `bs-acid` | acid bass | 303風のベース。ソー＋LP＋高いレゾ＋カットオフエンベ。ld-acid より1オクターブ低く、胴を残す。 | 36 | 1.15 |
| `bs:am` | yes | `bs-amen-sub` | dnb amen sub | Amen横のDnBサブ。短いクリックのあと丸い正弦。sub-bass より短くタイト。キックではなくベース。 | 36 | 1.05 |
| `bs:dq` | yes | `bs-dist-square` | distorted square bass | 歪んだスクエア／パルスベース。高いフィードバックで砂状。ミッドの芯。スーパーソーでもReeseでもない。 | 36 | 1.25 |
| `bs:fc` | yes | `bs-frenchcore` | frenchcore mid-bass | フレンチコアのミッドベース。HPで羊毛サブを切る。攻撃的なパルス＋高比。キックではなくベース。 | 43 | 1.05 |
| `bs:gb` | yes | `bs-gabber` | gabber bass | ガバの歪みミッドベース。短いパンチ、パルス＋フィードバック。ストンプキックではなくベースワンショット。 | 41 | 0.95 |
| `bs:g2` | yes | `bs-growl-2` | growl bass 2 | 2つ目のグロウル。既存 growl-bass / bp-growl よりソー寄りでフォルマント比が違う。ミッド。サブは薄く。 | 43 | 1.5 |
| `bs:hv` | yes | `bs-hoover` | hoover bass | フーバー／アルファレーン寄りのベース。デチューンソー＋フォルマント。ld-hoover より低く、ミッドベース。 | 40 | 1.65 |
| `bs:ht` | yes | `bs-house-tight` | tight house bass | タイトなハウスベース。短い減衰でサイドチェイン向き。サブの芯＋短いミッドクリック。長いReeseや808ではない。 | 36 | 0.85 |
| `bs:mt` | yes | `bs-metal-fm` | metal fm bass | 金属FMベース。非整数比のリンがミッドに乗る。サブは薄く、既存グロウルやスーパーソーとは別キャラ。 | 43 | 1.2 |
| `bs:rb` | yes | `bs-reese-bright` | bright wide reese | 明るい広いReese。super-sawのデチューンを既存 supersaw-bass より広くし、ミッドハイを残す。糊帯のBPではない。 | 40 | 1.45 |
| `bs:rd` | yes | `bs-reese-dark` | dark reese bass | 暗いフルレンジReese。サブ＋ミッドの厚いデチューンソー。reese-mid（800–1200糊）とは別。LPでハイを抑える。 | 36 | 1.7 |
| `bs:rn` | yes | `bs-reese-neuro` | neuro reese bass | ニューロ寄りのReese。非整数比とフィードバックで喉。BPは300–700付近。reese-midの1k糊でも既存グロウルのクローンでもない。 | 40 | 1.55 |
| `bs:ss` | yes | `bs-sine-sub` | clean sine sub | クリーンな正弦サブ。クリックも歪みもほぼなし。レイヤー用の純粋な低域。808ブームや既存 sub-bass より単純。 | 36 | 1.8 |
| `bs:wb` | yes | `bs-wobble` | wobble bass | ベース用ウォブルワンショット。速めのピッチLFOとBPエンベ。リードの ld-wobble より低く、サブ〜ミッド。 | 36 | 1.9 |
| `bs:bp` | yes | `bp-growl` | bp-growl | バンドパスで喉のあたりを抉るミッドグロウル。カットオフADSRでフォルマントが動く。 | 40 | 1.5 |
| `bs:gr` | yes | `growl-bass` | growl-bass | ミッドのグロウル。フィードバックと非整数比で喉声っぽい歪み。Reeseの下地。 | 40 | 1.6 |
| `bs:sb` | yes | `sub-bass` | sub-bass | キックの下に置くDnBサブ。短いクリックのあと正弦に近い胴が残るワンショット。 | 36 | 1.35 |
| `bs:sw` | yes | `supersaw-bass` | supersaw-bass | 厚みのあるミッドベース。キャリアを super-saw（擬似スーパーソー）にしたトランス寄りワンショット。 | 36 | 1.4 |
| `bs:rm` | yes | `reese-mid` | reese-mid | C3ミッドReeseの糊だけ。800–1200 Hzのバンドパス。サブなし。軽いデチューンと弱いFM。 alias `bs:rm` | 48 | 1.1 |
| `bs:hf` | yes | `bass-fm_house` | house floor bass | C2 tight house floor (batch 1). Native C2; write C4:…. Pairs with bd:hf. |  |  |
| `bs:su` | yes | `bass-fm_sub` | fm sine sub | C2 clean sine sub (batch 1). Floor only. Write C4:…. Do not stack with other subs. |  |  |
| `bs:dk` | yes | `reese-dark` | dark Reese | C3 full-range dark Reese with sub. Write C4:…. Do not stack with other subs. |  |  |

## `cp`

| call | in_bank | id | name | description | note | dur |
| --- | --- | --- | --- | --- | --- | --- |
| `cp:dr` | yes | `pc-clap-dry` | dry clap | ドライなクラップ。cp-house より短く、1k胴を薄くしてスラップ寄り。2/4用。 | 60 | 0.18 |
| `cp:gt` | yes | `pc-clap-gate` | gated clap | ゲートしたクラップ。胴のあと急に切れる。cp-house より短いゲート感。 | 60 | 0.2 |
| `cp:rm` | yes | `pc-clap-room` | room clap | ルーム寄りのクラップ。遅れた層で部屋感。cp-house より尾が長い。 | 60 | 0.42 |
| `cp:00` | yes | `cp-house` | cp-house | 短いドライなハウスクラップ。2/4専用。1 kHz付近の短い胴＋ハイのスラップ。スネア代用でもサブでもない。 alias `cp` | 60 | 0.28 |

## `dr`

| call | in_bank | id | name | description | note | dur |
| --- | --- | --- | --- | --- | --- | --- |
| `dr:ab` | no | `dr-abyss` | abyss rumble | 深淵のランブル。0.5正弦に薄いパルスの砂。 | 24 | 16.7 |
| `dr:ad` | no | `dr-ambient-dark` | dark ambient pad | 暗いアンビエントパッド。ゆっくりしたLFO。サブを残したまま空気を足す。 | 36 | 16.9 |
| `dr:bd` | no | `dr-brass-distant` | distant brass drone | 遠い金管。ミュートしたブラスの床。LPで遠さ。 | 33 | 16.7 |
| `dr:bp` | no | `dr-brass-pad` | low brass pad | 低いブラスパッド。リップのFMがすぐ落ち着き、長い胴が残る。 | 36 | 16.5 |
| `dr:ct` | no | `dr-cathedral` | cathedral low | 聖堂の低いドローバー。1+2+3+0.5。ミッドドローン（C3）。 | 48 | 16.6 |
| `dr:cd` | no | `dr-choir-dark` | dark choir drone | より暗いクワイア。短3度を薄く足す。ミッドドローン寄り。 | 33 | 16.7 |
| `dr:cl` | no | `dr-choir-low` | low choir drone | 低いクワイア寄りの重ねサイン。デチューンした加算。暗いホール。 | 36 | 16.5 |
| `dr:dy` | no | `dr-dystopia` | dystopian hum | ディストピアのハム。ソーとパルスの低い都市音。 | 31 | 16.5 |
| `dr:en` | no | `dr-engine` | engine rumble | エンジンの回転うなり。遅いLFOでピッチがわずかに揺れる。 | 28 | 16.5 |
| `dr:fh` | no | `dr-fifth-hollow` | hollow fifth drone | 中空5度の低ドローン。C+Gだけ。長3度なし。 | 36 | 16.4 |
| `dr:fb` | no | `dr-fm-bell-low` | low metallic fm | 低い金属FM。ベルというより遠いゴングの床。薄い高域ベルではない。 | 36 | 16.5 |
| `dr:fe` | no | `dr-fm-evolve` | evolving fm drone | 指数がゆっくり開くFMドローン。16秒かけて倍音が育つ。 | 33 | 16.7 |
| `dr:fi` | no | `dr-fm-index` | slow index fm | 並列モジュレータの遅い指数スイープ。暗い金属の粒がゆっくり増える。 | 36 | 16.5 |
| `dr:fg` | no | `dr-fog` | fog pad | 霧のパッド。カットオフが低く、輪郭が溶ける。 | 36 | 17.0 |
| `dr:fl` | no | `dr-formant-low` | low formant drone | 低いフォルマント。アブサインの母音がゆっくり動く。 | 36 | 16.5 |
| `dr:gc` | no | `dr-ghost-choir` | ghost choir | 幽霊クワイア。ミッド（C3）の薄い重ねサイン。サブは0.5で残す。 | 48 | 16.9 |
| `dr:hr` | no | `dr-horror` | horror drone | ホラーの不協和。1.07 / 2.13 のうなり。低い床は残す。 | 28 | 16.8 |
| `dr:hg` | no | `dr-hum-grid` | grid hum | 50/60 Hzの電源グリッド。固定周波数＋ノートのサブ。 | 36 | 16.4 |
| `dr:ic` | no | `dr-ice-cave` | ice cave drone | 氷穴のミッドドローン。冷たい倍音を薄く。サブは0.5で残す。 | 48 | 16.8 |
| `dr:ih` | no | `dr-impact-hold` | impact into hold | ピッチ落下のインパクトからそのまま床になる。ワンショットで消えない。 | 26 | 16.6 |
| `dr:id` | no | `dr-industrial` | industrial drone | 工場の低いハム。パルスとソー、フィードバックの砂。 | 31 | 16.4 |
| `dr:mb` | no | `dr-metal-bed` | metallic bed | 金属ベッド。共有モジュレータのリンが長く残る。低いキャリア。 | 36 | 16.5 |
| `dr:md` | no | `dr-metal-distant` | distant metallic drone | 遠い金属のうなり。非整数比。LPで手前に出さない。 | 36 | 16.6 |
| `dr:mn` | no | `dr-minor-dark` | dark minor drone | 暗い短3度寄り（6:5）。シネマのマイナー床。長三和音は使わない。 | 31 | 16.6 |
| `dr:nb` | no | `dr-noisy-bp` | noisy bandpass rumble | 高FBの砂を低いBPでランブルにする。カットオフは80 Hz付近。サブ隣接を残す。 | 28 | 16.4 |
| `dr:os` | no | `dr-octave-stack` | octave stack drone | オクターブ重ねの重いベッド。0.5 / 1 / 2。ミッドは薄く。 | 28 | 16.5 |
| `dr:pd` | no | `dr-pad-dark` | dark pad drone | 暗いパッドドローン。ソーの芯に正弦のサブ。 | 36 | 16.6 |
| `dr:pf` | no | `dr-pulse-fifth` | pulse fifth rumble | パルスの5度ランブル。中空で攻撃的な低域。 | 33 | 16.4 |
| `dr:pr` | no | `dr-pulse-rumble` | pulse rumble | パルスの低うなり。矩形の胴をLPで丸める。 | 28 | 16.3 |
| `dr:rc` | no | `dr-reactor` | reactor hum | 原子炉のハム。非整数比の低いうなり＋サブ。 | 31 | 16.6 |
| `dr:rd` | no | `dr-reese-dark` | dark reese drone | 暗いReeseスタック。デチューンソーのサブ〜ミッド。長いホールド。 | 31 | 16.6 |
| `dr:rw` | no | `dr-reese-wide` | wide reese drone | 広めの暗いReese。左右に広がるデチューン。サブは残す。 | 33 | 16.5 |
| `dr:rh` | no | `dr-reverse-hold` | reverse into hold | リバース風に開いてからホールド。アタックは短め（スモーク用）でフィルタがゆっくり開く。 | 31 | 17.0 |
| `dr:rs` | no | `dr-riser-slow` | slow riser drone | 遅いライザーがドローンになる。ピッチは少しだけ上がって止まる。 | 28 | 17.2 |
| `dr:ri` | no | `dr-ritual` | ritual drone | 儀式の低い重ね。5度と短3度。暗いホール。 | 28 | 16.8 |
| `dr:rm` | no | `dr-rumble` | low rumble | パルスの低ランブル。地面が揺れるような胴。LPでハイを抑える。 | 26 | 16.4 |
| `dr:sm` | no | `dr-saw-minor` | saw minor stack | ソーの短3度スタック。暗いコード床。長3度なし。 | 31 | 16.6 |
| `dr:sh` | no | `dr-scifi-hum` | sci-fi hum | SFの電源ハム。固定60 Hz層＋ノートのサブ。 | 36 | 16.5 |
| `dr:sc` | no | `dr-score-hold` | scored trailer hold | スコア／トレーラーのホールド。スーパーソー低域＋正弦サブ。ミッド（C3）。 | 48 | 17.2 |
| `dr:ss` | no | `dr-sine-sub` | sine sub bed | シネマティックな正弦サブベッド。20–40 Hzの胴を長くホールド。レイヤーの床。 | 24 | 16.4 |
| `dr:st` | no | `dr-storm` | storm rumble | 嵐のランブル。高FBノイズをLPで遠雷にする。サブ正弦が芯。 | 24 | 16.8 |
| `dr:so` | no | `dr-sub-octave` | sub octave bed | サブと1オクターブ上の正弦スタック。空洞のない重い床。 | 24 | 16.5 |
| `dr:sl` | no | `dr-supersaw-low` | low supersaw drone | 低いスーパーソードローン。トレーラーの厚いパッド床。LPでサブを残す。 | 36 | 16.8 |
| `dr:th` | no | `dr-tape-hum` | tape machine hum | テープ／機械のハム。わずかなデチューンと低いランブル。 | 28 | 16.4 |
| `dr:tb` | no | `dr-thunder-bed` | thunder bed | 雷のベッド。サブ正弦＋高FBの遠雷ノイズ。ワンショットではない。 | 24 | 16.8 |
| `dr:tl` | no | `dr-trailer-bloom` | trailer bloom drone | 短いインパクトが開いてドローンになる。トレーラーヒット→ホールド。 | 24 | 16.8 |
| `dr:uw` | no | `dr-underwater` | underwater drone | 水中の低いうなり。LPが狭く、ゆっくり揺れる。 | 24 | 16.6 |
| `dr:vd` | no | `dr-void` | void drone | 虚空。極端に暗いLP。ほぼサブだけの長い無。 | 24 | 17.0 |
| `dr:wf` | no | `dr-warfare` | warfare bed | 戦争映画の床。遠いブラスとサブのランブル。 | 31 | 16.7 |
| `dr:ws` | no | `dr-wobble-slow` | slow wobble drone | ごく遅いウォブル。0.15 HzのLFO。ベースワンショットではない。 | 33 | 16.6 |

## `ep`

| call | in_bank | id | name | description | note | dur |
| --- | --- | --- | --- | --- | --- | --- |
| `ep:mt` | yes | `ep-muted` | ep-muted | ミュート／ラウンジEP。LPで暗く、タインは控えめだがアタックに2×/3×は残す。C3。約3.0秒。 | 48 | 3.0 |
| `ep:rh` | yes | `ep-rhodes-hard` | ep-rhodes-hard | 硬いRhodes。同じ1×胴＋2×/3×タインだがFM指数とベロ感を上げた咬み。C–E–G向き（短3度の比は入れない）。C3。約2.8秒。 | 48 | 2.8 |
| `ep:rs` | yes | `ep-rhodes-soft` | ep-rhodes-soft | 柔らかいRhodes。アタックでタイン（比2＝約262 Hz、比3＝約392 Hz）が立ち、減衰してサイン寄りの胴（比1＝約131 Hz）へ。C3。約3.2秒。ベル（3.5）ではない。 | 48 | 3.2 |
| `ep:tb` | yes | `ep-tine-bell` | ep-tine-bell | タイン前のめりEP。2×/3×を強く出すが整数倍のまま（3.5や ld-bell-pluck の非整数比は使わない）。胴（比1）は残す。C3。約2.4秒。 | 48 | 2.4 |
| `ep:wr` | yes | `ep-wurli` | ep-wurli | ウーリッツァー寄り。パルス／アブサインのモジュレータでミッドの樹皮感。Rhodesより短い（約1.8秒）がクリックではない。タインは2×/3×。C3。 | 48 | 1.8 |
| `ep:ky` | yes | `keys-fm_ep` | FM EP one-shot | C3 EP tines (2×/3×). Write C4:…. Not the live 2-op lead. |  |  |

## `fx`

| call | in_bank | id | name | description | note | dur |
| --- | --- | --- | --- | --- | --- | --- |
| `fx:al` | yes | `fx-alarm` | fx-alarm | アラーム。2音の交互に近いLFOピッチ。短め。 | 76 | 0.9 |
| `fx:bl` | yes | `fx-blip` | fx-blip | 極短いブリップ。UI／グリッチ／アクセント。 | 84 | 0.22 |
| `fx:bm` | yes | `fx-boom` | fx-boom | ブーム。サブもあるがミッドの胴を残す。シネマティック寄り。 | 48 | 0.95 |
| `fx:cg` | yes | `fx-clang` | fx-clang | インダストリアルの金属クラング。固定周波数の打撃。 | 60 | 0.85 |
| `fx:ck` | yes | `fx-crackle` | fx-crackle | ビニールクラックルのバースト。短い砂粒。 | 84 | 0.45 |
| `fx:dk` | yes | `fx-down-to-kick` | fx-down-to-kick | ダウンリフターからキックへ。後半でピッチが落ちミッドが残る。 | 48 | 1.4 |
| `fx:dl` | yes | `fx-downlifter` | fx-downlifter | ダウンリフター。トーンが落ちてフィルタが閉じる。 | 72 | 1.8 |
| `fx:dn` | yes | `fx-downlifter-noise` | fx-downlifter-noise | ノイズのダウンリフター。砂が落ちて閉じる。 | 60 | 2.0 |
| `fx:fl` | yes | `fx-fall` | fx-fall | 急なフォール。レーザー寄りの落下。短め。 | 67 | 1.2 |
| `fx:fa` | yes | `fx-formant-ah` | fx-formant-ah | アー母音のFXヒット。リードではなくワンショットの声。 | 60 | 0.7 |
| `fx:fo` | yes | `fx-formant-oh` | fx-formant-oh | オー母音のFXヒット。低いフォルマント。 | 55 | 0.75 |
| `fx:fc` | yes | `fx-frenchcore-ns` | fx-frenchcore-ns | フレンチコアのノイズスネアFX。スネアバンクではなくトランジション用の割れ。 | 67 | 0.5 |
| `fx:gb` | yes | `fx-gabber-stab` | fx-gabber-stab | ガバのスタブFX。リードではなく短い歪みヒット。 | 55 | 0.38 |
| `fx:gs` | yes | `fx-glass-smash` | fx-glass-smash | ガラス破砕。高整数比とインハーモニックの短いスマッシュ。 | 84 | 0.7 |
| `fx:hv` | yes | `fx-hoover-fall` | fx-hoover-fall | フーバー寄りのフォールFX。リードではなく落下ワンショット。 | 55 | 1.3 |
| `fx:im` | yes | `fx-impact` | fx-impact | ミッド寄りのインパクト。クリック＋胴。サブだけにしない。 | 48 | 0.55 |
| `fx:id` | yes | `fx-impact-dnb` | fx-impact-dnb | DnBインパクト。タイトなミッドヒット＋短い砂。 | 52 | 0.5 |
| `fx:md` | yes | `fx-impact-mid` | fx-impact-mid | ミッド専用インパクト。胴のドサッ。サブはHPで切る。 | 52 | 0.45 |
| `fx:lz` | yes | `fx-laser` | fx-laser | 上昇レーザーFX。リードの ld-zap とは別。短いワンショット。 | 72 | 0.42 |
| `fx:lf` | yes | `fx-laser-fall` | fx-laser-fall | 落下レーザー。フィルタも閉じる。フィルの終端。 | 80 | 0.55 |
| `fx:mc` | yes | `fx-metal-crash` | fx-metal-crash | 金属FMクラッシュ。固定周波数のリン＋高比。本物のシンバルではない。 | 72 | 1.1 |
| `fx:nb` | yes | `fx-noise-burst` | fx-noise-burst | 少し長いノイズバースト。フィルやトランジションの砂。 | 67 | 0.85 |
| `fx:nh` | yes | `fx-noise-hit` | fx-noise-hit | ホワイト寄りの短いノイズヒット。トップやグリッチ。 | 72 | 0.35 |
| `fx:pb` | yes | `fx-passby` | fx-passby | 通過音。ピッチが落ち、BPが横切るドップラー風。 | 60 | 1.1 |
| `fx:rd` | yes | `fx-radio-stab` | fx-radio-stab | ラジオスタブ。狭いBPとパルス。通信ノイズ風。 | 67 | 0.4 |
| `fx:ra` | yes | `fx-rev-air` | fx-rev-air | エア寄りのリバースハット。スーパーソーの砂＋HPスウェル。 | 84 | 1.8 |
| `fx:rc` | yes | `fx-rev-crash` | fx-rev-crash | 明るいリバースクラッシュ。HPが開いて砂状のクラッシュで切れる。 | 76 | 2.1 |
| `fx:rm` | yes | `fx-rev-crash-metal` | fx-rev-crash-metal | 金属FMのリバースクラッシュ。固定周波数のリンが後半で開く。 | 72 | 2.2 |
| `fx:ry` | yes | `fx-rev-cym` | fx-rev-cym | クラシックなリバースシンバル。暗いノイズからHP/LPが開き、上昇ピッチでクラッシュへ。 | 72 | 2.6 |
| `fx:rb` | yes | `fx-rev-cym-bright` | fx-rev-cym-bright | 短い明るいリバースシンバル。BPが上へ開いてスプラッシュ気味。 | 80 | 1.7 |
| `fx:rk` | yes | `fx-rev-cym-dark` | fx-rev-cym-dark | 暗いリバースシンバル。胴寄り。ライドの逆再生印象。 | 55 | 3.0 |
| `fx:rl` | yes | `fx-rev-cym-long` | fx-rev-cym-long | 長いダークなリバースライド。3.8秒。ビルドの奥で使う。後で切る前提。 | 60 | 3.8 |
| `fx:rn` | yes | `fx-rev-cym-noise` | fx-rev-cym-noise | ノイズ寄りのリバースシンバル。高FBと高比でホワイトに近い砂。 | 67 | 2.4 |
| `fx:rh` | yes | `fx-rev-hat` | fx-rev-hat | リバースハットのスウェル。短め・明るい。ハイハットの逆再生印象。 | 84 | 1.55 |
| `fx:rs` | yes | `fx-rev-splash` | fx-rev-splash | 短いリバーススプラッシュ。明るいクラッシュの逆再生。1.5秒。 | 84 | 1.5 |
| `fx:rv` | yes | `fx-rev-verb` | fx-rev-verb | リバースリバーブ風。遅いアタック＋HPのウォッシュ。フェイク。 | 67 | 2.0 |
| `fx:rf` | yes | `fx-riser-filter` | fx-riser-filter | フィルタ開放のライザー。ピッチは控えめ、カットオフが主役。 | 60 | 2.6 |
| `fx:nr` | yes | `fx-riser-noise` | fx-riser-noise | ノイズライザー。砂が濃くなりピッチも上がる。3秒超。 | 55 | 3.2 |
| `fx:rp` | yes | `fx-riser-pitch` | fx-riser-pitch | ピッチ主体のライザー。トーンがはっきり上がる。 | 60 | 2.4 |
| `fx:rw` | yes | `fx-riser-saw` | fx-riser-saw | スーパーソーのライザー。厚みのある上昇。 | 48 | 3.0 |
| `fx:sr` | yes | `fx-siren` | fx-siren | 短いサイレン風。深いLFOピッチ。長いループではない。 | 72 | 1.15 |
| `fx:sd` | yes | `fx-sub-drop` | fx-sub-drop | サブドロップ。ピッチが大きく落ちる。キック前のダウン。 | 48 | 1.1 |
| `fx:sw` | yes | `fx-sweep-bp` | fx-sweep-bp | バンドパス掃引。カットオフが上へ開くFXスイープ。 | 60 | 1.6 |
| `fx:ts` | yes | `fx-tape-stop` | fx-tape-stop | テープストップ風。ピッチが後半急落。本物のテープではない。 | 60 | 1.0 |
| `fx:tf` | yes | `fx-trans-fill` | fx-trans-fill | トランジションフィル。ノイズ＋短いピッチ落ち。1拍用。 | 60 | 0.65 |
| `fx:up` | yes | `fx-uplifter` | fx-uplifter | アップリフター。ピッチ上昇＋フィルタ開放。ビルド用。 | 60 | 2.8 |
| `fx:wh` | yes | `fx-whoosh` | fx-whoosh | ウーシュ。BPが横切る風切り。 | 67 | 1.4 |
| `fx:wp` | yes | `fx-whoosh-hp` | fx-whoosh-hp | ハイパスのウーシュ。空気だけが横切る。 | 80 | 1.2 |
| `fx:wd` | yes | `fx-wind` | fx-wind | 風。高FBの持続ノイズ＋遅いLFO。パッドではなくワンショット。 | 72 | 2.2 |
| `fx:zp` | yes | `fx-zap` | fx-zap | ノイズ寄りの落下ザップ。ld-zap / zap より砂が多く、FX専用。 | 76 | 0.32 |
| `fx:fr` | yes | `fm-riser` | fm-riser | ノイズ寄りのFMライザー。ピッチ上昇と変調量スイープ。ビルドのFX。 | 48 | 2.4 |
| `fx:ha` | yes | `hp-air` | hp-air | ハイパスで胴を切ったエア／ティック。トップやトランジションの短いワンショット。 | 84 | 0.5 |
| `fx:zz` | yes | `zap` | zap | 下向きピッチのレーザー／ザップ。フィルやトランジションのワンショット。 | 72 | 0.38 |

## `hh`

| call | in_bank | id | name | description | note | dur |
| --- | --- | --- | --- | --- | --- | --- |
| `hh:cl` | yes | `pc-hat-closed` | closed hat | クローズドハット。短いノイズFM。キックやスネアではない。 | 84 | 0.18 |
| `hh:ch` | yes | `pc-hat-chip` | chip hat | チップチューン寄りの短いハット。パルスの粒が見える。 | 96 | 0.13 |
| `hh:dk` | yes | `pc-hat-dark` | dark hat | 暗いクローズドハット。HPを低めにして胴っぽい砂を残す。 | 76 | 0.22 |
| `hh:dn` | yes | `pc-hat-dnb-cl` | dnb closed hat | DnBのクローズドハット。少し暗く、短い砂。 | 80 | 0.16 |
| `hh:fc` | yes | `pc-hat-fc` | frenchcore hat | フレンチコアの攻撃的クローズハット。硬いハイ、短い。 | 86 | 0.12 |
| `hh:hs` | yes | `pc-hat-house` | house closed hat | ハウスのクローズドハット。タイトで明るい。4つ打ちの16分向き。 | 84 | 0.14 |
| `hh:ns` | yes | `pc-hat-noise` | noise hat | ノイズ寄りのハット。フィードバック最大付近。 | 84 | 0.2 |
| `hh:pd` | yes | `pc-hat-pedal` | pedal hat | ペダル／フットハット。クローズより暗く短いチック。 | 72 | 0.15 |
| `hh:tt` | yes | `pc-hat-tight` | tight hat | 極短いタイトハット。16分の隙間向き。 | 88 | 0.1 |

## `ld`

| call | in_bank | id | name | description | note | dur |
| --- | --- | --- | --- | --- | --- | --- |
| `ld:ac` | no | `ld-acid` | ld-acid | 303風のジェスチャ。ソー＋LP＋レゾ＋カットオフエンベ。完全な303ではない。 | 48 | 8.2 |
| `ld:an` | no | `ld-anthem` | ld-anthem | アンセムソー。太い単ソー＋オクターブ。フェスティバルEDMのロングノート。 | 48 | 8.2 |
| `ld:ap` | no | `ld-arp-pluck` | ld-arp-pluck | アルペジオ向きの短いプラック。明るい減衰。C4。 | 60 | 8.2 |
| `ld:bl` | no | `ld-bell-pluck` | ld-bell-pluck | ベルプラック。非整数比のトリプルキャリア。C4寄りの高いワンショット。 | 60 | 8.2 |
| `ld:br` | no | `ld-brass` | ld-brass | ブラス寄りのFMスタブ。アタックに高比、すぐ落ち着く。 | 48 | 8.2 |
| `ld:ch` | no | `ld-chip` | ld-chip | チップチューンのパルスリード。細い矩形。C4。 | 60 | 8.2 |
| `ld:cr` | no | `ld-choir` | ld-choir | クワイア寄りの重ねサイン。デチューンした加算。パッド兼リード。 | 48 | 8.2 |
| `ld:cn` | no | `ld-cinematic` | ld-cinematic | 長いシネマティックリード。ゆっくり開くフィルタとLFO。後で切って使う。 | 48 | 8.2 |
| `ld:cy` | no | `ld-crystal` | ld-crystal | クリスタルのキラキラリード。高い部分音。C4。 | 60 | 8.2 |
| `ld:dp` | no | `ld-dist-pulse` | ld-dist-pulse | 歪んだパルスリード。高FBで砂状。HPでサブを切る。 | 48 | 8.2 |
| `ld:ds` | no | `ld-dnb-stab` | ld-dnb-stab | タイトなDnBスタブ。ミッドBP、短い減衰。Amen隣接のメロディワンショット。 | 48 | 8.2 |
| `ld:dr` | no | `ld-drop-pluck` | ld-drop-pluck | ドロッププラック。短いピッチ落下＋LPエンベ。ビルド後のワンショット。 | 48 | 8.2 |
| `ld:et` | no | `ld-ethereal` | ld-ethereal | 空気感のあるパッドリード。遅いアタック、HP。ソフトなロングノート。 | 48 | 8.2 |
| `ld:fp` | no | `ld-fifth-pad` | ld-fifth-pad | 5度パッドリード。持続するC+Gに薄い倍音。コードの下地。 | 48 | 8.2 |
| `ld:fl` | no | `ld-flute` | ld-flute | フルート寄り。遅いアタックのサイン＋薄い息。HPで胴を薄く。 | 48 | 8.2 |
| `ld:fm` | no | `ld-fm-pluck` | ld-fm-pluck | 短いシリアルFMプラック。C3基音。リードバンク用（既存のlead-fm-pluckとは別パッチ）。 | 48 | 8.2 |
| `ld:fo` | no | `ld-formant` | ld-formant | フォルマント寄りのアブサインリード。BPで口の形。 | 48 | 8.2 |
| `ld:fc` | no | `ld-frenchcore` | ld-frenchcore | フレンチコアのスクリームリード。HP/BP、羊毛サブなし。攻撃的ミッドハイ。 | 60 | 8.2 |
| `ld:gb` | no | `ld-gabber` | ld-gabber | ガバリード。歪んだパルスのミッド。短めのホールド、会場のメロディ。 | 48 | 8.2 |
| `ld:gl` | no | `ld-glass` | ld-glass | ガラス／クリスタルリード。高い非整数比。C4。尾は少し長め。 | 60 | 8.2 |
| `ld:gr` | no | `ld-growl` | ld-growl | ミッドのグロウルリード。BP＋フィードバック。キック／ベースのサブは奪わない。 | 48 | 8.2 |
| `ld:hs` | no | `ld-half-sine` | ld-half-sine | ハーフサインの柔らかい三角波寄りリード。丸いメロディ。 | 48 | 8.2 |
| `ld:hd` | no | `ld-hardstyle` | ld-hardstyle | ハードスタイルのスクリーチ寄り。HP、高いモジュレータ。C4。 | 60 | 8.2 |
| `ld:hp` | no | `ld-harpsi` | ld-harpsi | ハープシコード寄り。明るいプラック、速い減衰、高い部分音。 | 48 | 8.2 |
| `ld:hf` | no | `ld-hollow-fifth` | ld-hollow-fifth | 中空5度リード。CとG（比1と1.5）だけ。長3度（5:4）は出さない。少し長い尾。 | 48 | 8.2 |
| `ld:hv` | no | `ld-hoover` | ld-hoover | フーバー／アルファレーン寄り。デチューンソー＋アブサイン。ミッドのうねり。 | 48 | 8.2 |
| `ld:hu` | no | `ld-house-pluck` | ld-house-pluck | ドライなハウスプラック。短いLPエンベ、素のソー＋薄いFM。キックを奪わない。 | 48 | 8.2 |
| `ld:in` | no | `ld-industrial` | ld-industrial | インダストリアルリード。パルス＋高FB。HPで羊毛のようなサブを切る。 | 48 | 8.2 |
| `ld:lz` | no | `ld-laser` | ld-laser | レーザーリード。高い開始ピッチからノートへ着地。C4。 | 60 | 8.2 |
| `ld:ml` | no | `ld-mallet` | ld-mallet | マレット／木琴寄り。ハーフサインの胴と短い減衰。メロディワンショット。 | 48 | 8.2 |
| `ld:mt` | no | `ld-metallic` | ld-metallic | 金属FMリード。インハーモニック比＋固定周波数のリン。攻撃的ミッド。 | 48 | 8.2 |
| `ld:mx` | no | `ld-music-box` | ld-music-box | オルゴール。高いベル＋短い減衰。C4ワンショット。 | 60 | 8.2 |
| `ld:nb` | no | `ld-noisy-bp` | ld-noisy-bp | ノイズ寄りのBPリード。高FBの砂をバンドパスで音符にする。 | 48 | 8.2 |
| `ld:ny` | no | `ld-nylon` | ld-nylon | ミュートしたナイロン寄りのプラック。ハーフサイン＋低いLP。柔らかいメロディ用。 | 48 | 8.2 |
| `ld:oc` | no | `ld-octave` | ld-octave | オクターブスタック。比1と2のソー／サイン。シンプルな厚いリード。 | 48 | 8.2 |
| `ld:or` | no | `ld-organ` | ld-organ | オルガン寄り。並列オペ（ドローバー風 1+2+3）。加算＋薄いFB。 | 48 | 8.2 |
| `ld:pc` | no | `ld-perc` | ld-perc | パーカッション寄りのリード。クリックアタック＋短いトーン。HPでサブを切る。 | 60 | 8.2 |
| `ld:pu` | no | `ld-pulse` | ld-pulse | スクエア／パルスリード。奇数倍音。ハウスやテクノのメロディ。 | 48 | 8.2 |
| `ld:rd` | no | `ld-reed` | ld-reed | リード／クラリネット寄り。奇数倍音のパルス＋サイン。 | 48 | 8.2 |
| `ld:rv` | no | `ld-reverse` | ld-reverse | リバース風。遅いアタック＋上昇ピッチエンベ。スイープ兼ノート。 | 48 | 8.2 |
| `ld:sw` | no | `ld-saw-pluck` | ld-saw-pluck | ドライな単ソーのプラック。飾りなし。ハウス／テクノの基本。 | 48 | 8.2 |
| `ld:si` | no | `ld-sine` | ld-sine | クリーンなサインリード。薄いビブラート。メロディの芯。 | 48 | 8.2 |
| `ld:ss` | yes | `ld-supersaw` | ld-supersaw | クラシックなトランス／EDMスーパーソーリード。長め。HPでサブを抑える。 | 48 | 8.2 |
| `ld:st` | no | `ld-supersaw-stab` | ld-supersaw-stab | 広いスーパーソースタブ。短いアンプ、厚みはユニゾン。トランス／EDMのコードヒット。 | 48 | 8.2 |
| `ld:sf` | no | `ld-sync-fm` | ld-sync-fm | シンク風FM。高い比のモジュレータでオシレータシンクっぽい倍音。 | 48 | 8.2 |
| `ld:tg` | no | `ld-trance-gate` | ld-trance-gate | トランスのゲート風。アンプLFOは無いのでフィルタADSRで開閉する。 | 48 | 8.2 |
| `ld:us` | no | `ld-unison-saw` | ld-unison-saw | ユニゾンソー（デチューン控え）。スーパーソーより締まったリード。 | 48 | 8.2 |
| `ld:vw` | no | `ld-vowel` | ld-vowel | 母音FM。非整数モジュレータでアー／オー。BPが口。 | 48 | 8.2 |
| `ld:wb` | no | `ld-wobble` | ld-wobble | ミッドウォブル。ピッチLFO＋フィルタエンベ。ニューロ寄りの音符。 | 48 | 8.2 |
| `ld:zp` | no | `ld-zap` | ld-zap | ザップリード。下向きピッチでもノートとして使える。フィル兼メロディ。 | 48 | 8.2 |
| `ld:lp` | no | `lead-fm-pluck` | lead-fm-pluck | C3（MIDI 48、約130.8 Hz）の短いFMプラック。C4ではない。速いアンプとフィルタADSR。メロディ用。 alias `plk:lp` | 48 | 8.2 |

## `oh`

| call | in_bank | id | name | description | note | dur |
| --- | --- | --- | --- | --- | --- | --- |
| `oh:op` | yes | `pc-hat-open` | open hat | オープンハット。尾は約1秒。クローズより長いノイズ減衰。 | 84 | 1.15 |
| `oh:dn` | yes | `pc-hat-dnb-op` | dnb open hat | DnBのオープンハット。中くらいの尾。ロールやオフビート向き。 | 80 | 1.05 |
| `oh:fc` | yes | `pc-hat-fc-op` | frenchcore open hat | フレンチコアのオープンハット。硬いハイ＋少し長い尾。 | 86 | 0.95 |

## `perc`

| call | in_bank | id | name | description | note | dur |
| --- | --- | --- | --- | --- | --- | --- |
| `perc:ah` | yes | `pc-agogo-hi` | high agogo | アゴゴの高音。明るい金属ベル。 | 84 | 0.38 |
| `perc:al` | yes | `pc-agogo-lo` | low agogo | アゴゴの低音。ハイより丸い金属。 | 76 | 0.42 |
| `perc:bh` | yes | `pc-bongo-hi` | high bongo | ハイボンゴ。コンガより明るく短い。 | 67 | 0.28 |
| `perc:bl` | yes | `pc-bongo-lo` | low bongo | ローボンゴ。ハイより丸い膜。 | 60 | 0.32 |
| `perc:cb` | yes | `pc-cabasa` | cabasa | カバサ。ざらついた短いスクレイプ。シェイカーより暗く粗い。 | 72 | 0.24 |
| `perc:cm` | yes | `pc-chime` | chime hit | 短いチャイム。ガラスヒットより低く、ベルの粒。 | 84 | 0.55 |
| `perc:cv` | yes | `pc-clave` | clave | クラべ。乾いた短い木のクリック。 | 76 | 0.16 |
| `perc:gh` | yes | `pc-conga-hi` | high conga | ハイコンガ。短い膜のトーン。キックではない。 | 62 | 0.35 |
| `perc:gl` | yes | `pc-conga-lo` | low conga | ローコンガ。ハイより低く長い膜。キックの808ブームではない。 | 53 | 0.42 |
| `perc:cw` | yes | `pc-cowbell` | cowbell | カウベル。2つの固定部分音。短い金属。 | 72 | 0.4 |
| `perc:fk` | yes | `pc-foley-click` | foley click | フォリーのクリック。乾いたプラスチック／スイッチ。 | 84 | 0.12 |
| `perc:fs` | yes | `pc-foley-scratch` | foley scratch | フォリーの短いスクラッチ。ざらついたノイズの一擦り。 | 72 | 0.2 |
| `perc:ft` | yes | `pc-foley-thud` | foley thud | フォリーの短いドサッ。ミュートした胴。キックや808ではない。 | 48 | 0.22 |
| `perc:gr` | yes | `pc-guiro` | guiro scrape | ギロの短いスクレイプ。ざらついた一擦り。 | 67 | 0.26 |
| `perc:rb` | yes | `pc-ride-bell` | ride bell | ライドベル。短い金属の芯。ライド本体よりタイト。 | 84 | 0.55 |
| `perc:rf` | yes | `pc-ride-fm` | fm ride | ライド寄りのFM。長いベル＋砂。オープンハットより金属。最大約1.2秒。 | 80 | 1.18 |
| `perc:rm` | yes | `pc-rim` | wood rim | 木寄りのリム。sd-rimshot より金属リンが弱く、短いウッドクリック。スネア代用ではない。 | 60 | 0.22 |
| `perc:sk` | yes | `pc-shaker` | shaker | シェイカー。粒状のハイノイズ。短いワンショット（ループ前提ではない）。 | 84 | 0.28 |
| `perc:ss` | yes | `pc-shaker-short` | short shaker | 極短いシェイカーティック。16分の粒。 | 88 | 0.12 |
| `perc:sn` | yes | `pc-snap` | finger snap | フィンガースナップ。短いミッドの肉＋ハイのクリック。 | 72 | 0.18 |
| `perc:sl` | yes | `pc-snap-lo` | low snap | 低いスナップ。ミッド寄りの指。ハイの粒は薄く。 | 60 | 0.2 |
| `perc:np` | yes | `pc-snaps` | snap cluster | スナップの重ね。わずかに遅れた2粒。pc-snap より広い。 | 72 | 0.26 |
| `perc:st` | yes | `pc-stick` | stick click | スティックのクリック。リムより乾いて短い。 | 76 | 0.14 |
| `perc:tb` | yes | `pc-tamb` | tambourine | タンバリン。ジングルの金属＋短い皮。ハットよりリンが残る。 | 84 | 0.45 |
| `perc:tr` | yes | `pc-tamb-roll` | tambourine roll hit | タンバリンの短いロール風ヒット。粒を重ねたワンショット。 | 84 | 0.38 |
| `perc:tk` | yes | `pc-tick-clock` | clock tick | 時計のティック。乾いた短いクリック。 | 88 | 0.11 |
| `perc:ti` | yes | `pc-tick-indust` | industrial tick | インダストリアルなティック。暗い金属＋砂。 | 72 | 0.16 |
| `perc:tm` | yes | `pc-tick-metal` | metallic tick | 金属ティック。極短いリン。ハットよりピッチがある。 | 96 | 0.14 |
| `perc:tg` | yes | `pc-triangle` | triangle | トライアングル。高い非整数のリン。尾は短めワンショット。 | 96 | 0.7 |
| `perc:wb` | yes | `pc-woodblock` | woodblock | ウッドブロック。乾いた中高域のトン。クラべより胴がある。 | 72 | 0.2 |
| `perc:zp` | yes | `pc-zap` | perc zap | パーカッションのザップ。短い下降ピッチ。リードやFXのレーザーとは別の短いヒット。 | 72 | 0.22 |
| `perc:zl` | yes | `pc-zap-lo` | low perc zap | 低いパーカッションザップ。ミッドの短い落下。キックではない。 | 55 | 0.28 |
| `perc:gs` | yes | `glass-hit` | glass-hit | ガラス／ベル系の短いヒット。高整数比とトリプルキャリア。トップやアクセント。 | 84 | 0.7 |
| `perc:mh` | yes | `metallic-hit` | metallic-hit | 金属ヒット。固定周波数オペでピッチに追従しない倍音。パーカッション／トップ。 | 72 | 0.55 |
| `perc:fm` | yes | `perc-fm_metal` | FM metal hit | Unpitched metallic hit. Not a stab or kick. |  |  |

## `pf`

| call | in_bank | id | name | description | note | dur |
| --- | --- | --- | --- | --- | --- | --- |
| `pf:al` | no | `pf-alpine` | alpine pad | 高山の澄んだ5度。明るいが軽い。 | 67 | 16.5 |
| `pf:br` | no | `pf-breeze` | breeze pad | そよ風。パルスの薄い息＋サイン。HP。 | 64 | 16.6 |
| `pf:ca` | no | `pf-choir-air` | choir air pad | 柔らかいクワイアの空気。加算サイン、HP。泥は入れない。 | 60 | 16.7 |
| `pf:cw` | no | `pf-chorus-wide` | wide chorus pad | 広いコーラスパッド。デチューンしたサインの重ね。午前の空気。 | 60 | 16.4 |
| `pf:cs` | no | `pf-clear-saw` | clear saw pad | 澄んだソーパッド。HPで低域を切り、午前のアナログ。 | 60 | 16.5 |
| `pf:cl` | no | `pf-cloud` | soft cloud pad | 柔らかい雲。ハーフサインの層。湿った空気、泥なし。 | 60 | 16.9 |
| `pf:dn` | no | `pf-dawn` | dawn bloom pad | 夜明けのブルーム。フィルタがゆっくり開く。メジャー寄り。 | 55 | 17.0 |
| `pf:fo` | no | `pf-fifth-open` | open fifth pad | 開いた5度パッド（C+G）。長3度なし。C3でもHPで150 Hz以上。 | 48 | 16.5 |
| `pf:fl` | no | `pf-flute-pad` | flute pad | フルートパッド。遅い息＋サイン。HPで胴なし。C5。 | 72 | 16.8 |
| `pf:ga` | no | `pf-glass-air` | glass air pad | ガラスの空気。薄い非整数比。ベルワンショットではない。 | 67 | 16.9 |
| `pf:hl` | no | `pf-halo` | halo choir pad | ハローのクワイア空気。広いデチューン、HP。 | 60 | 17.0 |
| `pf:ha` | no | `pf-harp-air` | harp air pad | ハープの空気。アタックは少し立つが16秒ホールド。 | 67 | 16.7 |
| `pf:hz` | no | `pf-horizon` | horizon pad | 地平線。広い5度＋9度。C3の開いた配置。 | 48 | 16.7 |
| `pf:iv` | no | `pf-ivory` | ivory pad | 象牙／柔らかい鍵盤のパッド。倍音は薄い。ホールド。 | 64 | 16.5 |
| `pf:ju` | no | `pf-juno-air` | juno air pad | Juno風の広いが軽いパッド。ソー＋サイン、HPで胴を切る。 | 60 | 16.6 |
| `pf:ln` | no | `pf-linen` | linen pad | リネンの質感。アブサインの薄いフォルマント。爽やか。 | 60 | 16.7 |
| `pf:ly` | no | `pf-lydian-sky` | lydian sky pad | リディアン（#4=11/8）の空。明るいが軽い。 | 60 | 16.8 |
| `pf:ms` | no | `pf-major-soft` | soft major pad | 柔らかい長三和音パッド（C–E–G）。開いた配置。 | 60 | 16.4 |
| `pf:md` | no | `pf-meadow` | meadow pad | 草原。長3度＋5度の柔らかい加算。朝。 | 64 | 16.4 |
| `pf:mn` | no | `pf-morning` | morning chorus pad | 朝のアナログコーラスパッド。軽いスーパーソー＋HP。低域はドローンに任せる。 | 60 | 16.5 |
| `pf:ni` | no | `pf-ninth-open` | open ninth pad | 開いた9度（根音＋9度＋5度）。ワイドだがサブなし。 | 48 | 16.6 |
| `pf:oc` | no | `pf-octave-light` | light octave pad | 軽いオクターブ重ね（1+2）。サブの0.5は使わない。 | 60 | 16.3 |
| `pf:or` | no | `pf-organ-light` | light organ pad | 軽いオルガン（1・2・3・4）。ドローバーだがHPで床なし。 | 55 | 16.4 |
| `pf:pu` | no | `pf-pulse-air` | pulse air pad | 中空のパルス空気。スクエアの隙間、HP。 | 55 | 16.6 |
| `pf:rs` | no | `pf-reed-soft` | soft reed pad | 柔らかいリード／リード管。パルス芯＋サイン。HP。 | 60 | 16.6 |
| `pf:sk` | no | `pf-silk` | silk sine pad | 絹のサインパッド。ごく薄いコーラス。澄んでいる。 | 67 | 16.8 |
| `pf:so` | no | `pf-sky-open` | open sky pad | 開いた空。オクターブ＋5度の高い配置。 | 64 | 16.5 |
| `pf:sp` | no | `pf-spring` | spring pad | 春。リディアン寄り＋空気。明るく開く。 | 64 | 16.8 |
| `pf:wa` | no | `pf-water-air` | water air pad | 水の空気。遅いLFO、薄いモジュレーション。泥なし。 | 60 | 17.1 |
| `pf:wm` | no | `pf-wide-major` | wide major pad | 開いた長三和音（根音＋10度＋12度）。泥のないワイド。 | 48 | 16.6 |
| `pf:ff` | yes | `pad-fm_fifth` | fifth pad | C3 hollow C+G pad (~8 s). Write C4:…. Cannot invent a third. |  |  |

## `plk`

| call | in_bank | id | name | description | note | dur |
| --- | --- | --- | --- | --- | --- | --- |
| `plk:ac` | yes | `pl-acid-short` | acid short pluck | アシッド。レゾで明るいミッド。短い（0.36秒）。ソー＋高いQのLP。 | 48 | 0.36 |
| `plk:am` | yes | `pl-ambient-soft` | ambient soft pluck | アンビエント。柔らかく暗い〜中庸。やや長いワンショット（1.18秒）。サステインほぼなし。 | 60 | 1.18 |
| `plk:aj` | yes | `pl-arp-major` | arp major pluck | アルペジオ。長3度（5:4）で明るい。極短い（0.30秒）。 | 67 | 0.3 |
| `plk:an` | yes | `pl-arp-minor` | arp minor pluck | アルペジオ。短3度（6:5）で中庸の明るさ。極短い（0.30秒）。 | 67 | 0.3 |
| `plk:bp` | yes | `pl-bass-pluck` | bass pluck | ベースプラック。暗い。短い（0.48秒）。ソー＋低いLP。C3。 | 48 | 0.48 |
| `plk:ch` | yes | `pl-chime-high` | chime high pluck | チャイム。非常に明るい高域。中短（0.85秒）。高い非整数比。 | 72 | 0.85 |
| `plk:cv` | yes | `pl-clav-funk` | clav funk pluck | ファンククラビ。ミッド明るく鼻にかかったBP。極短い（0.28秒）。パルス＋ソー。 | 60 | 0.28 |
| `plk:nn` | yes | `pl-dnb-neuro` | dnb neuro pluck | ニューロファンク。金属質でミッド暗い。短い（0.33秒）。非整数FM＋BP。 | 50 | 0.33 |
| `plk:dt` | yes | `pl-dnb-tight` | dnb tight pluck | DnB。タイトでミッド寄りの明るさ。極短い（0.26秒）。パルス＋速いBP。 | 53 | 0.26 |
| `plk:fc` | yes | `pl-fm-crystal` | fm crystal pluck | クリスタルFM。非常に明るい。中短（0.68秒）。高い比のシリアル。 | 72 | 0.68 |
| `plk:ep` | yes | `pl-fm-ep` | fm ep pluck | FMエレクトリックピアノ。暖かめで中庸の明るさ。やや長め（0.95秒）。1:14タイン。 | 60 | 0.95 |
| `plk:fg` | yes | `pl-future-glass` | future glass pluck | フューチャーベース。ガラス質で明るい。中短（0.65秒）。非整数比のシリアルFM。 | 69 | 0.65 |
| `plk:gm` | yes | `pl-guitar-mute` | guitar mute pluck | ミュートギター。暗く短い。極短い（0.27秒）。ソー＋爪、低いLP。 | 52 | 0.27 |
| `plk:hp` | yes | `pl-harp-open` | harp open pluck | ハープ。開いた明るさ。やや長いワンショット（1.15秒）。サイン＋5度。 | 67 | 1.15 |
| `plk:hb` | yes | `pl-house-bright` | house bright pluck | ハウス。明るい。短いワンショット（0.38秒）。ソー＋2:1/3:1のキラッと。 | 64 | 0.38 |
| `plk:hd` | yes | `pl-house-dry` | house dry pluck | ハウス。ドライで中庸の明るさ。短いワンショット（0.42秒）。単ソー＋控えめLP。 | 60 | 0.42 |
| `plk:kl` | yes | `pl-kalimba` | kalimba pluck | カリンバ。金属＋木の明るさ。短い（0.58秒）。タインの非整数比。 | 67 | 0.58 |
| `plk:kt` | yes | `pl-koto` | koto pluck | 箏（琴）。明るく鋭いアタック。中短（0.68秒）。わずかなピッチ落下。 | 64 | 0.68 |
| `plk:lf` | yes | `pl-lofi-dust` | lofi dust pluck | ローファイ。暗くダストっぽい。中短（0.72秒）。デチューン＋低いLP。 | 55 | 0.72 |
| `plk:mb` | yes | `pl-mallet-bell` | mallet bell pluck | マレット／ベル。明るく金属質。中短（0.72秒）。非整数のトリプルキャリア。 | 71 | 0.72 |
| `plk:mm` | yes | `pl-mallet-marimba` | mallet marimba pluck | マレット／木琴。木質で中庸の明るさ。短い（0.48秒）。ハーフサインの胴。 | 65 | 0.48 |
| `plk:mx` | yes | `pl-musicbox` | musicbox pluck | オルゴール。明るく高い。短い（0.55秒）。奇数倍音のサイン。 | 72 | 0.55 |
| `plk:pk` | yes | `pl-perc-click` | perc click pluck | パーカッション寄りのクリックプラック。明るく極短い（0.25秒）。パルス＋HP。 | 72 | 0.25 |
| `plk:ny` | yes | `pl-pop-nylon` | pop nylon pluck | ポップ。ナイロン質でやや暗い。中短（0.52秒）。ハーフサイン＋爪クリック。 | 64 | 0.52 |
| `plk:ps` | yes | `pl-pop-soft` | pop soft pluck | ポップ。柔らかく中庸の明るさ。中短（0.62秒）。サイン重ね。 | 62 | 0.62 |
| `plk:rv` | yes | `pl-reverse-swell` | reverse swell pluck | リバーススウェル。中庸の明るさ。唯一やや長いプラック（1.75秒）。遅いアタックのあと消える。 | 60 | 1.75 |
| `plk:sf` | yes | `pl-stab-fifth` | stab fifth pluck | スタブ。中空5度、中庸の明るさ。短い（0.38秒）。C+Gのみ。 | 55 | 0.38 |
| `plk:sm` | yes | `pl-stab-major` | stab major pluck | スタブ。長三和音で明るい。短い（0.40秒）。C–E–G。 | 55 | 0.4 |
| `plk:ss` | yes | `pl-supersaw-short` | supersaw short pluck | EDM。厚いスーパーソーで中庸の明るさ。短い（0.36秒）。ユニゾン5本。 | 55 | 0.36 |
| `plk:tg` | yes | `pl-trance-gate` | trance gate pluck | トランス。明るくゲートしたスーパーソー。極短い（0.32秒）。速いLP閉じ。 | 60 | 0.32 |
| `plk:fp` | yes | `filter-pluck` | filter-pluck | カットオフADSRで開いて閉じるプラック。低めのLPから3–6 kHz付近まで開く。 | 60 | 8.2 |
| `plk:sp` | yes | `stab-pluck` | stab-pluck | 短いスタブ／プラック。デュアルスタックで芯と倍音を分離。コードのワンショット。 | 60 | 8.2 |
| `plk:s5` | yes | `stab-fm-fifth` | stab-fm-fifth | C3の中空DnBスタブ。完全5度（CとG、比1と3/2）の2パーシャルだけ。長3度（E / 5:4）は出さない。 alias `plk:s5` | 48 | 8.2 |
| `plk:s3` | yes | `stab-fm-major` | stab-fm-major | C3の明るい長三和音スタブ。C–E–G（比1、5/4、3/2）の3パーシャル。中空の stab-fm-fifth（C–Gのみ）の対。 alias `plk:s3` | 48 | 8.2 |
| `plk:bl` | yes | `lead-fm_bell` | FM bell pluck | C3 inharmonic bell / glass (ratio 3.5). Write C4:…. Not a pad. |  |  |
| `plk:lp` | yes | `lead-fm_pluck` | FM pluck | C3 short FM pluck. Write C4:…. .cut(1) if monophonic. |  |  |

## `ps`

| call | in_bank | id | name | description | note | dur |
| --- | --- | --- | --- | --- | --- | --- |
| `ps:au` | no | `ps-aurora` | aurora pad | オーロラ。ゆっくり色が変わるFMの輝き。 | 64 | 17.2 |
| `ps:ba` | no | `ps-bell-air` | bell air pad | ベルの空気。金属というより光。ホールド。 | 72 | 16.6 |
| `ps:bh` | no | `ps-bell-hold` | held bell pad | ベルのホールド。アタックは立つがサステインで16秒残る。 | 72 | 16.8 |
| `ps:ce` | no | `ps-celesta` | celesta pad | チェレスタパッド。鍵盤ベルを伸ばしたホールド。 | 72 | 16.5 |
| `ps:cs` | no | `ps-celestial` | celestial pad | 天のパッド。クワイア＋高い輝き。 | 64 | 17.0 |
| `ps:cp` | no | `ps-chime-pad` | chime pad | チャイムパッド。金属の高い層をホールド。 | 72 | 16.8 |
| `ps:ch` | no | `ps-chorus-shine` | chorus shine pad | 遅いコーラスの輝き。キラキラが横に広がる。 | 64 | 16.7 |
| `ps:cr` | no | `ps-crystal` | crystal pad | クリスタルパッド。高い部分音。ホールド（ワンショットベルではない）。 | 72 | 16.6 |
| `ps:cc` | no | `ps-crystal-choir` | crystal choir pad | クリスタルクワイア。サイン重ね＋高い部分音。 | 67 | 16.9 |
| `ps:dm` | no | `ps-diamond` | diamond pad | ダイヤモンド。硬い高次、明るいホールド。 | 72 | 16.5 |
| `ps:fs` | no | `ps-fm-sparkle` | evolving fm sparkle | ゆっくり指数が開くFMスパークル。キラキラが育つ。 | 67 | 17.2 |
| `ps:fr` | no | `ps-frost` | frost pad | 霜。冷たい高域の層。キラキラは控えめ。 | 69 | 16.8 |
| `ps:gb` | no | `ps-glass-bell` | glass bell pad | ガラスベルのパッド。高い非整数比を持続。 | 72 | 16.9 |
| `ps:gl` | no | `ps-glisten` | glisten pad | きらめき。高域がゆっくり呼吸する。 | 69 | 16.9 |
| `ps:gt` | no | `ps-glitter` | glitter pad | グリッター。高次倍音の粉。パッドとして残る。 | 72 | 16.6 |
| `ps:gp` | no | `ps-glock-pad` | glock pad | グロッケンパッド。鉄琴の輝きを伸ばす。 | 72 | 16.5 |
| `ps:hs` | no | `ps-halo-shine` | halo shine pad | ハローの輝き。広いデチューン＋高次。 | 64 | 16.8 |
| `ps:hp` | no | `ps-high-partials` | high partials pad | 高次奇数倍音（1・3・5・7）。キラキラの骨格。 | 60 | 16.4 |
| `ps:ic` | no | `ps-ice-choir` | ice choir pad | 氷のクワイア。冷たい重ねサイン＋輝き。 | 67 | 17.0 |
| `ps:is` | no | `ps-ice-shine` | ice shine pad | 氷の輝き。冷たい高域。ホールド。 | 72 | 16.7 |
| `ps:ih` | no | `ps-inharmonic` | inharmonic sparkle | 非整数比のスパークル。金属だがパッドとして残る。 | 67 | 16.8 |
| `ps:mx` | no | `ps-music-box` | music box pad | オルゴールパッド。高いベル層をホールド（減衰しきらない）。 | 72 | 16.5 |
| `ps:pr` | no | `ps-prism` | prism pad | プリズム。スペクトルがゆっくり割れる。 | 67 | 17.1 |
| `ps:qz` | no | `ps-quartz` | quartz pad | 石英。硬い透明感。高い部分音のホールド。 | 69 | 16.7 |
| `ps:sh` | no | `ps-shimmer` | shimmer pad | シマー。高い部分音がゆっくり揺れる。キラキラのホールド。 | 67 | 16.9 |
| `ps:sf` | no | `ps-shine-fifth` | shine fifth pad | 輝く5度。開いたC+Gに高い粉。 | 60 | 16.6 |
| `ps:sv` | no | `ps-silver` | silver pad | 銀。冷たい金属の層。キラキラ控えめのホールド。 | 67 | 16.7 |
| `ps:se` | no | `ps-spark-evolve` | evolving spark pad | 火花がゆっくり育つ。モジュレーションスイープ。 | 67 | 17.2 |
| `ps:sl` | no | `ps-starlight` | starlight pad | 星明かり。高い非整数比がゆっくり動く。 | 69 | 17.0 |
| `ps:tw` | no | `ps-twinkle` | twinkle pad | トゥインクル。星のまたたきをホールド（ワンショットではない）。 | 72 | 16.6 |

## `sd`

| call | in_bank | id | name | description | note | dur |
| --- | --- | --- | --- | --- | --- | --- |
| `sd:8s` | yes | `sd-808-snap` | 808 snap | TR-808風スネア。短いトーン2本とノイズ。胴は短め、ノイズに尾を残す。 | 40 | 0.75 |
| `sd:9s` | yes | `sd-909-snappy` | 909 snappy | TR-909風スネア。スナップが強く胴もある。ノイズボディに余韻。 | 42 | 0.9 |
| `sd:br` | yes | `sd-brush-dust` | brush dust | ブラシ／ダストのローファイ。柔らかいアタックと砂状の尾。テープっぽい揺れ。 | 39 | 1.25 |
| `sd:cs` | yes | `sd-clap-snare` | clap snare | クラップ寄りのスネア。重ねノイズの粒だが、短いトーン芯は残す（純クラップではない）。 | 42 | 0.85 |
| `sd:dn` | yes | `sd-dnb-tight` | dnb tight | DnBのタイトスネア。ミッドの割れ。短いが使える尾は残す（チョークしすぎない）。 | 43 | 0.72 |
| `sd:fb` | yes | `sd-fat-backbeat` | fat backbeat | 太いバックビート。低ミッドの胴が長く、ノイズは後ろで余韻。 | 38 | 1.15 |
| `sd:fm` | yes | `sd-fm-long` | fm long | 実験的な長いFMスネア。使えるリンとヒスの尾。ドローンにはしない。 | 42 | 1.75 |
| `sd:fc` | yes | `sd-frenchcore` | frenchcore | フレンチコア／ハードコアの割れ。攻撃的なミッド。サブは削って裂けるように。 | 45 | 0.68 |
| `sd:gb` | yes | `sd-gabber-indust` | gabber indust | ガバ／インダストリアル。金属の踏み込みと歪んだミッド。工場っぽい尾。 | 46 | 0.78 |
| `sd:gt` | yes | `sd-gated-80s` | gated 80s | 80年代ゲートスネア。長いノイズボディ。DAW側でゲートする前提で尾を残す。 | 40 | 1.55 |
| `sd:hd` | yes | `sd-house-disco` | house disco | ハウス／ディスコのドライスネア。ブーム無し、乾いた割れ。部屋ヒスは薄く残す。 | 38 | 0.58 |
| `sd:jg` | yes | `sd-jungle-round` | jungle round | ジャングルの丸いスネア。胴が温かく、スナップは控えめ。ブレイク向き。 | 40 | 1.0 |
| `sd:mt` | yes | `sd-metal-ping` | metal ping | 金属的なFMピン・スネア。固定周波数のリンと短い胴。インダストリアル寄り。 | 48 | 0.88 |
| `sd:ng` | yes | `sd-neuro-growl` | neuro growl | ニューロ／グロウルのFMスネア。喉声ミッドと砂状の尾。 | 41 | 1.05 |
| `sd:ns` | yes | `sd-noise-layer` | noise layer | ノイズだけのレイヤースネア。胴なし。他スネアの上に重ねる砂／ヒス。 | 44 | 0.82 |
| `sd:pi` | yes | `sd-piccolo` | piccolo | ピッコロ／ハイクラック。高い割れと短い胴。ヒスの尾は残す。 | 50 | 0.6 |
| `sd:pp` | yes | `sd-pop-tight` | pop tight | タイトなアコースティック／ポップスネア。短い胴でも部屋っぽい尾は残す。 | 38 | 0.55 |
| `sd:rm` | yes | `sd-rimshot` | rimshot | リムショット。鋭いクリックと金属のリン。短い胴、リンの尾は残す。 | 48 | 0.7 |
| `sd:tn` | yes | `sd-tone-layer` | tone layer | 胴／トーンだけのレイヤースネア。ノイズはごく薄い。他スネアの下に重ねる。 | 38 | 0.7 |
| `sd:tr` | yes | `sd-trap-crisp` | trap crisp | トラップスネア。クリスプな割れと少し長めの尾。808スネアより明るい。 | 40 | 0.95 |

## `tom`

| call | in_bank | id | name | description | note | dur |
| --- | --- | --- | --- | --- | --- | --- |
| `tom:hi` | yes | `pc-tom-hi` | high tom | ハイタム。短い膜。キックバンク（bd-*）ではない。 | 62 | 0.38 |
| `tom:lo` | yes | `pc-tom-lo` | low tom | ロータム。膜の胴。808ブームや bd-* キックではない（ピッチ落下は小さくサブなし）。 | 48 | 0.55 |
| `tom:md` | yes | `pc-tom-mid` | mid tom | ミッドタム。フロアより高くキックより明るい。 | 55 | 0.45 |
