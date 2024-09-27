use std::fmt::Display;

use nom::{
    branch::alt,
    character::complete::i64,
    combinator::{fail, value, verify},
    IResult,
};

use crate::{
    format_utils::SpaceSepFormatter,
    parse_utils::{keyworded_options, token, token_tag, try_opt_once},
};

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
    pub fn parser(original_input: &str) -> IResult<&str, Self> {
        let is_keyword = verify(token, |s: &str| {
            matches!(s, "name" | "type" | "default" | "min" | "max" | "var")
        });

        let (input, options) = keyworded_options(is_keyword)(original_input)?;

        let mut name = None;
        let mut option_type = None;
        let mut default = None;
        let mut min = None;
        let mut max = None;
        let mut var = Vec::new();

        for (option_name, value) in options {
            match option_name {
                "name" => name = Some(value.to_string()),
                "type" => option_type = Some(value),
                "default" => default = Some(value),
                "min" => min = try_opt_once(i64, value),
                "max" => max = try_opt_once(i64, value),
                "var" => var.push(value.to_string()),
                _ => unreachable!(),
            }
        }

        let name = match name {
            Some(name) => name,
            None => return fail(original_input),
        };

        let result = if option_type == Some("check") {
            let default = default.and_then(|s| {
                try_opt_once(
                    alt((
                        value(true, token_tag("true")),
                        value(false, token_tag("false")),
                    )),
                    s,
                )
            });

            Self::Check { name, default }
        } else if option_type == Some("spin") {
            let default = default.and_then(|s| try_opt_once(i64, s));

            Self::Spin {
                name,
                default,
                min,
                max,
            }
        } else if option_type == Some("combo") {
            let default = default.map(|s| s.to_string());
            Self::Combo { name, default, var }
        } else if option_type == Some("button") {
            Self::Button { name }
        } else if option_type == Some("string") {
            let default = default.map(|s| {
                if s == "<empty>" {
                    "".to_string()
                } else {
                    s.to_string()
                }
            });
            Self::String { name, default }
        } else {
            return fail(original_input);
        };

        Ok((input, result))
    }
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
