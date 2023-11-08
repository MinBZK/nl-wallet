import 'dart:async';

import 'package:app_links/app_links.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/service/app_lifecycle_service.dart';
import 'package:wallet/src/data/service/deeplink_service.dart';
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';

import '../../mocks/wallet_mocks.dart';

void main() {
  late MockAppLinks appLinks;
  late MockDecodeUriUseCase mockDecodeUriUseCase;
  late MockNavigationService mockNavigationService;
  late AppLifecycleService mockAppLifecycleService;

  setUp(() {
    provideDummy<NavigationRequest>(const GenericNavigationRequest('/mock_destination'));
    appLinks = MockAppLinks();
    mockNavigationService = MockNavigationService();
    mockAppLifecycleService = AppLifecycleService(); // Uses the real implementation because it's trivial
    mockDecodeUriUseCase = MockDecodeUriUseCase();

    DeeplinkService(
      appLinks,
      mockNavigationService,
      mockDecodeUriUseCase,
      mockAppLifecycleService,
    );
  });

  group('uri events', () {
    test('Navigation request should be passed on to the navigation service if the app is resumed', () async {
      const navigationRequest = GenericNavigationRequest('/mock');
      when(mockDecodeUriUseCase.invoke(any)).thenAnswer((_) async => navigationRequest);
      await appLinks.mockUriEvent(Uri.parse('https://example.org'));

      // Make sure the navigation request was passed on
      verify(mockNavigationService.handleNavigationRequest(navigationRequest, queueIfNotReady: true));
    });

    test('Navigation request should not be passed on to the navigation service when the app is paused', () async {
      // Provide NavigationRequest
      mockAppLifecycleService.notifyStateChanged(AppLifecycleState.paused);
      const navigationRequest = GenericNavigationRequest('/mock');
      when(mockDecodeUriUseCase.invoke(any)).thenAnswer((_) async => navigationRequest);
      // Make sure it gets queued
      await appLinks.mockUriEvent(Uri.parse('https://example.org'));

      // Make sure navigation request was not passed on
      verifyNever(mockNavigationService.handleNavigationRequest(navigationRequest, queueIfNotReady: true));
    });

    test('Navigation request be queued and passed on once the app is resumed', () async {
      // Provide NavigationRequest
      mockAppLifecycleService.notifyStateChanged(AppLifecycleState.paused);
      const navigationRequest = GenericNavigationRequest('/mock');
      when(mockDecodeUriUseCase.invoke(any)).thenAnswer((_) async => navigationRequest);
      // Make sure it gets queued
      await appLinks.mockUriEvent(Uri.parse('https://example.org'));

      // Make sure it's not passed on yet
      verifyNever(mockNavigationService.handleNavigationRequest(navigationRequest, queueIfNotReady: true));

      // Transition the app to the resumed state
      mockAppLifecycleService.notifyStateChanged(AppLifecycleState.resumed);
      await Future.delayed(kResumeDebounceDuration * 1.5);

      // Make sure the navigation request is now passed on
      verify(mockNavigationService.handleNavigationRequest(navigationRequest, queueIfNotReady: true));
    });

    test('Navigation request is only handled once as the app cycles through lifecycles', () async {
      // Provide NavigationRequest
      const navigationRequest = GenericNavigationRequest('/mock');
      when(mockDecodeUriUseCase.invoke(any)).thenAnswer((_) async => navigationRequest);
      // Insert the uri
      await appLinks.mockUriEvent(Uri.parse('https://example.org'));

      // Transition the app through paused and resumed states
      mockAppLifecycleService.notifyStateChanged(AppLifecycleState.paused);
      mockAppLifecycleService.notifyStateChanged(AppLifecycleState.resumed);
      await Future.delayed(kResumeDebounceDuration * 1.5);

      // Make sure the navigation request was only passed on once
      // (note that this test fails when commenting out the clearController in the DeeplinkService)
      verify(mockNavigationService.handleNavigationRequest(navigationRequest, queueIfNotReady: true)).called(1);
    });
  });
}

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
