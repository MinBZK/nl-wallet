import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/service/work_manager_service.dart';

import '../../mocks/wallet_mocks.dart';

void main() {
  late MockWorkmanager mockWorkmanager;

  setUp(() {
    mockWorkmanager = MockWorkmanager();

    // Stub initialize to return a completed future
    when(
      mockWorkmanager.initialize(
        any,
        isInDebugMode: anyNamed('isInDebugMode'),
      ),
    ).thenAnswer((_) async => {});

    // Stub registerPeriodicTask
    when(
      mockWorkmanager.registerPeriodicTask(
        any,
        any,
        existingWorkPolicy: anyNamed('existingWorkPolicy'),
        initialDelay: anyNamed('initialDelay'),
        constraints: anyNamed('constraints'),
        frequency: anyNamed('frequency'),
        tag: anyNamed('tag'),
      ),
    ).thenAnswer((_) async => {});
  });

  test('WorkManagerService initializes Workmanager and registers periodic task', () async {
    WorkManagerService(mockWorkmanager);

    // Await async constructor initialization
    await Future.delayed(Duration.zero);

    verify(mockWorkmanager.initialize(callbackDispatcher)).called(1);
    verify(
      mockWorkmanager.registerPeriodicTask(
        kBackgroundSyncTask,
        kBackgroundSyncTask,
        existingWorkPolicy: .update,
        initialDelay: const Duration(minutes: 5),
        constraints: anyNamed('constraints'),
        frequency: const Duration(hours: 1),
        tag: 'background-sync',
      ),
    ).called(1);
  });

  group('static', () {
    test('executeTask unsupported task', () async {
      final success = await executeTask('random_task_id', {});

      // Unsupported task should shortcut to reporting it completed.
      expect(success, isTrue);
    });

    test('executeTask supported task', () async {
      final success = await executeTask(kBackgroundSyncTask, {});

      // Supported task (which fails to execute) should report it failed, triggering reschedule.
      // Note: task fails in tests because dependencies are not available, so exception leading to false is expected.
      expect(success, isFalse);
    });

    test('resolveAndroidDetails resolves the expected details', () async {
      final result = resolveAndroidDetails(MockActiveLocaleProvider(), .cardUpdates);

      expect(result, isNotNull);
      expect(result?.channelId, 'cardUpdates');
      expect(result?.autoCancel, isTrue);
    });
  });
}
