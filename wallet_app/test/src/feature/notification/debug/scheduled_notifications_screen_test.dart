import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:plugin_platform_interface/plugin_platform_interface.dart';
import 'package:wallet/src/domain/model/notification/os_notification.dart';
import 'package:wallet/src/domain/usecase/notification/observe_os_notifications_usecase.dart';
import 'package:wallet/src/feature/notification/debug/scheduled_notifications_screen.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mocks.mocks.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('ScheduledNotificationsScreen simple interaction test', (tester) async {
    FlutterLocalNotificationsPlatform.instance = MockFlutterLocalNotificationsPlugin();
    final mockUseCase = MockObserveOsNotificationsUseCase();
    when(mockUseCase.invoke()).thenAnswer(
      (_) => Stream.value([
        OsNotification(
          id: 1,
          channel: .cardUpdates,
          title: 'title',
          body: 'body',
          notifyAt: DateTime.now(),
        ),
      ]).asBroadcastStream(),
    );

    await tester.pumpWidgetWithAppWrapper(
      RepositoryProvider<ObserveOsNotificationsUseCase>.value(
        value: mockUseCase,
        child: const ScheduledNotificationsScreen(),
      ),
    );

    await tester.pumpAndSettle();

    // Switch between tabs to gain coverage
    await tester.tap(find.text('Pending'));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Active'));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Core'));
    await tester.pumpAndSettle();

    // Basic assertion to confirm the screen rendered
    expect(find.byType(ScheduledNotificationsScreen), findsOneWidget);
  });

  testWidgets('ScheduledNotificationsScreen simple interaction test - Empty states', (tester) async {
    FlutterLocalNotificationsPlatform.instance = MockFlutterLocalNotificationsPlugin(empty: true);
    final mockUseCase = MockObserveOsNotificationsUseCase();
    when(mockUseCase.invoke()).thenAnswer((_) => const Stream.empty());

    await tester.pumpWidgetWithAppWrapper(
      RepositoryProvider<ObserveOsNotificationsUseCase>.value(
        value: mockUseCase,
        child: const ScheduledNotificationsScreen(),
      ),
    );

    await tester.pumpAndSettle();

    // Switch between tabs to gain coverage
    await tester.tap(find.text('Pending'));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Active'));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Core'));
    await tester.pumpAndSettle();

    // Basic assertion to confirm the screen rendered
    expect(find.byType(ScheduledNotificationsScreen), findsOneWidget);
  });
}

class MockMethodChannel extends Mock implements MethodChannel {}

class MockFlutterLocalNotificationsPlugin extends Mock
    with MockPlatformInterfaceMixin
    implements FlutterLocalNotificationsPlatform {
  final bool empty;

  MockFlutterLocalNotificationsPlugin({this.empty = false});

  @override
  Future<List<PendingNotificationRequest>> pendingNotificationRequests() async {
    return empty ? [] : [const PendingNotificationRequest(1, 'title', 'body', 'payload')];
  }

  @override
  Future<List<ActiveNotification>> getActiveNotifications() async {
    return empty ? [] : [const ActiveNotification(id: 1, title: 'title', body: 'body', payload: 'payload')];
  }
}
