import 'package:flutter/material.dart';

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/card/wallet_card.dart';
import '../../../../theme/base_wallet_theme.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';
import '../../decoration/shadow_decoration.dart';
import '../default_text_and_focus_style.dart';
import 'wallet_card_item.dart';

const _kCornerRadius = Radius.circular(12);
const _kHeaderStripHeight = 40.0;

/// A Card like component that lists all the titles of the provided [attributes].
/// Used in e.g. the disclosure flow to show which attributes can be shared.
class SharedAttributesCard extends StatefulWidget {
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
  State<SharedAttributesCard> createState() => _SharedAttributesCardState();
}

class _SharedAttributesCardState extends State<SharedAttributesCard> {
  late WidgetStatesController _statesController;

  @override
  void initState() {
    super.initState();
    _statesController = WidgetStatesController();
    WidgetsBinding.instance.addPostFrameCallback((_) => _statesController.addListener(() => setState(() {})));
  }

  @override
  void dispose() {
    _statesController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: true,
      child: DecoratedBox(
        decoration: CardShadowDecoration(),
        child: TextButton(
          onPressed: widget.onTap,
          statesController: _statesController,
          style: context.theme.iconButtonTheme.style?.copyWith(
            padding: const WidgetStatePropertyAll(EdgeInsets.zero),
            foregroundColor: WidgetStateProperty.resolveWith(
              // Only override the color when the button is not pressed or focused
              (states) => states.isPressedOrFocused ? null : context.colorScheme.onSurface,
            ),
            side: WidgetStateProperty.resolveWith(
              (states) => _resolveBorderSide(context, states),
            ),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              SizedBox(
                width: double.infinity,
                height: _kHeaderStripHeight,
                child: _buildHeaderStrip(context),
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
    final textPressedColor = context.theme.textButtonTheme.style?.foregroundColor?.resolve({WidgetState.pressed});
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        DefaultTextAndFocusStyle(
          statesController: _statesController,
          textStyle: context.textTheme.titleMedium,
          pressedOrFocusedColor: textPressedColor,
          child: Text.rich(
            context.l10n
                .sharedAttributesCardTitle(
                  widget.card.title.l10nValue(context),
                  widget.attributes.length,
                )
                .toTextSpan(context),
          ),
        ),
        const SizedBox(height: 8),
        DefaultTextAndFocusStyle(
          statesController: _statesController,
          textStyle: context.textTheme.bodyLarge,
          pressedOrFocusedColor: textPressedColor,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: _buildAttributeList(context),
          ),
        ),
        const SizedBox(height: 16),
        Focus(
          // Prevents the button from being focused
          canRequestFocus: false,
          descendantsAreFocusable: false,
          child: TextButton.icon(
            onPressed: widget.onTap,
            statesController: _statesController,
            icon: const Icon(Icons.arrow_forward),
            iconAlignment: IconAlignment.end,
            label: Semantics(
              button: true,
              attributedLabel: context.l10n
                  .sharedAttributesCardCtaSemanticsLabel(widget.card.title.l10nValue(context))
                  .toAttributedString(context),
              excludeSemantics: true /* exclude semantics of descendants */,
              child: Text.rich(context.l10n.sharedAttributesCardCta.toTextSpan(context)),
            ),
            style: context.theme.textButtonTheme.style?.copyWith(
              backgroundColor: WidgetStateProperty.all(Colors.transparent),
              minimumSize: const WidgetStatePropertyAll(Size.zero),
              padding: const WidgetStatePropertyAll(EdgeInsets.zero),
              side: WidgetStateBorderSide.resolveWith((states) => BorderSide.none),
              tapTargetSize: MaterialTapTargetSize.shrinkWrap,
            ),
          ),
        ),
        const SizedBox(height: 4),
      ],
    );
  }

  Widget _buildHeaderStrip(BuildContext context) {
    return ClipRRect(
      borderRadius: const BorderRadius.only(
        topLeft: _kCornerRadius,
        topRight: _kCornerRadius,
      ),
      child: widget.card.getL10nBackground(context),
    );
  }

  List<Widget> _buildAttributeList(BuildContext context) {
    return widget.attributes
        .map(
          (attribute) => Text.rich(
            attribute.label.l10nSpan(context),
          ),
        )
        .toList();
  }

  BorderSide? _resolveBorderSide(BuildContext context, Set<WidgetState> states) {
    // Override all non-focused states to always display a border
    return !states.isFocused
        ? BaseWalletTheme.buttonBorderSideFocused.copyWith(
            color: context.colorScheme.outlineVariant,
            strokeAlign: BorderSide.strokeAlignOutside,
            width: 1,
          )
        : null;
  }
}
