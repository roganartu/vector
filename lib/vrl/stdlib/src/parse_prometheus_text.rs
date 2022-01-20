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
                    // let mut entry = map![
                    // TODO export the type as a string
                    // Then the log_to_metric config might become something like:
                    //   [[transforms.my_transform_id.metrics]]
                    //   type = "{{type}}"
                    //   field = "value"
                    //   name = "{{name}}"
                    //   tags = "{{tags}}"
                    // This might... just work?
                    // "name": metric_group.name,
                    // "timestamp": group_key.timestamp,
                    // "labels": group_key.labels,
                    // ];
                    match metric_group.metrics {
                        prometheus_parser::GroupKind::Counter(metric_map) => {
                            vec![]
                            // entry.insert("type".to_string(), Value::from("counter"));
                        }
                        prometheus_parser::GroupKind::Gauge(metric_map) => {
                            vec![]
                            // entry.insert("type".to_string(), Value::from("gauge"));
                        }
                        prometheus_parser::GroupKind::Summary(metric_map) => {
                            vec![]
                            // entry.insert("type".to_string(), Value::from("summary"));
                        }
                        prometheus_parser::GroupKind::Histogram(metric_map) => {
                            vec![]
                            // entry.insert("type".to_string(), Value::from("histogram"));
                        }
                        prometheus_parser::GroupKind::Untyped(metric_map) => metric_map
                            .into_iter()
                            .map(|(group_key, sample)| {
                                let mut entry = map![
                                    "name": Value::from(metric_group.name.clone()),
                                    "value": Value::from(sample.value),
                                    // TODO map this into k: Value::from(v)
                                    // "labels": Value::from(group_key.labels),
                                ];
                                // match group_key.timestamp {
                                //     Some(v) => entry.insert("timestamp", Value::from(v)),
                                //     None => {},
                                // };
                                entry
                            })
                            .collect::<Vec<_>>()
                            .into(),
                    }
                    // })
                })
                .flatten()
                .collect::<Vec<_>>()
                .into()),
            Err(err) => Err(ExpressionError::from(format!(
                "failed parsing Prometheus text format: {}",
                err.to_string()
            ))),
        }
    }

    fn type_def(&self, _: &state::Compiler) -> TypeDef {
        TypeDef::new().fallible().array::<TypeDef>(vec![
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
        // Is there a union type? Maybe a match statement of some kind? Generics?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    test_function![
        parse_prometheus_text => ParsePrometheusText;

        // TODO add more examples
        //    - with labels
        //    - with timestamps
        //    - with help text
        //    - with type
        //    - counter
        //    - gauge
        //    - histogram
        //    - summary
        no_labels_gauge_valid {
            args: func_args![value: r#"metric_without_timestamp_and_labels 12.47"#],
            want: Ok(vec![
                map![
                    "name": "metric_without_timestamp_and_labels",
                    "value": 12.47,
                    // TODO what will the timestamp be? How do we assert that?
                ],
            ]),
            tdef: TypeDef::new().fallible().array::<TypeDef>(vec![
                TypeDef::new().object::<&str, TypeDef>(inner_type_def())
            ]),
            tz: shared::TimeZone::default(),
        }
    ];
}
