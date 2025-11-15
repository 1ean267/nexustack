/*
 * This file is part of the nexustack (https://github.com/1ean267/nexustack) distribution.
 *
 * Copyright (c) Cato Truetschel and contributors. All rights reserved.
 * Licensed under the MIT license. See LICENSE file in the project root for details.
 */

use crate::openapi::Schema;
use cron::Schedule;

macro_rules! build_component_regex {
    ($base:expr, $steps:expr) => {
        const_format::formatcp!(
            r"\*|(?:{base}(?:\/(?:{steps}))?(?:-{base}(?:\/(?:{steps}))?)?,)*{base}(?:\/(?:{steps}))?(?:-{base}(?:\/(?:{steps}))?)?",
            base = $base,
            steps = $steps,
        )
    };
    ($base:expr) => {
        const_format::formatcp!(r"\*|(?:{base}(?:-{base})?,)*{base}(?:-{base})?", base = $base)
    };
}

const SEC_MIN_BASE_REGEX: &str = r"[1-5]?[0-9]";
const SEC_MIN_STEPS: &str = r"2|3|4|5|6|10|12|15|20|30";
const SEC_MIN_REGEX: &str = build_component_regex!(SEC_MIN_BASE_REGEX, SEC_MIN_STEPS);

const HOUR_BASE_REGEX: &str = r"2[0-3]|1[0-9]|[0-9]";
const HOUR_STEPS: &str = r"2|3|4|6|8|12";
const HOUR_REGEX: &str = build_component_regex!(HOUR_BASE_REGEX, HOUR_STEPS);

const DAY_OF_MONTH_BASE_REGEX: &str = r"3[01]|[12][0-9]|[1-9]";
const DAY_OF_MONTH_REGEX: &str = build_component_regex!(DAY_OF_MONTH_BASE_REGEX);

// TODO: ALLOW JAN-DEC
const MONTH_BASE_REGEX: &str = r"1[0-2]|[1-9]";
const MONTH_STEPS: &str = r"2|3|4|6";
const MONTH_REGEX: &str = build_component_regex!(MONTH_BASE_REGEX, MONTH_STEPS);

// TODO: ALLOW SUN-SAT
const DAY_OF_WEEK_BASE_REGEX: &str = r"[0-6]";
const DAY_OF_WEEK_STEPS: &str = r"2|3";
const DAY_OF_WEEK_REGEX: &str = build_component_regex!(DAY_OF_WEEK_BASE_REGEX, DAY_OF_WEEK_STEPS);

const YEAR_BASE_REGEX: &str = r"19[7-9][0-9]|20[0-9][0-9]";
const YEAR_STEPS: &str = r"/d+";
const YEAR_REGEX: &str = build_component_regex!(YEAR_BASE_REGEX, YEAR_STEPS);

const FULL_CRON_REGEX: &str = const_format::formatcp!(
    r"^(?:@(?:yearly|monthly|weekly|daily|hourly))|(?:(?:{sec}\s+)?{min}\s+{hour}\s+{dom}\s+{month}\s+{dow}(?:\s+{year})?))$",
    sec = SEC_MIN_REGEX,
    min = SEC_MIN_REGEX,
    hour = HOUR_REGEX,
    dom = DAY_OF_MONTH_REGEX,
    month = MONTH_REGEX,
    dow = DAY_OF_WEEK_REGEX,
    year = YEAR_REGEX
);

impl Schema for Schedule {
    type Example = &'static str;
    type Examples = <[Self::Example; 3] as IntoIterator>::IntoIter;

    fn describe<B>(schema_builder: B) -> Result<B::Ok, B::Error>
    where
        B: crate::openapi::SchemaBuilder<Self::Examples>,
    {
        schema_builder.describe_str(
            None,
            None,
            Some(FULL_CRON_REGEX),
            None,
            None,
            Some("A cron expression specifying scheduled times."),
            || {
                Ok([
                    "0 0 * * *",         // Every day at midnight
                    "*/15 9-17 * * 1-5", // Every 15 minutes during business hours on weekdays
                    "@hourly",           // Every hour
                ])
            },
            false,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openapi::{self, SchemaExamples};

    #[derive(Debug, PartialEq)]
    struct Error(String);

    impl openapi::Error for Error {
        fn custom<T>(msg: T) -> Self
        where
            T: std::fmt::Display,
        {
            Self(msg.to_string())
        }
    }

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
            f.write_str(&self.0)
        }
    }

    impl std::error::Error for Error {}

    #[test]
    fn test_cron_examples() {
        let examples = <Schedule as SchemaExamples>::examples::<Error>(false)
            .unwrap()
            .collect::<Vec<_>>();
        pretty_assertions::assert_eq!(examples, vec!["0 0 * * *", "*/15 9-17 * * 1-5", "@hourly",]);
    }

    #[test]
    fn test_cron_regex() {
        pretty_assertions::assert_eq!(
            FULL_CRON_REGEX,
            r"^(?:@(?:yearly|monthly|weekly|daily|hourly))|(?:(?:\*|(?:[1-5]?[0-9](?:\/(?:2|3|4|5|6|10|12|15|20|30))?(?:-[1-5]?[0-9](?:\/(?:2|3|4|5|6|10|12|15|20|30))?)?,)*[1-5]?[0-9](?:\/(?:2|3|4|5|6|10|12|15|20|30))?(?:-[1-5]?[0-9](?:\/(?:2|3|4|5|6|10|12|15|20|30))?)?\s+)?\*|(?:[1-5]?[0-9](?:\/(?:2|3|4|5|6|10|12|15|20|30))?(?:-[1-5]?[0-9](?:\/(?:2|3|4|5|6|10|12|15|20|30))?)?,)*[1-5]?[0-9](?:\/(?:2|3|4|5|6|10|12|15|20|30))?(?:-[1-5]?[0-9](?:\/(?:2|3|4|5|6|10|12|15|20|30))?)?\s+\*|(?:2[0-3]|1[0-9]|[0-9](?:\/(?:2|3|4|6|8|12))?(?:-2[0-3]|1[0-9]|[0-9](?:\/(?:2|3|4|6|8|12))?)?,)*2[0-3]|1[0-9]|[0-9](?:\/(?:2|3|4|6|8|12))?(?:-2[0-3]|1[0-9]|[0-9](?:\/(?:2|3|4|6|8|12))?)?\s+\*|(?:3[01]|[12][0-9]|[1-9](?:-3[01]|[12][0-9]|[1-9])?,)*3[01]|[12][0-9]|[1-9](?:-3[01]|[12][0-9]|[1-9])?\s+\*|(?:1[0-2]|[1-9](?:\/(?:2|3|4|6))?(?:-1[0-2]|[1-9](?:\/(?:2|3|4|6))?)?,)*1[0-2]|[1-9](?:\/(?:2|3|4|6))?(?:-1[0-2]|[1-9](?:\/(?:2|3|4|6))?)?\s+\*|(?:[0-6](?:\/(?:2|3))?(?:-[0-6](?:\/(?:2|3))?)?,)*[0-6](?:\/(?:2|3))?(?:-[0-6](?:\/(?:2|3))?)?(?:\s+\*|(?:19[7-9][0-9]|20[0-9][0-9](?:\/(?:/d+))?(?:-19[7-9][0-9]|20[0-9][0-9](?:\/(?:/d+))?)?,)*19[7-9][0-9]|20[0-9][0-9](?:\/(?:/d+))?(?:-19[7-9][0-9]|20[0-9][0-9](?:\/(?:/d+))?)?)?))$"
        );
    }
}
