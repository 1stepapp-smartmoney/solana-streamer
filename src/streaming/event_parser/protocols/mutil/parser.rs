use crate::{
    impl_event_parser_delegate,
    streaming::event_parser::{
        common::filter::EventTypeFilter,
        core::traits::{GenericEventParseConfig, GenericEventParser},
        EventParserFactory, Protocol,
    },
};

pub struct MutilEventParser {
    inner: GenericEventParser,
}

impl MutilEventParser {
    pub fn new(protocols: Vec<Protocol>, event_type_filter: Option<EventTypeFilter>) -> Self {
        let mut inner = GenericEventParser::new(vec![], vec![]);
        // Configure all event types
        for protocol in protocols {
            let parse = EventParserFactory::create_parser(protocol);

            // Merge instruction_configs, append configurations to existing Vec
            for (key, configs) in parse.instruction_configs() {
                let filtered_configs: Vec<GenericEventParseConfig> = configs
                    .into_iter()
                    .filter(|config| {
                        event_type_filter
                            .as_ref()
                            .map(|filter| filter.include.contains(&config.event_type))
                            .unwrap_or(true)
                    })
                    .collect();
                inner
                    .instruction_configs
                    .entry(key)
                    .or_insert_with(Vec::new)
                    .extend(filtered_configs);
            }

            // Append program_ids (this is already appending)
            inner.program_ids.extend(parse.supported_program_ids().clone());
        }
        Self { inner }
    }
}

impl_event_parser_delegate!(MutilEventParser);
