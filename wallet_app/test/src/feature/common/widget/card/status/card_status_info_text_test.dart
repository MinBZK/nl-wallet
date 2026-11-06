import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/card/status/card_status.dart';
import 'package:wallet/src/domain/model/card/status/card_status_metadata.dart';
import 'package:wallet/src/feature/common/widget/card/status/card_status_info_text.dart';
import 'package:wallet/src/util/mapper/card/status/card_status_metadata_mapper.dart';
import 'package:wallet/src/util/mapper/card/status/card_status_render_type.dart';

import '../../../../../../wallet_app_test_widget.dart';
import '../../../../../mocks/wallet_mock_data.dart';
import '../../../../../test_util/test_utils.dart';

void main() {
  group('semantics', () {
    testWidgets('icon label and text are merged into a single node', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardStatusInfoText(
          CardStatusMetadata(
            text: 'Explanation',
            textColor: Colors.red,
            icon: Icons.close,
            iconSemanticsLabel: 'Status',
          ),
        ),
      );

      expect(tester.getSemantics(find.text('Explanation')), matchesSemantics(label: 'Status\nExplanation'));
    });

    testWidgets('without an icon label only the text is exposed', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        CardStatusInfoText(
          CardStatusMetadata(
            text: 'Explanation',
            textColor: Colors.red,
            icon: Icons.close,
          ),
        ),
      );

      expect(tester.getSemantics(find.text('Explanation')), matchesSemantics(label: 'Explanation'));
    });

    testWidgets('revoked status on the card data screen is named, not conveyed by the icon only', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            final metadata = CardStatusMetadataMapper.map(
              context,
              WalletMockData.cardWithStatus(const CardStatusRevoked()),
              CardStatusRenderType.cardDataScreen,
            );
            return CardStatusInfoText(metadata!);
          },
        ),
      );

      final l10n = await TestUtils.englishLocalizations;
      expect(
        tester.getSemantics(find.text(l10n.cardStatusMetadataCardDataScreenRevoked)),
        matchesSemantics(
          label: '${l10n.cardStatusMetadataWalletItemRevoked}\n${l10n.cardStatusMetadataCardDataScreenRevoked}',
        ),
      );
    });

    testWidgets('revoked status on the shared attributes card is named', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            final metadata = CardStatusMetadataMapper.map(
              context,
              WalletMockData.cardWithStatus(const CardStatusRevoked()),
              CardStatusRenderType.sharedAttributesCard,
            );
            return CardStatusInfoText(metadata!);
          },
        ),
      );

      final l10n = await TestUtils.englishLocalizations;
      expect(
        tester.getSemantics(find.text(l10n.cardStatusMetadataSharedAttributesCardRevoked)),
        matchesSemantics(
          label: '${l10n.cardStatusMetadataWalletItemRevoked}\n${l10n.cardStatusMetadataSharedAttributesCardRevoked}',
        ),
      );
    });

    testWidgets('revoked status on the card detail screen is not prefixed, its text already names it', (tester) async {
      late CardStatusMetadata metadata;
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            metadata = CardStatusMetadataMapper.map(
              context,
              WalletMockData.cardWithStatus(const CardStatusRevoked()),
              CardStatusRenderType.cardDetailScreen,
            )!;
            return CardStatusInfoText(metadata);
          },
        ),
      );

      expect(tester.getSemantics(find.text(metadata.text)), matchesSemantics(label: metadata.text));
    });
  });
}
