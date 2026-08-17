import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/wallet/wallet_repository.dart';
import 'package:wallet/src/domain/model/disclosure/disclose_card_request.dart';
import 'package:wallet/src/domain/usecase/app/check_is_app_initialized_usecase.dart';
import 'package:wallet/src/domain/usecase/biometrics/is_biometric_login_enabled_usecase.dart';
import 'package:wallet/src/domain/usecase/pin/unlock_wallet_with_pin_usecase.dart';
import 'package:wallet/src/feature/check_attributes/check_attributes_screen.dart';
import 'package:wallet/src/feature/common/widget/card/shared_attributes_card.dart';
import 'package:wallet/src/feature/login/login_detail_screen.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/util/extension/localized_text_extension.dart';
import 'package:wallet/src/util/manager/biometric_unlock_manager.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mock_data.dart';
import '../../mocks/wallet_mocks.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';
import '../pin/pin_page_test.dart';

void main() {
  Widget buildLoginDetailScreen({required List<DiscloseCardRequest> cardRequests}) {
    return LoginDetailScreen(
      organization: WalletMockData.organization,
      policy: WalletMockData.policy,
      cardRequests: cardRequests,
      sharedDataWithOrganizationBefore: false,
    );
  }

  group('goldens', () {
    testGoldens('Login overview - light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        buildLoginDetailScreen(
          cardRequests: [WalletMockData.discloseCardRequestSingleCard],
        ),
      );

      await screenMatchesGolden('login.light');
    });
  });

  group('Requested credentials', () {
    testWidgets('renders the section header title, body and icon', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        buildLoginDetailScreen(cardRequests: [WalletMockData.discloseCardRequestSingleCard]),
      );

      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.loginDetailScreenCredentialsTitle), findsOneWidget);
      expect(find.text(l10n.loginDetailScreenCredentialsBody), findsOneWidget);
      expect(find.byIcon(Icons.credit_card_outlined), findsOneWidget);
    });

    testWidgets('renders one SharedAttributesCard per card request', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        buildLoginDetailScreen(
          cardRequests: [
            WalletMockData.discloseCardRequestSingleCard,
            WalletMockData.discloseCardRequestMultiCard,
          ],
        ),
      );

      expect(find.byType(SharedAttributesCard), findsNWidgets(2));
    });

    testWidgets('renders the currently selected candidate for each card request', (tester) async {
      final multiCardRequest = DiscloseCardRequest(
        candidates: [WalletMockData.card, WalletMockData.altCard],
        selectedIndex: 1,
      );

      await tester.pumpWidgetWithAppWrapper(
        buildLoginDetailScreen(cardRequests: [multiCardRequest]),
      );

      // The selected (alt) candidate's title should be shown, not the first candidate's.
      expect(find.textContaining(WalletMockData.altCard.title.testValue), findsOneWidget);
      expect(find.textContaining(WalletMockData.card.title.testValue), findsNothing);
    });

    testWidgets('renders no cards when there are no card requests', (tester) async {
      await tester.pumpWidgetWithAppWrapper(buildLoginDetailScreen(cardRequests: []));

      expect(find.byType(SharedAttributesCard), findsNothing);
      // Section header should still be shown.
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.loginDetailScreenCredentialsTitle), findsOneWidget);
    });

    testWidgets('tapping a card navigates to the CheckAttributesScreen for that card', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        buildLoginDetailScreen(cardRequests: [WalletMockData.discloseCardRequestSingleCard]),
        providers: [
          RepositoryProvider<WalletRepository>(
            create: (_) {
              final mockRepo = MockWalletRepository();
              when(mockRepo.isLockedStream).thenAnswer((_) => Stream.value(false));
              return mockRepo;
            },
          ),
          RepositoryProvider<IsWalletInitializedUseCase>(create: (_) => MockIsWalletInitializedUseCase()),
          RepositoryProvider<PinBloc>(create: (_) => MockPinBloc()),
          RepositoryProvider<UnlockWalletWithPinUseCase>(create: (_) => MockUnlockWalletWithPinUseCase()),
          RepositoryProvider<IsBiometricLoginEnabledUseCase>(create: (_) => MockIsBiometricLoginEnabledUseCase()),
          RepositoryProvider<BiometricUnlockManager>(create: (_) => MockBiometricUnlockManager()),
        ],
      );

      final cardFinder = find.byType(SharedAttributesCard);
      expect(cardFinder, findsOneWidget);
      final tapTarget = find.descendant(of: cardFinder, matching: find.byType(TextButton)).first;
      await tester.tap(tapTarget);
      await tester.pumpAndSettle();

      expect(find.byType(CheckAttributesScreen), findsOneWidget);
    });
  });
}
