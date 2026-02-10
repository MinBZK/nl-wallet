import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:wallet/src/feature/notification/bloc/manage_notifications_bloc.dart';
import 'package:wallet/src/feature/notification/manage_notifications_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/golden_utils.dart';

class MockManageNotificationsBloc extends MockBloc<ManageNotificationsEvent, ManageNotificationsState>
    implements ManageNotificationsBloc {}

void main() {
  late ManageNotificationsBloc manageNotificationsBloc;

  setUp(() {
    manageNotificationsBloc = MockManageNotificationsBloc();
  });

  group('goldens', () {
    testGoldens('ManageNotificationsScreen light enabled', (tester) async {
      when(() => manageNotificationsBloc.state).thenReturn(const ManageNotificationsLoaded(pushEnabled: true));

      await tester.pumpWidgetWithAppWrapper(
        BlocProvider.value(
          value: manageNotificationsBloc,
          child: const ManageNotificationsScreen(),
        ),
      );

      await screenMatchesGolden('manage_notifications/light_enabled');
    });

    testGoldens('ManageNotificationsScreen light disabled - scaled', (tester) async {
      when(() => manageNotificationsBloc.state).thenReturn(const ManageNotificationsLoaded(pushEnabled: false));

      await tester.pumpWidgetWithAppWrapper(
        BlocProvider.value(
          value: manageNotificationsBloc,
          child: const ManageNotificationsScreen(),
        ),
        textScaleSize: 2,
      );

      await screenMatchesGolden('manage_notifications/light_disabled.scaled');
    });

    testGoldens('ManageNotificationsScreen dark enabled', (tester) async {
      when(() => manageNotificationsBloc.state).thenReturn(const ManageNotificationsLoaded(pushEnabled: true));

      await tester.pumpWidgetWithAppWrapper(
        brightness: Brightness.dark,
        BlocProvider.value(
          value: manageNotificationsBloc,
          child: const ManageNotificationsScreen(),
        ),
      );

      await screenMatchesGolden('manage_notifications/dark_enabled');
    });

    testGoldens('ManageNotificationsScreen dark disabled landscape', (tester) async {
      when(() => manageNotificationsBloc.state).thenReturn(const ManageNotificationsLoaded(pushEnabled: false));

      await tester.pumpWidgetWithAppWrapper(
        brightness: Brightness.dark,
        surfaceSize: iphoneXSizeLandscape,
        BlocProvider.value(
          value: manageNotificationsBloc,
          child: const ManageNotificationsScreen(),
        ),
      );
      await screenMatchesGolden('manage_notifications/dark_disabled.landscape');
    });
  });
}
