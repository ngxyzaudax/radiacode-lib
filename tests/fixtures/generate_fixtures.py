"""Generate wire-level RadiaCode response fixtures for tests/captured_frames.rs.

Each fixture is one complete response frame exactly as a transport receives it:
a little-endian u32 length prefix, the four echoed request header bytes
(command u16, reserved zero, sequence) and the response body. Payloads follow
the reverse-engineered RadiaCode protocol and are encoded here independently of
the Rust decoders so the tests cross-check both implementations.

Run from the repository root: python3 tests/fixtures/generate_fixtures.py
"""

import math
import struct
from pathlib import Path

FIXTURE_DIR = Path(__file__).resolve().parent
BYTES_PER_LINE = 32

GET_VERSION = 0x000A
GET_SERIAL = 0x000B
RD_VIRT_STRING = 0x0826
RD_VIRT_SFR_BATCH = 0x082A

VS_CONFIGURATION = 2
VS_DATA_BUF = 0x100
VS_SFR_FILE = 0x101
VS_SPECTRUM = 0x200
VS_ENERGY_CALIB = 0x202

CS137_CALIBRATION = (-6.25, 2.4512, 0.00038)
CHANNEL_COUNT = 1024
SPECTRUM_SECONDS = 1800


def u8(value):
    """Encode an unsigned byte."""
    return struct.pack("<B", value)


def u16(value):
    """Encode a little-endian unsigned 16-bit integer."""
    return struct.pack("<H", value)


def u32(value):
    """Encode a little-endian unsigned 32-bit integer."""
    return struct.pack("<I", value)


def i32(value):
    """Encode a little-endian signed 32-bit integer."""
    return struct.pack("<i", value)


def f32(value):
    """Encode a little-endian IEEE-754 single."""
    return struct.pack("<f", value)


def ascii_field(text):
    """Encode a u8 length-prefixed ASCII string."""
    data = text.encode("ascii")
    return u8(len(data)) + data


def response_frame(command, sequence, body):
    """Wrap a body with the echoed header and the length prefix."""
    payload = u16(command) + u8(0) + u8(sequence) + body
    return u32(len(payload)) + payload


def virt_string_body(data):
    """Encode a successful virtual-string read: retcode 1, length, data."""
    return u32(1) + u32(len(data)) + data


def firmware_body():
    """GET_VERSION body: boot 4.1 and target 4.14 with build dates."""
    return u16(1) + u16(4) + ascii_field("Jul 12 2022") + u16(14) + u16(4) + ascii_field("Mar 18 2025")


def serial_body():
    """GET_SERIAL body: byte length followed by three serial words."""
    groups = (0x31415926, 0x53589793, 0x23846264)
    return u32(4 * len(groups)) + b"".join(u32(group) for group in groups)


def configuration_text():
    """Device configuration INI in CP1251, including Cyrillic units."""
    lines = [
        "[DeviceParams]",
        "SpecFormatVersion=1",
        "[[GRP_RealTimeData]]",
        "Id=0",
        "GId=0",
        "[CHN_CountRate]",
        "Id=0",
        "Unit=имп/с",
        "P1=1",
        "[CHN_DoseRate]",
        "Id=1",
        "Unit=мкЗв/ч",
        "P1=10000",
        "P2=0",
        "[[GRP_RareData]]",
        "Id=0",
        "GId=3",
        "[CHN_Temperature]",
        "Id=2",
        "Unit=°C",
        "P1=0.01",
        "P2=-20",
    ]
    return ("\r\n".join(lines) + "\r\n").encode("cp1251")


def sfr_file_text():
    """SFR_FILE register directory as reported by firmware 4.14."""
    entries = [
        ("VSFR_DEVICE_CTRL", 0x0500, 4, 4),
        ("VSFR_DEVICE_LANG", 0x0502, 1, 1),
        ("VSFR_DEVICE_TIME", 0x0504, 4, 4),
        ("VSFR_DISP_BRT", 0x0511, 1, 1),
        ("VSFR_SOUND_ON", 0x0522, 1, 1),
        ("VSFR_VIBRO_ON", 0x0531, 1, 1),
        ("VSFR_LEDS_ON", 0x0545, 1, 1),
        ("VSFR_DS_UNITS", 0x8004, 1, 1),
        ("VSFR_CHN_TO_keV_A0", 0x8010, 4, 5),
        ("VSFR_CR_UNITS", 0x8013, 1, 1),
        ("VSFR_CPS", 0x8020, 4, 4),
        ("VSFR_TEMP_degC", 0x8024, 4, 5),
    ]
    sections = [f"[{name}]\nAddr=0x{address:08X}\nSize={size}\nType={kind}\n" for name, address, size, kind in entries]
    return "".join(sections).encode("ascii")


def batch_body():
    """RD_VIRT_SFR_BATCH body for DISP_BRT, SOUND_ON, TEMP_degC and CPS."""
    values = (0x07000000, 1, struct.unpack("<I", f32(24.5))[0], 42)
    return u32(0b1111) + b"".join(u32(value) for value in values)


def calibration_bytes():
    """Three f32 energy calibration coefficients."""
    return b"".join(f32(value) for value in CS137_CALIBRATION)


def channel_energy(channel):
    """Energy in keV of a channel under the fixture calibration."""
    a0, a1, a2 = CS137_CALIBRATION
    return a0 + a1 * channel + a2 * channel * channel


def lcg_noise(seed):
    """Deterministic pseudo-random sequence in [0, 1)."""
    state = seed
    while True:
        state = (state * 1103515245 + 12345) & 0x7FFFFFFF
        yield state / 0x80000000


def cs137_counts():
    """Synthetic 30-minute Cs-137 spectrum: continuum, Compton edge, 662 keV peak, overflow bin."""
    noise = lcg_noise(137)
    counts = []
    for channel in range(CHANNEL_COUNT):
        energy = channel_energy(channel)
        continuum = 1800.0 * math.exp(-energy / 140.0) + 40.0 * math.exp(-energy / 900.0)
        compton = 120.0 if 150.0 < energy < 477.0 else 0.0
        peak = 5200.0 * math.exp(-0.5 * ((energy - 661.7) / 22.0) ** 2)
        expected = 0.0 if channel < 3 or energy > 2400.0 else continuum + compton + peak
        counts.append(int(expected * (0.9 + 0.2 * next(noise))))
    counts[-1] = 3_250_000
    return counts


def spectrum_header():
    """Spectrum preamble: acquisition seconds and calibration."""
    return u32(SPECTRUM_SECONDS) + calibration_bytes()


def encode_plain(counts):
    """SpecFormatVersion 0: one u32 per channel."""
    return b"".join(u32(count) for count in counts)


def value_length(count, last):
    """Pick the smallest version-1 encoding for one channel."""
    delta = count - last
    if count == 0:
        return 0
    if -128 <= delta <= 127:
        return 2
    if count <= 0xFF:
        return 1
    if -32768 <= delta <= 32767:
        return 3
    if -(1 << 23) <= delta < (1 << 23):
        return 4
    return 5


def encode_value(length, count, last):
    """Encode one channel value for a version-1 run."""
    delta = count - last
    encoders = {
        0: lambda: b"",
        1: lambda: u8(count),
        2: lambda: struct.pack("<b", delta),
        3: lambda: struct.pack("<h", delta),
        4: lambda: struct.pack("<i", delta)[:3],
        5: lambda: i32(delta),
    }
    return encoders[length]()


def encode_delta(counts):
    """SpecFormatVersion 1: runs of (repeat << 4 | value length) followed by values."""
    runs = []
    last = 0
    for count in counts:
        length = value_length(count, last)
        if runs and runs[-1][0] == length and len(runs[-1][1]) < 0xFFF:
            runs[-1][1].append(encode_value(length, count, last))
        else:
            runs.append((length, [encode_value(length, count, last)]))
        last = count
    return b"".join(u16((len(values) << 4) | length) + b"".join(values) for length, values in runs)


def record_header(seq, entity, group, ticks):
    """DataBuf record header: seq, entity, group, i32 ticks (10 ms)."""
    return u8(seq) + u8(entity) + u8(group) + i32(ticks)


def real_time(seq, ticks, cps, dose_rh):
    """RealTimeData record with 4.5 % / 6.0 % errors."""
    return record_header(seq, 0, 0, ticks) + f32(cps) + f32(dose_rh) + u16(45) + u16(60) + u16(0) + u8(0)


def raw_rates(seq, ticks, cps, dose_rh):
    """RawData record."""
    return record_header(seq, 0, 1, ticks) + f32(cps) + f32(dose_rh)


def dose_rate_db(seq, ticks, count, cps, dose_rh):
    """DoseRateDB record."""
    return record_header(seq, 0, 2, ticks) + u32(count) + f32(cps) + f32(dose_rh) + u16(52) + u16(0)


def rare(seq, ticks):
    """RareData record: 1 day, 2.9 mR, 23.75 C, 82.5 % battery."""
    return record_header(seq, 0, 3, ticks) + u32(86400) + f32(0.0029) + u16(4375) + u16(8250) + u16(0)


def user_data(seq, ticks):
    """UserData record with its opaque 16-byte payload."""
    return record_header(seq, 0, 4, ticks) + bytes(range(16))


def accel(seq, ticks):
    """AccelData record."""
    return record_header(seq, 0, 6, ticks) + u16(512) + u16(498) + u16(16100)


def event(seq, ticks, event_id):
    """Event record."""
    return record_header(seq, 0, 7, ticks) + u8(event_id) + u8(0) + u16(0)


def waveform8(seq, ticks):
    """Waveform record with two 8-byte samples."""
    return record_header(seq, 1, 1, ticks) + u16(2) + u32(0) + bytes(16)


def data_buf_records():
    """A realistic background-monitoring DataBuf session (0.12 uSv/h, about 5 cps)."""
    return [
        real_time(40, 1000, 5.2, 0.000012),
        raw_rates(41, 1000, 5.6, 0.000013),
        dose_rate_db(42, 1000, 312, 5.2, 0.000012),
        rare(43, 1050),
        event(44, 1075, 4),
        accel(45, 1090),
        user_data(46, 1095),
        waveform8(47, 1099),
        real_time(48, 1100, 4.9, 0.000011),
        real_time(49, 1200, 5.4, 0.000013),
    ]


def corrupt_data_buf():
    """The session with injected garbage after record 3 and a truncated final record."""
    records = data_buf_records()
    garbage = bytes([0x55, 0x7F, 0x7F, 0x00, 0x00, 0x00, 0x00, 0xAA, 0xBB])
    truncated = real_time(50, 1300, 5.0, 0.000012)[:10]
    return b"".join(records[:3]) + garbage + b"".join(records[3:]) + truncated


def fixtures():
    """Map fixture file names to frame bytes."""
    counts = cs137_counts()
    return {
        "get_version": response_frame(GET_VERSION, 0x81, firmware_body()),
        "get_serial": response_frame(GET_SERIAL, 0x82, serial_body()),
        "configuration": response_frame(RD_VIRT_STRING, 0x83, virt_string_body(configuration_text())),
        "sfr_file": response_frame(RD_VIRT_STRING, 0x84, virt_string_body(sfr_file_text())),
        "vsfr_batch": response_frame(RD_VIRT_SFR_BATCH, 0x85, batch_body()),
        "energy_calibration": response_frame(RD_VIRT_STRING, 0x86, virt_string_body(calibration_bytes())),
        "spectrum_v0": response_frame(
            RD_VIRT_STRING, 0x87, virt_string_body(spectrum_header() + encode_plain(counts))
        ),
        "spectrum_v1": response_frame(
            RD_VIRT_STRING, 0x88, virt_string_body(spectrum_header() + encode_delta(counts))
        ),
        "data_buf_session": response_frame(
            RD_VIRT_STRING, 0x89, virt_string_body(b"".join(data_buf_records()))
        ),
        "data_buf_corrupt": response_frame(RD_VIRT_STRING, 0x8A, virt_string_body(corrupt_data_buf())),
    }


def to_hex_lines(data):
    """Format bytes as lowercase hex, 32 bytes per line."""
    chunks = [data[index : index + BYTES_PER_LINE] for index in range(0, len(data), BYTES_PER_LINE)]
    return "\n".join(chunk.hex() for chunk in chunks) + "\n"


def main():
    """Write every fixture to tests/fixtures/<name>.hex."""
    for name, frame in fixtures().items():
        (FIXTURE_DIR / f"{name}.hex").write_text(to_hex_lines(frame))
        print(f"{name}.hex: {len(frame)} bytes")


if __name__ == "__main__":
    main()
