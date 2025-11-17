import 'package:flutter_test/flutter_test.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet/src/data/service/auto_lock_service.dart';

void main() {
  group('AutoLockService', () {
    late AutoLockService service;

    setUp(() {
      service = AutoLockService();
    });

    tearDown(() {
      service.dispose();
    });

    test('initial autoLockEnabled is true', () {
      expect(service.autoLockEnabled, isTrue);
    });

    test('resetIdleTimeout adds an event to the activityStream', () {
      expectLater(service.activityStream, emits(null));
      service.resetIdleTimeout();
    });

    test('setAutoLock updates autoLockEnabled and emits on autoLockStream', () {
      expect(service.autoLockEnabled, isTrue);
      expectLater(service.autoLockStream, emitsInOrder([true, false]));

      service.setAutoLock(enabled: false);

      expect(service.autoLockEnabled, isFalse);
    });

    test('setAutoLock does not emit if the value is the same', () {
      expectLater(service.autoLockStream, emitsInOrder([true, false]));

      // Setting AutoLock multiple times does not cause autoLockStream to emit multiple times
      service.setAutoLock(enabled: false);
      service.setAutoLock(enabled: false);
      service.setAutoLock(enabled: false);
    });

    test('enabling autoLock resets the idle timeout', () {
      // Set to false first
      service.setAutoLock(enabled: false);

      // Expect an event on activity stream when we enable it
      expectLater(service.activityStream, emits(null));
      service.setAutoLock(enabled: true);
    });

    test('dispose closes streams', () {
      final activityController = PublishSubject<void>();
      final autoLockController = BehaviorSubject<bool>();
      final customService = AutoLockService(
        activityStream: activityController,
        autoLockStream: autoLockController,
      );

      customService.dispose();

      expect(activityController.isClosed, isTrue);
      expect(autoLockController.isClosed, isTrue);
    });
  });
}
