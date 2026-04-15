use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// BMS のヘッダーキーを表す列挙型。
///
/// よく使われるキー（`TITLE`, `ARTIST`, `BPM` など）は個別のバリアントを持ち、
/// `WAVnn` のように番号付きのキーは `Wav(u8)` にパースされます。
/// 未知のキーは `Other(String)` に入ります（大文字化済み）。
pub enum HeaderKey {
    Player,
    Genre,
    Title,
    Artist,
    Bpm,
    PlayLevel,
    Rank,
    Subtitle,
    SubArtist,
    Difficulty,
    Total,
    Comment,
    LnType,
    /// ステージ用ファイル指定 (`#STAGEFILE ...`)。
    StageFile,
    Wav(u8),
    /// BMP イメージ参照 (`#BMPnn ...`)。
    Bmp(u8),
    Other(String),
}

impl fmt::Display for HeaderKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HeaderKey::Player => write!(f, "PLAYER"),
            HeaderKey::Genre => write!(f, "GENRE"),
            HeaderKey::Title => write!(f, "TITLE"),
            HeaderKey::Artist => write!(f, "ARTIST"),
            HeaderKey::Bpm => write!(f, "BPM"),
            HeaderKey::PlayLevel => write!(f, "PLAYLEVEL"),
            HeaderKey::Rank => write!(f, "RANK"),
            HeaderKey::Subtitle => write!(f, "SUBTITLE"),
            HeaderKey::SubArtist => write!(f, "SUBARTIST"),
            HeaderKey::Difficulty => write!(f, "DIFFICULTY"),
            HeaderKey::Total => write!(f, "TOTAL"),
            HeaderKey::Comment => write!(f, "COMMENT"),
            HeaderKey::LnType => write!(f, "LNTYPE"),
            HeaderKey::Wav(n) => write!(f, "WAV{:02X}", n),
            HeaderKey::StageFile => write!(f, "STAGEFILE"),
            HeaderKey::Bmp(n) => write!(f, "BMP{:02X}", n),
            HeaderKey::Other(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for HeaderKey {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(parse_header_key(s))
    }
}

impl From<&str> for HeaderKey {
    fn from(s: &str) -> Self {
        parse_header_key(s)
    }
}

#[derive(Debug, Clone)]
/// BMSファイルをパースした結果を表す構造体。
///
/// `Bms` はファイル内のヘッダー（キー/値）と小節ごとの譜面データを保持します。
/// 軽量な表現を意図しており、テストや小さなツールから扱いやすいようにしています。
pub struct Bms {
    /// ヘッダーのキー/値ペア（キーは `HeaderKey`）。
    pub headers: HashMap<HeaderKey, String>,

    /// 小節番号 -> その小節に含まれるチャンネルの一覧。
    pub measures: HashMap<u32, Vec<ChannelData>>,
}

impl Bms {
    /// `HeaderKey` を使ってヘッダー値を取得します。
    pub fn header(&self, key: &HeaderKey) -> Option<&str> {
        self.headers.get(key).map(|s| s.as_str())
    }

    /// 文字列キーを受け取り、パースしてヘッダー値を取得します。
    /// 例: `b.header_by_str("TITLE")` や `b.header_by_str("wav01")`。
    pub fn header_by_str(&self, key: &str) -> Option<&str> {
        let hk = HeaderKey::from(key);
        self.header(&hk)
    }

    /// `WAVnn` のような WAV エントリを直接取得します。
    pub fn wav(&self, id: u8) -> Option<&str> {
        self.header(&HeaderKey::Wav(id))
    }
    /// `BMPnn` のエントリを取得します。
    pub fn bmp(&self, id: u8) -> Option<&str> {
        self.header(&HeaderKey::Bmp(id))
    }

    /// `#STAGEFILE` の値を取得します。
    pub fn stagefile(&self) -> Option<&str> {
        self.header(&HeaderKey::StageFile)
    }
}

#[derive(Debug, Clone)]
pub struct ChannelData {
    pub channel: u8,
    pub data: Vec<Option<String>>,
}

/// Parse a BMS file from text into a simple `Bms` structure.
///
/// This parser is intentionally small and aims to cover common patterns in
/// `test.bms`: header lines like `#TITLE ...` and data lines like
/// `#00111:00010000` where the first three digits are measure and the rest is
/// the channel number. Data payloads are parsed as two-character pairs; `00`
/// is treated as empty.
pub fn parse_bms(s: &str) -> Result<Bms, String> {
    let mut headers = HashMap::new();
    let mut measures: HashMap<u32, Vec<ChannelData>> = HashMap::new();

    for raw_line in s.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('*') {
            continue;
        }
        if !line.starts_with('#') {
            continue;
        }
        let after = &line[1..];
        if let Some(colon_idx) = after.find(':') {
            // Try to treat as a data line (#mmmcc:payload). If the left side
            // doesn't parse as measure+channel, fall back to header parsing
            // because some header values may contain ':' (e.g., "obj : LTFE").
            let left = after[..colon_idx].trim();
            let data = after[colon_idx + 1..].trim();

            let mut handled_as_data = false;
            if left.len() >= 4 {
                let measure_str = &left[..3];
                let channel_str = &left[3..];
                if let (Ok(measure), Ok(channel)) = (
                    measure_str.parse::<u32>(),
                    channel_str.parse::<u8>(),
                ) {
                    let mut chunks = Vec::new();
                    let mut idx = 0usize;
                    while idx + 2 <= data.len() {
                        let pair = &data[idx..idx + 2];
                        if pair != "00" {
                            chunks.push(Some(pair.to_string()));
                        } else {
                            chunks.push(None);
                        }
                        idx += 2;
                    }
                    let ch = ChannelData {
                        channel,
                        data: chunks,
                    };
                    measures.entry(measure).or_default().push(ch);
                    handled_as_data = true;
                }
            }

            if !handled_as_data {
                // Treat as a header line: parse key and value by whitespace.
                let mut parts = after.splitn(2, |c: char| c.is_whitespace());
                let key_raw = parts.next().unwrap_or("").trim();
                let key = parse_header_key(key_raw);
                let value = parts
                    .next()
                    .map(|v| v.trim().trim_matches('"').to_string())
                    .unwrap_or_default();
                headers.insert(key, value);
            }
        } else {
            // header line like: KEY value
            let mut parts = after.splitn(2, |c: char| c.is_whitespace());
            let key_raw = parts.next().unwrap_or("").trim();
            let key = parse_header_key(key_raw);
            let value = parts
                .next()
                .map(|v| v.trim().trim_matches('"').to_string())
                .unwrap_or_default();
            headers.insert(key, value);
        }
    }

    Ok(Bms { headers, measures })
}

fn parse_header_key(s: &str) -> HeaderKey {
    let up = s.trim().to_ascii_uppercase();
    match up.as_str() {
        "PLAYER" => HeaderKey::Player,
        "GENRE" => HeaderKey::Genre,
        "TITLE" => HeaderKey::Title,
        "ARTIST" => HeaderKey::Artist,
        "BPM" => HeaderKey::Bpm,
        "PLAYLEVEL" => HeaderKey::PlayLevel,
        "RANK" => HeaderKey::Rank,
        "SUBTITLE" => HeaderKey::Subtitle,
        "SUBARTIST" => HeaderKey::SubArtist,
        "DIFFICULTY" => HeaderKey::Difficulty,
        "TOTAL" => HeaderKey::Total,
        "COMMENT" => HeaderKey::Comment,
        "LNTYPE" => HeaderKey::LnType,
        "STAGEFILE" => HeaderKey::StageFile,
        _ => {
            // WAV/BMP は後続の識別子を16進数として扱う (例: WAV0A -> 0x0A)
            if up.starts_with("WAV") && up.len() > 3 {
                let num_str = &up[3..];
                if let Ok(n) = u8::from_str_radix(num_str, 16) {
                    HeaderKey::Wav(n)
                } else {
                    HeaderKey::Other(up)
                }
            } else if up.starts_with("BMP") && up.len() > 3 {
                let num_str = &up[3..];
                if let Ok(n) = u8::from_str_radix(num_str, 16) {
                    HeaderKey::Bmp(n)
                } else {
                    HeaderKey::Other(up)
                }
            } else {
                HeaderKey::Other(up)
            }
        }
    }
}
