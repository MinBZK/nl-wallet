import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/service/navigation_service.dart';
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';

import '../../mocks/wallet_mocks.dart';

void main() {
  late NavigationService service;
  late MockCheckNavigationPrerequisitesUseCase mockCheckNavigationPrerequisitesUseCase;
  late MockPerformPreNavigationActionsUseCase mockPerformPreNavigationActionsUseCase;
  late MockNavigatorKey navigatorKey;

  setUp(() {
    provideDummy<NavigationRequest>(GenericNavigationRequest('/mock_destination'));
    navigatorKey = MockNavigatorKey();
    // Usecases
    mockCheckNavigationPrerequisitesUseCase = MockCheckNavigationPrerequisitesUseCase();
    mockPerformPreNavigationActionsUseCase = MockPerformPreNavigationActionsUseCase();

    service = NavigationService(
      navigatorKey,
      mockCheckNavigationPrerequisitesUseCase,
      mockPerformPreNavigationActionsUseCase,
    );
  });

  group('handleNavigationRequest', () {
    test('When navigation prerequisites are fulfilled, trigger actual navigation', () async {
      // Allow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);

      await service.handleNavigationRequest(GenericNavigationRequest('/mock'));

      // Make sure navigation was triggered (note: currently only shallow validation by checking interaction with the navigator)
      verify(navigatorKey.currentState);
    });

    test('Verify preNavigationActions are executed before the navigation occures', () async {
      // Allow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);

      await service.handleNavigationRequest(GenericNavigationRequest('/mock'));

      verifyInOrder([mockPerformPreNavigationActionsUseCase.invoke(any), navigatorKey.currentState]);
    });

    test('When navigation prerequisites are not fulfilled, do not trigger actual navigation', () async {
      // Disallow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => false);

      await service.handleNavigationRequest(GenericNavigationRequest('/mock'));

      // Make sure navigation was NOT triggered
      verifyNever(navigatorKey.currentState);
    });
  });

  group('processQueue', () {
    test('Verify request is not queued if not explicitly requested', () async {
      // Disallow navigation, meaning the request should be queued if that is requested
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => false);
      await service.handleNavigationRequest(GenericNavigationRequest('/mock'));

      // Now allow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);
      // And process any queue if it exists
      await service.processQueue();

      // Make sure navigation was NOT triggered
      verifyNever(navigatorKey.currentState);
    });

    test('Verify request is queued when requested', () async {
      // Disallow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => false);
      await service.handleNavigationRequest(GenericNavigationRequest('/mock'), queueIfNotReady: true);

      // Now allow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);
      // And process any queue if it exists
      await service.processQueue();

      // Make sure navigation was triggered
      verify(navigatorKey.currentState);
    });

    test('Verify even if request is queued, it is not processed if the app is still not ready', () async {
      // Disallow navigation
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => false);
      await service.handleNavigationRequest(GenericNavigationRequest('/mock'), queueIfNotReady: true);

      // And process any queue if it exists, while app is still NOT ready
      await service.processQueue();

      // Make sure navigation was NOT triggered
      verifyNever(navigatorKey.currentState);
    });
  });
}

// ignore: must_be_immutable
class MockNavigatorKey extends Mock implements GlobalKey<NavigatorState> {}
