import 'dart:async';
import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/data/store/impl/active_localization_delegate.dart';

void main() {
  late ActiveLocalizationDelegate delegate;

  setUp(() {
    delegate = ActiveLocalizationDelegate();
  });

  test('delegate should not reload', () async {
    expect(delegate.shouldReload(ActiveLocalizationDelegate()), isFalse);
  });

  test('provided locale defaults to english', () async {
    expect(delegate.activeLocale.languageCode, 'en');
  });

  test('when notified about a different language, that language is emitted as the new activeLocale', () async {
    await delegate.load(Locale('nl'));
    expect(delegate.activeLocale.languageCode, 'nl');
  });

  test('stream provides new locales when they are loaded', () async {
    unawaited(
      expectLater(
        delegate.observe(),
        emitsInOrder(
          [
            Locale('en'),
            Locale('nl'),
            Locale('de'),
          ],
        ),
      ),
    );
    await delegate.load(Locale('nl'));
    await delegate.load(Locale('de'));
  });
}
