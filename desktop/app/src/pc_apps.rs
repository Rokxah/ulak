// SPDX-License-Identifier: GPL-3.0-or-later
//! Phone apps that also have a desktop app on this PC, so a notification
//! doesn't pop up twice: once from the phone and once from the PC's own
//! WhatsApp, Telegram, Slack... (the [`AppRule::WhenPcClosed`] rule).
//!
//! [`AppRule::WhenPcClosed`]: crate::state::AppRule::WhenPcClosed

use std::{
    collections::HashSet,
    sync::Mutex,
    time::{Duration, Instant},
};

/// Android package → the desktop programs (lowercase executable names) that
/// show the same notifications on the PC.
const KNOWN: &[(&str, &[&str])] = &[
    ("com.whatsapp", &["whatsapp.exe", "whatsapp.root.exe", "whatsapp"]),
    ("com.whatsapp.w4b", &["whatsapp.exe", "whatsapp.root.exe", "whatsapp"]),
    ("org.telegram.messenger", &["telegram.exe", "telegram", "telegram-desktop"]),
    ("org.telegram.messenger.web", &["telegram.exe", "telegram", "telegram-desktop"]),
    ("org.thoughtcrime.securesms", &["signal.exe", "signal-desktop"]),
    ("com.discord", &["discord.exe", "discord"]),
    ("com.Slack", &["slack.exe", "slack"]),
    ("com.microsoft.teams", &["ms-teams.exe", "teams.exe", "teams-for-linux"]),
    ("com.microsoft.office.outlook", &["olk.exe", "outlook.exe"]),
    ("com.facebook.orca", &["messenger.exe"]),
    ("com.skype.raider", &["skype.exe", "skypeforlinux"]),
    ("com.viber.voip", &["viber.exe", "viber"]),
    ("com.google.android.gm", &["thunderbird.exe", "thunderbird"]),
    ("com.spotify.music", &["spotify.exe", "spotify"]),
    ("us.zoom.videomeetings", &["zoom.exe", "zoom"]),
    ("com.tencent.mm", &["weixin.exe", "wechat.exe"]),
    ("jp.naver.line.android", &["line.exe"]),
    ("im.vector.app", &["element.exe", "element-desktop"]),
    ("com.mattermost.rn", &["mattermost.exe", "mattermost-desktop"]),
];

/// How long a process list is reused: notifications often come in bursts.
const CACHE_FOR: Duration = Duration::from_secs(3);

static CACHE: Mutex<Option<(Instant, HashSet<String>)>> = Mutex::new(None);

/// The desktop programs that stand for `app` on the PC, if it's a known one.
pub fn desktop_programs(app: &str) -> &'static [&'static str] {
    KNOWN.iter().find(|(package, _)| *package == app).map_or(&[], |(_, programs)| programs)
}

/// Whether `app` has a known desktop counterpart (shown as a hint in Settings).
pub fn has_desktop_app(app: &str) -> bool {
    !desktop_programs(app).is_empty()
}

/// Whether the desktop app for `app` is running on this PC right now.
pub fn desktop_app_running(app: &str) -> bool {
    let programs = desktop_programs(app);
    if programs.is_empty() {
        return false;
    }
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let fresh = cache.as_ref().is_some_and(|(at, _)| at.elapsed() < CACHE_FOR);
    if !fresh {
        *cache = Some((Instant::now(), running_programs()));
    }
    let running = &cache.as_ref().expect("filled above").1;
    programs.iter().any(|p| running.contains(*p))
}

#[cfg(windows)]
fn running_programs() -> HashSet<String> {
    crate::win::processes::running_programs()
}

#[cfg(target_os = "linux")]
fn running_programs() -> HashSet<String> {
    let Ok(dir) = std::fs::read_dir("/proc") else { return HashSet::new() };
    dir.flatten()
        .filter(|e| e.file_name().to_string_lossy().bytes().all(|b| b.is_ascii_digit()))
        .filter_map(|e| std::fs::read_to_string(e.path().join("comm")).ok())
        .map(|name| name.trim().to_lowercase())
        .collect()
}

#[cfg(not(any(windows, target_os = "linux")))]
fn running_programs() -> HashSet<String> {
    HashSet::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knows_common_chat_apps_and_not_others() {
        assert!(has_desktop_app("com.whatsapp"));
        assert!(desktop_programs("org.telegram.messenger").contains(&"telegram.exe"));
        assert!(!has_desktop_app("com.example.unknown"));
        assert!(!desktop_app_running("com.example.unknown"));
    }

    #[test]
    fn package_names_are_unique() {
        let mut seen = HashSet::new();
        assert!(KNOWN.iter().all(|(package, _)| seen.insert(*package)));
    }
}
