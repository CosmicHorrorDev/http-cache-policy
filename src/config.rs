/// Indicates the mode the cache is operating in
///
/// This influences the impact of things like the `private` or `s-maxage` directives or the
/// [`http::header::AUTHORIZATION`] header impact storability.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Mode {
    /// A shared cache (default) e.g. for proxy or some other multi-user cache
    ///
    /// The `CachePolicy` will be evaluated from the perspective of a shared cache
    #[default]
    Shared,
    /// A private cache e.g. for a web browser
    ///
    /// The `CachePolicy` will be evaluated from the perspective of a shared cache.
    Private,
}

impl Mode {
    /// The default Mode [`Mode::Shared`]
    pub const fn default() -> Self {
        Self::Shared
    }

    /// If the mode is [`Mode::Shared`]
    pub fn is_shared(self) -> bool {
        self == Self::Shared
    }

    /// If the mode is [`Mode::Private`]
    pub fn is_private(self) -> bool {
        !self.is_shared()
    }
}

/// TODO
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Config {
    /// TODO
    pub mode: Mode,
    /// TODO
    pub cache_heuristic: f32,
    /// TODO
    pub ignore_cargo_cult: bool,
}

impl Config {
    /// The default cache config
    ///
    /// See the various fields' docs for more details.
    ///
    /// | field | value |
    /// | :---: | :--- |
    /// | [`mode`][Self::mode] | [`Mode::Shared`] |
    /// | [`cache_heuristic`][Self::cache_heuristic] | 10% of the time since last modified |
    /// | [`ignore_cargo_cult`][Self::ignore_cargo_cult] | [`false`] |
    pub const fn default() -> Self {
        Self {
            mode: Mode::default(),
            cache_heuristic: 0.1, // 10% matches IE
            ignore_cargo_cult: false,
        }
    }

    /// Set the mode that the cache operates in
    #[must_use]
    pub const fn mode(self, mode: Mode) -> Self {
        Self { mode, ..self }
    }

    /// Sets the cache's last modified freshness heuristic
    ///
    /// See [`cache_heuristic`][Self::cache_heuristic] for more details.
    #[must_use]
    pub const fn cache_heuristic(self, heuristic: f32) -> Self {
        Self {
            cache_heuristic: heuristic,
            ..self
        }
    }

    /// Ignores the effect of some ill-advised directive usage
    ///
    /// See [`ignore_cargo_cult`][Self::ignore_cargo_cult] for more details.
    #[must_use]
    pub const fn ignore_cargo_cult(self, ignore: bool) -> Self {
        Self {
            ignore_cargo_cult: ignore,
            ..self
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::default()
    }
}
