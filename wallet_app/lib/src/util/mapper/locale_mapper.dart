import 'dart:ui';

export 'dart:ui';

abstract class LocaleMapper<I, O> {
  O map(Locale locale, I input);
}
