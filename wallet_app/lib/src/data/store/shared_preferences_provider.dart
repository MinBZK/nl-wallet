import 'package:shared_preferences/shared_preferences.dart';

/// A type definition for a function that provides an instance of [SharedPreferences].
///
/// Enables dependency injection of different SharedPreferences implementations,
/// such as real instances for production or mock instances for testing.
///
/// Returns a [Future] that completes with a [SharedPreferences] instance.
typedef PreferenceProvider = Future<SharedPreferences> Function();
