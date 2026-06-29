import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/util/extension/build_context_extension.dart';

void main() {
  group('screenReaderListCacheExtent', () {
    testWidgets('returns a viewport cache extent when a screen reader is enabled', (tester) async {
      await tester.pumpWidget(
        MediaQuery(
          data: const MediaQueryData(accessibleNavigation: true),
          child: Builder(
            builder: (BuildContext context) {
              final extent = context.screenReaderListCacheExtent;
              expect(extent, isNotNull);
              expect(extent!.style, CacheExtentStyle.viewport);
              expect(extent.value, 5.0);
              return const Placeholder();
            },
          ),
        ),
      );
    });

    testWidgets('returns null when no screen reader is enabled', (tester) async {
      await tester.pumpWidget(
        MediaQuery(
          data: const MediaQueryData(accessibleNavigation: false),
          child: Builder(
            builder: (BuildContext context) {
              expect(context.screenReaderListCacheExtent, isNull);
              return const Placeholder();
            },
          ),
        ),
      );
    });
  });
}
