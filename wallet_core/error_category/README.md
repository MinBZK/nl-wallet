# Sentry categorized error logging

Sentry is an error tracking and performance monitoring platform. You can send it
stacktraces, minidumps, and accompanying messages.

## Overview

This crate makes it possible to categorize various types of errors, which in
turn makes it possible to decide what/if to send things to Sentry. There are
essentially three important Rust attributes this crate provides:

  * `#[sentry_capture_error]`: Indicates that you want to capture errors and
    send them to Sentry. Can be applied to a function (`fn`) or implementation
    (`impl`). When applied to an implementation, it applies to all functions
    in the implementation. Note that `#[sentry_capture_error]` is applied
    transitively; if annotated on function `a`, which calls `b`, which in turn
    calls `c`, and `c` causes an (unhandled) error, it will be reported to
    Sentry
  * `#[derive(ErrorCategory)]`: Opt to derive a category for `Error` types
  * `#[category(..)]`: Set a category. On an enum or struct, this sets a
    default category for any un-annotated field within the enum or struct
    On a field, this sets the category for that field
  * `#[defer]`: Used on a field in an enum or struct. Indicates that you want
    to defer the categorization of this field to the type which the field
    references

The following categories exist (which you set using the `#[category(..)]`
attribute):

  * `expected`: Expected errors, will not be sent to Sentry
  * `critical`: Critical error, report to Sentry, with message(s)
  * `pd`: Critical error with personal data, sent call stack without messages
  * `defer`: Analysis of categorization is deferred to one of the fields
    of this variant
  * `unexpected`: This is an unexpected error and should never be encountered
    by `sentry_capture_error`. Causes a panic

## Configuration

Sentry client, within Rust is configured by `sentry::init` which takes two
arguments: a `DSN` url, which configures which Sentry project your app talks to
(also handles the authentication part), and a `ClientOptions` struct that can
contain various other configuration settings like `release` and `environment`.

For more in-depth details with regards to how Sentry client initializes on Rust,
see: https://docs.sentry.io/platforms/rust/

## Things to consider

1. This crate does categorization and private-data filtering only for Rust.
   i.e., Something like Flutter, which might interact with the `wallet` crate
   through a `flutter-api` crate, has its own Flutter client configuration,
   and hence, needs to either do something similar to what this crate enables,
   or needs to globally disable the sending of any privacy-sensitive information
   at the Flutter Sentry client-level.

2. Using this crate does not magically guarantee that you are not logging any
   sensitive data to Sentry! You need to use the tools provided by this crate
   (i.e., use the attributes to annotate your functions, implementations, enums,
   structs and fields) to indicate to the Rust Sentry client how to handle a
   specific instance of a log-to-sentry event. In particular, if a log message
   associated with an error may contain privacy-sensitive data, you need to
   annotate it with `#[category(pd)]`. Not doing so **will** result in unwanted
   disclosure of privacy-sensitive data.

3. If you do not provide a configured Sentry client (i.e., a successful call to
   `sentry::init` with a valid `DSN` url and `ClientOptions` struct), then any
   exception and/or error captured by `#[sentry_capture_error]` will eventually
   be no-opped (not by this crate, but by the `sentry` crate).

4. As of the 10th of October, 2024, The Rust code in the `wallet_core` directory
   has 109 lines of code in 29 files which contain a `#[category(critical)]`.
   None of the annotated fields, enums and structs contain obvious issues where
   the field, enum or struct could be considered as miscategorized (i.e., the
   field, enum or struct should be annotated as `#[category(pd)]` instead).
   This consideration is no guarantee of non-existing faulty annotations.
