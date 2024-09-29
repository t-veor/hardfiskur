use std::fmt::Display;

use crate::format_utils::SpaceSepFormatter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UCIOptionConfig {
    Check {
        name: String,
        default: Option<bool>,
    },
    Spin {
        name: String,
        default: Option<i64>,
        min: Option<i64>,
        max: Option<i64>,
    },
    Combo {
        name: String,
        default: Option<String>,
        var: Vec<String>,
    },
    Button {
        name: String,
    },
    String {
        name: String,
        default: Option<String>,
    },
}

impl UCIOptionConfig {
    pub(crate) fn from_raw(
        name: &str,
        option_type: &str,
        default: Option<&str>,
        min: Option<i64>,
        max: Option<i64>,
        var: Vec<&str>,
    ) -> Option<Self> {
        let name = name.to_string();

        let result = if option_type == "check" {
            let default = default.and_then(|s| match s {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            });

            Self::Check { name, default }
        } else if option_type == "spin" {
            let default = default.and_then(|s| s.parse().ok());

            Self::Spin {
                name,
                default,
                min,
                max,
            }
        } else if option_type == "combo" {
            let default = default.map(|s| {
                if s == "<empty>" {
                    "".to_string()
                } else {
                    s.to_string()
                }
            });

            Self::Combo {
                name,
                default,
                var: var
                    .into_iter()
                    .map(|s| {
                        if s == "<empty>" {
                            "".to_string()
                        } else {
                            s.to_string()
                        }
                    })
                    .collect(),
            }
        } else if option_type == "button" {
            Self::Button { name }
        } else if option_type == "string" {
            let default = default.map(|s| {
                if s == "<empty>" {
                    "".to_string()
                } else {
                    s.to_string()
                }
            });
            Self::String { name, default }
        } else {
            return None;
        };

        Some(result)
    }
}

impl Display for UCIOptionConfig {
    // FIXME: Gotta use <empty>s.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut formatter = SpaceSepFormatter::new(f);

        match self {
            UCIOptionConfig::Check { name, default } => {
                formatter.push_pair("name", name)?;
                formatter.push_str("type check")?;
                formatter.push_option("default", *default)?;
            }

            UCIOptionConfig::Spin {
                name,
                default,
                min,
                max,
            } => {
                formatter.push_pair("name", name)?;
                formatter.push_str("type spin")?;
                formatter.push_option("default", *default)?;
                formatter.push_option("min", *min)?;
                formatter.push_option("max", *max)?;
            }

            UCIOptionConfig::Combo { name, default, var } => {
                formatter.push_pair("name", name)?;
                formatter.push_str("type combo")?;
                formatter.push_option("default", default.as_ref())?;

                for var in var {
                    formatter.push_pair("var", var)?;
                }
            }

            UCIOptionConfig::Button { name } => {
                formatter.push_pair("name", name)?;
                formatter.push_str("type button")?;
            }

            UCIOptionConfig::String { name, default } => {
                formatter.push_pair("name", name)?;
                formatter.push_str("type string")?;
                formatter.push_option("default", default.as_ref())?;
            }
        }

        Ok(())
    }
}
