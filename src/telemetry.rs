//! Modified by itay7891 (2026) under Apache License 2.0 section 4(b).
//! Privacy hardening: usage reporting is compiled out. See HARDENING.md.
//!
//! Call sites still use these functions. They do not collect counts, write a
//! report, start a thread, or contact a server. There is no telemetry endpoint
//! in this build, including no `HERDR_TELEMETRY_URL` override.

use serde_json::{Value, json};

/// What the window used to describe. Kept so call sites compile; nothing reads it.
#[derive(Default)]
#[allow(dead_code)]
pub struct Observation {
    pub foreground: bool,
    pub agents: Vec<Option<String>>,
    pub agents_working: usize,
    pub spaces: usize,
    pub tabs: usize,
    pub terminals: usize,
    pub layout: String,
    pub mods: Vec<String>,
    pub theme: String,
    pub welcome: Option<String>,
}

pub fn collecting() -> bool {
    false
}

pub fn observe(_: Observation) {}

pub fn command(_: &'static str) {}

pub fn count(_: &'static str) {}

pub fn window_open() {}

pub fn set_setting(_: bool) {}

/// What the settings screen and `telemetry()` in mods read.
pub fn status() -> Value {
    json!({
        "enabled": false,
        "setting": false,
        "off_because": "compiled out",
        "sends": false,
        "endpoint": Value::Null,
        "draft": {},
        "queued": [],
        "last_sent": Value::Null,
    })
}

pub fn quit() {}

pub fn clock() {}

pub fn start(_: bool) {}

/// A release version (`0.2.0`, `0.2.0-rc.1`), else `other`: never a free string.
pub(crate) fn version(v: &str) -> String {
    let v = v.trim().trim_start_matches("hc ").trim_start_matches('v');
    let number = |s: &str| !s.is_empty() && s.len() <= 4 && s.bytes().all(|b| b.is_ascii_digit());
    let (core, pre) = v.split_once('-').map_or((v, None), |(c, p)| (c, Some(p)));
    let parts: Vec<&str> = core.split('.').collect();
    let pre_ok = pre.is_none_or(|p| {
        let (label, n) = p.split_once('.').map_or((p, None), |(l, n)| (l, Some(n)));
        matches!(label, "alpha" | "beta" | "rc" | "preview") && n.is_none_or(number)
    });
    if parts.len() == 3 && parts.iter().all(|p| number(p)) && pre_ok {
        v.to_owned()
    } else {
        "other".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use serde_json::json;
    use std::collections::{BTreeMap, BTreeSet};

    const VERSION: u32 = 1;

    #[derive(Serialize, Default, Clone)]
    #[serde(default)]
    struct Day {
        day: String,
        open_secs: u64,
        foreground_secs: u64,
        startups: u32,
        startup_ms: Vec<u64>,
        unclean_exits: u32,
        agents: BTreeSet<String>,
        agents_open: usize,
        agents_working: usize,
        spaces: usize,
        tabs: usize,
        terminals: usize,
        layouts: BTreeSet<String>,
        mods: BTreeSet<String>,
        commands: BTreeSet<String>,
        theme_custom: bool,
        core: Option<String>,
        welcome: Option<String>,
        events: BTreeMap<String, u32>,
    }

    impl Day {
        fn new(day: String) -> Day {
            Day {
                day,
                ..Day::default()
            }
        }

        fn observe(&mut self, o: Observation, layouts: &[&str]) {
            for a in &o.agents {
                self.agents
                    .insert(crate::programs::agent_name(a.as_deref()));
            }
            self.agents_open = self.agents_open.max(o.agents.len());
            self.agents_working = self.agents_working.max(o.agents_working);
            self.spaces = self.spaces.max(o.spaces);
            self.tabs = self.tabs.max(o.tabs);
            self.terminals = self.terminals.max(o.terminals);
            self.layouts
                .insert(if layouts.contains(&o.layout.as_str()) {
                    o.layout
                } else {
                    "custom".into()
                });
            self.mods
                .extend(o.mods.into_iter().filter(|m| !m.starts_with("frame-")));
            if let Some(w) = o.welcome.filter(|w| w == "finished" || w == "skipped") {
                self.welcome = Some(w);
            }
            self.theme_custom |= !crate::settings::PRESETS.contains(&o.theme.as_str())
                && !crate::settings::SHIPPED_THEMES.contains(&o.theme.as_str());
        }

        fn report(&self, install: &str) -> Value {
            let (builtin, edited, custom) = crate::mods::kinds(&self.mods);
            let mut ms = self.startup_ms.clone();
            ms.sort_unstable();
            json!({
                "v": VERSION,
                "install": install,
                "day": self.day,
                "app": {
                    "version": version(env!("CARGO_PKG_VERSION")),
                    "os": std::env::consts::OS,
                    "arch": std::env::consts::ARCH,
                    "channel": "stable",
                },
                "core": { "version": self.core.as_deref().map(version) },
                "usage": {
                    "open": time_bucket(self.open_secs),
                    "foreground": time_bucket(self.foreground_secs),
                    "startups": self.startups.min(99),
                    "startup": ms.get(ms.len() / 2).map(|m| startup_bucket(*m)),
                    "unclean_exits": self.unclean_exits.min(99),
                },
                "agents": self.agents,
                "agents_open": bucket(self.agents_open),
                "agents_working": bucket(self.agents_working),
                "spaces": bucket(self.spaces),
                "tabs": bucket(self.tabs),
                "terminals": bucket(self.terminals),
                "layouts": self.layouts,
                "mods": { "builtin": builtin, "builtin_edited": edited, "custom": custom },
                "commands": self.commands,
                "custom": { "keys": keys_custom(), "theme": self.theme_custom },
                "welcome": self.welcome.as_deref().unwrap_or("none"),
                "events": self.events,
            })
        }
    }

    fn bucket(n: usize) -> &'static str {
        match n {
            0 => "0",
            1 => "1",
            2..=5 => "2-5",
            6..=20 => "6-20",
            _ => "21+",
        }
    }

    fn time_bucket(secs: u64) -> &'static str {
        match secs / 60 {
            0 => "0",
            1..=14 => "<15m",
            15..=59 => "15m-1h",
            60..=239 => "1-4h",
            240..=479 => "4-8h",
            _ => "8h+",
        }
    }

    fn startup_bucket(ms: u64) -> &'static str {
        match ms {
            0..=499 => "<0.5s",
            500..=999 => "0.5-1s",
            1000..=1999 => "1-2s",
            2000..=4999 => "2-5s",
            _ => "5s+",
        }
    }

    fn keys_custom() -> bool {
        let parse = |s: &str| serde_json::from_str::<Value>(s).ok();
        match std::fs::read_to_string(crate::keys::path()) {
            Ok(text) => parse(&text) != parse(crate::keys::DEFAULTS),
            Err(_) => false,
        }
    }

    fn date(days: i64) -> String {
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z.rem_euclid(146_097);
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = yoe + era * 400 + i64::from(m <= 2);
        format!("{y:04}-{m:02}-{d:02}")
    }

    #[test]
    fn reporting_is_compiled_out() {
        assert!(!collecting());
        let status = status();
        assert_eq!(status["enabled"], false);
        assert_eq!(status["sends"], false);
        assert!(status["endpoint"].is_null());
        start(true);
        assert!(!collecting());
    }

    #[test]
    fn dates_are_utc_calendar_days() {
        assert_eq!(date(0), "1970-01-01");
        assert_eq!(date(19_723), "2024-01-01");
        assert_eq!(date(19_782), "2024-02-29");
        assert_eq!(date(20_745), "2026-10-19");
    }

    #[test]
    fn versions_are_releases_or_other() {
        assert_eq!(version("0.2.0"), "0.2.0");
        assert_eq!(version("hc 0.1.5"), "0.1.5");
        assert_eq!(version("0.2.0-rc.1"), "0.2.0-rc.1");
        assert_eq!(version("0.2.0-my/secret path"), "other");
        assert_eq!(version("hc 0.1.5-acmeclient"), "other");
        assert_eq!(version("0.1.5-rc.x"), "other");
        assert_eq!(version("dev"), "other");
    }

    #[test]
    fn a_report_holds_no_free_text() {
        let mut day = Day::new("2026-10-19".into());
        let o = Observation {
            agents: vec![
                Some("claude-code".into()),
                Some("/home/me/secret-tool".into()),
                None,
            ],
            layout: "my-private-layout".into(),
            mods: vec![
                "agents".into(),
                "my-secret-mod".into(),
                "frame-settings".into(),
            ],
            theme: "my-theme".into(),
            ..Observation::default()
        };
        day.observe(o, &["main", "terminals"]);
        day.core = Some("hc /home/me/build".into());
        let text = day.report("0123456789abcdef0123456789abcdef").to_string();
        for leak in ["secret", "private", "/home", "my-theme", "frame-"] {
            assert!(!text.contains(leak), "{leak} leaked: {text}");
        }
        assert!(
            text.contains("\"claude\""),
            "known agents are named: {text}"
        );
        assert!(text.contains("\"other\""), "{text}");
        assert!(text.contains("\"custom\""), "{text}");
    }
}
