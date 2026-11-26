#!/usr/bin/python3
"""Builds the input songs of the Edit menu's Initialize commands (task-306).

`init_loop.rbs` and `init_zero.rbs` are the same song, in song mode with the loop on, whose
device chunks (the pattern-mode values) differ from its track events: `init_loop.rbs` has the
loop at bars 3..5 (ticks 1536..3840), `init_zero.rbs` at bars 1..2 (ticks 0..1536), so the
tick-0 rule of the original's delete (`sub_10014910`) shows. Built from the Silent Default Song
template with `tools/rbs.py build`. The original's results are the files
`<song>.after_<command>.rbs` next to them, saved on the test machine (README.md).
"""
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(HERE)))
RBS = os.path.join(ROOT, "tools", "rbs.py")
TEMPLATE = os.path.join(ROOT, "tests", "songs", "shipped", "Silent Default Song.rbs")


def step303(note, rest=0, accent=0):
    return {"note": note, "slide": 0, "accent": accent, "up": 0, "down": 0, "rest": rest}


def pattern303(p):
    steps = [step303((p * 3 + i * 5) % 13, rest=int(i % 4 == 3), accent=int(i % 8 == 0))
             for i in range(16)]
    return {"shuffle": 0, "length": 16, "steps": steps}


def drums(p):
    steps = []
    for i in range(16):
        s = [0] * 12
        s[0] = 1 if i % 4 == 0 else 0
        s[1 + p % 3] = 1 if (i + p) % 6 == 2 else 0
        steps.append(s)
    return {"shuffle": 0, "length": 16, "steps": steps}


def ev(pos, ctrl, value):
    return {"pos": pos, "ctrl": ctrl, "value": value}


def spec(loop_start, loop_end):
    base = json.loads(subprocess.check_output([sys.executable, RBS, "json", TEMPLATE]))
    tracks = base["tracks"]
    # automation over the template's tick-0 events: inside, at the edges of and outside both
    # loops; positions in 32nds (32 per bar)
    tracks["tb303_1"] += [
        ev(32, "pattern", 1), ev(64, "pattern", 2), ev(96, "pattern", 3), ev(128, "pattern", 4),
        ev(160, "pattern", 5), ev(192, "pattern", 6),
        ev(40, "cutoff", 20), ev(64, "cutoff", 30), ev(100, "cutoff", 40), ev(159, "cutoff", 50),
        ev(160, "cutoff", 60), ev(170, "cutoff", 70), ev(130, "resonance", 90)]
    tracks["tb303_2"] += [ev(48, "enabled", 0), ev(200, "enabled", 1)]
    tracks["tr808"] += [
        ev(64, "pattern", 2), ev(128, "pattern", 3), ev(80, "bass_level", 50),
        ev(165, "bass_level", 90)]
    tracks["tr909"] += [
        ev(32, "pattern", 4), ev(96, "pattern", 5), ev(192, "pattern", 6),
        ev(60, "accent_level", 90)]
    tracks["mixer"] += [
        ev(50, "tb303_1_level", 60), ev(100, "tb303_1_level", 80),
        ev(180, "tb303_1_level", 110), ev(120, "tr909_pan", 20)]
    tracks["delay"] += [ev(70, "feedback", 20), ev(150, "feedback", 100)]
    for name in tracks:
        tracks[name].sort(key=lambda e: e["pos"])
    patterns303 = {str(p): pattern303(p) for p in range(10)}
    return {
        "version": "4.2",
        "glob": {"song_mode": 1, "loop": 1, "tempo": 130000, "loop_start": loop_start,
                 "loop_end": loop_end, "shuffle": 0, "vintage": 0},
        "usri": {"show_on_open": 0},
        "mixer": {"master": 100, "comp_id": 0, "pcf_id": 0, "channels": [
            {"enabled": 1, "level": 90, "pan": 40, "delay_send": 10, "dist": 0},
            {"enabled": 1, "level": 70, "pan": 64, "delay_send": 0, "dist": 0},
            {"enabled": 1, "level": 100, "pan": 80, "delay_send": 0, "dist": 0},
            {"enabled": 1, "level": 95, "pan": 64, "delay_send": 30, "dist": 0}]},
        "delay": {"values": {"enabled": 1, "steps": 3, "mode": 1, "feedback": 45, "pan": 64}},
        "dist": {"values": {"enabled": 1, "amount": 40, "shape": 0}},
        "tb303": {
            "0": {"params": {"enabled": 1, "pattern": 9, "tune": 70, "cutoff": 33,
                             "resonance": 50, "env_mod": 60, "decay": 64, "accent": 80,
                             "waveform": 1},
                  "patterns": patterns303},
            "1": {"params": {"enabled": 0, "pattern": 7, "tune": 64, "cutoff": 64,
                             "resonance": 64, "env_mod": 64, "decay": 64, "accent": 64,
                             "waveform": 0},
                  "patterns": patterns303}},
        "tr808": {"params": {"pattern": 8, "bass_level": 110, "accent_level": 70},
                  "patterns": {str(p): drums(p) for p in range(10)}},
        "tr909": {"params": {"pattern": 11, "bass_level": 120, "accent_level": 50},
                  "patterns": {str(p): drums(p) for p in range(12)}},
        "tracks": tracks,
    }


def main():
    for name, (ls, le) in {"init_loop": (1536, 3840), "init_zero": (0, 1536)}.items():
        path = os.path.join(HERE, name + ".json")
        with open(path, "w") as f:
            json.dump(spec(ls, le), f, indent=1)
        out = os.path.join(HERE, name + ".rbs")
        subprocess.check_call([sys.executable, RBS, "build", path, out])
        os.remove(path)


if __name__ == "__main__":
    main()
