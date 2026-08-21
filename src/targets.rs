//! Supported target triplets.
//!
//! This module holds the canonical list of target triplets that the
//! `cart init` command presents to the user. The list mirrors the
//! supported-targets table in the `std` library `README.md`.

/// A supported target triplet with its CPU name and platform name.
///
/// The tuple holds `(triplet, cpu_name, platform_name)`.
pub const SUPPORTED_TARGETS: &[(&str, &str, &str)] = &[
    ("mos6502-apple-ii", "MOS 6502", "Apple II"),
    ("mos6502-apple-iic", "MOS 6502", "Apple IIc"),
    ("mos6502-apple-iie", "MOS 6502", "Apple IIe"),
    ("mos6502-apple-iie-enhanced", "MOS 6502", "Apple IIe Enhanced"),
    ("mos6502-atari-800-ntsc", "MOS 6502", "Atari 800 NTSC"),
    ("mos6502-atari-800-pal", "MOS 6502", "Atari 800 PAL"),
    ("mos6502-atari-2600", "MOS 6502", "Atari 2600"),
    ("mos6502-atari-5200", "MOS 6502", "Atari 5200"),
    ("mos6502-atari-7800", "MOS 6502", "Atari 7800"),
    ("mos65sc02-atari-lynx", "MOS 65SC02", "Atari Lynx"),
    ("mos6502-commodore-64", "MOS 6502", "Commodore 64"),
    ("mos6502-nec-pcengine", "MOS 6502", "NEC PC Engine"),
    ("rp2A03-nintendo-nes-ntsc", "Ricoh RP2A03", "NES NTSC"),
    ("rp2A07-nintendo-nes-pal", "Ricoh RP2A07", "NES PAL"),
    ("m68000-neogeo-aes", "Motorola 68000", "Neo Geo AES"),
    ("m68000-sega-genesis", "Motorola 68000", "Sega Genesis"),
    ("wdc65c816-apple-iigs", "WDC 65C816", "Apple IIgs"),
    ("wdc65c816-nintendo-snes", "WDC 65C816", "SNES"),
    ("z80-neogeo-aes", "Zilog Z80", "Neo Geo AES"),
    ("z80-nintendo-gameboy", "Sharp LR35902", "Game Boy"),
    ("z80-nintendo-gameboy-color", "Sharp LR35902", "Game Boy Color"),
    ("z80-sega-gamegear", "Zilog Z80", "Sega Game Gear"),
    ("z80-sega-genesis", "Zilog Z80", "Sega Genesis"),
    ("z80-sega-mastersystem", "Zilog Z80", "Sega Master System"),
    ("z80-sega-sg1000", "Zilog Z80", "Sega SG-1000"),
    ("z80-sinclair-zx80", "Zilog Z80", "Sinclair ZX80"),
    ("z80-sinclair-zx81", "Zilog Z80", "Sinclair ZX81"),
    ("z80-sinclair-spectrum", "Zilog Z80", "Sinclair Spectrum"),
    ("z80-ti-85", "Zilog Z80", "Texas Instruments TI-85"),
];