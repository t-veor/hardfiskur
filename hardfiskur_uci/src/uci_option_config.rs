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

impl Display for UCIOptionConfig {
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
