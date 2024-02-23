import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/data/service/app_lifecycle_service.dart';

import '../../../wallet_app_test_widget.dart';

void main() {
  testWidgets('AppLifecycleObserver renders the provided child', (tester) async {
    await tester.pumpWidgetWithAppWrapper(
      const AppLifecycleObserver(
        child: Text('Lorem Ipsum'),
      ),
    );

    final childFinder = find.text('Lorem Ipsum');
    expect(childFinder, findsOneWidget);
  });
}
