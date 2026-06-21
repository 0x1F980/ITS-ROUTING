//! AEH last-resort carrier — embed epoch cells φ ~ 𝒟_benign (no WIKI_STEGO in release).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::aeh_channel::AehChannel;

fn hex_encode(data: &[u8]) -> String {
    let mut s = String::with_capacity(data.len() * 2);
    for b in data {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    for i in (0..s.len()).step_by(2) {
        let hi = bytes[i] as char;
        let lo = bytes[i + 1] as char;
        let v = u8::from_str_radix(&format!("{hi}{lo}"), 16).ok()?;
        out.push(v);
    }
    Some(out)
}

/// Embed fixed-size epoch cells into benign-looking public observations.
pub(crate) trait AehCarrier {
    fn embed(&self, epoch: u64, cell: &[u8], channel: AehChannel) -> io::Result<String>;
    fn publish(&self, epoch: u64, cell: &[u8], channel: AehChannel) -> io::Result<()>;
    /// L3': harvest all observations (constant-rate; no selective poll).
    fn harvest_all(&self) -> io::Result<Vec<(u64, String, AehChannel)>>;
}

/// File-backed benign E-channel (DNS TXT / NASA telemetry / domestic news formats).
pub(crate) struct FileAehCarrier {
    dir: PathBuf,
}

impl FileAehCarrier {
    pub(crate) fn new(dir: impl AsRef<Path>) -> Self {
        FileAehCarrier {
            dir: dir.as_ref().to_path_buf(),
        }
    }

    fn obs_path(&self, epoch: u64, channel: AehChannel) -> PathBuf {
        self.dir
            .join(format!("obs_{:08}_{}.txt", epoch, channel.file_suffix()))
    }
}

impl AehCarrier for FileAehCarrier {
    fn embed(&self, epoch: u64, cell: &[u8], channel: AehChannel) -> io::Result<String> {
        Ok(channel.embed_cell(epoch, cell))
    }

    fn publish(&self, epoch: u64, cell: &[u8], channel: AehChannel) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        let text = self.embed(epoch, cell, channel)?;
        fs::write(self.obs_path(epoch, channel), text)
    }

    fn harvest_all(&self) -> io::Result<Vec<(u64, String, AehChannel)>> {
        if !self.dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for entry in fs::read_dir(&self.dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("obs_") || !name.ends_with(".txt") {
                continue;
            }
            let mid = &name[4..name.len() - 4];
            let (epoch_str, suffix) = match mid.split_once('_') {
                Some(v) => v,
                None => continue,
            };
            let epoch: u64 = match epoch_str.parse() {
                Ok(v) => v,
                Err(_) => continue,
            };
            let channel = match AehChannel::from_suffix(suffix) {
                Some(c) => c,
                None => continue,
            };
            let text = fs::read_to_string(entry.path())?;
            out.push((epoch, text, channel));
        }
        out.sort_by_key(|(e, _, _)| *e);
        Ok(out)
    }
}

impl AehChannel {
    pub(crate) fn file_suffix(&self) -> &'static str {
        match self {
            AehChannel::DnsTxt => "dns",
            AehChannel::NasaTelemetry => "nasa",
            AehChannel::DomesticNews => "news",
            AehChannel::SneakernetFile => "sneak",
            #[cfg(feature = "dev-onion-mix")]
            AehChannel::Wikipedia => "wiki",
            #[cfg(feature = "dev-onion-mix")]
            AehChannel::GitHubGists => "gist",
            #[cfg(feature = "dev-onion-mix")]
            AehChannel::Reddit => "reddit",
            #[cfg(not(feature = "dev-onion-mix"))]
            AehChannel::Wikipedia => "dns",
            #[cfg(not(feature = "dev-onion-mix"))]
            AehChannel::GitHubGists => "dns",
            #[cfg(not(feature = "dev-onion-mix"))]
            AehChannel::Reddit => "dns",
        }
    }

    pub(crate) fn from_suffix(s: &str) -> Option<Self> {
        match s {
            "dns" => Some(AehChannel::DnsTxt),
            "nasa" => Some(AehChannel::NasaTelemetry),
            "news" => Some(AehChannel::DomesticNews),
            "sneak" => Some(AehChannel::SneakernetFile),
            #[cfg(feature = "dev-onion-mix")]
            "wiki" => Some(AehChannel::Wikipedia),
            #[cfg(feature = "dev-onion-mix")]
            "gist" => Some(AehChannel::GitHubGists),
            #[cfg(feature = "dev-onion-mix")]
            "reddit" => Some(AehChannel::Reddit),
            _ => None,
        }
    }

    /// φ(cell) ~ 𝒟_benign — production formats (no WIKI_STEGO prefix).
    pub(crate) fn embed_cell(&self, epoch: u64, cell: &[u8]) -> String {
        let hx = hex_encode(cell);
        match self {
            AehChannel::DnsTxt => format!(
                "v=spf1 ip4:192.168.1.1 include:_spf.google.com mx-id={epoch} data={hx} ~all"
            ),
            AehChannel::NasaTelemetry => format!(
                "SENS_ID={epoch};GEOM_X={hx};NOISE_FILTER=0.02;SYS_STATUS=OK;SOURCE=USGS"
            ),
            AehChannel::DomesticNews => format!(
                "Municipal infrastructure bulletin #{epoch}: scheduled maintenance completed. \
                 Telemetry digest [{hx}] archived per directive 12/2024."
            ),
            AehChannel::SneakernetFile => format!(
                "OFFLINE_ARCHIVE;EPOCH={epoch};PAYLOAD_HEX={hx};CHECK=OK"
            ),
            #[cfg(feature = "dev-onion-mix")]
            AehChannel::Wikipedia | AehChannel::GitHubGists | AehChannel::Reddit => {
                self.stego_encode_dev(epoch, &hx)
            }
            #[cfg(not(feature = "dev-onion-mix"))]
            AehChannel::Wikipedia | AehChannel::GitHubGists | AehChannel::Reddit => format!(
                "v=spf1 ip4:192.168.1.1 include:_spf.google.com mx-id={epoch} data={hx} ~all"
            ),
        }
    }

    /// Extract cell bytes from a benign observation.
    pub(crate) fn extract_cell(&self, text: &str) -> Option<Vec<u8>> {
        match self {
            AehChannel::DnsTxt => {
                let data = text.split("data=").nth(1)?.split(' ').next()?.trim();
                hex_decode(data)
            }
            AehChannel::NasaTelemetry => {
                let data = text.split("GEOM_X=").nth(1)?.split(';').next()?.trim();
                hex_decode(data)
            }
            AehChannel::DomesticNews => {
                let inner = text.split('[').nth(1)?.split(']').next()?.trim();
                hex_decode(inner)
            }
            AehChannel::SneakernetFile => {
                let data = text.split("PAYLOAD_HEX=").nth(1)?.split(';').next()?.trim();
                hex_decode(data)
            }
            #[cfg(feature = "dev-onion-mix")]
            AehChannel::Wikipedia | AehChannel::GitHubGists | AehChannel::Reddit => {
                self.stego_decode_dev(text)
            }
            #[cfg(not(feature = "dev-onion-mix"))]
            AehChannel::Wikipedia | AehChannel::GitHubGists | AehChannel::Reddit => {
                let data = text.split("data=").nth(1)?.split(' ').next()?.trim();
                hex_decode(data)
            }
        }
    }

    #[cfg(feature = "dev-onion-mix")]
    fn stego_encode_dev(&self, epoch: u64, hx: &str) -> String {
        match self {
            AehChannel::Wikipedia => format!("WIKI_STEGO:enwiki;id={epoch};points={hx};tags=dev"),
            AehChannel::GitHubGists => format!("GIST_STEGO:id={epoch};payload={hx}"),
            AehChannel::Reddit => format!("REDDIT_STEGO:id={epoch};payload={hx}"),
            _ => format!("DEV_STEGO:{epoch}:{hx}"),
        }
    }

    #[cfg(feature = "dev-onion-mix")]
    fn stego_decode_dev(&self, text: &str) -> Option<Vec<u8>> {
        let hx = if let Some(p) = text.split("points=").nth(1) {
            p.split(';').next()?.trim()
        } else if let Some(p) = text.split("payload=").nth(1) {
            p.split(';').next()?.trim()
        } else {
            return None;
        };
        hex_decode(hx)
    }
}

/// Production AEH channel rotation (benign formats only).
pub(crate) fn production_channels() -> [AehChannel; 4] {
    [
        AehChannel::DnsTxt,
        AehChannel::NasaTelemetry,
        AehChannel::DomesticNews,
        AehChannel::SneakernetFile,
    ]
}
