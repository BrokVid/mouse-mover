use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const APP_DIR: &str = "MouseMover";
const FILE_NAME: &str = "config.json";

const INTERVAL_MIN: u32 = 1;
const INTERVAL_MAX: u32 = 300;
const JITTER_MIN: u32 = 1;
const JITTER_MAX: u32 = 90;

fn default_jitter() -> u32 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub enabled: bool,
    pub zen: bool,
    pub interval_secs: u32,
    pub randomize: bool,
    #[serde(default = "default_jitter")]
    pub jitter_percent: u32,
    pub minimize_to_tray: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            zen: true,
            interval_secs: 30,
            randomize: true,
            jitter_percent: 10,
            minimize_to_tray: false,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = config_path();
        let Ok(raw) = fs::read_to_string(&path) else {
            let cfg = Self::default();
            cfg.save();
            return cfg;
        };
        match serde_json::from_str::<Config>(&raw) {
            Ok(mut cfg) => {
                cfg.sanitize();
                cfg
            }
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) {
        let path = config_path();
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }

    pub fn sanitize(&mut self) {
        self.interval_secs = self.interval_secs.clamp(INTERVAL_MIN, INTERVAL_MAX);
        self.jitter_percent = self.jitter_percent.clamp(JITTER_MIN, JITTER_MAX);
    }

    pub fn next_delay_ms(&self, rng: &mut u32) -> u32 {
        let base_ms = self
            .interval_secs
            .clamp(INTERVAL_MIN, INTERVAL_MAX)
            .saturating_mul(1000);
        if !self.randomize {
            return base_ms;
        }
        let pct = self.jitter_percent.clamp(JITTER_MIN, JITTER_MAX);
        // Равномерно в диапазоне [-pct; +pct] процентов.
        let span = pct * 2 + 1;
        let signed = (xorshift(rng) % span) as i32 - pct as i32;
        let ms = (base_ms as i64) * (100 + signed as i64) / 100;
        ms.clamp(200, 3_600_000) as u32
    }
}

pub fn config_path() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join(APP_DIR).join(FILE_NAME)
}

pub fn seed_rng() -> u32 {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| (d.as_nanos() as u32) ^ (d.subsec_nanos()))
        .unwrap_or(0xA5A5_A5A5);
    t | 1
}

pub fn xorshift(state: &mut u32) -> u32 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *state = x | 1;
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_clamps_interval() {
        let mut cfg = Config {
            interval_secs: 9999,
            ..Default::default()
        };
        cfg.sanitize();
        assert_eq!(cfg.interval_secs, INTERVAL_MAX);
        cfg.interval_secs = 0;
        cfg.sanitize();
        assert_eq!(cfg.interval_secs, INTERVAL_MIN);
    }

    #[test]
    fn randomize_stays_in_range() {
        let cfg = Config {
            interval_secs: 30,
            randomize: true,
            jitter_percent: 10,
            ..Default::default()
        };
        let mut rng = 0xC0FFEE01;
        for _ in 0..400 {
            let ms = cfg.next_delay_ms(&mut rng);
            assert!((27_000..=33_000).contains(&ms), "ms={ms}");
        }
    }

    #[test]
    fn jitter_20_covers_full_plus_minus_range() {
        let cfg = Config {
            interval_secs: 30,
            randomize: true,
            jitter_percent: 20,
            ..Default::default()
        };
        let mut rng = 0xC0FFEE01;
        let mut min_ms = u32::MAX;
        let mut max_ms = 0u32;
        for _ in 0..2000 {
            let ms = cfg.next_delay_ms(&mut rng);
            assert!((24_000..=36_000).contains(&ms), "ms={ms}");
            min_ms = min_ms.min(ms);
            max_ms = max_ms.max(ms);
        }
        assert_eq!(min_ms, 24_000);
        assert_eq!(max_ms, 36_000);
    }

    #[test]
    fn jitter_40_is_plus_minus_forty() {
        let cfg = Config {
            interval_secs: 30,
            randomize: true,
            jitter_percent: 40,
            ..Default::default()
        };
        let mut rng = 0xC0FFEE01;
        let mut min_ms = u32::MAX;
        let mut max_ms = 0u32;
        for _ in 0..4000 {
            let ms = cfg.next_delay_ms(&mut rng);
            assert!((18_000..=42_000).contains(&ms), "ms={ms}");
            min_ms = min_ms.min(ms);
            max_ms = max_ms.max(ms);
        }
        assert_eq!(min_ms, 18_000);
        assert_eq!(max_ms, 42_000);
    }

    #[test]
    fn fixed_interval_is_exact() {
        let cfg = Config {
            interval_secs: 20,
            randomize: false,
            ..Default::default()
        };
        let mut rng = 1;
        assert_eq!(cfg.next_delay_ms(&mut rng), 20_000);
    }

    #[test]
    fn old_json_gets_default_jitter() {
        let back: Config = serde_json::from_str(
            r#"{"enabled":true,"zen":true,"interval_secs":30,"randomize":true,"minimize_to_tray":false}"#,
        )
        .unwrap();
        assert_eq!(back.jitter_percent, 10);
    }

    #[test]
    fn json_roundtrip() {
        let cfg = Config {
            enabled: false,
            zen: false,
            interval_secs: 45,
            randomize: true,
            jitter_percent: 15,
            minimize_to_tray: true,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: Config = serde_json::from_str(&json).unwrap();
        assert!(!back.enabled);
        assert!(back.minimize_to_tray);
        assert_eq!(back.interval_secs, 45);
        assert_eq!(back.jitter_percent, 15);
    }
}
