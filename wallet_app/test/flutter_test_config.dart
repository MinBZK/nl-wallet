import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/disclosure/disclosure_repository.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/attribute/data_attribute.dart';
import 'package:wallet/src/domain/model/card_front.dart';
import 'package:wallet/src/domain/model/disclosure/disclosure_session_type.dart';
import 'package:wallet/src/domain/model/disclosure/disclosure_type.dart';
import 'package:wallet/src/domain/model/navigation/navigation_request.dart';
import 'package:wallet/src/domain/model/organization.dart';
import 'package:wallet/src/util/extension/bloc_extension.dart';
import 'package:wallet/src/util/extension/string_extension.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import 'src/mocks/wallet_mock_data.dart';
import 'src/mocks/wallet_mocks.dart';
import 'src/util/golden_diff_comparator.dart';

Future<void> testExecutable(FutureOr<void> Function() testMain) async {
  await loadAppFonts();
  _provideDefaultCheckHasInternetMock();
  _setupMockitoDummies();
  _setupGoldenFileComparator();
  return testMain();
}

/// Some BLoCs rely on the static [BlocExtensions.checkHasInternetUseCase], provide a default
/// implementation for all tests.
void _provideDefaultCheckHasInternetMock() {
  final mockCheckHasInternetUseCase = MockCheckHasInternetUseCase();
  when(mockCheckHasInternetUseCase.invoke()).thenAnswer((realInvocation) async => true);
  BlocExtensions.checkHasInternetUseCase = mockCheckHasInternetUseCase;
}

/// Configure some basic mockito dummies
void _setupMockitoDummies() {
  provideDummy<DataAttribute>(
    DataAttribute.untranslated(key: '', label: '', value: const StringValue(''), sourceCardDocType: ''),
  );
  provideDummy<Organization>(WalletMockData.organization);
  provideDummy<AttributeValue>(const StringValue(''));
  provideDummy<CardFront>(CardFront(title: ''.untranslated, backgroundImage: '', theme: CardFrontTheme.light));
  provideDummy<NavigationRequest>(const GenericNavigationRequest('/mock_destination'));
  provideDummy<StartDisclosureResult>(StartDisclosureReadyToDisclose(
    WalletMockData.organization,
    'http://origin.org',
    'requestPurpose'.untranslated,
    false,
    DisclosureSessionType.crossDevice,
    DisclosureType.regular,
    {},
    WalletMockData.policy,
  ));
  provideDummy<CoreError>(const CoreGenericError('dummy'));
}

/// Overrides the default [LocalFileComparator] with our [GoldenDiffComparator] that has
/// a configurable tolerance (defaults to 0.5%) when comparing goldens.
void _setupGoldenFileComparator() {
  final testFilePath = (goldenFileComparator as LocalFileComparator).basedir;
  goldenFileComparator = GoldenDiffComparator(testFilePath.toString());
}
