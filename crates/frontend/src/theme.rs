//! Theme system — Platform-aware design tokens and CSS generation.
//!
//! Design philosophy:
//! - Web: High-contrast, branded, landing-page aesthetic
//! - Desktop: Native feel, system fonts, subtle borders, compact
//! - Mobile: Touch-optimized, larger tap targets, bottom navigation feel
//!
//! Color palette (Pickando brand):
//! - Primary: #10B981 (Emerald green — direction, movement)
//! - Secondary: #2563EB (Blue — trust, technology)
//! - Accent: #F59E0B (Amber — energy, attention)
//! - Dark: #0F172A (Slate 900)
//! - Light: #F8FAFC (Slate 50)

/// Detected platform for UI adaptation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Platform {
    Web,
    Desktop,
    Mobile,
}

impl Platform {
    /// Detect the current platform at runtime.
    pub fn detect() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            // Check if mobile via user agent
            if let Some(window) = web_sys::window() {
                if let Ok(ua) = window.navigator().user_agent() {
                    if ua.contains("Android") || ua.contains("iPhone") {
                        return Self::Mobile;
                    }
                }
            }
            Self::Web
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::Desktop
        }
    }
}

/// Generate the complete CSS stylesheet adapted for the current platform.
pub fn global_css(platform: &Platform) -> String {
    let base = include_str!("../assets/style.css");

    let platform_overrides = match platform {
        Platform::Web => WEB_OVERRIDES,
        Platform::Desktop => DESKTOP_OVERRIDES,
        Platform::Mobile => MOBILE_OVERRIDES,
    };

    format!("{base}\n{platform_overrides}")
}

const WEB_OVERRIDES: &str = r#"
/* Web: Full landing-page experience with generous spacing */
.nav-bar { padding: 16px 32px; }
.main-content { max-width: 1200px; margin: 0 auto; padding: 32px; }
.hero { min-height: 80vh; }
"#;

const DESKTOP_OVERRIDES: &str = r#"
/* Desktop: Native feel — compact, system fonts, window chrome awareness */
.nav-bar {
    padding: 8px 16px;
    -webkit-app-region: drag;
}
.nav-links { -webkit-app-region: no-drag; }
.main-content { max-width: 900px; margin: 0 auto; padding: 16px; }
.hero { min-height: 60vh; }
body { font-size: 14px; }
.card { border-radius: 8px; }
"#;

const MOBILE_OVERRIDES: &str = r#"
/* Mobile: Touch-optimized — larger targets, bottom-friendly */
.nav-bar { padding: 12px 16px; }
.nav-link { padding: 12px 16px; font-size: 15px; }
.main-content { padding: 16px; }
.hero { min-height: 70vh; }
.card { border-radius: 16px; }
button { min-height: 48px; }
input { min-height: 48px; font-size: 16px; }
"#;
