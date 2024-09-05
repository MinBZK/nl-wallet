use std::panic;

use backtrace::Backtrace;

use tracing::error;

pub fn init_panic_logger() {
    panic::set_hook(Box::new(|panic_info| {
        // Invoke the Sentry panic handler, since this one replaces the previous error panic handler.
        sentry_panic::panic_handler(panic_info);

        // Unfortunately, std::backtrace::Backtrace does not work on Android.
        // This is why we use the "backtrace" crate instead.
        let backtrace = Backtrace::new();

        // The payload may either be a reference to a [`String`] or a `&'static str`.
        let payload = panic_info.payload();
        let message = match (payload.downcast_ref::<String>(), payload.downcast_ref::<&'static str>()) {
            (Some(s), _) => Some(s.as_ref()),
            (_, Some(s)) => Some(*s),
            (_, _) => None,
        };

        // Log the panic message and backtrace, each on separate lines
        // because OSLog on iOS has a 1024 character limit.
        // See
        // https://stackoverflow.com/questions/39584707/nslog-on-devices-in-ios-10-xcode-8-seems-to-truncate-why/40283623#40283623
        //
        // Note that we need to use string formatting to prevent
        // the [`error!`] macro from printing the variable name.
        error!("Panic occurred: {}", message.unwrap_or("UNKNOWN"));
        format!("{:?}", backtrace)
            .split('\n')
            .filter(|backtrace_line| !backtrace_line.is_empty())
            .for_each(|backtrace_line| error!("{}", backtrace_line));

        // Make sure that spawned tasks exit upon panic as well.
        std::process::exit(1);
    }));
}
