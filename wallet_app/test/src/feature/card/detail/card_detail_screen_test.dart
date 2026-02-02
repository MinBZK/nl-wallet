import 'package:bloc_test/bloc_test.dart';
import 'package:clock/clock.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/card/status/card_status.dart';
import 'package:wallet/src/domain/model/card/wallet_card.dart';
import 'package:wallet/src/domain/model/wallet_card_detail.dart';
import 'package:wallet/src/feature/card/detail/argument/card_detail_screen_argument.dart';
import 'package:wallet/src/feature/card/detail/bloc/card_detail_bloc.dart';
import 'package:wallet/src/feature/card/detail/card_detail_screen.dart';
import 'package:wallet/src/feature/common/widget/card/wallet_card_item.dart';
import 'package:wallet/src/feature/common/widget/centered_loading_indicator.dart';
import 'package:wallet/src/util/extension/string_extension.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

class MockCardSummaryBloc extends MockBloc<CardDetailEvent, CardDetailState> implements CardDetailBloc {}

void main() {
  CardDetailLoadSuccess cardDetailLoadSuccessMock({CardStatus? status, bool isPidCard = false}) {
    return CardDetailLoadSuccess(
      WalletCardDetail(
        card: WalletMockData.cardWithStatus(status ?? WalletMockData.status),
        mostRecentIssuance: WalletMockData.issuanceEvent,
        mostRecentSuccessfulDisclosure: WalletMockData.disclosureEvent,
      ),
      isPidCard: isPidCard,
    );
  }

  group('goldens', () {
    testGoldens('ltc25 CardDetailLoadSuccess light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          cardDetailLoadSuccessMock(),
        ),
      );
      await screenMatchesGolden('success.light');
    });

    testGoldens('ltc25 CardDetailLoadSuccess dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          cardDetailLoadSuccessMock(),
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('success.dark');
    });

    testGoldens('ltc25 CardDetailLoadSuccess - renewable card - light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          CardDetailLoadSuccess(
            WalletCardDetail(
              card: WalletMockData.card,
              mostRecentIssuance: WalletMockData.issuanceEvent,
              mostRecentSuccessfulDisclosure: WalletMockData.disclosureEvent,
            ),
            isPidCard: true,
          ),
        ),
      );
      await screenMatchesGolden('success.renewable.light');
    });

    testGoldens('ltc25 CardDetailLoadInProgress light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          const CardDetailLoadInProgress(),
        ),
      );
      await screenMatchesGolden('loading.light');
    });

    testGoldens('ltc25 CardDetailLoadFailure light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          CardDetailLoadFailure(WalletMockData.card.attestationId!),
        ),
      );
      await screenMatchesGolden('error.light');
    });

    testGoldens('ltc25 CardDetailLoadSuccess status - validSoon', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          cardDetailLoadSuccessMock(status: CardStatusValidSoon(validFrom: WalletMockData.validFrom)),
        ),
      );
      await screenMatchesGolden('status.valid.soon');
    });

    testGoldens('ltc25 CardDetailLoadSuccess status - valid', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          cardDetailLoadSuccessMock(status: const CardStatusValid(validUntil: null)),
        ),
      );
      await screenMatchesGolden('status.valid');
    });

    testGoldens('ltc25 CardDetailLoadSuccess status - valid - dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        brightness: Brightness.dark,
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          cardDetailLoadSuccessMock(status: const CardStatusValid(validUntil: null)),
        ),
      );
      await screenMatchesGolden('status.valid.dark');
    });

    testGoldens('ltc25 CardDetailLoadSuccess status - valid - PID', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          cardDetailLoadSuccessMock(
            status: const CardStatusValid(validUntil: null),
            isPidCard: true,
          ),
        ),
      );
      await screenMatchesGolden('status.valid.pid');
    });

    testGoldens('ltc25 CardDetailLoadSuccess status - valid until', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          cardDetailLoadSuccessMock(status: CardStatusValid(validUntil: WalletMockData.validUntil)),
        ),
      );
      await screenMatchesGolden('status.valid.until');
    });

    testGoldens('ltc25 CardDetailLoadSuccess status - valid until - PID', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          cardDetailLoadSuccessMock(
            status: CardStatusValid(validUntil: WalletMockData.validUntil),
            isPidCard: true,
          ),
        ),
      );
      await screenMatchesGolden('status.valid.until.pid');
    });

    testGoldens('ltc25 CardDetailLoadSuccess status - expiresSoon', (tester) async {
      await withClock(Clock.fixed(DateTime(2025, 1, 1)), () async {
        await tester.pumpWidgetWithAppWrapper(
          CardDetailScreen(
            cardTitle: WalletMockData.card.title.testValue,
          ).withState<CardDetailBloc, CardDetailState>(
            MockCardSummaryBloc(),
            cardDetailLoadSuccessMock(status: CardStatusExpiresSoon(validUntil: WalletMockData.validUntil)),
          ),
        );
        await screenMatchesGolden('status.expires.soon');
      });
    });

    testGoldens('ltc25 CardDetailLoadSuccess status - expiresSoon - PID', (tester) async {
      await withClock(Clock.fixed(DateTime(2025, 1, 1)), () async {
        await tester.pumpWidgetWithAppWrapper(
          CardDetailScreen(
            cardTitle: WalletMockData.card.title.testValue,
          ).withState<CardDetailBloc, CardDetailState>(
            MockCardSummaryBloc(),
            cardDetailLoadSuccessMock(
              status: CardStatusExpiresSoon(validUntil: WalletMockData.validUntil),
              isPidCard: true,
            ),
          ),
        );
        await screenMatchesGolden('status.expires.soon.pid');
      });
    });

    testGoldens('ltc25 CardDetailLoadSuccess status - expired', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          cardDetailLoadSuccessMock(status: CardStatusExpired(validUntil: WalletMockData.validUntil)),
        ),
      );
      await screenMatchesGolden('status.expired');
    });

    testGoldens('ltc25 CardDetailLoadSuccess status - expired - scaled', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        textScaleSize: 2,
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          cardDetailLoadSuccessMock(status: CardStatusExpired(validUntil: WalletMockData.validUntil)),
        ),
      );
      await screenMatchesGolden('status.expired.scaled');
    });

    testGoldens('ltc25 CardDetailLoadSuccess status - expired - dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        brightness: Brightness.dark,
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          cardDetailLoadSuccessMock(status: CardStatusExpired(validUntil: WalletMockData.validUntil)),
        ),
      );
      await screenMatchesGolden('status.expired.dark');
    });

    testGoldens('ltc25 CardDetailLoadSuccess status - expired - PID', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          cardDetailLoadSuccessMock(
            status: CardStatusExpired(validUntil: WalletMockData.validUntil),
            isPidCard: true,
          ),
        ),
      );
      await screenMatchesGolden('status.expired.pid');
    });

    testGoldens('ltc25 CardDetailLoadSuccess status - revoked', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          cardDetailLoadSuccessMock(status: const CardStatusRevoked()),
        ),
      );
      await screenMatchesGolden('status.revoked');
    });

    testGoldens('ltc25 CardDetailLoadSuccess status - revoked - PID', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          cardDetailLoadSuccessMock(
            status: const CardStatusRevoked(),
            isPidCard: true,
          ),
        ),
      );
      await screenMatchesGolden('status.revoked.pid');
    });

    testGoldens('ltc25 CardDetailLoadSuccess status - corrupted', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          cardDetailLoadSuccessMock(status: const CardStatusCorrupted()),
        ),
      );
      await screenMatchesGolden('status.corrupted');
    });

    testGoldens('ltc25 CardDetailLoadSuccess status - corrupted - PID', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          cardDetailLoadSuccessMock(
            status: const CardStatusCorrupted(),
            isPidCard: true,
          ),
        ),
      );
      await screenMatchesGolden('status.corrupted.pid');
    });

    testGoldens('ltc25 CardDetailLoadSuccess status - undetermined', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          cardDetailLoadSuccessMock(status: const CardStatusUndetermined()),
        ),
      );
      await screenMatchesGolden('status.undetermined');
    });

    testGoldens('ltc25 CardDetailLoadSuccess status - undetermined - dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        brightness: Brightness.dark,
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          cardDetailLoadSuccessMock(status: const CardStatusUndetermined()),
        ),
      );
      await screenMatchesGolden('status.undetermined.dark');
    });

    testGoldens('ltc25 CardDetailLoadSuccess status - undetermined - PID', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          cardDetailLoadSuccessMock(
            status: const CardStatusUndetermined(),
            isPidCard: true,
          ),
        ),
      );
      await screenMatchesGolden('status.undetermined.pid');
    });
  });

  group('widgets', () {
    testWidgets('ltc25 card is visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          cardDetailLoadSuccessMock(),
        ),
      );

      // Validate that the widget exists
      final cardTitleFinder = find.text(WalletMockData.card.title.testValue);
      expect(cardTitleFinder, findsNWidgets(2)); // Screen title + title on card
    });

    testWidgets('ltc25 loading renders as expected, with title and loading indicator', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          const CardDetailLoadInProgress(),
        ),
      );

      final cardTitleFinder = find.text(WalletMockData.card.title.testValue);
      expect(cardTitleFinder, findsAtLeast(1));

      // Validate that the loader is shown
      final loadingIndicatorFinder = find.byType(CenteredLoadingIndicator);
      expect(loadingIndicatorFinder, findsOneWidget);
    });

    testWidgets('ltc25 loading with card renders as expected, with title and card', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          CardDetailLoadInProgress(card: WalletMockData.card),
        ),
      );

      // Find the card title
      final cardTitleFinder = find.text(WalletMockData.card.title.testValue);
      expect(cardTitleFinder, findsAtLeast(1));

      // Find the preview card
      final cardFinder = find.byType(WalletCardItem);
      expect(cardFinder, findsOneWidget);

      // Validate that the loader is shown
      final loadingIndicatorFinder = find.byType(CenteredLoadingIndicator);
      expect(loadingIndicatorFinder, findsOneWidget);
    });

    testWidgets('ltc25 error renders with expected, with retry cta', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardDetailScreen(
          cardTitle: WalletMockData.card.title.testValue,
        ).withState<CardDetailBloc, CardDetailState>(
          MockCardSummaryBloc(),
          CardDetailLoadFailure(WalletMockData.card.attestationId!),
        ),
      );

      final l10n = await TestUtils.englishLocalizations;
      final retryFinder = find.text(l10n.generalRetry);
      expect(retryFinder, findsOneWidget);
    });
  });

  group('unit', () {
    test('CardDetailScreenArgument can be extracted from RouteSettings', () async {
      final inputCard = WalletCard(
        attestationId: 'id',
        attestationType: 'com.example.docType',
        issuer: WalletMockData.organization,
        status: WalletMockData.status,
        attributes: const [],
      );
      final CardDetailScreenArgument inputArgument = CardDetailScreenArgument(
        card: inputCard,
        cardId: inputCard.attestationId!,
        cardTitle: ''.untranslated,
      );
      final resultArgument = CardDetailScreen.getArgument(RouteSettings(arguments: inputArgument.toJson()));
      expect(resultArgument, inputArgument);
    });
  });
}
