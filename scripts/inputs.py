"""Writes `inputs.jsonl` for one ported libavutil unit: the vectors from ffmpeg's own
libavutil/tests/<unit>.c and FATE references, regenerated here, plus seeded edge cases.

    python3 inputs.py <unit> <out.jsonl>
"""
import json
import random
import sys

rng = random.Random(7)


def hexs(data):
    return bytes(data).hex()


def rand_bytes(n, text=False):
    if text:
        return bytes(rng.randrange(0x20, 0x7f) for _ in range(n))
    return bytes(rng.randrange(256) for _ in range(n))


LENGTHS = [0, 1, 2, 3, 4, 5, 7, 8, 9, 15, 16, 17, 31, 63, 64, 65, 127, 183, 184, 185, 191, 192,
           193, 255, 256, 1000, 4095, 4096, 5552, 5553, 7001, 65536]


def adler32():
    fate = bytes((((i * i) >> 3) + 123 * i) & 0xff for i in range(7001))
    yield {"fn": "av_adler32_update", "args": [1, hexs(fate)]}
    for n in LENGTHS:
        for start in (1, 0, 0x50E6E508, 0xFFFFFFFF, 0xFFF0FFF0, 0x0000FFFF, 0xFFFF0000):
            yield {"fn": "av_adler32_update", "args": [start, hexs(rand_bytes(n))]}
        yield {"fn": "av_adler32_update", "args": [1, "ff" * n]}
        yield {"fn": "av_adler32_update", "args": [1, "00" * n]}
    for _ in range(200):
        n = rng.randrange(0, 3000)
        yield {"fn": "av_adler32_update",
               "args": [rng.randrange(1 << 32), hexs(rand_bytes(n, rng.random() < 0.5))]}


def base64():
    import base64 as b64
    fate = [("", ""), ("1", "MQ=="), ("22", "MjI="), ("333", "MzMz"), ("4444", "NDQ0NA=="),
            ("55555", "NTU1NTU="), ("666666", "NjY2NjY2"), ("abc:def", "YWJjOmRlZg==")]
    datas = [d.encode() for d, _ in fate]
    datas += [rand_bytes(n) for n in list(range(0, 40)) + [100, 255, 256, 1000, 4096]]
    for data in datas:
        size = (len(data) + 2) // 3 * 4 + 1
        for out in (size - 1, size, size + 3, 0, 1):
            if out >= 0:
                yield {"fn": "av_base64_encode", "args": [out, hexs(data)]}
        yield {"fn": "av_base64_encode", "args": ["ff" * (size + 2), hexs(data)]}
        text = b64.b64encode(data).decode()
        for out in sorted({0, 1, 2, 3, 4, len(data) // 2, len(data) - 1, len(data), len(data) + 1,
                           len(data) + 4, len(text)}):
            if out >= 0:
                yield {"fn": "av_base64_decode", "args": [out, text]}
                yield {"fn": "av_base64_decode", "args": ["ff" * out, text]}
        yield {"fn": "av_base64_decode", "args": [None, text]}
        yield {"fn": "av_base64_decode", "args": [len(data) + 4, text.rstrip("=")]}
        for i in range(0, len(text) + 1, max(1, len(text) // 6)):
            for bad in ("!", " ", "\n", "=", "é", "-", "_", "A=", "=A"):
                broken = text[:i] + bad + text[i:]
                yield {"fn": "av_base64_decode", "args": [len(data) + 4, broken]}
                yield {"fn": "av_base64_decode", "args": [None, broken]}
    for text in ["", "=", "==", "A", "AB", "ABC", "ABCD", "AB==CD", "QUJD\u0000", "////",
                 "++++", "YWJj ZGVm", "YWJjZA", "YWJjZA=", "YWJjZA===", "a" * 1001]:
        for out in (None, 0, 1, 3, 4, 5, 8, 64, 1024):
            yield {"fn": "av_base64_decode", "args": [out, text]}


I64 = [0, 1, -1, 2, 3, 7, -7, 12, -18, 1000, 90000, 48000, 44100, 1001, 30000, 2**31 - 1, 2**31,
       -2**31, 2**32, 2**32 + 1, 2**40 + 3, 2**53 + 1, 2**62, 2**63 - 1, -2**63, -2**63 + 1,
       2**63 - 2, 3 * 2**60]


def rand_i64():
    bits = rng.choice([8, 16, 31, 32, 33, 48, 62, 63])
    v = rng.randrange(1 << bits)
    return -v if rng.random() < 0.3 else v


def mathematics():
    for a in I64:
        for b in I64:
            yield {"fn": "av_gcd", "args": [a, b]}
    for _ in range(2000):
        yield {"fn": "av_gcd", "args": [rand_i64(), rand_i64()]}
    rnds = [0, 1, 2, 3, 5, 8192, 8193, 8194, 8195, 8197, 4, 6, -1, 8196, 16384]
    abc = [0, 1, -1, 3, -3, 7, 2**31 - 1, 2**31, 2**33 + 5, 2**62, 2**63 - 1, -2**63, -2**63 + 1]
    for a in abc:
        for b in [0, 1, 2, 3, 1001, 2**31 - 1, 2**31, 2**40, 2**63 - 1, -1]:
            for c in [1, 2, 3, 1000, 90000, 2**31 - 1, 2**31, 2**45 + 7, 2**63 - 1, 0, -5]:
                for rnd in rnds:
                    yield {"fn": "av_rescale_rnd", "args": [a, b, c, rnd]}
                yield {"fn": "av_rescale", "args": [a, b, c]}
    for _ in range(3000):
        a, b, c = rand_i64(), abs(rand_i64()), abs(rand_i64()) or 1
        yield {"fn": "av_rescale_rnd", "args": [a, b, c, rng.choice(rnds)]}
        yield {"fn": "av_rescale", "args": [a, b, c]}
    for _ in range(1000):
        mod = 1 << rng.randrange(0, 64)
        yield {"fn": "av_compare_mod",
               "args": [rng.randrange(1 << 64), rng.randrange(1 << 64), mod]}
    for a in [0, 1, 2**63, 2**64 - 1, 5]:
        for b in [0, 1, 2**63, 2**64 - 1, 3]:
            for mod in [0, 1, 2, 2**32, 2**63, 6]:
                yield {"fn": "av_compare_mod", "args": [a, b, mod]}
    xs = [0.0, -0.0, 1e-300, 0.5, 1.0, -1.0, 2.5, 14.999999, 15.0, 15.000001, 20.0, 100.0,
          700.0, 709.0, 710.0, 713.0, 800.0, 1e308, {"$num": "Infinity"},
          {"$num": "-Infinity"}, {"$num": "NaN"}, {"$num": "-0"}]
    xs += [rng.uniform(-30, 30) for _ in range(500)] + [rng.uniform(0, 720) for _ in range(300)]
    for x in xs:
        yield {"fn": "av_bessel_i0", "args": [x]}


TOKENS = ["''", "", ":", "\\", "'", "    ''    :", "    ''  ''  :", "foo   '' :", "'foo'",
          "foo     ", "  '  foo  '  ", "foo\\", "foo':  blah:blah", "foo\\:  blah:blah", "foo'",
          "'foo :  '  :blahblah", "\\ :blah", "     foo", "      foo       ",
          "      foo     \\ ", "foo ':blah", " foo   bar    :   blahblah", "\\f\\o\\o",
          "'foo : \\ \\  '   : blahblah", "'\\fo\\o:': blahblah",
          "\\'fo\\o\\:':  foo  '  :blahblah"]
HAYSTACK = "Education consists mainly in what we have unlearned."
NEEDLES = ["learned.", "unlearned.", "Unlearned"]
WORDS = ["", "a", "A", "ab", "AB", "abc", "aBc", "mp4", "MP4", "mov", "m4a", "ALL", "all", "-mp4",
         "mov,mp4,m4a,3gp,3g2,mj2", "matroska,webm", "-ALL", "h264", "H264", "hevc,h265", ",",
         ",,", "a,", ",a", "é", "É", "straße", "ß", "İ", "ı", "\u00ff", "x\u00e9y", "😀",
         "Hello World", "WORLD", "hello", "a,b", "a:b", "a|b"]


def avstring():
    for s in TOKENS + WORDS + [HAYSTACK]:
        for t in NEEDLES + WORDS + TOKENS[:6]:
            yield {"fn": "av_stristr", "args": [s, t]}
            yield {"fn": "av_strcasecmp", "args": [s, t]}
            for n in (0, 1, 2, 3, 100, 2**64 - 1):
                yield {"fn": "av_strncasecmp", "args": [s, t, n]}
            yield {"fn": "av_match_name", "args": [s, t]}
            for sep in (44, 58, 124, 0, -1):
                yield {"fn": "av_match_list", "args": [s, t, sep]}
            for n in sorted({0, len(s.encode()) // 2, len(s.encode())}):
                yield {"fn": "av_strnstr", "args": [s, t, n]}
    for t in NEEDLES + [""]:
        yield {"fn": "av_strnstr", "args": [HAYSTACK, t, len(HAYSTACK)]}
    for src in TOKENS + WORDS + [HAYSTACK, "x" * 300]:
        for size in (0, 1, 2, 3, 4, 8, 16, 64, 512):
            yield {"fn": "av_strlcpy", "args": [size, src]}
            yield {"fn": "av_strlcpy", "args": ["ff" * size, src]}
            for prefix in ("", "ab", "abcdef"):
                if len(prefix) < size:
                    init = prefix.encode().hex() + "00" + "ff" * (size - len(prefix) - 1)
                    yield {"fn": "av_strlcat", "args": [init, src]}
    names = ["mp4", "mov", "matroska", "webm", "x", "ALL", "-", ""]
    lists = ["mov,mp4,m4a", "-mp4,ALL", "ALL", "ALLx", "AL", "-ALL", "matroska,webm", "", ",",
             "mp4,", ",mp4", "-mov,-mp4", "MP4", "mp", "mp4x"]
    for n in names:
        for l in lists:
            yield {"fn": "av_match_name", "args": [n, l]}
            yield {"fn": "av_match_name", "args": [None, l]}
            yield {"fn": "av_match_list", "args": [n, l, 44]}
    alphabet = "aAbB,:-|É "
    for _ in range(3000):
        s = "".join(rng.choice(alphabet) for _ in range(rng.randrange(0, 12)))
        t = "".join(rng.choice(alphabet) for _ in range(rng.randrange(0, 6)))
        fn = rng.choice(["av_stristr", "av_strcasecmp", "av_match_name", "av_match_list",
                         "av_strncasecmp", "av_strnstr"])
        args = [s, t]
        if fn in ("av_strncasecmp", "av_strnstr"):
            args.append(rng.randrange(0, len(s.encode()) + 1))
        if fn == "av_match_list":
            args.append(rng.choice([44, 58, 124]))
        yield {"fn": fn, "args": args}


def rational():
    numlist = [-2**63, -2**31, -2**16, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
               15, 16, 17, 42, 2**16, 2**31 - 1, 2**31, 2**31 + 1, 2**32, 2**62, 2**63 - 1]
    maxes = [0, 1, 2, 10, 255, 1001, 65535, 2**16, 2**31 - 1, 2**31, 2**62, 2**63 - 1, -1]
    for num in numlist:
        for den in numlist:
            for mx in maxes:
                yield {"fn": "av_reduce", "args": [0, 0, num, den, mx]}
    for num, den in [(30000, 1001), (24000, 1001), (60000, 1001), (314159265, 100000000),
                     (2**31 - 1, 2**31 - 2), (1, 3), (2, 3), (48000, 44100), (90000, 1)]:
        for mx in maxes:
            yield {"fn": "av_reduce", "args": [7, -7, num, den, mx]}
            yield {"fn": "av_reduce", "args": [None, 0, num, den, mx]}
            yield {"fn": "av_reduce", "args": [0, None, num, den, mx]}
    for _ in range(5000):
        yield {"fn": "av_reduce",
               "args": [0, 0, rand_i64(), rand_i64(), rng.choice(maxes + [rand_i64()])]}


SIZE_ABBRS = ["ntsc", "pal", "qntsc", "qpal", "sntsc", "spal", "film", "ntsc-film", "sqcif",
              "qcif", "cif", "4cif", "16cif", "qqvga", "qvga", "vga", "svga", "xga", "uxga",
              "qxga", "sxga", "qsxga", "hsxga", "wvga", "wxga", "wsxga", "wuxga", "woxga",
              "wqhd", "wqsxga", "wquxga", "whsxga", "whuxga", "cga", "ega", "hd480", "hd720",
              "hd1080", "quadhd", "2k", "2kdci", "2kflat", "2kscope", "4k", "4kdci", "4kflat",
              "4kscope", "nhd", "hqvga", "wqvga", "fwqvga", "hvga", "qhd", "uhd2160", "uhd4320"]


def parseutils():
    sizes = SIZE_ABBRS + [a.upper() for a in SIZE_ABBRS[:10]] + [a + " " for a in SIZE_ABBRS[:5]]
    sizes += ["640x480", "640X480", "640*480", "640:480", " 640x 480", "+640x+480", "-640x480",
              "640x-480", "0x0", "1x1", "640x480x", "640x480 ", "640", "x480", "640x", "",
              "x", "2147483647x2147483647", "2147483648x10", "4294967297x10",
              "9223372036854775807x1", "9223372036854775808x1", "-9223372036854775808x1",
              "99999999999999999999x1", "1920x1080foobar", "0x10", "10x0", "\t\n12x13",
              "12\u00e9x13", "1e3x2", "0x1A", "007x08", "640\u00d7480", "1x1x1", "  "]
    for size in sizes:
        for init in (0, -5):
            yield {"fn": "av_parse_video_size", "args": [init, init, size]}
    for _ in range(2000):
        w = rng.choice([rng.randrange(-10, 5000), rng.randrange(-2**40, 2**40)])
        h = rng.choice([rng.randrange(-10, 5000), rng.randrange(-2**40, 2**40)])
        sep = rng.choice(["x", "X", ":", " ", "", "xx"])
        tail = rng.choice(["", "", "", " ", "a"])
        yield {"fn": "av_parse_video_size", "args": [0, 0, f"{w}{sep}{h}{tail}"]}
    fate_args = "?tag1=val1&tag2=val2&tag3=val3&tag41=value 41&tag42=random1"
    infos = [fate_args, fate_args[1:], "", "?", "&", "=", "?=", "a", "a=", "=b", "a=b",
             "a=b&", "&a=b", "a=b&&c=d", "a=1&a=2", "a=hello+world", "a=+++", "x" * 200 + "=v",
             "x" * 127 + "=v", "x" * 128 + "=v", "a=" + "v" * 300, "?a=%20&b=c+d&c",
             "tag=é&b=ü", "a&b&c=3"]
    tags = ["tag1", "tag2", "tag3", "tag4", "tag41", "41", "random1", "", "a", "b", "c", "x",
            "x" * 127, "x" * 128, "x" * 200, "tag", "?a"]
    for info in infos:
        for tag in tags:
            for size in (0, 1, 2, 4, 16, 64, 512):
                yield {"fn": "av_find_info_tag", "args": [size, tag, info]}
            yield {"fn": "av_find_info_tag", "args": ["ff" * 8, tag, info]}


HASH_LENGTHS = [0, 1, 3, 15, 16, 17, 31, 55, 56, 57, 63, 64, 65, 119, 120, 127, 128, 129, 1000,
                4096, 10007]
CHUNKS = [0, 1, 3, 7, 16, 63, 64, 65, 1000, -1]


def hash_cases(fn, fate, extra_args, digest):
    for data in fate:
        for chunk in (0, 1, 64):
            yield {"fn": fn, "args": [digest] + extra_args + [hexs(data), chunk]}
    for n in HASH_LENGTHS:
        data = rand_bytes(n)
        for chunk in CHUNKS:
            yield {"fn": fn, "args": [digest] + extra_args + [hexs(data), chunk]}
    for size in (0, 1, digest - 1, digest + 8):
        yield {"fn": fn, "args": [size] + extra_args + [hexs(b"abc"), 0]}
    yield {"fn": fn, "args": ["ff" * (digest + 8)] + extra_args + [hexs(b"abc"), 0]}
    for _ in range(300):
        data = rand_bytes(rng.randrange(0, 3000), rng.random() < 0.5)
        yield {"fn": fn, "args": [digest] + extra_args + [hexs(data), rng.choice(CHUNKS)]}


def md5():
    sq = bytes((i * i) & 0xff for i in range(1000))
    mod = bytes(i % 127 for i in range(1000))
    fate = [sq, sq[:63], sq[:64], sq[:65], mod[:999]]
    yield from hash_cases("oneshot_md5", fate, [], 16)


def sha():
    fate = [b"abc", b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"]
    for bits in (160, 224, 256):
        yield from hash_cases("oneshot_sha", fate, [bits], 32)
        yield {"fn": "oneshot_sha", "args": [32, bits, "61" * 1000000, 0]}
    for bits in (0, 128, 384, 512, -160, 161):
        yield {"fn": "oneshot_sha", "args": [32, bits, hexs(b"abc"), 0]}


def murmur3():
    fate = []
    for i in range(256):
        yield {"fn": "oneshot_murmur3", "args": [16, 256 - i, hexs(bytes(range(i))), 0]}
    for seed in (0, 1, 0x725ACC55DADDCA55, 2**64 - 1):
        yield from hash_cases("oneshot_murmur3", fate, [seed], 16)


def crc():
    fate = bytes((i + i * i) & 0xff for i in range(1999))
    for cid in range(8):
        for start in (0, 1, 0xFFFFFFFF, 0x12345678):
            yield {"fn": "oneshot_crc", "args": [cid, start, hexs(fate)]}
            for n in HASH_LENGTHS:
                yield {"fn": "oneshot_crc", "args": [cid, start, hexs(rand_bytes(n))]}
    for cid in (8, -1, 100):
        yield {"fn": "oneshot_crc", "args": [cid, 0, hexs(b"abc")]}
    for le in (0, 1):
        for bits in (7, 8, 12, 16, 17, 24, 31, 32, 33):
            for poly in (0, 1, 0x07, 0x1D, 0x8005, 0x1021, 0xA001, 0x864CFB, 0x04C11DB7,
                         0xEDB88320, 0x82F63B78, 0xFFFFFFFF, 0x100):
                for size in (257, 1024, 256, 0):
                    yield {"fn": "oneshot_crc_custom",
                           "args": [le, bits, poly, size, 0xFFFFFFFF, hexs(fate[:100])]}
    for _ in range(500):
        yield {"fn": "oneshot_crc_custom",
               "args": [rng.randrange(2), rng.randrange(8, 33), rng.randrange(1 << 32),
                        rng.choice([257, 1024]), rng.randrange(1 << 32),
                        hexs(rand_bytes(rng.randrange(0, 600)))]}


UNITS = {"adler32": adler32, "parseutils": parseutils, "md5": md5, "sha": sha,
         "murmur3": murmur3, "crc": crc, "base64": base64, "mathematics": mathematics, "avstring": avstring,
         "rational": rational}

if __name__ == "__main__":
    unit, out = sys.argv[1], sys.argv[2]
    with open(out, "w") as f:
        for case in UNITS[unit]():
            f.write(json.dumps(case) + "\n")
