import 'dart:async';

import 'package:app_links/app_links.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/bridge_generated.dart';
import 'package:wallet/src/data/service/app_lifecycle_service.dart';
import 'package:wallet/src/data/service/deeplink_service.dart';
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';
import 'package:wallet/src/wallet_core/typed/typed_wallet_core.dart';

import '../../mocks/wallet_mocks.dart';

void main() {
  late MockAppLinks appLinks;
  late DeeplinkService service;
  late MockTypedWalletCore mockWalletCore;
  late MockDecodeDeeplinkUseCase mockDecodeDeeplinkUseCase;
  late MockUpdatePidIssuanceStatusUseCase updatePidIssuanceStatusUseCase;
  late MockObserveWalletLockedUseCase mockObserveWalletLockUseCase;
  late MockCheckNavigationPrerequisitesUseCase mockCheckNavigationPrerequisitesUseCase;
  late MockPerformPreNavigationActionsUseCase mockPerformPreNavigationActionsUseCase;
  late MockNavigatorKey navigatorKey;
  late AppLifecycleService mockAppLifecycleService;

  setUp(() {
    appLinks = MockAppLinks();
    navigatorKey = MockNavigatorKey();
    mockWalletCore = Mocks.create<TypedWalletCore>() as MockTypedWalletCore;
    mockAppLifecycleService = AppLifecycleService();
    // Usecases
    mockCheckNavigationPrerequisitesUseCase = MockCheckNavigationPrerequisitesUseCase();
    mockPerformPreNavigationActionsUseCase = MockPerformPreNavigationActionsUseCase();
    mockObserveWalletLockUseCase = MockObserveWalletLockedUseCase();
    mockDecodeDeeplinkUseCase = MockDecodeDeeplinkUseCase();
    updatePidIssuanceStatusUseCase = MockUpdatePidIssuanceStatusUseCase();

    service = DeeplinkService(
      appLinks,
      navigatorKey,
      mockDecodeDeeplinkUseCase,
      updatePidIssuanceStatusUseCase,
      mockCheckNavigationPrerequisitesUseCase,
      mockPerformPreNavigationActionsUseCase,
      mockObserveWalletLockUseCase,
      mockWalletCore,
      mockAppLifecycleService,
    );
  });

  group('processUri', () {
    test('Wallet core should not be called when the DecodeDeeplinkUsecase can resolve the url', () async {
      when(mockDecodeDeeplinkUseCase.invoke(any)).thenReturn(NavigationRequest('/mock'));
      await appLinks.mockUriEvent(Uri.parse('https://example.org'));
      verifyZeroInteractions(mockWalletCore);
    });

    test('A navigation event should be triggered when a supported deeplink url is provided', () async {
      // Allow deeplink_service to navigate
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);

      // Navigation is triggered when supported deeplink is provided
      when(mockDecodeDeeplinkUseCase.invoke(any)).thenReturn(NavigationRequest('/mock'));
      await appLinks.mockUriEvent(Uri.parse('https://example.org'));

      // Make sure navigation was triggered (note: currently only shallow validation by checking interaction with the navigator)
      verify(navigatorKey.currentState);
    });

    test(
      'Wallet core should be called when the DecodeDeeplinkUsecase can not resolve the url',
      () async {
        await appLinks.mockUriEvent(Uri.parse('https://example.org'));
        verify(mockWalletCore.processUri(any));
      },
    );

    test(
      'Result should be passed on to the updateDigidAuthStatusUseCase when the result is relevant',
      () async {
        when(mockWalletCore.processUri(any)).thenAnswer(
          (_) => Stream.value(ProcessUriEvent.pidIssuance(event: PidIssuanceEvent.success(previewCards: List.empty()))),
        );
        await appLinks.mockUriEvent(Uri.parse('https://example.org'));
        verify(updatePidIssuanceStatusUseCase.invoke(any));
      },
    );

    test(
      'Result should not be passed on to the updateDigidAuthStatusUseCase when the result is irrelevant',
      () async {
        when(mockWalletCore.processUri(any)).thenAnswer((_) => Stream.error('Error'));
        await appLinks.mockUriEvent(Uri.parse('https://example.org'));
        verifyNever(updatePidIssuanceStatusUseCase.invoke(any));
      },
    );
  });

  group('processQueue', () {
    test('No navigation requested when queue is empty', () async {
      when(mockDecodeDeeplinkUseCase.invoke(any)).thenReturn(NavigationRequest('/mock'));
      await appLinks.mockUriEvent(Uri.parse('https://example.org'));
      verifyZeroInteractions(mockWalletCore);
    });

    test('Navigation requested when queue is filled but navigation can now be done', () async {
      // Provide NavigationRequest
      when(mockDecodeDeeplinkUseCase.invoke(any)).thenReturn(NavigationRequest('/mock'));
      // Make sure it gets queued
      await appLinks.mockUriEvent(Uri.parse('https://example.org'));
      // Allow queue to be processed
      when(mockCheckNavigationPrerequisitesUseCase.invoke(any)).thenAnswer((_) async => true);
      // Process the queue
      await service.processQueue();
      // Make sure navigation was triggered (note: currently only shallow validation by checking interaction with the navigator)
      verify(navigatorKey.currentState);
    });
  });
}

// ignore: must_be_immutable
class MockNavigatorKey extends Mock implements GlobalKey<NavigatorState> {}

class MockAppLinks implements AppLinks {
  MockAppLinks();

  final StreamController<Uri> _uriStreamController = StreamController();

  Future<void> mockUriEvent(Uri uri) async {
    _uriStreamController.add(uri);
    // Give the event some time to propagate
    await Future.delayed(kResumeDebounceDuration * 1.5);
  }

  @override
  Stream<String> get allStringLinkStream => throw UnimplementedError();

  @override
  Stream<Uri> get allUriLinkStream => _uriStreamController.stream;

  @override
  Future<Uri?> getInitialAppLink() async => null;

  @override
  Future<String?> getInitialAppLinkString() => throw UnimplementedError();

  @override
  Future<Uri?> getLatestAppLink() => throw UnimplementedError();

  @override
  Future<String?> getLatestAppLinkString() => throw UnimplementedError();

  @override
  Stream<String> get stringLinkStream => throw UnimplementedError();

  @override
  Stream<Uri> get uriLinkStream => throw UnimplementedError();
}
