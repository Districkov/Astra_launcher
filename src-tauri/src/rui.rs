//! rui.rs — RAGE:MP `index.rui` archive editor
//!
//! Faithful Rust port of `tiny-ragemp-launcher/tools/build.py`
//! (https://github.com/winnerchester/tiny-ragemp-launcher, MIT).
//!
//! ## Archive format
//!
//! `rui\index.rui` is a flat archive of **265 contiguous files** preceded by an
//! 8488-byte header holding two tables. Each table entry is **32 bytes**:
//!
//! | Table | Count | size position        | offset position      | hash position        |
//! |-------|-------|----------------------|----------------------|----------------------|
//! | 1     | 132   | `(2k+1) * 16`        | `(2k+2) * 16`        | `size_pos + 8`       |
//! | 2     | 133   | `4240 + (2k) * 16`   | `4240 + (2k+1) * 16` | `size_pos + 8`       |
//!
//! The 8 bytes at `size_pos + 8` hold **`XXH64(file content, seed = 0)`** —
//! the client verifies every entry against it before it will render the
//! launcher UI, so [`Archive::serialize`] recomputes them alongside the
//! sizes and offsets. (The 8 bytes at `size_pos + 24` hold `XXH64(file name)`
//! and are unrelated to content, so they never need updating on an edit.)
//!
//! Entries are unordered in the header, so the reader sorts them by offset.
//! File data starts at 8488 and runs contiguously; any bytes after the last
//! file are preserved verbatim as `trailing`.
//!
//! The header begins with `u32 count = 265` at 0x00 and a u64 magic at 0x08.
//!
//! ## Splice vs. rebuild
//!
//! Two write strategies exist and they are **not** interchangeable:
//!
//! * [`splice_auto_connect`] rewrites a fixed-size region of `app.js` with an
//!   equal-length replacement. The archive never changes size, so every header
//!   offset stays valid and the client's own reader sees a file that is
//!   byte-for-byte the same length as the factory one.
//! * [`Archive::serialize`] is a full rebuild. It is required when a file
//!   actually changes size (images especially, colour text replacements).
//!
//! The distinction is not academic: a *growing* auto-connect injection was
//! shipped, and the client's CEF launcher UI then failed to load at all
//! ("404 error: File not found"). Any size-changing edit recomputes the tables
//! and offsets correctly — our parser round-trips byte-exact — but the shipped
//! build proved it needs its archive layout left exactly as delivered.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::logging::{log_info, log_warn};

/// Size of the archive header (both tables live inside it).
pub const HEADER_END: usize = 8488;
/// Start of the second (size, offset) table.
pub const TABLE2_BASE: usize = 4240;
const TABLE1_COUNT: usize = 132;
const TABLE2_COUNT: usize = 133;
/// Total files in a well-formed archive.
pub const TOTAL_ENTRIES: usize = TABLE1_COUNT + TABLE2_COUNT;

/// Backup suffix written before the first modification.
pub const BACKUP_SUFFIX: &str = ".astra.bak";

const MARKER_START: &[u8] = b"/*astra-installed-start*/";
const MARKER_END: &[u8] = b"/*astra-installed-end*/";
const LEGACY_AUTO_CONNECT_PREFIX: &[u8] =
    b"_.servers.refresh),setTimeout(()=>{rageApi.isALauncher&&this.launchGame({ip:";

// ── Equal-length auto-connect splice (preferred over the growing rewrite) ──
//
// The launcher UI's root component mounts with:
//
//     async mounted(){this.loader("core",!0),await this.getServers(this.cache.favorites,!0),
//       this.tabs.favorites.cache=…,this.refreshInterval=setInterval(this.refresh,_.servers.refresh)},methods:{loader(A,n){…
//
// Everything after the `await this getServers(...)` call and before the `}`
// that closes `mounted()` is what the splice rewrites. The prefix through
// `getServers` is deliberately **kept**: that call drives `mp.getServers()`,
// the native handshake that precedes `_rageInitialize` — removing it left the
// client with `_rageInitialize is not defined` and a dead renderer.
//
// The opening anchor must also stay unique: the bundle contains other
// components that open with a bare `async mounted(){`.

/// Left edge of the region the splice replaces — through the `getServers` call.
const MOUNTED_OPEN: &[u8] =
    b"async mounted(){this.loader(\"core\",!0),await this.getServers(this.cache.favorites,!0),";
/// Right edge: `}` closing `mounted()`, followed by the methods object.
const MOUNTED_CLOSE: &[u8] = b"},methods:{loader(A,n)";
/// Embedded in the spliced payload so the injection stays detectable.
const SPLICE_MARKER: &[u8] = b"/*astra-connect*/";
/// Longest body the splice will rewrite — guards against a mis-anchored span.
const MAX_SPLICE_BODY: usize = 4096;

// ── Anchor strings taken verbatim from upstream build.py ──
const ANCHOR_AUTO_CONNECT: &[u8] = b"_.servers.refresh)}";
const ANCHOR_SERVER_LIST: &[u8] = b"async mounted(){this.loader(\"core\",!0)";
const HEAD_APP_JS: &[u8] = b"(functio";
const HEAD_INDEX_HTML: &[u8] = b"<!DO";

// ─────────────────────────────────────────────────────────────────────────────
// Archive
// ─────────────────────────────────────────────────────────────────────────────

/// One directory entry: where its size/offset live, and the values themselves.
#[derive(Debug, Clone, Copy)]
pub struct Entry {
    /// Byte position of the u32 size field inside the header.
    pub size_pos: usize,
    /// Byte position of the u64 `XXH64(content)` field inside the header.
    ///
    /// The client verifies this on every read, so it **must** be recomputed
    /// alongside `size_pos` whenever a file's bytes change. Leaving it stale
    /// makes the whole archive unreadable to CEF.
    pub hash_pos: usize,
    /// Byte position of the u32 offset field inside the header.
    pub off_pos: usize,
    pub size: u32,
    pub off: u32,
}

/// A parsed `index.rui`.
#[derive(Debug, Clone)]
pub struct Archive {
    /// Verbatim first `HEADER_END` bytes (we patch the tables in place).
    pub header: Vec<u8>,
    /// Entries sorted by file offset.
    pub entries: Vec<Entry>,
    /// File contents, parallel to `entries`.
    pub blobs: Vec<Vec<u8>>,
    /// Any bytes following the final file.
    pub trailing: Vec<u8>,
}

fn u32_at(data: &[u8], pos: usize) -> Result<u32, String> {
    let end = pos + 4;
    if end > data.len() {
        return Err(format!("archive truncated at 0x{:X}", pos));
    }
    Ok(u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]))
}

// ── XXH64 ───────────────────────────────────────────────────────────────────
//
// The archive stores `XXH64(file content, seed = 0)` for every entry at
// `size_pos + 8`, and the client verifies it when it reads the archive. Any
// edit to a file that does not update this field makes the CEF launcher UI
// fail to load at all ("404 error: File not found") — which is exactly what
// the first auto-connect release did.

const XXH_PRIME64_1: u64 = 0x9E3779B185EBCA87;
const XXH_PRIME64_2: u64 = 0xC2B2AE3D27D4EB4F;
const XXH_PRIME64_3: u64 = 0x165667B19E3779F9;
const XXH_PRIME64_4: u64 = 0x85EBCA77C2B2AE63;
const XXH_PRIME64_5: u64 = 0x27D4EB2F165667C5;

#[inline]
fn xxh_rotl(x: u64, r: u32) -> u64 {
    x.rotate_left(r)
}

#[inline]
fn xxh_round(acc: u64, input: u64) -> u64 {
    acc.wrapping_add(input.wrapping_mul(XXH_PRIME64_2))
        .rotate_left(31)
        .wrapping_mul(XXH_PRIME64_1)
}

#[inline]
fn xxh_merge_round(acc: u64, val: u64) -> u64 {
    let val = xxh_round(0, val);
    (acc ^ val).wrapping_mul(XXH_PRIME64_1).wrapping_add(XXH_PRIME64_4)
}

#[inline]
fn xxh_read_u64(data: &[u8], at: usize) -> u64 {
    u64::from_le_bytes([
        data[at], data[at + 1], data[at + 2], data[at + 3],
        data[at + 4], data[at + 5], data[at + 6], data[at + 7],
    ])
}

#[inline]
fn xxh_read_u32(data: &[u8], at: usize) -> u64 {
    u32::from_le_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]]) as u64
}

/// XXH64 with `seed = 0` — the variant `index.rui` stores.
pub fn xxh64(data: &[u8]) -> u64 {
    let len = data.len();
    let mut idx;
    let mut h;

    if len >= 32 {
        let mut v1 = XXH_PRIME64_1.wrapping_add(XXH_PRIME64_2);
        let mut v2 = XXH_PRIME64_2;
        let mut v3 = 0u64;
        let mut v4 = 0u64.wrapping_sub(XXH_PRIME64_1);

        idx = 0;
        while idx + 32 <= len {
            v1 = xxh_round(v1, xxh_read_u64(data, idx));
            v2 = xxh_round(v2, xxh_read_u64(data, idx + 8));
            v3 = xxh_round(v3, xxh_read_u64(data, idx + 16));
            v4 = xxh_round(v4, xxh_read_u64(data, idx + 24));
            idx += 32;
        }

        h = xxh_rotl(v1, 1)
            .wrapping_add(xxh_rotl(v2, 7))
            .wrapping_add(xxh_rotl(v3, 12))
            .wrapping_add(xxh_rotl(v4, 18));
        h = xxh_merge_round(h, v1);
        h = xxh_merge_round(h, v2);
        h = xxh_merge_round(h, v3);
        h = xxh_merge_round(h, v4);
    } else {
        h = XXH_PRIME64_5;
        idx = 0;
    }

    h = h.wrapping_add(len as u64);

    while idx + 8 <= len {
        let k1 = xxh_round(0, xxh_read_u64(data, idx));
        h ^= k1;
        h = xxh_rotl(h, 27)
            .wrapping_mul(XXH_PRIME64_1)
            .wrapping_add(XXH_PRIME64_4);
        idx += 8;
    }
    if idx + 4 <= len {
        h ^= xxh_read_u32(data, idx).wrapping_mul(XXH_PRIME64_1);
        h = xxh_rotl(h, 23)
            .wrapping_mul(XXH_PRIME64_2)
            .wrapping_add(XXH_PRIME64_3);
        idx += 4;
    }
    while idx < len {
        h ^= (data[idx] as u64).wrapping_mul(XXH_PRIME64_5);
        h = xxh_rotl(h, 11).wrapping_mul(XXH_PRIME64_1);
        idx += 1;
    }

    h ^= h >> 33;
    h = h.wrapping_mul(XXH_PRIME64_2);
    h ^= h >> 29;
    h = h.wrapping_mul(XXH_PRIME64_3);
    h ^= h >> 32;
    h
}

impl Archive {
    /// Parse an archive from raw bytes.
    pub fn parse(data: &[u8]) -> Result<Self, String> {
        if data.len() < HEADER_END {
            return Err(format!(
                "not a RAGE:MP rui archive ({} bytes, need at least {})",
                data.len(),
                HEADER_END
            ));
        }

        // Collect every entry with the position of its header fields.
        let mut entries: Vec<Entry> = Vec::with_capacity(TOTAL_ENTRIES);

        for k in 0..TABLE1_COUNT {
            let size_pos = (2 * k + 1) * 16;
            let off_pos = (2 * k + 2) * 16;
            entries.push(Entry {
                size_pos,
                hash_pos: size_pos + 8,
                off_pos,
                size: u32_at(data, size_pos)?,
                off: u32_at(data, off_pos)?,
            });
        }
        for k in 0..TABLE2_COUNT {
            let size_pos = TABLE2_BASE + (2 * k) * 16;
            let off_pos = TABLE2_BASE + (2 * k + 1) * 16;
            entries.push(Entry {
                size_pos,
                hash_pos: size_pos + 8,
                off_pos,
                size: u32_at(data, size_pos)?,
                off: u32_at(data, off_pos)?,
            });
        }

        // The header order is arbitrary — the data is laid out by offset.
        entries.sort_by_key(|e| e.off);

        // Slice out the blobs.
        let mut blobs = Vec::with_capacity(entries.len());
        for e in &entries {
            let start = e.off as usize;
            let end = start
                .checked_add(e.size as usize)
                .ok_or_else(|| format!("entry at 0x{:X} overflows", start))?;
            if end > data.len() {
                return Err(format!(
                    "entry at 0x{:X} ({} bytes) runs past end of file ({})",
                    start,
                    e.size,
                    data.len()
                ));
            }
            blobs.push(data[start..end].to_vec());
        }

        let last = entries.last().ok_or("archive has no entries")?;
        let trailing_start = last.off as usize + last.size as usize;

        Ok(Archive {
            header: data[..HEADER_END].to_vec(),
            entries,
            blobs,
            trailing: data[trailing_start..].to_vec(),
        })
    }

    /// True when every file sits immediately after the previous one, and the
    /// first one starts right at the end of the header.
    ///
    /// This inspects the layout as parsed from disk. Note that
    /// [`Archive::serialize`] always *produces* a contiguous layout, so a
    /// non-contiguous source is normalised rather than preserved.
    pub fn is_contiguous(&self) -> bool {
        match self.entries.first() {
            Some(first) if first.off as usize != HEADER_END => return false,
            None => return false,
            _ => {}
        }
        self.entries
            .windows(2)
            .all(|w| (w[0].off as u64 + w[0].size as u64) == w[1].off as u64)
    }

    /// Index of the first file whose content starts with `head`.
    pub fn find_by_head(&self, head: &[u8]) -> Option<usize> {
        self.blobs.iter().position(|b| b.starts_with(head))
    }

    /// The bundled application script (`app.js`).
    pub fn app_js_index(&self) -> Option<usize> {
        self.find_by_head(HEAD_APP_JS)
    }

    /// The bundled `index.html`.
    pub fn index_html_index(&self) -> Option<usize> {
        self.find_by_head(HEAD_INDEX_HTML)
    }

    /// Serialise back to bytes, recomputing every table entry and offset.
    pub fn serialize(&self) -> Result<Vec<u8>, String> {
        // Offsets are cumulative from the end of the header.
        let mut offs: Vec<u32> = Vec::with_capacity(self.blobs.len());
        let mut cursor: u64 = HEADER_END as u64;
        for b in &self.blobs {
            if cursor > u32::MAX as u64 {
                return Err("archive exceeds 4 GiB — cannot be represented".to_string());
            }
            offs.push(cursor as u32);
            cursor += b.len() as u64;
        }

        let mut header = self.header.clone();
        for (i, e) in self.entries.iter().enumerate() {
            let size = self.blobs[i].len() as u32;
            let off = offs[i];

            if e.size_pos + 4 > header.len() || e.off_pos + 4 > header.len() {
                return Err(format!(
                    "table position out of range (size_pos=0x{:X}, off_pos=0x{:X})",
                    e.size_pos, e.off_pos
                ));
            }
            if e.hash_pos + 8 > header.len() {
                return Err(format!(
                    "hash position out of range (hash_pos=0x{:X})",
                    e.hash_pos
                ));
            }
            header[e.size_pos..e.size_pos + 4].copy_from_slice(&size.to_le_bytes());
            header[e.off_pos..e.off_pos + 4].copy_from_slice(&off.to_le_bytes());
            // The client verifies XXH64(content) on read — keep it in sync.
            header[e.hash_pos..e.hash_pos + 8].copy_from_slice(&xxh64(&self.blobs[i]).to_le_bytes());
        }

        let total: usize = HEADER_END + self.blobs.iter().map(|b| b.len()).sum::<usize>() + self.trailing.len();
        let mut out = Vec::with_capacity(total);
        out.extend_from_slice(&header);
        for b in &self.blobs {
            out.extend_from_slice(b);
        }
        out.extend_from_slice(&self.trailing);
        Ok(out)
    }

    /// Sanity-check a freshly serialised archive by re-parsing it.
    pub fn verify_roundtrip(&self) -> Result<(), String> {
        let bytes = self.serialize()?;
        let reparsed = Archive::parse(&bytes)?;
        if reparsed.blobs.len() != self.blobs.len() {
            return Err("round-trip changed the entry count".to_string());
        }
        for (i, (a, b)) in self.blobs.iter().zip(reparsed.blobs.iter()).enumerate() {
            if a != b {
                return Err(format!("round-trip corrupted entry {}", i));
            }
        }
        Ok(())
    }
}

/// Lightweight archive metadata, read from the header alone.
///
/// [`Archive::parse`] copies every file out of the archive — for the real
/// 15 MB `index.rui` that is ~265 allocations and a 15 MB memcpy, which is far
/// too expensive to do just to render a status line in the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveInfo {
    pub entries: usize,
    pub contiguous: bool,
    pub app_js_index: Option<usize>,
    pub index_html_index: Option<usize>,
    pub total_size: usize,
    pub payload_size: usize,
    pub trailing_size: usize,
}

/// Read only the header tables plus the first few bytes of each entry.
///
/// Cost is proportional to the entry count, not the archive size.
pub fn inspect(data: &[u8]) -> Result<ArchiveInfo, String> {
    if data.len() < HEADER_END {
        return Err(format!(
            "not a RAGE:MP rui archive ({} bytes, need at least {})",
            data.len(),
            HEADER_END
        ));
    }

    let mut entries: Vec<(u32, u32)> = Vec::with_capacity(TOTAL_ENTRIES); // (off, size)

    for k in 0..TABLE1_COUNT {
        let size_pos = (2 * k + 1) * 16;
        let off_pos = (2 * k + 2) * 16;
        entries.push((u32_at(data, off_pos)?, u32_at(data, size_pos)?));
    }
    for k in 0..TABLE2_COUNT {
        let size_pos = TABLE2_BASE + (2 * k) * 16;
        let off_pos = TABLE2_BASE + (2 * k + 1) * 16;
        entries.push((u32_at(data, off_pos)?, u32_at(data, size_pos)?));
    }

    entries.sort_by_key(|(off, _)| *off);

    // Peek at the head of each entry to locate app.js / index.html.
    let mut app_js_index = None;
    let mut index_html_index = None;
    for (i, (off, size)) in entries.iter().enumerate() {
        let start = *off as usize;
        if start >= data.len() {
            continue;
        }
        let end = (start + 8).min(data.len()).min(start + *size as usize);
        let head = &data[start..end];

        if app_js_index.is_none() && head.starts_with(HEAD_APP_JS) {
            app_js_index = Some(i);
        }
        if index_html_index.is_none() && head.starts_with(HEAD_INDEX_HTML) {
            index_html_index = Some(i);
        }
        if app_js_index.is_some() && index_html_index.is_some() {
            break;
        }
    }

    // Entries are (offset, size) — a gap-free layout means
    // offset[i] + size[i] == offset[i + 1].
    let contiguous = entries
        .first()
        .map(|(off, _)| *off as usize == HEADER_END)
        .unwrap_or(false)
        && entries
            .windows(2)
            .all(|w| (w[0].0 as u64 + w[0].1 as u64) == w[1].0 as u64);

    let payload_size: usize = entries.iter().map(|(_, s)| *s as usize).sum();
    let last_end = entries
        .last()
        .map(|(off, size)| *off as usize + *size as usize)
        .unwrap_or(HEADER_END);

    Ok(ArchiveInfo {
        entries: entries.len(),
        contiguous,
        app_js_index,
        index_html_index,
        total_size: data.len(),
        payload_size,
        trailing_size: data.len().saturating_sub(last_end),
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Byte helpers
// ─────────────────────────────────────────────────────────────────────────────

fn count_occurrences(haystack: &[u8], needle: &[u8]) -> usize {
    if needle.is_empty() || needle.len() > haystack.len() {
        return 0;
    }
    haystack.windows(needle.len()).filter(|w| *w == needle).count()
}

/// Replace the single occurrence of `needle`, erroring if it is not unique.
/// Uniqueness matters: a blind `replace_all` could corrupt unrelated code.
fn replace_unique(haystack: &[u8], needle: &[u8], replacement: &[u8], what: &str) -> Result<Vec<u8>, String> {
    let n = count_occurrences(haystack, needle);
    if n != 1 {
        return Err(format!("{} anchor found {} time(s), expected exactly 1", what, n));
    }
    let pos = haystack
        .windows(needle.len())
        .position(|w| w == needle)
        .ok_or("anchor vanished")?;

    let mut out = Vec::with_capacity(haystack.len() - needle.len() + replacement.len());
    out.extend_from_slice(&haystack[..pos]);
    out.extend_from_slice(replacement);
    out.extend_from_slice(&haystack[pos + needle.len()..]);
    Ok(out)
}

fn replace_all(haystack: &[u8], needle: &[u8], replacement: &[u8]) -> (Vec<u8>, usize) {
    let n = count_occurrences(haystack, needle);
    if n == 0 || needle.is_empty() {
        return (haystack.to_vec(), 0);
    }
    let mut out = Vec::with_capacity(haystack.len());
    let mut rest = haystack;
    while let Some(pos) = rest.windows(needle.len()).position(|w| w == needle) {
        out.extend_from_slice(&rest[..pos]);
        out.extend_from_slice(replacement);
        rest = &rest[pos + needle.len()..];
    }
    out.extend_from_slice(rest);
    (out, n)
}

/// Escape a value for embedding inside a double-quoted JS string literal.
fn js_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// Configuration (mirrors upstream tools/config.json)
// ─────────────────────────────────────────────────────────────────────────────

/// `auto_connect` block — injects a `launchGame(ip, port)` call on UI load.
///
/// This is the raw-IP whitelist bypass: `ragemp_v.exe` filters *domains* in
/// `launch2.ip`, but this path calls the JS API directly with a literal IP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoConnect {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub ip: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_ac_name")]
    pub name: String,
    #[serde(default = "default_delay")]
    pub delay_ms: u32,
}

fn default_port() -> u16 {
    22005
}
fn default_ac_name() -> String {
    "AutoConnect".to_string()
}
fn default_delay() -> u32 {
    2000
}

impl Default for AutoConnect {
    fn default() -> Self {
        Self {
            enabled: false,
            ip: String::new(),
            port: default_port(),
            name: default_ac_name(),
            delay_ms: default_delay(),
        }
    }
}

/// One entry of the replaced server browser list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEntry {
    pub name: String,
    pub ip: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub players: u32,
    #[serde(default = "default_maxplayers")]
    pub maxplayers: u32,
    #[serde(default = "default_ping")]
    pub ping: u32,
    #[serde(default = "default_lang")]
    pub lang: String,
    #[serde(default = "default_gamemode")]
    pub gamemode: String,
    #[serde(default)]
    pub url: String,
}

fn default_maxplayers() -> u32 {
    1000
}
fn default_ping() -> u32 {
    30
}
fn default_lang() -> String {
    "en".to_string()
}
fn default_gamemode() -> String {
    "Freeroam".to_string()
}

/// `server_list` block — replaces what the browser *shows*.
///
/// Clicking a row still connects through the normal `rageApi.launchGame` path.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerList {
    #[serde(default, rename = "override")]
    pub override_list: bool,
    #[serde(default)]
    pub servers: Vec<ServerEntry>,
}

/// Accepts either `{"a":"b"}` or `[["a","b"]]` so upstream configs load as-is.
fn de_pairs<'de, D>(deserializer: D) -> Result<Vec<(String, String)>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let value = serde_json::Value::deserialize(deserializer)?;
    let mut out = Vec::new();
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                out.push((k, v.as_str().unwrap_or_default().to_string()));
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                if let Some(pair) = item.as_array() {
                    let a = pair.first().and_then(|v| v.as_str()).unwrap_or_default();
                    let b = pair.get(1).and_then(|v| v.as_str()).unwrap_or_default();
                    out.push((a.to_string(), b.to_string()));
                }
            }
        }
        _ => {}
    }
    Ok(out)
}

/// Full editor configuration — field names match upstream `tools/config.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuiConfig {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub auto_connect: AutoConnect,
    #[serde(default)]
    pub server_list: ServerList,
    /// Theme colour remap, e.g. `{"#f64a46": "#00ff00"}`.
    #[serde(default, deserialize_with = "de_pairs")]
    pub colors: Vec<(String, String)>,
    /// Arbitrary UI string replacements, as `[[old, new], ...]`.
    #[serde(default, deserialize_with = "de_pairs")]
    pub texts: Vec<(String, String)>,
    /// Optional image overrides: archive index → absolute path on disk.
    #[serde(default)]
    pub images: BTreeMap<String, String>,
}

/// What an edit run actually changed, for reporting back to the UI.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EditReport {
    pub entries_changed: usize,
    pub auto_connect: bool,
    pub server_list_entries: usize,
    pub colors_applied: usize,
    pub texts_applied: usize,
    pub title_applied: bool,
    pub images_applied: usize,
    pub bytes_before: usize,
    pub bytes_after: usize,
    pub contiguity_ok: bool,
    pub warnings: Vec<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Edits
// ─────────────────────────────────────────────────────────────────────────────

/// Replace a bounded byte range without changing any surrounding bytes.
fn replace_span(data: &[u8], start: usize, end: usize, replacement: &[u8]) -> Result<Vec<u8>, String> {
    if start > end || end > data.len() {
        return Err("auto-connect replacement span is invalid".to_string());
    }
    let mut out = Vec::with_capacity(data.len() - (end - start) + replacement.len());
    out.extend_from_slice(&data[..start]);
    out.extend_from_slice(replacement);
    out.extend_from_slice(&data[end..]);
    Ok(out)
}

/// Find and validate the old markerless injector used by v1.10–v1.11.3.
/// Returns the span from `_.servers.refresh)` through the end of its setTimeout.
fn legacy_auto_connect_span(data: &[u8]) -> Result<Option<(usize, usize)>, String> {
    let count = count_occurrences(data, LEGACY_AUTO_CONNECT_PREFIX);
    if count == 0 {
        return Ok(None);
    }
    if count != 1 {
        return Err(format!(
            "legacy auto-connect code found {} times; refusing ambiguous migration",
            count
        ));
    }

    let start = data
        .windows(LEGACY_AUTO_CONNECT_PREFIX.len())
        .position(|w| w == LEGACY_AUTO_CONNECT_PREFIX)
        .ok_or("legacy auto-connect prefix vanished")?;
    let search_start = start + LEGACY_AUTO_CONNECT_PREFIX.len();
    let search_end = (search_start + 1024).min(data.len());
    let tail = &data[search_start..search_end];

    for (rel, window) in tail.windows(3).enumerate() {
        if window != b")}," {
            continue;
        }
        let mut i = search_start + rel + 3;
        let digits_start = i;
        while i < search_end && data[i].is_ascii_digit() {
            i += 1;
        }
        if i == digits_start || !data[i..search_end].starts_with(b")}") {
            continue;
        }
        return Ok(Some((start, i + 2)));
    }

    Err("legacy auto-connect code is malformed; restore the index.rui backup before launching".to_string())
}

/// Rewrite the root component's `async mounted()` body so the launcher UI
/// connects straight to `ip:port` instead of showing the server browser.
///
/// # Why equal-length
///
/// The previous approach *grew* `app.js`, which shifts the offsets of every
/// archive entry stored after it — and the shipped build's CEF launcher UI
/// stopped loading entirely ("404 error: File not found"). Byte-length
/// preservation keeps every header table entry valid, so this splice is
/// indistinguishable from the factory archive as far as the client's own
/// `index.rui` reader is concerned.
///
/// # Why delayed, and why the component's own method
///
/// The connect is deferred with `setTimeout(..., delay_ms)` and routed through
/// `this.launchGame({ip, port}, true)` — the component's method — rather than
/// calling `rageApi.launchGame` directly. Two reasons:
///
/// * the method first writes `launch.ip` / `launch.port` through
///   `mp.setConf(...)`, which the native download path expects to be set;
/// * an immediate call races against `window._rageInitialize`, which the
///   native side invokes separately — calling before it lands crashes the
///   renderer.
///
/// It is wrapped in `try/catch` so a failure can never abort the mount and
/// blank the launcher UI.
///
/// Returns `None` when the anchors are missing/ambiguous or the payload cannot
/// fit, so the caller can fall back to [`apply_auto_connect`].
pub fn splice_auto_connect(
    app_js: &[u8],
    ip: &str,
    port: u16,
    delay_ms: u32,
) -> Option<Vec<u8>> {
    // Only a plain host or IPv4 literal is safe to inline into JS source.
    // Anything else (quotes, backslashes, control chars) is refused so a
    // hostile value can never escape the string literal.
    if ip.is_empty()
        || !ip
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_' | b':'))
    {
        return None;
    }

    if count_occurrences(app_js, MOUNTED_OPEN) != 1 || count_occurrences(app_js, MOUNTED_CLOSE) != 1 {
        return None;
    }

    let start = app_js
        .windows(MOUNTED_OPEN.len())
        .position(|w| w == MOUNTED_OPEN)?;
    let body_start = start + MOUNTED_OPEN.len();

    let close = app_js
        .windows(MOUNTED_CLOSE.len())
        .position(|w| w == MOUNTED_CLOSE)?;
    if close < body_start {
        return None;
    }
    // The body runs up to and including the `}` that closes `mounted()`.
    let body_end = close + 1;
    let body_len = body_end - body_start;
    if body_len == 0 || body_len > MAX_SPLICE_BODY {
        return None;
    }

    let payload = format!(
        "{}setTimeout(()=>{{try{{this.launchGame({{ip:\"{}\",port:{}}},!0)}}catch(e){{}}}},{}),this.loader(\"core\",!1)",
        String::from_utf8_lossy(SPLICE_MARKER),
        ip,
        port,
        delay_ms
    );
    let payload = payload.as_bytes();

    // Need the payload, at least one space, and the closing brace.
    if payload.len() + 1 >= body_len {
        return None;
    }

    let mut out = Vec::with_capacity(app_js.len());
    out.extend_from_slice(&app_js[..body_start]);
    out.extend_from_slice(payload);
    out.resize(out.len() + (body_len - payload.len() - 1), b' ');
    out.push(b'}');
    out.extend_from_slice(&app_js[body_end..]);

    debug_assert_eq!(out.len(), app_js.len(), "splice must be length preserving");
    Some(out)
}

/// Whether `app.js` carries the equal-length auto-connect splice.
pub fn is_connect_spliced(app_js: &[u8]) -> bool {
    count_occurrences(app_js, SPLICE_MARKER) == 1
}

/// Span of a marker-wrapped injection, validated. `None` when absent.
fn marker_block_span(data: &[u8]) -> Result<Option<(usize, usize)>, String> {
    let start_count = count_occurrences(data, MARKER_START);
    let end_count = count_occurrences(data, MARKER_END);
    if start_count == 0 && end_count == 0 {
        return Ok(None);
    }
    if start_count != 1 || end_count != 1 {
        return Err(format!(
            "auto-connect markers are inconsistent (start={}, end={})",
            start_count, end_count
        ));
    }
    let start = data
        .windows(MARKER_START.len())
        .position(|w| w == MARKER_START)
        .ok_or("auto-connect start marker vanished")?;
    let end_marker_start = data
        .windows(MARKER_END.len())
        .position(|w| w == MARKER_END)
        .ok_or("auto-connect end marker vanished")?;
    let end = end_marker_start + MARKER_END.len();
    if end_marker_start < start {
        return Err("auto-connect markers are out of order".to_string());
    }
    Ok(Some((start, end)))
}

/// Remove a marker-wrapped injection, leaving every other byte intact.
fn strip_marker_block(data: &[u8]) -> Result<Vec<u8>, String> {
    match marker_block_span(data)? {
        Some((start, end)) => replace_span(data, start, end, b""),
        None => Ok(data.to_vec()),
    }
}

/// Remove a legacy markerless injection, restoring the `_.servers.refresh)}`
/// anchor it consumed so a later inject can still find it.
fn strip_legacy_block(data: &[u8]) -> Result<Vec<u8>, String> {
    match legacy_auto_connect_span(data)? {
        Some((start, end)) => {
            // The legacy code replaced the anchor's closing brace with
            // `,setTimeout(…)}`; put the brace back so ANCHOR_AUTO_CONNECT is
            // intact for the fallback injector.
            const KEEP: &[u8] = b"_.servers.refresh),";
            let from = start + KEEP.len();
            if from > end || !data[start..].starts_with(KEEP) {
                return Err("legacy auto-connect layout is unexpected".to_string());
            }
            replace_span(data, from, end, b"}")
        }
        None => Ok(data.to_vec()),
    }
}

/// Inject the auto-connect bootstrap into `app.js`.
///
/// Strategy, in order:
/// 1. A previous marker-wrapped injection is replaced — preferably by an
///    equal-length splice, else in place by a fresh marker block.
/// 2. A legacy markerless injection (v1.10–v1.11.3) is migrated the same way.
/// 3. Nothing injected yet — the equal-length splice is preferred.
/// 4. Fallback: grow the file at the anchor and wrap the block in markers.
///
/// The splice exists because the growing path shifts the offsets of every
/// archive entry stored after `app.js`, and the shipped client's CEF launcher
/// UI then failed to load at all ("404 error: File not found").
pub fn apply_auto_connect(app_js: &[u8], ac: &AutoConnect) -> Result<Vec<u8>, String> {
    if !ac.enabled {
        return Ok(app_js.to_vec());
    }
    if ac.ip.trim().is_empty() {
        return Err("auto_connect.ip is empty".to_string());
    }

    let call = format!(
        "{}setTimeout(()=>{{rageApi.isALauncher&&this.launchGame({{ip:\"{}\",port:{},name:\"{}\"}},!0)}},{}){}",
        String::from_utf8_lossy(MARKER_START),
        js_escape(&ac.ip),
        ac.port,
        js_escape(&ac.name),
        ac.delay_ms,
        String::from_utf8_lossy(MARKER_END),
    );

    // 1. Replace an existing marker-wrapped injection.
    if let Some((start, end)) = marker_block_span(app_js)? {
        if let Some(spliced) = splice_auto_connect(&strip_marker_block(app_js)?, ac.ip.trim(), ac.port, ac.delay_ms) {
            log_info!("rui: auto-connect re-spliced (equal-length) -> {}:{}", ac.ip, ac.port);
            return Ok(spliced);
        }
        return replace_span(app_js, start, end, call.as_bytes());
    }

    // 2. Migrate a legacy markerless injection.
    if let Some((start, end)) = legacy_auto_connect_span(app_js)? {
        if let Some(spliced) = splice_auto_connect(&strip_legacy_block(app_js)?, ac.ip.trim(), ac.port, ac.delay_ms) {
            log_info!("rui: legacy injector replaced by an equal-length splice -> {}:{}", ac.ip, ac.port);
            return Ok(spliced);
        }
        let mut replacement = b"_.servers.refresh),".to_vec();
        replacement.extend_from_slice(call.as_bytes());
        replacement.push(b'}');
        return replace_span(app_js, start, end, &replacement);
    }

    // 3. Nothing injected yet — the equal-length splice wins.
    if let Some(spliced) = splice_auto_connect(app_js, ac.ip.trim(), ac.port, ac.delay_ms) {
        log_info!(
            "rui: auto-connect spliced into mounted() (equal-length) -> {}:{}",
            ac.ip,
            ac.port
        );
        return Ok(spliced);
    }

    // 4. Fallback: grow at the anchor.
    log_warn!(
        "rui: anchored splice unavailable for this app.js build — falling back to the growing injector"
    );
    let injection = format!("_.servers.refresh),{}", call);
    let mut replacement = injection.into_bytes();
    replacement.push(b'}');
    replace_unique(app_js, ANCHOR_AUTO_CONNECT, &replacement, "auto-connect")
}

/// Override the server browser list inside `app.js`.
pub fn apply_server_list(app_js: &[u8], sl: &ServerList) -> Result<Vec<u8>, String> {
    if !sl.override_list {
        return Ok(app_js.to_vec());
    }

    // Build the same JSON shape upstream produces.
    let mut arr = Vec::with_capacity(sl.servers.len());
    for (i, s) in sl.servers.iter().enumerate() {
        let name = if s.name.trim().is_empty() {
            format!("Server {}", i)
        } else {
            s.name.clone()
        };
        arr.push(serde_json::json!({
            "id": i,
            "name": name,
            "ip": if s.ip.trim().is_empty() { "127.0.0.1" } else { &s.ip },
            "port": s.port,
            "players": s.players,
            "maxplayers": s.maxplayers,
            "ping": s.ping,
            "lang": s.lang,
            "gamemode": s.gamemode,
            "url": s.url,
        }));
    }
    let json = serde_json::to_string(&arr).map_err(|e| format!("failed to encode server list: {}", e))?;

    let injection = format!(
        "async mounted(){{try{{rageApi.getServers=function(_c){{_c(null,{})}}}}catch(e){{}}this.loader(\"core\",!0)",
        json
    );

    replace_unique(app_js, ANCHOR_SERVER_LIST, injection.as_bytes(), "server-list")
}

/// Remap theme colours across a blob.
///
/// Applied longest-key-first so `#ffffff` is not partially consumed by a
/// shorter `#fff` rule.
pub fn apply_colors(blob: &[u8], colors: &[(String, String)]) -> (Vec<u8>, usize) {
    let mut rules: Vec<&(String, String)> = colors.iter().collect();
    rules.sort_by_key(|(old, _)| std::cmp::Reverse(old.len()));

    let mut out = blob.to_vec();
    let mut total = 0;
    for (old, new) in rules {
        if old.is_empty() {
            continue;
        }
        let (next, n) = replace_all(&out, old.as_bytes(), new.as_bytes());
        out = next;
        total += n;
    }
    (out, total)
}

/// Apply literal UI string replacements.
pub fn apply_texts(blob: &[u8], texts: &[(String, String)]) -> (Vec<u8>, usize) {
    let mut out = blob.to_vec();
    let mut total = 0;
    for (old, new) in texts {
        if old.is_empty() {
            continue;
        }
        let (next, n) = replace_all(&out, old.as_bytes(), new.as_bytes());
        out = next;
        total += n;
    }
    (out, total)
}

/// Replace `<title>...</title>` inside `index.html`.
pub fn apply_title(html: &[u8], title: &str) -> (Vec<u8>, bool) {
    let open = b"<title>";
    let close = b"</title>";

    let Some(start) = html.windows(open.len()).position(|w| w == open) else {
        return (html.to_vec(), false);
    };
    let Some(rel_end) = html[start + open.len()..]
        .windows(close.len())
        .position(|w| w == close)
    else {
        return (html.to_vec(), false);
    };
    let end = start + open.len() + rel_end + close.len();

    let mut out = Vec::with_capacity(html.len());
    out.extend_from_slice(&html[..start]);
    out.extend_from_slice(b"<title>");
    out.extend_from_slice(title.as_bytes());
    out.extend_from_slice(b"</title>");
    out.extend_from_slice(&html[end..]);
    (out, true)
}

// ─────────────────────────────────────────────────────────────────────────────
// High-level driver
// ─────────────────────────────────────────────────────────────────────────────

/// Backup path for a given archive.
pub fn backup_path(rui: &Path) -> PathBuf {
    let name = rui
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "index.rui".to_string());
    rui.with_file_name(format!("{}{}", name, BACKUP_SUFFIX))
}

/// Copy `rui` to its backup once. Returns the backup path.
pub fn ensure_backup(rui: &Path) -> Result<PathBuf, String> {
    let backup = backup_path(rui);
    if !backup.exists() {
        fs::copy(rui, &backup).map_err(|e| format!("failed to back up index.rui: {}", e))?;
        log_info!("rui backup created: {:?}", backup);
    }
    Ok(backup)
}

/// Restore the archive from its backup, if one exists.
pub fn restore_backup(rui: &Path) -> Result<bool, String> {
    let backup = backup_path(rui);
    if !backup.exists() {
        return Ok(false);
    }
    fs::copy(&backup, rui).map_err(|e| format!("failed to restore index.rui: {}", e))?;
    log_info!("rui restored from {:?}", backup);
    Ok(true)
}

/// Locate `rui\index.rui` inside a RAGE:MP install.
pub fn rui_path(ragemp_dir: &Path) -> PathBuf {
    ragemp_dir.join("rui").join("index.rui")
}

/// Whether this archive data contains an ASTRA auto-connect injection.
fn contains_auto_connect(data: &[u8]) -> bool {
    count_occurrences(data, MARKER_START) == 1
        || count_occurrences(data, LEGACY_AUTO_CONNECT_PREFIX) == 1
        || is_connect_spliced(data)
}

/// Whether the archive contains either the current marked injector or a
/// markerless injector written by an older ASTRA build.
pub fn is_injected(ragemp_dir: &Path) -> bool {
    let path = rui_path(ragemp_dir);
    match fs::read(&path) {
        Ok(data) => contains_auto_connect(&data),
        Err(_) => false,
    }
}

/// Apply a full [`RuiConfig`] to the archive at `ragemp_dir`, writing in place.
///
/// A backup is taken before the first write. Returns a report of what changed.
pub fn apply_config(ragemp_dir: &Path, config: &RuiConfig) -> Result<EditReport, String> {
    let path = rui_path(ragemp_dir);
    if !path.exists() {
        return Err(format!("index.rui not found at {:?}", path));
    }

    let original = fs::read(&path).map_err(|e| format!("failed to read index.rui: {}", e))?;
    let mut archive = Archive::parse(&original)?;

    let mut report = EditReport {
        bytes_before: original.len(),
        ..Default::default()
    };

    let app_idx = archive.app_js_index();
    let html_idx = archive.index_html_index();

    // ── app.js: auto-connect + server list + colour/text replacements ──
    if let Some(i) = app_idx {
        let before = archive.blobs[i].clone();
        let mut blob = before.clone();

        if config.auto_connect.enabled {
            blob = apply_auto_connect(&blob, &config.auto_connect)?;
            report.auto_connect = true;
            log_info!(
                "rui: auto-connect -> {}:{} ({}ms)",
                config.auto_connect.ip,
                config.auto_connect.port,
                config.auto_connect.delay_ms
            );
        }

        if config.server_list.override_list {
            blob = apply_server_list(&blob, &config.server_list)?;
            report.server_list_entries = config.server_list.servers.len();
            log_info!("rui: server list override ({} entries)", report.server_list_entries);
        }

        if !config.colors.is_empty() {
            let (next, n) = apply_colors(&blob, &config.colors);
            blob = next;
            report.colors_applied = n;
        }
        if !config.texts.is_empty() {
            let (next, n) = apply_texts(&blob, &config.texts);
            blob = next;
            report.texts_applied = n;
        }

        if blob != before {
            archive.blobs[i] = blob;
            report.entries_changed += 1;
        }
    } else {
        report
            .warnings
            .push("app.js not found in archive — auto-connect/server-list skipped".to_string());
        log_warn!("rui: app.js entry not found");
    }

    // ── index.html: title + colour/text replacements ──
    if let Some(i) = html_idx {
        let before = archive.blobs[i].clone();
        let mut blob = before.clone();

        if let Some(title) = &config.title {
            let (next, applied) = apply_title(&blob, title);
            blob = next;
            report.title_applied = applied;
            if applied {
                log_info!("rui: title -> {}", title);
            }
        }
        if !config.colors.is_empty() {
            let (next, n) = apply_colors(&blob, &config.colors);
            blob = next;
            report.colors_applied += n;
        }
        if !config.texts.is_empty() {
            let (next, n) = apply_texts(&blob, &config.texts);
            blob = next;
            report.texts_applied += n;
        }

        if blob != before {
            archive.blobs[i] = blob;
            report.entries_changed += 1;
        }
    }

    // ── image overrides by archive index ──
    for (key, file_path) in &config.images {
        let Ok(idx) = key.parse::<usize>() else {
            report.warnings.push(format!("image key '{}' is not an index", key));
            continue;
        };
        if idx >= archive.blobs.len() {
            report
                .warnings
                .push(format!("image index {} out of range (0..{})", idx, archive.blobs.len()));
            continue;
        }
        match fs::read(file_path) {
            Ok(bytes) => {
                if archive.blobs[idx] != bytes {
                    archive.blobs[idx] = bytes;
                    report.entries_changed += 1;
                    report.images_applied += 1;
                    log_info!("rui: image idx{} <- {}", idx, file_path);
                }
            }
            Err(e) => report
                .warnings
                .push(format!("cannot read image '{}': {}", file_path, e)),
        }
    }

    // ── rebuild + verify + write ──
    archive.verify_roundtrip()?;
    let rebuilt = archive.serialize()?;

    // Re-parse what we are about to write and confirm contiguity.
    let check = Archive::parse(&rebuilt)?;
    report.contiguity_ok = check.is_contiguous();
    if !report.contiguity_ok {
        return Err("rebuilt archive failed the contiguity check — aborting write".to_string());
    }

    report.bytes_after = rebuilt.len();

    ensure_backup(&path)?;
    fs::write(&path, &rebuilt).map_err(|e| format!("failed to write index.rui: {}", e))?;

    log_info!(
        "rui written: {} -> {} bytes, {} entr(ies) changed",
        report.bytes_before,
        report.bytes_after,
        report.entries_changed
    );

    Ok(report)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

/// Auto-connect the way the reference project does it (`tiny-ragemp-launcher`).
///
/// Two details matter here:
///
/// * The injection goes **before the closing brace of `mounted()`**, at the
///   `_.servers.refresh)}` anchor. Everything the method already does
///   (`getServers`, the tab caches, `refreshInterval`) stays untouched.
///   Replacing the whole body instead left the UI without
///   `window._rageInitialize` and CEF reported "File not found".
/// * The blob is allowed to **grow**, and every size/offset in the header is
///   rebuilt afterwards. The XXH64 fields are left alone, exactly like the
///   reference implementation — the client does not appear to verify them.
pub fn apply_upstream_auto_connect(
    ragemp_dir: &Path,
    ip: &str,
    port: u16,
    name: &str,
    delay_ms: u32,
) -> Result<bool, String> {
    const HEADER_END: usize = 8488;
    const TABLE2_BASE: usize = 4240;
    const ANCHOR: &[u8] = b"_.servers.refresh)}";

    if ip.is_empty() || !ip.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_')) {
        return Err(format!("refusing to inline '{}' into JS", ip));
    }

    let path = ragemp_dir.join("rui").join("index.rui");
    let data = fs::read(&path).map_err(|e| format!("read {}: {}", path.display(), e))?;
    if data.len() < HEADER_END {
        return Err("index.rui is too small to be an archive".to_string());
    }

    // Header tables: two runs of 16-byte slots holding (size, offset) as u32.
    let mut entries: Vec<(usize, usize, usize, usize)> = Vec::with_capacity(TOTAL_ENTRIES);
    for k in 0..132usize {
        let sp = (2 * k + 1) * 16;
        let op = (2 * k + 2) * 16;
        let size = u32::from_le_bytes(data[sp..sp + 4].try_into().unwrap()) as usize;
        let off = u32::from_le_bytes(data[op..op + 4].try_into().unwrap()) as usize;
        entries.push((sp, op, size, off));
    }
    for k in 0..133usize {
        let sp = TABLE2_BASE + (2 * k) * 16;
        let op = TABLE2_BASE + (2 * k + 1) * 16;
        let size = u32::from_le_bytes(data[sp..sp + 4].try_into().unwrap()) as usize;
        let off = u32::from_le_bytes(data[op..op + 4].try_into().unwrap()) as usize;
        entries.push((sp, op, size, off));
    }
    entries.sort_by_key(|e| e.3);

    // Locate app.js: the one entry carrying the mounted() anchor.
    let mut target: Option<usize> = None;
    for (i, (_, _, size, off)) in entries.iter().enumerate() {
        if off.checked_add(*size).map_or(true, |end| end > data.len()) {
            return Err(format!("entry {} is out of bounds", i));
        }
        if count_occurrences(&data[*off..*off + *size], ANCHOR) == 1 {
            target = Some(i);
            break;
        }
    }
    let idx = target.ok_or("auto-connect anchor not found — unsupported index.rui build")?;

    let (_, _, size, off) = entries[idx];
    let blob = &data[off..off + size];

    // Already injected → nothing to do (keeps the call idempotent).
    if blob.windows(24).any(|w| w == b"setTimeout(()=>{rageApi.is") {
        log_info!("rui: auto-connect already present");
        return Ok(true);
    }

    let injection = format!(
        "_.servers.refresh),setTimeout(()=>{{rageApi.isALauncher&&this.launchGame({{ip:\"{}\",port:{},name:\"{}\"}},!0)}},{})}}",
        ip, port, name, delay_ms
    );
    let new_blob = replace_unique(blob, ANCHOR, injection.as_bytes(), "auto-connect")?;

    let mut blobs: Vec<Vec<u8>> = entries
        .iter()
        .map(|(_, _, s, o)| data[*o..*o + *s].to_vec())
        .collect();
    blobs[idx] = new_blob;

    let last_end = entries.last().map(|e| e.3 + e.2).unwrap_or(HEADER_END);
    let trailing = data.get(last_end..).unwrap_or(&[]).to_vec();

    let mut offs = Vec::with_capacity(blobs.len());
    let mut cursor = HEADER_END;
    for b in &blobs {
        offs.push(cursor);
        cursor += b.len();
    }

    let mut header = data[..HEADER_END].to_vec();
    for (i, (sp, op, _, _)) in entries.iter().enumerate() {
        if *sp + 4 > header.len() || *op + 4 > header.len() {
            return Err("header slot out of range".to_string());
        }
        header[*sp..*sp + 4].copy_from_slice(&(blobs[i].len() as u32).to_le_bytes());
        header[*op..*op + 4].copy_from_slice(&(offs[i] as u32).to_le_bytes());
    }

    let mut out = header;
    for b in &blobs {
        out.extend_from_slice(b);
    }
    out.extend_from_slice(&trailing);

    fs::write(&path, &out).map_err(|e| format!("write {}: {}", path.display(), e))?;
    log_info!(
        "rui: auto-connect injected upstream-style -> {}:{} ({} -> {} bytes)",
        ip,
        port,
        data.len(),
        out.len()
    );
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a synthetic archive with the real layout so we can round-trip it.
    fn synthetic(count_small: usize) -> Vec<u8> {
        // 265 files: a few recognisable ones plus filler.
        let mut blobs: Vec<Vec<u8>> = Vec::new();
        for i in 0..TOTAL_ENTRIES {
            let body = match i {
                // Place app.js / index.html at the same indices as the real build.
                i if i == 262 => {
                    let mut b = b"(function(){".to_vec();
                    b.extend_from_slice(b"x".repeat(64).as_slice());
                    b.extend_from_slice(ANCHOR_AUTO_CONNECT);
                    b.extend_from_slice(b"y".repeat(16).as_slice());
                    b.extend_from_slice(ANCHOR_SERVER_LIST);
                    b.extend_from_slice(b"z".repeat(16).as_slice());
                    b
                }
                i if i == 261 => {
                    let mut b = b"<!DOCTYPE html><html><head>".to_vec();
                    b.extend_from_slice(b"<title>rage-ui</title>");
                    b.extend_from_slice(b"</head></html>".to_vec().as_slice());
                    b
                }
                _ => format!("file-{}-{}", i, "a".repeat(count_small)).into_bytes(),
            };
            blobs.push(body);
        }

        // Emit header tables in the archive's native (unsorted) order.
        let mut header = vec![0u8; HEADER_END];
        let mut cursor = HEADER_END as u32;
        let mut order: Vec<(usize, usize, usize)> = Vec::new(); // (size_pos, off_pos, blob_idx)

        for k in 0..TABLE1_COUNT {
            order.push(((2 * k + 1) * 16, (2 * k + 2) * 16, k));
        }
        for k in 0..TABLE2_COUNT {
            order.push((
                TABLE2_BASE + (2 * k) * 16,
                TABLE2_BASE + (2 * k + 1) * 16,
                TABLE1_COUNT + k,
            ));
        }

        for (size_pos, off_pos, idx) in &order {
            let size = blobs[*idx].len() as u32;
            header[*size_pos..*size_pos + 4].copy_from_slice(&size.to_le_bytes());
            header[*off_pos..*off_pos + 4].copy_from_slice(&cursor.to_le_bytes());
            // The client verifies XXH64(content) per entry, so a synthetic
            // archive must carry valid hashes too.
            header[*size_pos + 8..*size_pos + 16].copy_from_slice(&xxh64(&blobs[*idx]).to_le_bytes());
            cursor += size;
        }

        let mut out = header;
        for b in &blobs {
            out.extend_from_slice(b);
        }
        out
    }

    #[test]
    fn parses_the_real_layout() {
        let bytes = synthetic(4);
        let a = Archive::parse(&bytes).unwrap();
        assert_eq!(a.entries.len(), TOTAL_ENTRIES);
        assert_eq!(a.entries[0].off as usize, HEADER_END);
        assert!(a.is_contiguous());
    }

    #[test]
    fn roundtrip_is_byte_exact_for_an_unmodified_archive() {
        let bytes = synthetic(4);
        let a = Archive::parse(&bytes).unwrap();
        let rebuilt = a.serialize().unwrap();
        assert_eq!(rebuilt, bytes, "an untouched archive must repack identically");
    }

    #[test]
    fn detects_app_js_and_index_html() {
        let bytes = synthetic(4);
        let a = Archive::parse(&bytes).unwrap();
        assert_eq!(a.app_js_index(), Some(262));
        assert_eq!(a.index_html_index(), Some(261));
    }

    #[test]
    fn auto_connect_injects_exactly_once() {
        let bytes = synthetic(4);
        let a = Archive::parse(&bytes).unwrap();
        let i = a.app_js_index().unwrap();
        let out = apply_auto_connect(
            &a.blobs[i],
            &AutoConnect {
                enabled: true,
                ip: "185.176.94.21".into(),
                port: 22005,
                name: "ASTRA".into(),
                delay_ms: 1500,
            },
        )
        .unwrap();
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains(r#"launchGame({ip:"185.176.94.21",port:22005,name:"ASTRA"},!0)"#));
        assert!(s.contains("},1500)/*astra-installed-end*/}"));
        assert_eq!(count_occurrences(&out, ANCHOR_AUTO_CONNECT), 0, "anchor consumed");
        assert_eq!(count_occurrences(&out, MARKER_START), 1);
        assert_eq!(count_occurrences(&out, MARKER_END), 1);
        assert!(contains_auto_connect(&out));
    }

    #[test]
    fn auto_connect_updates_the_same_marked_block_on_repeated_launch() {
        let first = apply_auto_connect(
            b"_.servers.refresh)}",
            &AutoConnect {
                enabled: true,
                ip: "1.2.3.4".into(),
                port: 22005,
                name: "First".into(),
                delay_ms: 1500,
            },
        )
        .unwrap();
        let second = apply_auto_connect(
            &first,
            &AutoConnect {
                enabled: true,
                ip: "5.6.7.8".into(),
                port: 22006,
                name: "Second".into(),
                delay_ms: 3000,
            },
        )
        .unwrap();

        let s = String::from_utf8_lossy(&second);
        assert!(!s.contains("1.2.3.4"), "old server address must be removed");
        assert!(s.contains(r#"launchGame({ip:"5.6.7.8",port:22006,name:"Second"},!0)"#));
        assert_eq!(count_occurrences(&second, MARKER_START), 1);
        assert_eq!(count_occurrences(&second, MARKER_END), 1);
    }

    #[test]
    fn auto_connect_migrates_markerless_legacy_block() {
        let legacy = b"function(){_.servers.refresh),setTimeout(()=>{rageApi.isALauncher&&this.launchGame({ip:\"1.2.3.4\",port:22005,name:\"Old\"},!0)},2000)}";
        let out = apply_auto_connect(
            legacy,
            &AutoConnect {
                enabled: true,
                ip: "5.6.7.8".into(),
                port: 22006,
                name: "New".into(),
                delay_ms: 3000,
            },
        )
        .unwrap();
        let s = String::from_utf8_lossy(&out);
        assert!(!s.contains("1.2.3.4"));
        assert!(s.contains(r#"launchGame({ip:"5.6.7.8",port:22006,name:"New"},!0)"#));
        assert_eq!(count_occurrences(&out, MARKER_START), 1);
        assert!(contains_auto_connect(&out));
    }

    #[test]
    fn detects_legacy_and_marked_auto_connect_blocks() {
        assert!(contains_auto_connect(b"/*astra-installed-start*/setTimeout(/*astra-installed-end*/"));
        assert!(contains_auto_connect(LEGACY_AUTO_CONNECT_PREFIX));
        assert!(!contains_auto_connect(b"no auto connect here"));
    }

    #[test]
    fn auto_connect_escapes_hostile_name() {
        let src = b"_.servers.refresh)}".to_vec();
        let out = apply_auto_connect(
            &src,
            &AutoConnect {
                enabled: true,
                ip: "1.2.3.4".into(),
                port: 22005,
                name: "x\",evil:\"1".into(),
                delay_ms: 1000,
            },
        )
        .unwrap();
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains(r#"name:"x\",evil:\"1""#), "quotes escaped: {}", s);
    }

    #[test]
    fn auto_connect_disabled_is_a_noop() {
        let src = b"_.servers.refresh)}".to_vec();
        let out = apply_auto_connect(&src, &AutoConnect::default()).unwrap();
        assert_eq!(out, src);
    }

    #[test]
    fn auto_connect_requires_a_unique_anchor() {
        let src = b"_.servers.refresh)}___".to_vec();
        let dup = [src.as_slice(), src.as_slice()].concat();
        let err = apply_auto_connect(
            &dup,
            &AutoConnect {
                enabled: true,
                ip: "1.2.3.4".into(),
                ..Default::default()
            },
        );
        assert!(err.is_err(), "ambiguous anchor must be rejected");
    }

    #[test]
    fn server_list_override_emits_json() {
        let src = b"async mounted(){this.loader(\"core\",!0)".to_vec();
        let sl = ServerList {
            override_list: true,
            servers: vec![ServerEntry {
                name: "ASTRA RP".into(),
                ip: "185.176.94.21".into(),
                port: 22005,
                players: 42,
                maxplayers: 500,
                ping: 12,
                lang: "ru".into(),
                gamemode: "Roleplay".into(),
                url: "".into(),
            }],
        };
        let out = apply_server_list(&src, &sl).unwrap();
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains(r#""name":"ASTRA RP""#));
        assert!(s.contains(r#""port":22005"#));
        assert!(s.contains(r#""id":0"#));
        assert!(s.starts_with(r#"async mounted(){try{rageApi.getServers=function(_c){_c(null,["#));
    }

    #[test]
    fn title_replacement_works_and_is_bounded() {
        let html = b"<html><head><title>rage-ui</title></head></html>";
        let (out, applied) = apply_title(html, "ASTRA RP");
        assert!(applied);
        assert_eq!(
            String::from_utf8_lossy(&out),
            "<html><head><title>ASTRA RP</title></head></html>"
        );

        let (same, applied2) = apply_title(b"<html>no title here</html>", "X");
        assert!(!applied2);
        assert_eq!(&same, b"<html>no title here</html>");
    }

    #[test]
    fn colours_are_applied_longest_key_first() {
        // "#fff" is a substring of "#ffffff"; the longer rule must win.
        let blob = b"color:#ffffff;border:#fff";
        let (out, n) = apply_colors(
            blob,
            &[
                ("#fff".to_string(), "#000".to_string()),
                ("#ffffff".to_string(), "#123456".to_string()),
            ],
        );
        let s = String::from_utf8_lossy(&out);
        assert_eq!(s, "color:#123456;border:#000");
        assert_eq!(n, 2);
    }

    #[test]
    fn config_accepts_upstream_shapes() {
        // colours as an object, texts as an array of pairs — exactly like
        // upstream tools/config.json.
        let json = r##"{
            "title": "My Server",
            "auto_connect": {"enabled": true, "ip": "1.2.3.4", "port": 22005},
            "server_list": {"override": true, "servers": []},
            "colors": {"#f64a46": "#00ff00"},
            "texts": [["Play", "Играть"]]
        }"##;
        let cfg: RuiConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.title.as_deref(), Some("My Server"));
        assert!(cfg.auto_connect.enabled);
        assert_eq!(cfg.auto_connect.port, 22005);
        assert!(cfg.server_list.override_list);
        assert_eq!(cfg.colors, vec![("#f64a46".to_string(), "#00ff00".to_string())]);
        assert_eq!(cfg.texts, vec![("Play".to_string(), "Играть".to_string())]);
    }

    #[test]
    fn rejects_non_rui_input() {
        assert!(Archive::parse(b"too small").is_err());
        assert!(Archive::parse(&vec![0u8; HEADER_END]).is_ok()); // all-zero tables parse
    }

    #[test]
    fn corrupt_offsets_are_rejected() {
        let mut bytes = synthetic(4);
        // Point the first entry's offset far past the end of the file.
        let off_pos = 2 * 16; // (2*0+2)*16
        bytes[off_pos..off_pos + 4].copy_from_slice(&0xFFFF_FF00u32.to_le_bytes());
        assert!(Archive::parse(&bytes).is_err());
    }

    #[test]
    fn inspect_matches_full_parse_on_the_synthetic_archive() {
        let bytes = synthetic(4);
        let full = Archive::parse(&bytes).unwrap();
        let info = inspect(&bytes).unwrap();

        assert_eq!(info.entries, full.entries.len());
        assert_eq!(info.contiguous, full.is_contiguous());
        assert_eq!(info.app_js_index, full.app_js_index());
        assert_eq!(info.index_html_index, full.index_html_index());
        assert_eq!(info.total_size, bytes.len());
        assert_eq!(info.trailing_size, full.trailing.len());
    }

    #[test]
    fn inspect_matches_full_parse_on_the_real_archive() {
        let path = std::path::Path::new(r"C:\RAGEMP\rui\index.rui");
        if !path.exists() {
            eprintln!("skipping: no RAGE:MP install at C:\\RAGEMP");
            return;
        }
        let data = fs::read(path).unwrap();

        let info = inspect(&data).unwrap();
        assert_eq!(info.entries, TOTAL_ENTRIES);
        assert!(info.contiguous);
        assert_eq!(info.app_js_index, Some(262));
        assert_eq!(info.index_html_index, Some(261));
        assert_eq!(info.total_size, data.len());
        // Sanity: the payload should be the bulk of the file.
        assert!(info.payload_size > data.len() / 2);
    }

    #[test]
    fn inspect_rejects_non_rui_input() {
        assert!(inspect(b"tiny").is_err());
    }

    /// Round-trip the *real* 15 MB archive when one is installed.
    ///
    /// This is the strongest correctness check available: an untouched archive
    /// must repack byte-for-byte. Skips silently on machines without RAGE:MP,
    /// so CI stays green either way.
    #[test]
    fn real_archive_auto_connect_is_repeatable_without_writing_to_disk() {
        let path = std::path::Path::new(r"C:\RAGEMP\rui\index.rui");
        if !path.exists() {
            eprintln!("skipping: no RAGE:MP install at C:\\RAGEMP");
            return;
        }

        let data = fs::read(path).expect("read real index.rui");
        let mut archive = Archive::parse(&data).expect("parse real index.rui");
        let app_idx = archive.app_js_index().expect("find real app.js");
        let config = AutoConnect {
            enabled: true,
            ip: "185.176.94.21".into(),
            port: 22005,
            name: "ASTRA AutoConnect".into(),
            delay_ms: 2000,
        };

        let first = apply_auto_connect(&archive.blobs[app_idx], &config)
            .expect("inject into real app.js");
        let second = apply_auto_connect(&first, &config)
            .expect("repeat injection into real app.js");

        assert_eq!(first, second, "same config must be byte-stable across launches");
        // The shipped app.js accepts the equal-length splice, so no growing
        // marker block is written and the file length is untouched.
        assert_eq!(first.len(), archive.blobs[app_idx].len(), "splice must not grow app.js");
        assert!(is_connect_spliced(&second), "repeat must stay spliced");
        assert_eq!(count_occurrences(&second, MARKER_START), 0);
        let s = String::from_utf8_lossy(&second);
        assert!(s.contains(r#"this.launchGame({ip:"185.176.94.21",port:22005},!0)"#));
        assert!(!s.contains("setInterval(this.refresh,_.servers.refresh)"), "old body must be gone");
        archive.blobs[app_idx] = second;

        let rebuilt = archive.serialize().expect("serialize in memory");
        assert_eq!(rebuilt.len(), data.len(), "archive size must not change");
        let reparsed = Archive::parse(&rebuilt).expect("reparse in memory");
        assert!(reparsed.is_contiguous());
        assert!(contains_auto_connect(&reparsed.blobs[app_idx]));

        // Every entry's stored hash must match XXH64 of its content — this is
        // what the client verifies before it will render the launcher UI.
        for (i, e) in reparsed.entries.iter().enumerate() {
            let stored = u64::from_le_bytes(
                rebuilt[e.hash_pos..e.hash_pos + 8].try_into().expect("8 bytes"),
            );
            assert_eq!(
                stored,
                xxh64(&reparsed.blobs[i]),
                "entry {} has a stale content hash after auto-connect injection",
                i
            );
        }

        // This test is strictly in-memory; the installed file is untouched.
        assert_eq!(fs::read(path).unwrap(), data);
    }

    #[test]
    fn real_archive_roundtrip_if_present() {
        let path = std::path::Path::new(r"C:\RAGEMP\rui\index.rui");
        if !path.exists() {
            eprintln!("skipping: no RAGE:MP install at C:\\RAGEMP");
            return;
        }

        let data = fs::read(path).expect("read real index.rui");
        let archive = Archive::parse(&data).expect("parse real index.rui");

        assert_eq!(archive.entries.len(), TOTAL_ENTRIES, "expected 265 entries");
        assert_eq!(archive.entries[0].off as usize, HEADER_END);
        assert!(archive.is_contiguous(), "real archive must be contiguous");
        assert!(archive.app_js_index().is_some(), "app.js must be found");
        assert!(archive.index_html_index().is_some(), "index.html must be found");

        let rebuilt = archive.serialize().expect("serialize");
        assert_eq!(rebuilt.len(), data.len(), "size must be preserved");
        assert_eq!(rebuilt, data, "an untouched archive must repack byte-for-byte");
    }

    // ── Equal-length auto-connect splice ──

    /// Stand-in for the launcher UI's root component, quoting the anchors the
    /// splice keys off. Mirrors the real build: exactly one `}` between
    /// `_.servers.refresh)` and `,methods:`.
    fn mounted_source() -> Vec<u8> {
        let mut b = b"(function(){".to_vec();
        b.extend_from_slice(b"x".repeat(64).as_slice());
        b.extend_from_slice(b"async mounted(){this.loader(\"core\",!0),");
        b.extend_from_slice(
            b"await this.getServers(this.cache.favorites,!0),\
              this.tabs.favorites.cache=this.cache.favorites,this.tabs.history.cache=this.cache.history,\
              this.loader(\"core\",!1),this.refreshInterval=setInterval(this.refresh,_.servers.refresh)",
        );
        b.extend_from_slice(b"},methods:{loader(A,n){}}");
        b
    }

    #[test]
    fn splice_replaces_the_mounted_body() {
        let src = mounted_source();
        let out = splice_auto_connect(&src, "185.176.94.21", 22005, 2000)
            .expect("anchors present — splice must succeed");
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains(r#"this.launchGame({ip:"185.176.94.21",port:22005},!0)"#));
        assert!(s.contains("/*astra-connect*/"));
        assert!(is_connect_spliced(&out));
    }

    #[test]
    fn splice_is_length_preserving() {
        // The whole point: an equal-length edit leaves every archive offset
        // valid, unlike the growing injector that broke the CEF launcher UI.
        let src = mounted_source();
        let out = splice_auto_connect(&src, "185.176.94.21", 22005, 2000).expect("splice");
        assert_eq!(out.len(), src.len(), "splice must not change app.js length");
    }

    #[test]
    fn splice_keeps_the_closure_paired() {
        let src = mounted_source();
        let out = splice_auto_connect(&src, "185.176.94.21", 22005, 2000).expect("splice");
        // `mounted` must still be closed (the brace survives, merely preceded
        // by space padding), and `methods` must still open.
        let s = String::from_utf8_lossy(&out);
        assert!(
            s.contains("},methods:{loader(A,n){}"),
            "closing brace lost: {}",
            s
        );
        // Balanced enough for a parser: the splice may add a balanced pair
        // (the `function(){}` callback) but must never create an imbalance.
        let o_open = out.iter().filter(|b| **b == b'{').count();
        let o_close = out.iter().filter(|b| **b == b'}').count();
        let s_open = src.iter().filter(|b| **b == b'{').count();
        let s_close = src.iter().filter(|b| **b == b'}').count();
        assert_eq!(
            o_open - s_open,
            o_close - s_close,
            "splice introduced a brace imbalance (o={}/{}, s={}/{})",
            o_open,
            o_close,
            s_open,
            s_close
        );
    }

    #[test]
    fn splice_is_idempotent() {
        let src = mounted_source();
        let first = splice_auto_connect(&src, "185.176.94.21", 22005, 2000).expect("first splice");
        let second = splice_auto_connect(&first, "10.0.0.1", 22005, 2000).expect("second splice");
        assert_eq!(second.len(), first.len(), "repeat must stay length preserving");
        let s = String::from_utf8_lossy(&second);
        assert!(s.contains(r#"this.launchGame({ip:"10.0.0.1",port:22005},!0)"#));
        assert!(!s.contains("185.176.94.21"), "stale address must be gone");
        // Exactly one connect call, never two.
        assert_eq!(s.matches("this.launchGame(").count(), 1);
    }

    #[test]
    fn apply_auto_connect_prefers_the_splice() {
        let src = mounted_source();
        let out = apply_auto_connect(
            &src,
            &AutoConnect {
                enabled: true,
                ip: "185.176.94.21".into(),
                port: 22005,
                name: "ASTRA".into(),
                delay_ms: 2000,
            },
        )
        .unwrap();
        assert_eq!(out.len(), src.len(), "must not grow the file");
        assert!(is_connect_spliced(&out));
        // The anchors must survive so a later launch can re-splice idempotently.
        assert_eq!(count_occurrences(&out, MOUNTED_OPEN), 1, "open anchor must survive");
        assert_eq!(count_occurrences(&out, MOUNTED_CLOSE), 1, "close anchor must survive");
    }

    #[test]
    fn apply_auto_connect_replaces_a_previous_growing_injection() {
        // Simulate the broken path: a marker block already embedded in app.js.
        let mut src = mounted_source();
        src.extend_from_slice(
            b"/*astra-installed-start*/setTimeout(()=>{rageApi.isALauncher&&this.launchGame({ip:\"1.2.3.4\",port:22005,name:\"Old\"},!0)},2000)/*astra-installed-end*/",
        );

        let out = apply_auto_connect(
            &src,
            &AutoConnect {
                enabled: true,
                ip: "185.176.94.21".into(),
                port: 22005,
                name: "ASTRA".into(),
                delay_ms: 2000,
            },
        )
        .unwrap();

        let s = String::from_utf8_lossy(&out);
        assert!(is_connect_spliced(&out), "must end up spliced");
        assert!(!s.contains("astra-installed-start"), "old marker block must be gone");
        assert!(!s.contains("1.2.3.4"), "old address must be gone");
        assert_eq!(s.matches("this.launchGame(").count(), 1);
    }

    #[test]
    fn splice_refuses_hostile_values() {
        let src = mounted_source();
        for bad in [
            "1.2.3.4\"",
            "1.2.3.4\\",
            "ip\"],close()//",
            "1.2.3.4\n",
            "",
            "1.2.3.4;alert(1)",
        ] {
            assert!(
                splice_auto_connect(&src, bad, 22005, 2000).is_none(),
                "value {:?} must be refused",
                bad
            );
        }
        // A safe value still works.
        assert!(splice_auto_connect(&src, "server.example", 22005, 2000).is_some());
    }

    #[test]
    fn real_archive_splice_is_length_preserving_and_valid() {
        let path = std::path::Path::new(r"C:\RAGEMP\rui\index.rui");
        if !path.exists() {
            eprintln!("skipping: no RAGE:MP install at C:\\RAGEMP");
            return;
        }
        let data = fs::read(path).expect("read real index.rui");
        let archive = Archive::parse(&data).expect("parse");
        let i = archive.app_js_index().expect("app.js");
        let app_js = &archive.blobs[i];

        let spliced = splice_auto_connect(app_js, "185.176.94.21", 22005, 2000)
            .expect("the shipped app.js must accept the anchored splice");

        assert_eq!(spliced.len(), app_js.len(), "must not change app.js length");
        assert!(is_connect_spliced(&spliced));
        assert!(!is_connect_spliced(app_js), "factory app.js is not spliced");

        // The `},methods:` seam must survive so the surrounding object stays
        // parseable, and the body must be gone. (`getServers` also appears in
        // other components, so check the body's unique tail instead.)
        let s = String::from_utf8_lossy(&spliced);
        assert!(s.contains("},methods:{loader(A,n)"), "seam broken");
        assert!(
            !s.contains("setInterval(this.refresh,_.servers.refresh)"),
            "old body should be replaced"
        );

        // Emit for an external syntax check (`node --check`).
        let out = std::env::temp_dir().join("astra-real-app-spliced.js");
        fs::write(&out, &spliced).expect("write spliced app.js");
        eprintln!("[ok] real app.js splice written to {:?}", out);
    }

    #[test]
    fn xxh64_matches_known_vectors() {
        // From the reference implementation.
        assert_eq!(xxh64(b""), 0xEF46DB3751D8E999);
        assert_eq!(xxh64(b"a"), 0xD24EC4F1A98C6E5B);
        assert_eq!(xxh64(b"abc"), 0x44BC2CF5AD770999);
    }

    #[test]
    fn factory_hashes_match_xxh64_of_every_entry() {
        // The golden test: the real archive stores XXH64(content) at
        // size_pos+8 for every entry. If this passes, we understand the
        // format well enough to keep those hashes valid when editing.
        let path = std::path::Path::new(r"C:\RAGEMP\rui\index.rui");
        if !path.exists() {
            eprintln!("skipping: no RAGE:MP install at C:\\RAGEMP");
            return;
        }
        let data = fs::read(path).expect("read real index.rui");
        let archive = Archive::parse(&data).expect("parse");

        let mut checked = 0usize;
        let mut mismatched = Vec::new();
        for (i, e) in archive.entries.iter().enumerate() {
            let stored = u64::from_le_bytes(
                data[e.hash_pos..e.hash_pos + 8].try_into().expect("8 bytes"),
            );
            let actual = xxh64(&archive.blobs[i]);
            if stored == actual {
                checked += 1;
            } else {
                mismatched.push((i, e.size_pos));
            }
        }
        assert_eq!(
            mismatched.len(),
            0,
            "entries with a stale content hash: {:?}",
            mismatched
        );
        assert_eq!(checked, archive.entries.len(), "every entry must be checked");
    }

    #[test]
    fn serialise_keeps_the_content_hash_of_an_edited_entry_correct() {
        // Editing a file must refresh its XXH64, not just its size/offset.
        let mut a = Archive::parse(&synthetic(4)).unwrap();
        let app = a.app_js_index().unwrap();
        let original = a.blobs[app].clone();

        a.blobs[app].extend_from_slice(b"/* edited */");
        let rebuilt = a.serialize().expect("serialize");

        let reparsed = Archive::parse(&rebuilt).expect("reparse");
        let e = reparsed.entries[app];
        let stored = u64::from_le_bytes(
            rebuilt[e.hash_pos..e.hash_pos + 8].try_into().expect("8 bytes"),
        );
        assert_eq!(stored, xxh64(&reparsed.blobs[app]), "edited entry hash must be refreshed");
        assert_eq!(reparsed.blobs[app].len(), original.len() + b"/* edited */".len());
    }

    #[test]
    fn serialise_leaves_untouched_hashes_identical() {
        // An unmodified archive must repack with every hash bit-identical.
        let bytes = synthetic(4);
        let a = Archive::parse(&bytes).unwrap();
        let rebuilt = a.serialize().unwrap();
        assert_eq!(rebuilt, bytes, "an untouched archive must repack byte-for-byte");
    }

    #[test]
    fn splice_returns_none_when_anchors_move() {
        let mut src = mounted_source();
        // Break the opening anchor.
        let pos = src.windows(MOUNTED_OPEN.len()).position(|w| w == MOUNTED_OPEN).unwrap();
        src[pos] = b'X';
        assert!(splice_auto_connect(&src, "185.176.94.21", 22005, 2000).is_none());
    }
}
