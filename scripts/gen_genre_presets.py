#!/usr/bin/env python3
"""Rewrite bundled genre presets from strudel-genre-* skill fences.

Holds out catalog families (clockwork / glass-garden / night-market / tide-lantern).
01 keeps e2e titles; 02–30 transpose and apply named skill variants.
"""
from __future__ import annotations

import argparse
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SONGS = ROOT / "songs"

HOLD = {"clockwork", "glass-garden", "night-market", "tide-lantern"}

# 10 parent keys, three skill variants → 30 slots.
KEYS = [0, 2, 3, 4, 5, 7, 8, 9, 10, 11]
PC_NAMES = ["C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B"]

SCALE_TOKEN = re.compile(r"([A-G](?:#|b)?)(\d):([A-Za-z]+)")
TITLE_RE = re.compile(r"(// @title )[^\n]+")

# e2e-locked titles for slot 01.
TITLE_01 = {
    "house": "warehouse-intro",
    "four-on-the-floor": "sine-pulse",
    "dnb": "dnb-01",
    "techno-duck": "pump-core",
    "acid": "saw-303",
    "dnb-reese": "dnb-reese-01",
}

FENCES: dict[str, str] = {}

FENCES["house"] = r'''// @title warehouse-intro
// @genre house
setcpm(124/4)
// drums
$: s("bd:hf*4, [~ cp]*2, [~ hh:hs]*4, <~ ~ ~ [~@3 bd:hf ~@4]>").gain(0.58)
// bass
$: note("0 0 4 <0 2 4 0>").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("bs:hf").gain(0.42)
// lead
$: note("~ 7 4 <7 9 4 2>").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("ld:ss").gain(0.16).cut(1)
// hook
$: note("4 ~ 7 4  2 0 ~ -1").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("plk:lp").gain(0.22).cut(1)
// arp
$: note("~ 0 4 7  ~ 4 0 2").scale("<C5:minor C5:minor G5:dorian C5:minor>")
  .s("plk:hd").gain(0.14).cut(1)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("ep:ky").gain(0.22)
// pad
$: note("0").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("pf:ff").gain(0.16).room(0.3).orbit(2)
'''

FENCES["four-on-the-floor"] = r'''// @title sine-pulse
// @genre techno
setcpm(124/4)
// drums
$: s("bd*4, [~ hh]*4, <~ ~ ~ [~@3 bd ~@4]>").gain(0.62)
// bass
$: note("0 0 2 <4 0 3 0>").scale("<C2:minor C2:minor G2:phrygian C2:minor>")
  .s("sawtooth").lpf(400).gain(0.44)
  .attack(0.001).decay(0.08).sustain(0.2).release(0.05)
// lead
$: note("~ 7 6 <4 9 3 7>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("ld:ss").gain(0.15).cut(1)
// hook
$: note("~ 4 ~ <7 4 4 7>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("plk:s5").gain(0.18).cut(1)
// arp
$: note("0 3 0 7  3 0 5 ~").scale("<C5:minor C5:minor G5:phrygian C5:minor>")
  .s("plk:ac").gain(0.12).cut(1)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("ep:ky").gain(0.22)
// pad
$: note("0").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("pf:ff").gain(0.16).room(0.3).orbit(2)
'''

FENCES["techno-duck"] = r'''// @title pump-core
// @genre techno
setcpm(126/4)
// kick
$: s("bd*4").gain(0.9).duckorbit(2).duckattack(0.04).duckdepth(0.85)
// hats
$: s("[~ hh]*4, <~ ~ ~ hh*8>").gain(0.38)
// bass
$: note("0 0 2 <4 6 2 0>").scale("<C2:minor C2:minor G2:phrygian C2:minor>")
  .s("sine").fm(3).fmh(1.5).lpf(500).gain(0.52)
  .attack(0.005).decay(0.1).sustain(0.3).release(0.08)
  .orbit(2)
// lead
$: note("~ 7 4 <9 7 4 2>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("sine").fm(3).fmh(2).fmatt(0.01).fmdec(0.3).fmsus(0.25)
  .lpf(1800).lpenv(2).gain(0.16)
// hook
$: note("~ 4 ~ <7 4 4 7>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("plk:s5").gain(0.16).cut(1)
// arp
$: s("<~ perc:tm ~ perc:st>").gain(0.18)
// chords
$: note("[0,2,4] ~ ~ ~").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("ep:ky").gain(0.18).orbit(2)
// pad
$: note("0").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("pf:ff").orbit(2).gain(0.22).room(0.25)
'''

FENCES["acid"] = r'''// @title saw-303
// @genre acid techno
setcpm(130/4)
// drums
$: s("bd*4, [~ hh]*4, <~ ~ ~ [~@3 bd ~@4]>").gain(0.62)
// bass
$: note("0 ~ 0 <0 0 3 0>").scale("<C2:minor C2:minor G2:phrygian C2:minor>")
  .s("sine").lpf(180).gain(0.32)
// hook
$: note("0 0 3 0  7 3 2 0  4 4 3 0  -1 3 0 <2 5>")
  .scale("C2:minor")
  .s("sawtooth")
  .lpf("260 260 720 260  760 260 260 260  260 680 260 260  740 720 260 260")
  .lpq(14)
  .lpenv(3)
  .lpattack(0.001)
  .lpdecay(0.09)
  .lpsustain(0.05)
  .cut(1)
  .gain(0.44)
  .attack(0.001)
  .decay(0.1)
  .sustain(0.12)
  .release(0.04)
// lead
$: note("~ 7 ~ <9 7 4 12>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("plk:ac").gain(0.12).cut(1)
// arp
$: s("<~ perc:mh ~ perc:tm>").gain(0.14)
// chords
$: note("[0,4] ~ ~ [0,4]").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("ep:mt").gain(0.16)
// pad
$: note("0").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("pf:ff").gain(0.12).room(0.25).orbit(2)
'''

FENCES["dnb"] = r'''// @title dnb-01
// @genre drum and bass
setcpm(174/4)
// drums
$: s("[bd <~ sd> ~ sd ~ <bd ~> <bd sd> <bd ~>, hh*4, [~@5 oh ~@2]]*2").gain(0.7).lpf(4000)
// bass
$: note("0 3 0 <0 -1 0 3>").scale("C2:minor")
  .s("square").lpf(120).gain(0.42)
  .attack(0.01).decay(0.5).release(0.4)
// bass-mid
$: note("0 3 0 <0 -1 0 3>").scale("C2:minor")
  .s("sawtooth").lpf(1000).gain(0.32)
  .attack(0.01).decay(0.4).release(0.3)
// lead
$: note("~ 7 ~ <9 7 4 11>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("ld:ss").gain(0.14).cut(1)
// hook
$: note("~ 4 ~ <7 4 4 7>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("plk:dt").gain(0.16).cut(1)
// arp
$: s("<~ perc:st ~ perc:tm>").gain(0.16)
// chords
$: note("[0,4] ~ [0,4] ~").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("ep:mt").gain(0.16)
// pad
$: note("0").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("pf:ff").gain(0.12).room(0.22).orbit(2)
'''

FENCES["dnb-reese"] = r'''// @title dnb-reese-01
// @genre drum and bass
setcpm(174/4)
// drums
$: s("[bd <~ sd> ~ sd ~ <bd ~> <bd sd> <bd ~>, hh*4, [~@5 oh ~@2]]*2").gain(0.7).lpf(4000)
// bass
$: note("0 3 0 <0 -1>").scale("C2:minor").s("square").lpf(120).gain(0.42)
// bass-mid
$: note("0 3 0 <0 -1>").scale("C4:minor").s("bs:rm").gain(0.36)
// hook
$: note("~ 4 ~ <7 4>").scale("C4:minor").s("plk:s5").gain(0.2).cut(1)
// lead
$: note("~ 11 7 <12 9 7 4>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("ld:ss").gain(0.12).cut(1)
// arp
$: note("~ 0 7 12  7 0 ~ 4").scale("<C5:minor C5:minor G5:phrygian C5:minor>")
  .s("plk:dt").gain(0.12).cut(1)
// chords
$: note("[0,4] ~ ~ [0,4]").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("ep:mt").gain(0.14)
// pad
$: note("0").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("pf:ff").gain(0.12).room(0.2).orbit(2)
'''

FENCES["future-bass"] = r'''// @title future-bass-01
// @genre future-bass
setcpm(140/4)
// kick
$: s("bd ~ bd ~").gain(0.85).duckorbit(2).duckattack(0.05).duckdepth(0.8)
// hats
$: s("~ ~ sd ~, hh*8, <~ ~ ~ [hh*16]>").gain(0.42)
// bass
$: note("0 ~ 0 <0 4 0 2>").scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor>")
  .s("bs:su").gain(0.4).orbit(2)
// lead
$: note("4@2 7 9@2  7 4 <2 0> ~").scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor>")
  .s("ld:ss").gain(0.16).cut(1)
// hook
$: note("~ 11 ~ 12  ~ 9 ~ <11 12>").scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor>")
  .s("plk:mx").gain(0.18).cut(1)
// arp
$: note("0 4 ~ 7  4 ~ 9 4").scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor>")
  .s("plk:fg").gain(0.14).cut(1)
// chords
$: note("[0,4,9] ~ [0,4,9] ~").scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor>")
  .s("plk:ss").gain(0.2).orbit(2)
// pad
$: note("0").scale("<F4:lydian G4:mixolydian E4:phrygian A4:minor>")
  .s("pf:ff").gain(0.14).room(0.3).orbit(2)
'''

FENCES["chill-pop"] = r'''// @title chill-pop-01
// @genre chill-pop
setcpm(100/4)
// drums
$: s("bd ~ bd ~, [~ sd]*2, hh*8, <~ ~ ~ [bd sd bd sd]>").gain(0.5)
// bass
$: note("0 ~ 4 0  0 ~ <4 7 2 0>").scale("<F4:lydian E4:phrygian D4:dorian C4:major>")
  .s("bs:su").gain(0.42)
// lead
$: note("~ 2 4 6  4 2 ~ <0 2 4 6>").scale("<F4:lydian E4:phrygian D4:dorian C4:major>")
  .s("plk:ps").gain(0.16).cut(1)
// hook
$: note("0 ~ 2 6  ~ 4 2 <6 4 2 0>").scale("<F4:lydian E4:phrygian D4:dorian C4:major>")
  .s("ep:rs").gain(0.22)
// arp
$: note("0 2 4 6  4 2 0 ~").scale("<F5:lydian E5:phrygian D5:dorian C5:major>")
  .s("plk:ny").gain(0.12).cut(1)
// chords
$: note("[0,2,6] ~ [0,2,6] ~").scale("<F4:lydian E4:phrygian D4:dorian C4:major>")
  .s("ep:mt").gain(0.24).room(0.25).orbit(2)
// pad
$: note("0").scale("<F4:lydian E4:phrygian D4:dorian C4:major>")
  .s("pf:ff").gain(0.14).room(0.3).orbit(2)
'''

FENCES["chill"] = r'''// @title chill-01
// @genre chill
setcpm(90/4)
// drums
$: s("bd:hf ~ ~ bd:hf ~ ~ sd ~, [~ hh]*4, <~ ~ ~ oh>").gain(0.42)
// bass
$: note("0 ~ 2 ~ 0 <3 4 2 0>").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("bs:hf").gain(0.4)
// lead
$: note("~ 4 ~ 7 ~ <6 9 7 4>").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("plk:ps").gain(0.16).cut(1)
// hook
$: note("0@2 4 7@2 ~").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("plk:am").gain(0.18).cut(1)
// arp
$: note("~ 7 12 7  4 0 ~ 2").scale("<C5:minor C5:minor F5:dorian C5:minor>")
  .s("plk:hp").gain(0.12).cut(1)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("ep:rs").gain(0.22).room(0.3).orbit(2)
// pad
$: note("0").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("pf:ff").gain(0.16).room(0.35).orbit(2)
'''

FENCES["ambient"] = r'''// @title ambient-01
// @genre ambient
setcpm(70/4)
// drums
$: s("bd:lf ~ ~ ~").gain(0.18)
// bass
$: note("0 ~ ~ <0 0 ~ 0>").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("bs:su").gain(0.28)
// lead
$: note("~ 7 ~ <9 7 4 11>").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("plk:bl").gain(0.1).cut(1)
// hook
$: note("0@2 ~ 4@2 ~").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("plk:am").gain(0.16).cut(1)
// arp
$: s("<~ perc:cm ~ perc:tg>").gain(0.1)
// chords
$: note("[0,2,4] ~ ~ ~").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("ep:mt").gain(0.18)
// pad
$: note("0").scale("<C4:minor C4:minor G4:dorian C4:minor>")
  .s("pf:ff").gain(0.24).room(0.5).orbit(1)
'''

FENCES["dubstep"] = r'''// @title dubstep-01
// @genre dubstep
setcpm(140/4)
// drums
$: s("bd ~ ~ ~ bd ~ sd ~, hh*8, <~ ~ ~ [bd sd bd sd]>").gain(0.72)
// bass
$: note("0 0 3 <0 0 3 -1>").scale("<C2:minor C2:minor G2:phrygian C2:minor>")
  .s("sawtooth").lpf(sine.rangex(80, 600)).lpq(6).gain(0.5)
// lead
$: note("~ 7 ~ <10 7 3 7>").scale("<C3:minor C3:minor G3:phrygian C3:minor>")
  .s("sawtooth").fm(4).fmh(1).fmdec(0.25).fmsus(0.2)
  .lpf(800).gain(0.16)
// hook
$: note("~ 4 ~ <7 4 4 7>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("plk:s5").gain(0.18).cut(1)
// arp
$: note("~ ~ 12 ~").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("plk:dt").gain(0.1).cut(1)
// chords
$: note("[0,4] ~ ~ [0,4]").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("ep:mt").gain(0.16)
// pad
$: note("0").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("pf:ff").gain(0.12).room(0.25).orbit(2)
'''

FENCES["electro"] = r'''// @title electro-01
// @genre electro
setcpm(126/4)
// drums
$: s("bd*4, ~ sd ~ sd, hh*8, <~ ~ ~ [bd sd bd sd]>").gain(0.62)
// bass
$: note("0 ~ 0 <3 0 0 5>").scale("<C2:minor C2:minor G2:phrygian C2:minor>")
  .s("square").lpf(500).gain(0.46)
  .attack(0.001).decay(0.08).sustain(0.15).release(0.04)
// lead
$: note("~ 7 4 <9 7 12 7>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("ld:ss").gain(0.14).cut(1)
// hook
$: note("12 ~ 7 <12 15 12 7>").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("square").penv(12).pattack(0.001).pdecay(0.08).lpf(2400).gain(0.16)
// arp
$: note("~ 0 3 7  3 0 ~ 5").scale("<C5:minor C5:minor G5:phrygian C5:minor>")
  .s("plk:cv").gain(0.14).cut(1)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("plk:sp").gain(0.18)
// pad
$: note("0").scale("<C4:minor C4:minor G4:phrygian C4:minor>")
  .s("pf:ff").gain(0.14).room(0.25).orbit(2)
'''

FENCES["lofi-hiphop"] = r'''// @title lofi-hiphop-01
// @genre lofi-hiphop
setcpm(84/4)
// drums
$: s("bd:lf ~ ~ sd, [~ hh]*4, <~ ~ ~ [bd:lf sd bd:lf sd]>").gain(0.44)
// bass
$: note("0 ~ 2 <0 0 3 2>").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("bs:su").gain(0.38)
// lead
$: note("~ 4 7 <5 4 0 2>").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("plk:lf").gain(0.18).cut(1)
// hook
$: note("[0,2,4] ~ [0,3,5] ~").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("ep:rs").gain(0.22)
  .attack(0.05).decay(0.3).sustain(0.5).release(0.3)
// arp
$: note("~ 0 4 7  ~ 4 0 2").scale("<C5:minor C5:minor F5:dorian C5:minor>")
  .s("plk:ny").gain(0.12).cut(1)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("ep:mt").gain(0.2).room(0.3).orbit(1)
// pad
$: note("0").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("pf:ff").gain(0.14).room(0.35).orbit(1)
'''

FENCES["minimal-techno"] = r'''// @title minimal-techno-01
// @genre minimal-techno
setcpm(126/4)
// drums
$: s("bd*4, hh*8, <~ ~ ~ cp>").gain(0.7)
// bass
$: note("0 ~ 3 ~").scale("<C2:minor C2:minor C2:minor G2:phrygian>")
  .s("sawtooth").lpf(320).gain(0.4)
// lead
$: note("~ ~ 7 ~").scale("<C4:minor C4:minor C4:minor G4:phrygian>")
  .s("plk:pk").gain(0.1).cut(1)
// hook
$: note("~ 4 ~ ~").scale("<C4:minor C4:minor C4:minor G4:phrygian>")
  .s("plk:ac").gain(0.12).cut(1)
// arp
$: s("<~ perc:tk ~ perc:st>").gain(0.12)
// chords
$: note("~ [0,2,4] ~ ~").scale("<C4:minor C4:minor C4:minor G4:phrygian>")
  .s("ep:mt").gain(0.14)
// pad
$: note("0").scale("<C4:minor C4:minor C4:minor G4:phrygian>")
  .s("pf:ff").orbit(2).gain(0.1).room(0.25)
'''

FENCES["progressive-house"] = r'''// @title progressive-house-01
// @genre progressive-house
setcpm(128/4)
// drums
$: s("bd*4, [~ cp]*2, hh*8, <~ ~ ~ oh>").gain(0.58)
// bass
$: note("0 0 4 <0 2 4 0>").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("bs:hf").gain(0.42)
// lead
$: note("4@2 7 9@3 ~").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("ld:ss").gain(0.16).cut(1)
// hook
$: note("~ 7 4 <9 7 4 2>").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("plk:hb").gain(0.18).cut(1)
// arp
$: note("0 2 4 7  4 2 0 ~").scale("<C5:minor C5:minor F5:dorian C5:minor>")
  .s("plk:hd").gain(0.12).cut(1)
// chords
$: note("[0,2,4] ~ [0,2,4] ~").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("ep:ky").gain(0.24)
// pad
$: note("0").scale("<C4:minor C4:minor F4:dorian C4:minor>")
  .s("pf:ff").gain(0.18).room(0.4).orbit(2)
'''


def pc_name(pc: int) -> str:
    return PC_NAMES[pc % 12]


def note_pc(name: str) -> int:
    letter = name[0]
    base = {"C": 0, "D": 2, "E": 4, "F": 5, "G": 7, "A": 9, "B": 11}[letter]
    i = 1
    while i < len(name):
        if name[i] == "#":
            base += 1
            i += 1
        elif name[i] == "b":
            base -= 1
            i += 1
        else:
            break
    return base % 12


def transpose_token(name: str, oct_: int, semis: int) -> tuple[str, int]:
    midi = (oct_ + 1) * 12 + note_pc(name) + semis
    new_oct = midi // 12 - 1
    return pc_name(midi), new_oct


def transpose_text(text: str, semis: int) -> str:
    if semis == 0:
        return text

    def repl(m: re.Match[str]) -> str:
        name, oct_s, mode = m.group(1), int(m.group(2)), m.group(3)
        new_name, new_oct = transpose_token(name, oct_s, semis)
        return f"{new_name}{new_oct}:{mode}"

    return SCALE_TOKEN.sub(repl, text)


def scale4(oct_: int, pairs: list[tuple[int, str]]) -> str:
    """pairs: (semitone offset from C, mode) at the given octave."""
    parts = [f"{pc_name(off)}{oct_}:{mode}" for off, mode in pairs]
    return "<" + " ".join(parts) + ">"


ODO = [(5, "lydian"), (7, "mixolydian"), (4, "phrygian"), (9, "minor")]  # IV V iii vi
KOMURO = [(9, "minor"), (5, "lydian"), (7, "mixolydian"), (0, "major")]  # vi IV V I
CANON = [(0, "major"), (7, "mixolydian"), (9, "minor"), (5, "lydian")]  # I V vi IV
CITY = [(5, "lydian"), (4, "phrygian"), (2, "dorian"), (0, "major")]  # IV iii ii I
WEST = [(0, "major"), (9, "mixolydian"), (2, "mixolydian"), (7, "mixolydian")]
TWO5 = [(2, "dorian"), (7, "mixolydian"), (0, "major"), (9, "minor")]


def replace_scale_inners(text: str, pairs: list[tuple[int, str]]) -> str:
    def repl(m: re.Match[str]) -> str:
        inner = m.group(1)
        toks = inner.split()
        if len(toks) != 4:
            return m.group(0)
        octs = []
        for tok in toks:
            mm = SCALE_TOKEN.fullmatch(tok)
            if not mm:
                return m.group(0)
            octs.append(int(mm.group(2)))
        # keep each token's octave, rebuild degrees/modes from pairs
        rebuilt = []
        for oct_, (off, mode) in zip(octs, pairs):
            rebuilt.append(f"{pc_name(off)}{oct_}:{mode}")
        return '.scale("<' + " ".join(rebuilt) + '>")'

    return re.sub(r'\.scale\("<([^"]+)>"\)', repl, text)


def apply_variant(genre: str, text: str, variant: int) -> str:
    if variant == 0:
        return text
    if genre == "future-bass":
        return replace_scale_inners(text, KOMURO if variant == 1 else CANON)
    if genre == "chill-pop":
        return replace_scale_inners(text, WEST if variant == 1 else TWO5)
    if genre == "house" and variant == 1:
        return text.replace("[~ hh:hs]*4", "hh:hs*8")
    if genre == "house" and variant == 2:
        return text.replace(
            's("bd:hf*4, [~ cp]*2, [~ hh:hs]*4, <~ ~ ~ [~@3 bd:hf ~@4]>").gain(0.58)',
            's("bd:hf*4, [~ cp]*2, [~ hh:hs]*4, <~ ~ ~ [~@3 bd:hf ~@4]>").gain(0.58).lpf(4000)',
        )
    if genre == "four-on-the-floor" and variant >= 1:
        return text.replace(
            's("bd*4, [~ hh]*4, <~ ~ ~ [~@3 bd ~@4]>")',
            's("bd*4, [~ hh]*4, <~ ~ ~ [bd sd bd sd]>")',
        )
    if genre == "acid" and variant == 1:
        return text.replace('.s("sawtooth")', '.s("square")', 1)
    if genre == "acid" and variant == 2:
        return text.replace(
            '.lpf("260 260 720 260  760 260 260 260  260 680 260 260  740 720 260 260")',
            '.lpf("[260 260 720 260]*4")',
        )
    if genre == "dnb" and variant == 1:
        return text.replace("hh*4", "hh*8")
    if genre == "dnb" and variant == 2:
        return text.replace(".lpf(4000)", ".lpf(2500)")
    if genre == "dnb-reese" and variant == 1:
        return text.replace("hh*4", "hh*8")
    if genre == "dnb-reese" and variant == 2:
        return text.replace(".lpf(4000)", ".lpf(2500)")
    if genre == "techno-duck" and variant == 1:
        return text.replace("duckdepth(0.85)", "duckdepth(0.95)")
    if genre == "techno-duck" and variant == 2:
        return text.replace("[~ hh]*4", "hh*8")
    if genre == "chill" and variant == 1:
        return text.replace("[~ hh]*4", "[~ hh]*4, perc:sk")
    if genre == "ambient" and variant >= 1:
        return text.replace('s("bd:lf ~ ~ ~")', 's("~ ~ ~ ~")')
    if genre == "lofi-hiphop" and variant == 1:
        return text.replace("[~ hh]*4", "[~ hh]*4, perc:cb")
    if genre == "electro" and variant == 1:
        return text.replace("hh*8", "hh*16")
    if genre == "minimal-techno" and variant == 1:
        return text.replace("hh*8", "hh*4")
    if genre == "progressive-house" and variant == 1:
        return text.replace("hh*8", "hh*16")
    if genre == "dubstep" and variant == 1:
        return text.replace("hh*8", "hh*16")
    if variant == 2 and genre in {
        "house",
        "chill",
        "ambient",
        "lofi-hiphop",
        "electro",
        "minimal-techno",
        "progressive-house",
        "dubstep",
        "four-on-the-floor",
        "techno-duck",
    }:
        # Darker kit / lead lpf drop on a pitched track — skip drums-only files.
        return text.replace(".lpf(2800)", ".lpf(1800)").replace(".lpf(3200)", ".lpf(2200)")
    return text


def set_title(text: str, title: str) -> str:
    if TITLE_RE.search(text):
        return TITLE_RE.sub(rf"\g<1>{title}", text, count=1)
    return f"// @title {title}\n" + text


def validate(path: Path, text: str) -> None:
    n = len(re.findall(r"^\$:", text, re.M))
    if n < 7 or n > 8:
        raise SystemExit(f"{path}: expected 7–8 $: tracks, got {n}")
    if "setcpm(" not in text:
        raise SystemExit(f"{path}: missing setcpm")
    for bad in ("stack(", ".cpm(", ".lfo(", ".fast(", "kit:bd"):
        if bad in text:
            raise SystemExit(f"{path}: forbidden {bad}")
    if re.search(r"\bdb\b", text):
        raise SystemExit(f"{path}: sample db is silent")


def slot_spec(n: int) -> tuple[int, int]:
    idx = n - 1
    return KEYS[idx % 10], idx // 10


def render(genre: str, n: int) -> str:
    text = FENCES[genre].strip() + "\n"
    semis, variant = slot_spec(n)
    if n == 1:
        variant = 0
        semis = 0
    text = apply_variant(genre, text, variant)
    text = transpose_text(text, semis)
    if n == 1 and genre in TITLE_01:
        title = TITLE_01[genre]
    else:
        title = f"{genre}-{n:02d}"
    text = set_title(text, title)
    # four-on-the-floor 01 must not mention sd (e2e).
    if genre == "four-on-the-floor" and n == 1 and "sd" in text:
        raise SystemExit("four-on-the-floor 01 still contains sd")
    return text


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--genre", action="append", default=[])
    args = ap.parse_args()
    genres = args.genre or sorted(FENCES)
    written = 0
    for genre in genres:
        if genre in HOLD:
            continue
        if genre not in FENCES:
            raise SystemExit(f"unknown genre {genre}")
        dest = SONGS / genre
        dest.mkdir(parents=True, exist_ok=True)
        for n in range(1, 31):
            text = render(genre, n)
            path = dest / f"{n:02d}.strudel"
            validate(path, text)
            path.write_text(text, encoding="utf-8", newline="\n")
            written += 1
    print(f"wrote {written} songs")


if __name__ == "__main__":
    main()
