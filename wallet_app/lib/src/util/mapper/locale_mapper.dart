import 'dart:ui';

export 'dart:ui';

abstract class LocaleMapper<I, O> {
  O map(Locale locale, I input);

  List<O> mapList(Locale locale, List<I> input) => input.map((e) => map(locale, e)).toList();
}
