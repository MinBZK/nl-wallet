import 'dart:async';
import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/data/service/semantics_event_service.dart';

void main() {
  group('SemanticsEventService', () {
    late SemanticsEventService semanticsEventService;

    setUp(() {
      semanticsEventService = SemanticsEventService();
    });

    test('notifying onSemanticsActionEvent should emit event on actionEventStream', () async {
      // Arrange
      final testEvent = const SemanticsActionEvent(viewId: 1, nodeId: 2, type: SemanticsAction.tap, arguments: 'test');
      final Completer<SemanticsActionEvent> completer = Completer();
      semanticsEventService.actionEventStream.listen(completer.complete);

      // Act
      PlatformDispatcher.instance.onSemanticsActionEvent?.call(testEvent);

      // Assert
      final receivedEvent = await completer.future;
      expect(receivedEvent, equals(testEvent));
    });
  });
}
