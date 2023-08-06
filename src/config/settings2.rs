use crate::config::MissingRuntimeBehavior;
use log::LevelFilter;
use std::collections::HashMap;
use std::env;

#[derive(Debug)]
pub struct Settings2 {
    pub t: bool,
    pub log_level: LevelFilter,
    pub log_file_level: LevelFilter,
    pub missing_runtime_behavior: MissingRuntimeBehavior,
    pub raw: bool,
    pub jobs: usize,
    pub verbose: bool,
    pub experimental: bool,
    vars: HashMap<SettingsKey, Value>,
}

impl Settings2 {
    pub fn load_env(&mut self, env: HashMap<String, String>) {
        if let Some(log_level) = env.get("RTX_LOG_LEVEL") {
            self.set_str(SettingsKey::LogLevel, log_level);
        }
        if let Some(log_file_level) = env.get("RTX_LOG_FILE_LEVEL") {
            self.set_str(SettingsKey::LogFileLevel, log_file_level);
        }
        if let Some(raw) = env.get("RTX_RAW") {
            self.set_bool(SettingsKey::Raw, val_is_true(raw));
        }
        if let Some(jobs) = env.get("RTX_JOBS") {
            self.set_int(SettingsKey::Jobs, jobs.parse().unwrap());
        }
        if let Some(verbose) = env.get("RTX_VERBOSE") {
            self.set_bool(SettingsKey::Verbose, val_is_true(verbose));
        }
        if let Some(mbr) = env.get("RTX_MISSING_RUNTIME_BEHAVIOR") {
            self.set_str(SettingsKey::MissingRuntimeBehavior, mbr);
        }
        if let Some(experimental) = env.get("RTX_EXPERIMENTAL") {
            self.set_bool(SettingsKey::Experimental, val_is_true(experimental));
        }
        self.reset();
    }

    pub fn set_str(&mut self, key: SettingsKey, value: &str) {
        self.vars.insert(key, Value::String(value.to_string()));
    }

    pub fn set_int(&mut self, key: SettingsKey, value: usize) {
        self.vars.insert(key, Value::Int(value));
    }

    pub fn set_bool(&mut self, key: SettingsKey, value: bool) {
        self.vars.insert(key, Value::Bool(value));
    }

    pub fn apply(&mut self, other: &Settings2) {
        for (key, value) in &other.vars {
            self.vars.insert(key.clone(), value.clone());
        }
        self.reset();
    }

    pub fn reset(&mut self) {
        if let Some(Value::String(log_level)) = self.vars.get(&SettingsKey::LogLevel) {
            self.log_level = log_level.parse().unwrap();
        }
        if let Some(Value::String(level)) = self.vars.get(&SettingsKey::LogFileLevel) {
            self.log_file_level = level.parse().unwrap();
        } else {
            self.log_file_level = self.log_level;
        }
        if let Some(Value::Bool(experimental)) = self.vars.get(&SettingsKey::Experimental) {
            self.experimental = *experimental;
        }
        if let Some(Value::Bool(raw)) = self.vars.get(&SettingsKey::Raw) {
            self.raw = *raw;
            self.jobs = 1;
            self.verbose = true;
        } else {
            if let Some(Value::Int(jobs)) = self.vars.get(&SettingsKey::Jobs) {
                self.jobs = *jobs;
            }
            if let Some(Value::Bool(verbose)) = self.vars.get(&SettingsKey::Verbose) {
                self.verbose = *verbose;
            }
        }
        if let Some(Value::String(mrb)) = self.vars.get(&SettingsKey::MissingRuntimeBehavior) {
            self.missing_runtime_behavior = mrb.parse().unwrap();
        }
    }
}

impl Default for Settings2 {
    fn default() -> Self {
        Self {
            t: true,
            log_level: default_log_level(),
            log_file_level: default_file_log_level(),
            jobs: 4,
            missing_runtime_behavior: MissingRuntimeBehavior::Warn,
            experimental: false,
            raw: false,
            verbose: false,
            vars: HashMap::new(),
        }
    }
}

fn default_log_level() -> LevelFilter {
    for (i, arg) in env::args().enumerate() {
        if arg == "--" {
            break;
        }
        if let Some(("--log-level", level)) = arg.split_once('=') {
            return level.parse().unwrap();
        }
        if arg == "--log-level" {
            if let Some(level) = env::args().nth(i + 1) {
                return level.parse().unwrap();
            }
        }
        if arg == "--debug" {
            return LevelFilter::Debug;
        }
        if arg == "--trace" {
            return LevelFilter::Trace;
        }
    }
    env::var("RTX_LOG_LEVEL")
        .unwrap_or_default()
        .parse()
        .unwrap_or(LevelFilter::Info)
}

fn default_file_log_level() -> LevelFilter {
    if let Ok(log_file_level) = env::var("RTX_LOG_FILE_LEVEL") {
        return log_file_level.parse().unwrap();
    }
    default_log_level()
}

#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    String(String),
    Int(usize),
    // Array(Vec<Value>),
    // Object(HashMap<String, Value>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum SettingsKey {
    LogLevel,
    LogFileLevel,
    Raw,
    Jobs,
    Verbose,
    MissingRuntimeBehavior,
    Experimental,
}

fn val_is_true(val: &str) -> bool {
    let v = val.to_lowercase();
    v == "y" || v == "yes" || v == "true" || v == "1" || v == "on"
}
