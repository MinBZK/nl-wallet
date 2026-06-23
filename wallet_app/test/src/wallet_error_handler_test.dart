import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
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

  test('handleError reports it as handled and navigates to the invariant error screen', () {
    final handled = handler.handleError(Exception('boom'), StackTrace.current);

    expect(handled, isTrue);
    verify(
      navigatorState.pushNamedAndRemoveUntil(
        WalletRoutes.invariantErrorRoute,
        any,
        arguments: anyNamed('arguments'),
      ),
    ).called(1);
  });

  test('navigates to the invariant error screen on every handled error', () {
    handler.handleError(Exception('first'), StackTrace.current);
    handler.handleError(Exception('second'), StackTrace.current);

    verify(
      navigatorState.pushNamedAndRemoveUntil(any, any, arguments: anyNamed('arguments')),
    ).called(2);
  });
}
