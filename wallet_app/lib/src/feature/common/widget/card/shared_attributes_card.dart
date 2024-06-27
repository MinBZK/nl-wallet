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
const _kHeaderStripHeight = 40.0;

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
    return Semantics(
      button: true,
      child: InkWell(
        borderRadius: _kBorderRadius,
        onTap: onTap,
        child: DecoratedBox(
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
                child: _buildCardContent(context),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildCardContent(BuildContext context) {
    return Column(
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
        ..._buildAttributeList(context),
        const SizedBox(height: 16),
        _buildCtaText(context),
        const SizedBox(height: 4),
      ],
    );
  }

  /// Not using linkButton because that has a minHeight which conflicts with the design
  /// FIXME: Can likely be refactored to use [ButtonContent] after merging PVW-2595.
  Widget _buildCtaText(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        Text(
          context.l10n.sharedAttributesCardCta,
          style:
              context.theme.textButtonTheme.style?.textStyle?.resolve({})?.copyWith(color: context.colorScheme.primary),
        ),
        const SizedBox(width: 8),
        Icon(
          Icons.arrow_forward_outlined,
          color: context.colorScheme.primary,
          size: 16,
        ),
      ],
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
          Positioned.fill(
            child: SvgOrImage(
              asset: card.front.backgroundImage,
              fit: BoxFit.cover,
            ),
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
      color: context.colorScheme.surface,
      border: Border.all(
        color: context.colorScheme.outlineVariant,
        width: 1,
      ),
      boxShadow: const [
        BoxShadow(
          color: Color(0x0000000D),
          blurRadius: 15,
          offset: Offset(0, 1),
        ),
        BoxShadow(
          color: Color(0x152A621A),
          blurRadius: 4,
          offset: Offset(0, 4),
        ),
      ],
    );
  }
}
