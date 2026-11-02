"""Explicitly regenerate glyph references with native HarfBuzz, outside Cargo."""

import argparse
import ctypes as c
import ctypes.util
import hashlib
import json
from pathlib import Path


class Info(c.Structure):
    _fields_ = [(name, c.c_uint32) for name in ("glyph", "mask", "cluster", "var1", "var2")]


class Position(c.Structure):
    _fields_ = [(name, c.c_int32) for name in ("x_advance", "y_advance", "x_offset", "y_offset")]
    _fields_ += [("var", c.c_uint32)]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()
    library = c.CDLL(ctypes.util.find_library("harfbuzz") or "libharfbuzz.so.0")

    def bind(name, result, *parameters):
        function = getattr(library, name)
        function.restype = result
        function.argtypes = parameters
        return function

    ptr, uint, integer = c.c_void_p, c.c_uint32, c.c_int
    blob_create = bind("hb_blob_create", ptr, c.c_char_p, uint, integer, ptr, ptr)
    face_create = bind("hb_face_create", ptr, ptr, uint)
    font_create = bind("hb_font_create", ptr, ptr)
    font_funcs = bind("hb_ot_font_set_funcs", None, ptr)
    font_scale = bind("hb_font_set_scale", None, ptr, integer, integer)
    buffer_create = bind("hb_buffer_create", ptr)
    buffer_add = bind("hb_buffer_add_utf8", None, ptr, c.c_char_p, integer, uint, integer)
    set_direction = bind("hb_buffer_set_direction", None, ptr, integer)
    set_script = bind("hb_buffer_set_script", None, ptr, uint)
    set_flags = bind("hb_buffer_set_flags", None, ptr, uint)
    script = bind("hb_script_from_string", uint, c.c_char_p, integer)
    shape = bind("hb_shape", None, ptr, ptr, ptr, uint)
    infos = bind("hb_buffer_get_glyph_infos", c.POINTER(Info), ptr, c.POINTER(uint))
    positions = bind("hb_buffer_get_glyph_positions", c.POINTER(Position), ptr, c.POINTER(uint))
    destroy_buffer = bind("hb_buffer_destroy", None, ptr)
    version = bind("hb_version_string", c.c_char_p)().decode()
    font_bytes = (Path(__file__).resolve().parents[2] / "assets/NotoEmoji-Regular.ttf").read_bytes()
    digest = hashlib.sha256(font_bytes).hexdigest()
    blob = blob_create(font_bytes, len(font_bytes), 0, None, None)
    face = face_create(blob, 0)
    font = font_create(face)
    font_funcs(font)
    units = bind("hb_face_get_upem", uint, ptr)(face)
    font_scale(font, units, units)
    texts = [str(n) + "\ufe0f\u20e3" for n in range(10)] + [
        "1\u20e3", "#\ufe0f\u20e3", "*\ufe0f\u20e3", "👩‍👩‍👦", "👨‍👩‍👧‍👦",
        "👩🏽‍💻", "👍🏽", "🇺🇸", "🇯🇵", "❤️", "❤️‍🔥", "🏳️‍🌈", "🦀", "1",
    ]
    rows = []
    tsv = [
        f"# HarfBuzz {version}; REMOVE_DEFAULT_IGNORABLES; Noto Emoji SHA-256 {digest}",
        "# codepoints glyph x_advance y_advance x_offset y_offset",
    ]
    try:
        for text in texts:
            buffer = buffer_create()
            try:
                payload = text.encode()
                buffer_add(buffer, payload, len(payload), 0, len(payload))
                set_direction(buffer, 4)  # HB_DIRECTION_LTR
                set_script(buffer, script(b"Zyyy", 4))
                set_flags(buffer, 8)  # HB_BUFFER_FLAG_REMOVE_DEFAULT_IGNORABLES
                shape(font, buffer, None, 0)
                count = uint()
                glyph_info = infos(buffer, c.byref(count))
                glyph_pos = positions(buffer, c.byref(count))
                glyphs = [
                    {"glyph_id": glyph_info[n].glyph, "cluster": glyph_info[n].cluster,
                     "advance": [glyph_pos[n].x_advance, glyph_pos[n].y_advance],
                     "offset": [glyph_pos[n].x_offset, glyph_pos[n].y_offset]}
                    for n in range(count.value)
                ]
                assert len(glyphs) == 1 and glyphs[0]["glyph_id"] > 0, (text, glyphs)
                codepoints = ",".join(f"{ord(character):X}" for character in text)
                rows.append({"text": text, "codepoints": codepoints, "glyphs": glyphs})
                glyph = glyphs[0]
                values = [codepoints, glyph["glyph_id"], *glyph["advance"], *glyph["offset"]]
                tsv.append("\t".join(map(str, values)))
            finally:
                destroy_buffer(buffer)
    finally:
        for name, value in [("hb_font_destroy", font), ("hb_face_destroy", face), ("hb_blob_destroy", blob)]:
            bind(name, None, ptr)(value)
    Path(__file__).with_name("emoji-shaping.tsv").write_text("\n".join(tsv) + "\n")
    if args.report:
        args.report.write_text(json.dumps({"harfbuzz": version, "buffer_flags": "REMOVE_DEFAULT_IGNORABLES",
                                          "font_sha256": digest, "cases": rows}, ensure_ascii=False, indent=2))
    print(json.dumps({"harfbuzz": version, "cases": len(rows), "all_single_glyph": True}))


if __name__ == "__main__":
    main()
