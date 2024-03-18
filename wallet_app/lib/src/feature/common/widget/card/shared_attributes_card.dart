import 'package:flutter/material.dart';

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/attribute/data_attribute.dart';
import '../../../../domain/model/card_front.dart';
import '../../../../domain/model/wallet_card.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../svg_or_image.dart';
import 'card_holograph.dart';

const _kCornerRadius = Radius.circular(12);
const _kBorderRadius = BorderRadius.all(_kCornerRadius);
const _kHolographSize = 134.0;
const _kHeaderStripHeight = 48.0;

/// A Card like component that lists all the titles of the provided [attributes].
/// Used in e.g. the disclosure flow to show which attributes can be shared.
class SharedAttributesCard extends StatelessWidget {
  final WalletCard card;
  final List<DataAttribute> attributes;
  final VoidCallback? onTap;

  const SharedAttributesCard({
    required this.card,
    required this.attributes,
    this.onTap,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return InkWell(
      borderRadius: _kBorderRadius,
      onTap: onTap,
      child: Container(
        decoration: _createBorderDecoration(context),
        child: Column(
          children: [
            SizedBox(
              width: double.infinity,
              height: _kHeaderStripHeight,
              child: _buildHeaderStrip(),
            ),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 20),
              child: SizedBox(
                width: double.infinity,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      context.l10n.sharedAttributesCardTitle(
                        card.front.title.l10nValue(context),
                        attributes.length,
                      ),
                      style: context.textTheme.titleMedium,
                    ),
                    const SizedBox(height: 8),
                    Row(
                      crossAxisAlignment: CrossAxisAlignment.end,
                      children: [
                        Expanded(
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: _buildAttributeList(context),
                          ),
                        ),
                        const SizedBox(width: 8),
                        Icon(
                          Icons.arrow_forward,
                          color: context.colorScheme.primary,
                          size: 24,
                        ),
                      ],
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildHeaderStrip() {
    return ClipRRect(
      borderRadius: const BorderRadius.only(
        topLeft: _kCornerRadius,
        topRight: _kCornerRadius,
      ),
      child: Stack(
        children: [
          SvgOrImage(
            asset: card.front.backgroundImage,
            fit: BoxFit.cover,
          ),
          _buildPositionedHolograph(),
        ],
      ),
    );
  }

  Widget _buildPositionedHolograph() {
    if (card.front.holoImage == null) return const SizedBox.shrink();
    final holoBrightness = card.front.theme == CardFrontTheme.light ? Brightness.light : Brightness.dark;
    return Positioned(
      right: 32,
      top: _kHolographSize / -3 /* Shift the holo so the center part is shown */,
      height: _kHolographSize,
      width: _kHolographSize,
      child: CardHolograph(
        holograph: card.front.holoImage!,
        brightness: holoBrightness,
      ),
    );
  }

  List<Widget> _buildAttributeList(BuildContext context) {
    return attributes
        .map(
          (attribute) => Text(
            attribute.label.l10nValue(context),
            style: context.textTheme.bodyLarge,
          ),
        )
        .toList();
  }

  BoxDecoration _createBorderDecoration(BuildContext context) {
    return BoxDecoration(
      borderRadius: _kBorderRadius,
      border: Border.all(
        color: context.colorScheme.outlineVariant,
        width: 1,
      ),
    );
  }
}
