use std::collections::BTreeMap;

use prometheus_parser::parse_text;
use vrl::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct ParsePrometheusText;

impl Function for ParsePrometheusText {
    fn identifier(&self) -> &'static str {
        "parse_prometheus_text"
    }

    fn parameters(&self) -> &'static [Parameter] {
        &[Parameter {
            keyword: "value",
            kind: kind::BYTES,
            required: true,
        }]
    }

    fn compile(
        &self,
        _state: &state::Compiler,
        _ctx: &FunctionCompileContext,
        mut arguments: ArgumentList,
    ) -> Compiled {
        let value = arguments.required("value");

        Ok(Box::new(ParsePrometheusTextFn { value }))
    }

    fn examples(&self) -> &'static [Example] {
        &[Example {
            title: "parse basic metric with no labels",
            source: r#"encode_json(parse_prometheus_text!(s'metric_without_timestamp_and_labels 12.47'))"#,
            result: Ok(
                r#"s'[{"gauge":{"value": 12.47},"name":"metric_without_timestamp_and_labels","tags": {}}]'"#,
            ),
        }]
    }
}

#[derive(Debug, Clone)]
struct ParsePrometheusTextFn {
    value: Box<dyn Expression>,
}

impl Expression for ParsePrometheusTextFn {
    fn resolve(&self, ctx: &mut Context) -> Resolved {
        let bytes = self.value.resolve(ctx)?;
        let message = bytes.try_bytes_utf8_lossy()?;

        match parse_text(&message) {
            Ok(parsed) => Ok(parsed
                .into_iter()
                .map(|metric_group| {
                    // metric_group.metrics.into_iter().map(|group_key, metric| {
                    map![
                        "name": metric_group.name,
                        // "timestamp": group_key.timestamp,
                        // "labels": group_key.labels,
                    ]
                    // })
                })
                // .flatten()
                .collect::<Vec<_>>()
                .into()),
            Err(err) => Err(ExpressionError::from(format!(
                "failed parsing Prometheus text format: {}",
                err.to_string()
            ))),
        }
    }

    fn type_def(&self, _: &state::Compiler) -> TypeDef {
        TypeDef::new()
            .fallible()
            // TODO fix this up
            .array::<TypeDef>(vec![
                TypeDef::new().object::<&str, TypeDef>(inner_type_def())
            ])
    }
}

fn inner_type_def() -> BTreeMap<&'static str, TypeDef> {
    map! {
        "name": Kind::Bytes,
        "timestamp": Kind::Timestamp,
        "labels": TypeDef::new().object::<&str, Kind>(map! {}),
        // TODO figure out how to typedef the various different metric types.
        // Is there a union type?
    }
}

#[cfg(test)]
mod tests {
    use shared::btreemap;

    use super::*;

    test_function![
        parse_prometheus_text => ParsePrometheusText;

        no_labels_gauge_valid {
            args: func_args![value: r#"metric_without_timestamp_and_labels 12.47"#],
            // TODO fix this, need a type or something, btree map?
            want: Ok(vec![
                map![
                    "name": "metric_without_timestamp_and_labels",
                ],
            ]),
            tdef: TypeDef::new().fallible().array::<TypeDef>(vec![
                TypeDef::new().object::<&str, TypeDef>(inner_type_def())
            ]),
            tz: shared::TimeZone::default(),
        }
    ];
}
