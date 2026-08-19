# tuigreet Changelog

## 0.13.0

### Added

- Configuration is now loaded via `clap` (derive) and `figment`, replacing the
  previous `getopts`-based parser. Config is resolved in layered priority:
  clap defaults < system TOML < user TOML < environment variables < CLI args.
  Higher layers override lower layers, and explicitly-set values at any layer
  take precedence over defaults from lower layers.
- `--system-config` and `--config` CLI options to specify system and user TOML
  configuration file paths (default: `/etc/tuigreet/config.toml` and
  `~/.config/tuigreet/config.toml`).
- Individual `--theme-border`, `--theme-text`, `--theme-time`,
  `--theme-container`, `--theme-title`, `--theme-greet`, `--theme-prompt`,
  `--theme-input`, `--theme-action`, and `--theme-button` CLI options and
  `[theme]` TOML section for granular theme control.
- `--secret-mode` (`hidden`/`characters`) and `--secret-characters` CLI options
  and `[secret]` TOML section for controlling secret input display.
- `--asterisks` and `--asterisks-char` CLI options and `[display] asterisks` /
  `asterisks_char` TOML keys re-added as deprecated aliases for `--secret-mode`
  and `--secret-characters`. A deprecation warning is emitted when they are
  used. `secret.*` takes priority when both are set.
- `--power-use-setsid` CLI option and `[power] use_setsid` TOML key (default:
  true) for controlling `setsid` prefix on power commands.
- Individual `--doom-top-color`, `--doom-middle-color`, `--doom-bottom-color`,
  `--doom-height`, `--doom-spread`, `--matrix-head-color`,
  `--matrix-bright-color`, `--matrix-dim-color`, `--matrix-min-length`,
  `--matrix-max-length`, `--matrix-min-speed`, `--matrix-max-speed`, and
  `--matrix-mutate-chance` CLI options.
- Generated man page (`tuigreet-1`) via `clap_mangen`.
- added toml option for dumping config (only works if toml loaded properly,
  prefer cli option). Also available as `general.dump_config` in TOML and
  `TUIGREET_GENERAL__DUMP_CONFIG` via environment.
- Configuration validation now produces non-fatal warnings alongside errors for
  potentially problematic settings (e.g., all outputs disabled, excessive
  padding, hidden widgets with enabled features).
- Hot-reload watcher now watches both system and user config files.

### Changed

- **Breaking**: Configuration system replaced `getopts` with `clap` (derive) and
  `figment`. The old `schema.rs`, `parser.rs`, and `env.rs` modules have been
  removed and replaced by `config.rs`, `loader.rs`, `validation.rs`, and
  `error.rs`.
- **Breaking**: `--no-xsession-wrapper` CLI flag removed. Use
  `--xsession-wrapper ""` (empty string) to disable X11 session wrapping.
- **Breaking**: `--theme "spec"` CLI flag removed. Use individual
  `--theme-border`, `--theme-text`, etc. flags or the `[theme]` TOML section.
- **Breaking**: `--doom-colors TOP,MIDDLE,BOTTOM` removed. Use
  `--doom-top-color`, `--doom-middle-color`, `--doom-bottom-color` instead.
- **Breaking**: `--matrix-colors HEAD,BRIGHT,DIM` removed. Use
  `--matrix-head-color`, `--matrix-bright-color`, `--matrix-dim-color` instead.
- **Breaking**: `--matrix-length MIN,MAX` removed. Use `--matrix-min-length`
  and `--matrix-max-length` instead.
- **Breaking**: `--matrix-speed MIN,MAX` removed. Use `--matrix-min-speed` and
  `--matrix-max-speed` instead.
- **Breaking**: `--power-no-setsid` replaced by `--power-use-setsid` (default:
  true). The flag semantics are inverted.
- **Breaking**: `--sessions` now takes a single directory per flag repetition
  (e.g., `-s /one -s /two`) instead of a colon-separated list. Similarly,
  `--xsessions` and `--env` now use repeated flags instead of delimited
  strings.
- **Breaking**: Configuration environment variables now use double underscore
  (`__`) as the section/field separator (e.g., `TUIGREET_DISPLAY__SHOW_TIME`).
  Single underscores in field names like `log_file` are preserved correctly.
- **Breaking**: `container_padding` value is now used directly (no internal +1).
  The default changed from 1 to 2 to preserve the same effective padding.
- `Greeter` struct now holds a `Config` struct directly with `Deref`/`DerefMut`
  accessors, replacing the previous `Option<Matches>` + separate field storage
  pattern.
- Theme module consolidated: `tuigreet-theme` now imports `ThemeConfig` from
  `tuigreet-config` and provides `Theme::from_config()` directly.
- UID range for user menu defaults to 1000-60000. Reading `/etc/login.defs` for
  `UID_MIN`/`UID_MAX` has been removed.

### Removed

- `getopts` dependency for CLI argument parsing.
- `tuigreet-config::schema`, `tuigreet-config::parser`, and
  `tuigreet-config::env` modules.

### Added

- Suspend and hibernate actions are now available from the power menu. They use
  `loginctl suspend` and `loginctl hibernate` by default.
- `[power] suspend` and `[power] hibernate` configuration keys, plus the
  `--power-suspend` and `--power-hibernate` command-line options, allow
  overriding those actions.
- When user-menu filtering leaves exactly one eligible user, tuigreet now
  selects that user and begins session creation automatically.

### Changed

- `display.asterisks` is deprecated. Use `[secret] mode = "characters"` or
  `mode = "hidden"` instead. Existing configurations remain supported and emit a
  validation warning; `secret.mode` takes precedence when both are configured.
  The deprecated option **will be removed in a future release**.
- Configuration reload now uses the same configuration sources and command line
  overrides as initial startup.
- Higher-priority configuration files can now explicitly restore a setting to
  its default value, such as `show_time = false` or `width = 80`.

### Fixed

- **Breaking**: `sessions_dirs` and `xsessions_dirs` no longer default to
  hardcoded paths. Both now default to empty, causing tuigreet to fall back to
  `$XDG_DATA_DIRS` (defaulting to `/usr/local/share:/usr/share`) for session
  discovery. This restores the behavior of pre-0.11.0 tuigreet on systems like
  NixOS where sessions are not under `/usr/share/`.
- Reloading configuration now replaces removed session commands, session
  directories, wrappers, and user-menu state instead of retaining stale values.
- Reloading session directories now refreshes the session-selection menu while
  retaining the selected session when it is still available.
- Explicit `secret.mode = "hidden"` and the legacy `display.asterisks = false`
  setting now correctly override lower-priority configuration layers.
- `[general] debug` and `log_file` now configure logging during startup.
- Constrained terminal sizes and narrow layouts no longer trigger unsigned
  arithmetic overflows while rendering.
- Corrected the documented names of several configuration environment variables.
- The top infobar now respects `layout.window_padding`.
- Background animations no longer bleed into the login form.
