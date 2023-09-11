import 'package:flutter/material.dart';

/// Provide the active (i.e. the language used to currently render the app) [Locale].
abstract class ActiveLocaleProvider {
  Stream<Locale> observe();
}
