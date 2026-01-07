use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use axum::response::sse::{Event, Sse};
        use chrono::Utc;
        use futures::stream::{Stream, StreamExt};
        use once_cell::sync::Lazy;
        use std::{convert::Infallible, fmt::Debug};
        use tokio::sync::broadcast;
        use tokio_stream::wrappers::BroadcastStream;
        use tracing::{Event as TracingEvent, Subscriber};
        use tracing::field::{Field, Visit};
        use tracing_subscriber::fmt::format::FmtSpan;
        use tracing_subscriber::layer::Context;
        use tracing_subscriber::layer::Layer;
        use tracing_subscriber::prelude::*;
        use tracing_subscriber::registry::LookupSpan;

        // Global broadcast sender shared by the tracing layer and SSE handler
        static LOG_TX: Lazy<broadcast::Sender<String>> = Lazy::new(|| {
            // choose buffer capacity according to expected burstiness
            broadcast::channel::<String>(1024).0
        });

        /// A tiny visitor that collects fields from a `tracing::Event`.
        #[derive(Default)]
        struct FieldVisitor {
            parts: Vec<String>,
        }

        impl Visit for FieldVisitor {
            fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
                self.parts.push(format!("{}={:?}", field.name(), value));
            }
            fn record_i64(&mut self, field: &Field, value: i64) {
                self.parts.push(format!("{}={}", field.name(), value));
            }
            fn record_u64(&mut self, field: &Field, value: u64) {
                self.parts.push(format!("{}={}", field.name(), value));
            }
            fn record_bool(&mut self, field: &Field, value: bool) {
                self.parts.push(format!("{}={}", field.name(), value));
            }
            fn record_str(&mut self, field: &Field, value: &str) {
                self.parts.push(format!("{}={}", field.name(), value));
            }
            fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
                self.parts.push(format!("{}={}", field.name(), value));
            }
        }

        /// A tracing_subscriber layer, forwards formatted events to broadcast channel
        struct BroadcastLayer {
            tx: broadcast::Sender<String>,
        }

        impl<S> Layer<S> for BroadcastLayer
        where
            S: Subscriber + for<'a> LookupSpan<'a>,
        {
            fn on_event(&self, event: &TracingEvent<'_>, _ctx: Context<'_, S>) {
                let mut visitor = FieldVisitor::default();
                event.record(&mut visitor);

                let meta = event.metadata();
                let level = meta.level();
                let target = meta.target();
                let message = if !visitor.parts.is_empty() {
                    visitor.parts.join(", ")
                } else {
                    meta.name().to_string()
                };

                let line = format!("{} {} {} - {}", Utc::now().to_rfc3339(), level, target, message);

                let _ = self.tx.send(line);
            }
        }

        /// Initializes tracing logger with 2 layers:
        /// 1. To stdout
        /// 2. To broadcast layer
        pub fn init_tracing() {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .with_file(true)
                .with_line_number(true)
                .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
                .with_target(true)
                .with_writer(std::io::stdout);

            let forward_layer = BroadcastLayer {
                tx: LOG_TX.clone(),
            };

            tracing_subscriber::registry()
                .with(fmt_layer)
                .with(forward_layer)
                .init();
        }

        /// SSE endpoint returning stream of events
        pub async fn logs_sse() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
            let rx = LOG_TX.subscribe();

            let stream = BroadcastStream::new(rx)
                .filter_map(|res| async move { res.ok() })
                .map(|msg: String| Event::default().data(msg))
                .map(Ok);

            Sse::new(stream)
        }
    }
}