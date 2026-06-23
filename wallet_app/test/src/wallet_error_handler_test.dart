import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:sentry_flutter/sentry_flutter.dart';
import 'package:wallet/src/navigation/wallet_routes.dart';
import 'package:wallet/src/wallet_error_handler.dart';

import 'mocks/wallet_mocks.dart';

void main() {
  late MockNavigatorKey navigatorKey;
  late MockNavigatorState navigatorState;
  late WalletErrorHandler handler;

  setUp(() {
    navigatorKey = MockNavigatorKey();
    navigatorState = MockNavigatorState();
    when(navigatorKey.currentState).thenReturn(navigatorState);
    when(
      navigatorState.pushNamedAndRemoveUntil(any, any, arguments: anyNamed('arguments')),
    ).thenAnswer((_) async => null);
    handler = WalletErrorHandler(navigatorKey);
  });

  test('handleError reports it as handled and navigates to the invariant error screen', () async {
    final handled = handler.handleError(Exception('boom'), StackTrace.current);

    expect(handled, isTrue);
    verifyNever(
      navigatorState.pushNamedAndRemoveUntil(
        any,
        any,
        arguments: anyNamed('arguments'),
      ),
    );

    await Future<void>.delayed(Duration.zero);

    verify(
      navigatorState.pushNamedAndRemoveUntil(
        WalletRoutes.invariantErrorRoute,
        any,
        arguments: anyNamed('arguments'),
      ),
    ).called(1);
  });

  test('marks fatal Dart error events as unhandled PlatformDispatcher errors', () {
    final error = Exception('boom');
    final event = createFatalDartErrorEvent(error);
    final throwableMechanism = event.throwableMechanism as ThrowableMechanism;

    expect(event.level, SentryLevel.fatal);
    expect(throwableMechanism.throwable, same(error));
    expect(throwableMechanism.mechanism.type, 'PlatformDispatcher.onError');
    expect(throwableMechanism.mechanism.handled, isFalse);
  });

  test('navigates to the invariant error screen on every handled error', () async {
    handler.handleError(Exception('first'), StackTrace.current);
    handler.handleError(Exception('second'), StackTrace.current);

    await Future<void>.delayed(Duration.zero);

    verify(
      navigatorState.pushNamedAndRemoveUntil(any, any, arguments: anyNamed('arguments')),
    ).called(2);
  });
}
